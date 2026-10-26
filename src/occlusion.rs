use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde_json::Value;
use smithay_client_toolkit::reexports::calloop::ping::Ping;

/// Detector reactivo de oclusión / visibilidad del fondo en el compositor Wayland
#[derive(Clone)]
pub struct OcclusionDetector {
    is_occluded: Arc<AtomicBool>,
}

impl OcclusionDetector {
    pub fn new(enabled: bool, target_output: Option<String>, ping: Option<Ping>) -> Self {
        let is_occluded = Arc::new(AtomicBool::new(false));

        if !enabled {
            return Self { is_occluded };
        }

        // Detectar si estamos bajo el compositor Niri
        if let Ok(socket_path) = std::env::var("NIRI_SOCKET") {
            let flag = is_occluded.clone();
            thread::Builder::new()
                .name("niri-occlusion-detector".into())
                .spawn(move || {
                    run_niri_listener(&socket_path, target_output.as_deref(), flag, ping);
                })
                .expect("Failed to spawn niri occlusion listener thread");
        }

        Self { is_occluded }
    }

    pub fn is_occluded(&self) -> bool {
        self.is_occluded.load(Ordering::Relaxed)
    }
}

fn run_niri_listener(
    socket_path: &str,
    target_output: Option<&str>,
    flag: Arc<AtomicBool>,
    ping: Option<Ping>,
) {
    loop {
        match UnixStream::connect(socket_path) {
            Ok(mut stream) => {
                if stream.write_all(b"\"EventStream\"\n").is_err() {
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
                let mut reader = BufReader::new(stream);
                let mut line = String::new();

                let mut overview_open = false;
                let mut workspaces: Vec<Value> = Vec::new();
                let mut windows: Vec<Value> = Vec::new();

                while reader.read_line(&mut line).is_ok() {
                    if line.is_empty() {
                        break;
                    }

                    if let Ok(val) = serde_json::from_str::<Value>(&line) {
                        if let Some(ws_obj) = val.get("WorkspacesChanged") {
                            if let Some(list) = ws_obj.get("workspaces").and_then(|v| v.as_array()) {
                                workspaces = list.clone();
                            }
                        } else if let Some(win_obj) = val.get("WindowsChanged") {
                            if let Some(list) = win_obj.get("windows").and_then(|v| v.as_array()) {
                                windows = list.clone();
                            }
                        } else if let Some(ov_obj) = val.get("OverviewOpenedOrClosed") {
                            if let Some(open) = ov_obj.get("is_open").and_then(|v| v.as_bool()) {
                                overview_open = open;
                            }
                        }

                        let covered = calculate_occlusion(overview_open, target_output, &workspaces, &windows);
                        let prev = flag.swap(covered, Ordering::Relaxed);
                        if prev != covered {
                            if covered {
                                eprintln!("[chess-wallpaper] Pantalla ocupada por ventanas -> Pausando animación (0% CPU)");
                            } else {
                                eprintln!("[chess-wallpaper] Fondo visible -> Reanudando animación");
                                if let Some(ref p) = ping {
                                    p.ping();
                                }
                            }
                        }
                    }
                    line.clear();
                }
            }
            Err(_) => {
                thread::sleep(Duration::from_secs(2));
            }
        }
    }
}

pub fn calculate_occlusion(
    overview_open: bool,
    target_output: Option<&str>,
    workspaces: &[Value],
    windows: &[Value],
) -> bool {
    // Si Overview está abierto, el fondo es visible siempre
    if overview_open {
        return false;
    }

    // Buscar workspace activo en el output correspondiente
    let active_ws = workspaces.iter().find(|w| {
        let is_active = w.get("is_active").and_then(|v| v.as_bool()).unwrap_or(false);
        if !is_active {
            return false;
        }
        if let Some(target) = target_output {
            w.get("output").and_then(|v| v.as_str()) == Some(target)
        } else {
            // Si no se especificó output, tomar el enfocado o cualquier activo
            w.get("is_focused").and_then(|v| v.as_bool()).unwrap_or(true)
        }
    });

    if let Some(ws) = active_ws {
        if let Some(id) = ws.get("id").and_then(|v| v.as_u64()) {
            // Verificar si hay ventanas en este workspace activo
            let wins_count = windows
                .iter()
                .filter(|w| w.get("workspace_id").and_then(|v| v.as_u64()) == Some(id))
                .count();

            return wins_count > 0;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_overview_unoccluded() {
        let workspaces = vec![json!({
            "id": 1,
            "is_active": true,
            "is_focused": true,
            "output": "eDP-1"
        })];
        let windows = vec![json!({
            "id": 10,
            "workspace_id": 1
        })];

        // Aunque haya ventanas, si overview está abierto el fondo es visible
        assert!(!calculate_occlusion(true, None, &workspaces, &windows));
    }

    #[test]
    fn test_empty_workspace_unoccluded() {
        let workspaces = vec![json!({
            "id": 2,
            "is_active": true,
            "is_focused": true,
            "output": "eDP-1"
        })];
        let windows = vec![json!({
            "id": 10,
            "workspace_id": 1 // Ventana en el workspace 1, el 2 está vacío
        })];

        // Workspace activo 2 no tiene ventanas -> fondo visible
        assert!(!calculate_occlusion(false, None, &workspaces, &windows));
    }

    #[test]
    fn test_workspace_with_windows_is_occluded() {
        let workspaces = vec![json!({
            "id": 1,
            "is_active": true,
            "is_focused": true,
            "output": "eDP-1"
        })];
        let windows = vec![json!({
            "id": 10,
            "workspace_id": 1
        })];

        // Workspace activo 1 tiene una ventana -> fondo tapado
        assert!(calculate_occlusion(false, None, &workspaces, &windows));
    }
}
