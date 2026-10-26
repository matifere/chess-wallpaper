use std::collections::HashMap;
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

/// Estado en memoria sincronizado en tiempo real con Niri EventStream
#[derive(Default, Debug)]
pub struct NiriState {
    pub overview_open: bool,
    pub active_workspace_id: Option<u64>,
    pub workspaces: HashMap<u64, Value>,
    pub windows: HashMap<u64, u64>, // win_id -> workspace_id
}

impl NiriState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process_event(&mut self, val: &Value) {
        if let Some(ws_obj) = val.get("WorkspacesChanged") {
            if let Some(list) = ws_obj.get("workspaces").and_then(|v| v.as_array()) {
                self.workspaces.clear();
                for ws in list {
                    if let Some(id) = ws.get("id").and_then(|v| v.as_u64()) {
                        let is_active = ws.get("is_active").and_then(|v| v.as_bool()).unwrap_or(false);
                        let is_focused = ws.get("is_focused").and_then(|v| v.as_bool()).unwrap_or(false);
                        if is_active || is_focused {
                            self.active_workspace_id = Some(id);
                        }
                        self.workspaces.insert(id, ws.clone());
                    }
                }
            }
        } else if let Some(act) = val.get("WorkspaceActivated") {
            if let Some(id) = act.get("id").and_then(|v| v.as_u64()) {
                self.active_workspace_id = Some(id);
                for (&wid, ws) in self.workspaces.iter_mut() {
                    if let Some(map) = ws.as_object_mut() {
                        let active = wid == id;
                        map.insert("is_active".into(), Value::Bool(active));
                        map.insert("is_focused".into(), Value::Bool(active));
                    }
                }
            }
        } else if let Some(win_obj) = val.get("WindowsChanged") {
            if let Some(list) = win_obj.get("windows").and_then(|v| v.as_array()) {
                self.windows.clear();
                for win in list {
                    if let (Some(win_id), Some(ws_id)) = (
                        win.get("id").and_then(|v| v.as_u64()),
                        win.get("workspace_id").and_then(|v| v.as_u64()),
                    ) {
                        self.windows.insert(win_id, ws_id);
                    }
                }
            }
        } else if let Some(win_obj) = val.get("WindowOpenedOrChanged").and_then(|v| v.get("window")) {
            if let (Some(win_id), Some(ws_id)) = (
                win_obj.get("id").and_then(|v| v.as_u64()),
                win_obj.get("workspace_id").and_then(|v| v.as_u64()),
            ) {
                self.windows.insert(win_id, ws_id);
            }
        } else if let Some(closed) = val.get("WindowClosed") {
            if let Some(win_id) = closed.get("id").and_then(|v| v.as_u64()) {
                self.windows.remove(&win_id);
            }
        } else if let Some(ov_obj) = val.get("OverviewOpenedOrClosed") {
            if let Some(open) = ov_obj.get("is_open").and_then(|v| v.as_bool()) {
                self.overview_open = open;
            }
        }
    }

    pub fn is_occluded(&self, target_output: Option<&str>) -> bool {
        // Si Overview está abierto, el fondo es siempre visible
        if self.overview_open {
            return false;
        }

        let Some(active_ws_id) = self.active_workspace_id else {
            return false;
        };

        // Si se especificó un monitor concreto, verificar que el workspace pertenezca a él
        if let Some(target) = target_output {
            if let Some(ws) = self.workspaces.get(&active_ws_id) {
                if ws.get("output").and_then(|v| v.as_str()) != Some(target) {
                    return false;
                }
            }
        }

        // Contar ventanas que pertenecen al workspace activo
        let wins_in_active = self.windows.values().filter(|&&ws_id| ws_id == active_ws_id).count();
        wins_in_active > 0
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
                let mut state = NiriState::new();

                while reader.read_line(&mut line).is_ok() {
                    if line.is_empty() {
                        break;
                    }

                    if let Ok(val) = serde_json::from_str::<Value>(&line) {
                        state.process_event(&val);

                        let covered = state.is_occluded(target_output);
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

#[allow(dead_code)]
pub fn calculate_occlusion(
    overview_open: bool,
    target_output: Option<&str>,
    workspaces: &[Value],
    windows: &[Value],
) -> bool {
    let mut state = NiriState::new();
    state.overview_open = overview_open;
    state.process_event(&serde_json::json!({
        "WorkspacesChanged": {
            "workspaces": workspaces
        }
    }));
    state.process_event(&serde_json::json!({
        "WindowsChanged": {
            "windows": windows
        }
    }));
    state.is_occluded(target_output)
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
            "workspace_id": 1
        })];

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

        assert!(calculate_occlusion(false, None, &workspaces, &windows));
    }

    #[test]
    fn test_workspace_activated_dynamic_switch() {
        let mut state = NiriState::new();
        // Inicialmente en workspace 1 con 1 ventana
        state.process_event(&json!({
            "WorkspacesChanged": {
                "workspaces": [
                    {"id": 1, "is_active": true, "is_focused": true},
                    {"id": 2, "is_active": false, "is_focused": false}
                ]
            }
        }));
        state.process_event(&json!({
            "WindowsChanged": {
                "windows": [
                    {"id": 100, "workspace_id": 1}
                ]
            }
        }));

        assert!(state.is_occluded(None));

        // El usuario cambia al workspace 2 (vacío)
        state.process_event(&json!({
            "WorkspaceActivated": {
                "id": 2,
                "focused": true
            }
        }));

        // ¡Ahora debe estar desocluido!
        assert!(!state.is_occluded(None));

        // El usuario vuelve al workspace 1
        state.process_event(&json!({
            "WorkspaceActivated": {
                "id": 1,
                "focused": true
            }
        }));

        assert!(state.is_occluded(None));
    }

    #[test]
    fn test_window_open_and_close_dynamic() {
        let mut state = NiriState::new();
        state.process_event(&json!({
            "WorkspacesChanged": {
                "workspaces": [
                    {"id": 1, "is_active": true, "is_focused": true}
                ]
            }
        }));

        // Sin ventanas -> no ocluido
        assert!(!state.is_occluded(None));

        // Se abre una ventana en workspace 1
        state.process_event(&json!({
            "WindowOpenedOrChanged": {
                "window": {
                    "id": 55,
                    "workspace_id": 1
                }
            }
        }));

        assert!(state.is_occluded(None));

        // Se cierra la ventana
        state.process_event(&json!({
            "WindowClosed": {
                "id": 55
            }
        }));

        assert!(!state.is_occluded(None));
    }
}
