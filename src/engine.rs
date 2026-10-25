use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use shakmaty::san::San;
use shakmaty::{Chess, Position, Role};
use tiny_skia::Color as SkColor;

use crate::chess_sim::eval_to_white_pct;

/// Clasificación oficial de jugadas al estilo Chess.com Game Review
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveClassification {
    Book,        // 📖 Libro
    Brilliant,   // 💎 Brillante (!!)
    Great,       // 🔷 Gran jugada (!)
    Best,        // ⭐️ Mejor jugada (★)
    Excellent,   // 🟢 Excelente (✔)
    Good,        // 🟢 Buena
    Inaccuracy,  // 🟡 Imprecisión (?!)
    Mistake,     // 🟠 Error (?)
    Blunder,     // 🔴 Grave error (??)
    Miss,        // ✖ Oportunidad perdida
}

impl MoveClassification {
    pub fn symbol(&self) -> &'static str {
        match self {
            MoveClassification::Brilliant => "!!",
            MoveClassification::Great => "!",
            MoveClassification::Best => "★",
            MoveClassification::Excellent => "✔",
            MoveClassification::Good => "",
            MoveClassification::Book => "📖",
            MoveClassification::Inaccuracy => "?!",
            MoveClassification::Mistake => "?",
            MoveClassification::Blunder => "??",
            MoveClassification::Miss => "✖",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            MoveClassification::Brilliant => "Brillante",
            MoveClassification::Great => "Gran jugada",
            MoveClassification::Best => "Mejor",
            MoveClassification::Excellent => "Excelente",
            MoveClassification::Good => "Buena",
            MoveClassification::Book => "Libro",
            MoveClassification::Inaccuracy => "Imprecisión",
            MoveClassification::Mistake => "Error",
            MoveClassification::Blunder => "Grave error",
            MoveClassification::Miss => "Perdida",
        }
    }

    pub fn bg_color(&self) -> SkColor {
        match self {
            MoveClassification::Brilliant => SkColor::from_rgba8(38, 194, 163, 255), // #26c2a3 Cyan/Turquesa
            MoveClassification::Great => SkColor::from_rgba8(92, 139, 176, 255),     // #5c8bb0 Azul acero
            MoveClassification::Best => SkColor::from_rgba8(129, 182, 76, 255),      // #81b64c Verde Chess.com
            MoveClassification::Excellent => SkColor::from_rgba8(150, 188, 75, 255), // #96bc4b Verde claro
            MoveClassification::Good => SkColor::from_rgba8(130, 168, 65, 230),      // Verde oliva
            MoveClassification::Book => SkColor::from_rgba8(168, 139, 103, 255),     // #a88b67 Marrón pergamino
            MoveClassification::Inaccuracy => SkColor::from_rgba8(240, 193, 92, 255),// #f0c15c Amarillo/ámbar
            MoveClassification::Mistake => SkColor::from_rgba8(230, 145, 44, 255),   // #e6912c Naranja
            MoveClassification::Blunder => SkColor::from_rgba8(250, 65, 45, 255),     // #fa412d Rojo carmesí
            MoveClassification::Miss => SkColor::from_rgba8(195, 84, 133, 255),      // #c35485 Magenta oscuro
        }
    }

    pub fn text_color(&self) -> SkColor {
        match self {
            MoveClassification::Inaccuracy => SkColor::from_rgba8(30, 30, 35, 255), // Texto oscuro sobre amarillo
            _ => SkColor::from_rgba8(255, 255, 255, 255),
        }
    }
}

/// Evaluación de una posición específica generada por el motor
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct PositionEval {
    pub eval_score: f32, // En peones desde la perspectiva de Blancas (ej. +1.4, -3.2, +100.0)
    pub eval_pct: f32,   // 0.0 .. 1.0 mapeado para la Eval Bar
    pub mate_in: Option<i32>, // Some(1) = Blancas dan mate en 1, Some(-2) = Negras dan mate en 2
    pub classification: MoveClassification,
    pub best_move_uci: Option<String>,
}

/// Búsqueda del ejecutable de Stockfish en el sistema o carpeta local
pub fn find_stockfish_binary() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("STOCKFISH_PATH") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Some(path);
        }
    }

    if let Some(home) = std::env::var_os("HOME") {
        let local_p = PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("chess-wallpaper")
            .join("bin")
            .join("stockfish");
        if local_p.is_file() {
            return Some(local_p);
        }
    }

    for p in &["/usr/bin/stockfish", "/usr/local/bin/stockfish", "/usr/games/stockfish"] {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Some(path);
        }
    }

    None
}

/// Analiza una partida completa con Stockfish en un solo pase de ~0.5s a profundidad 8-10
pub fn analyze_game_with_stockfish(
    stockfish_path: &Path,
    moves_san: &[String],
) -> Result<Vec<PositionEval>, String> {
    let mut pos = Chess::default();
    let mut uci_moves = Vec::with_capacity(moves_san.len());
    let mut roles_moved = Vec::with_capacity(moves_san.len());
    let mut captures = Vec::with_capacity(moves_san.len());

    for san_str in moves_san {
        let san: San = san_str.parse().map_err(|e| format!("SAN parse error: {}", e))?;
        let m = san.to_move(&pos).map_err(|e| format!("Illegal move {}: {}", san_str, e))?;
        let uci = m.to_uci(shakmaty::CastlingMode::Standard).to_string();
        uci_moves.push(uci);
        roles_moved.push(m.role());
        captures.push(m.is_capture());
        pos.play_unchecked(m);
    }

    let mut child = Command::new(stockfish_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Error al ejecutar Stockfish: {}", e))?;

    let mut stdin = child.stdin.take().ok_or("No se pudo abrir stdin de Stockfish")?;
    let stdout = child.stdout.take().ok_or("No se pudo abrir stdout de Stockfish")?;
    let mut reader = BufReader::new(stdout);

    // Inicializar protocolo UCI
    writeln!(stdin, "uci").map_err(|e| e.to_string())?;
    stdin.flush().map_err(|e| e.to_string())?;

    let mut line = String::new();
    while reader.read_line(&mut line).map_err(|e| e.to_string())? > 0 {
        if line.trim() == "uciok" {
            break;
        }
        line.clear();
    }

    writeln!(stdin, "setoption name Threads value 2").map_err(|e| e.to_string())?;
    writeln!(stdin, "setoption name Hash value 16").map_err(|e| e.to_string())?;
    writeln!(stdin, "isready").map_err(|e| e.to_string())?;
    stdin.flush().map_err(|e| e.to_string())?;

    line.clear();
    while reader.read_line(&mut line).map_err(|e| e.to_string())? > 0 {
        if line.trim() == "readyok" {
            break;
        }
        line.clear();
    }

    // Evaluar cada una de las posiciones (k = 0..=N)
    let n = uci_moves.len();
    let mut raw_evals: Vec<(f32, Option<i32>, String)> = Vec::with_capacity(n + 1);

    for k in 0..=n {
        if k == 0 {
            writeln!(stdin, "position startpos").map_err(|e| e.to_string())?;
        } else {
            let moves_str = uci_moves[0..k].join(" ");
            writeln!(stdin, "position startpos moves {}", moves_str).map_err(|e| e.to_string())?;
        }
        writeln!(stdin, "go depth 8").map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;

        let mut best_move = String::new();
        let mut score_cp = 0i32;
        let mut mate_in = None;

        line.clear();
        while reader.read_line(&mut line).map_err(|e| e.to_string())? > 0 {
            let trimmed = line.trim();
            if trimmed.contains("score cp ") {
                if let Some(pos) = trimmed.find("score cp ") {
                    let rest = &trimmed[pos + 9..];
                    if let Some(token) = rest.split_whitespace().next() {
                        if let Ok(cp) = token.parse::<i32>() {
                            score_cp = cp;
                            mate_in = None;
                        }
                    }
                }
            } else if trimmed.contains("score mate ") {
                if let Some(pos) = trimmed.find("score mate ") {
                    let rest = &trimmed[pos + 11..];
                    if let Some(token) = rest.split_whitespace().next() {
                        if let Ok(m) = token.parse::<i32>() {
                            mate_in = Some(m);
                            score_cp = if m > 0 { 10000 } else { -10000 };
                        }
                    }
                }
            }

            if trimmed.starts_with("bestmove ") {
                if let Some(bm) = trimmed.split_whitespace().nth(1) {
                    best_move = bm.to_string();
                }
                break;
            }
            line.clear();
        }

        let is_white_turn = k % 2 == 0;
        let w_score = if is_white_turn {
            score_cp as f32 / 100.0
        } else {
            -score_cp as f32 / 100.0
        };
        let w_mate = if is_white_turn {
            mate_in
        } else {
            mate_in.map(|m| -m)
        };

        raw_evals.push((w_score, w_mate, best_move));
    }

    let _ = writeln!(stdin, "quit");
    let _ = child.wait();

    // Clasificar cada jugada según la caída de evaluación y sacrificios
    let mut plies: Vec<PositionEval> = Vec::with_capacity(n);

    for i in 0..n {
        let (w_before, m_bef, ref bm) = raw_evals[i];
        let (w_after, m_aft, _) = raw_evals[i + 1];
        let is_white = i % 2 == 0;

        let p_drop = if let Some(m_after_val) = m_aft {
            if (is_white && m_after_val > 0) || (!is_white && m_after_val < 0) {
                0.0
            } else {
                5.0
            }
        } else if let Some(m_before_val) = m_bef {
            if (is_white && m_before_val > 0) || (!is_white && m_before_val < 0) {
                0.0
            } else {
                5.0
            }
        } else if is_white {
            w_before - w_after
        } else {
            w_after - w_before
        };

        let is_best = &uci_moves[i] == bm;
        let role = roles_moved[i];
        let is_piece = matches!(role, Role::Queen | Role::Rook | Role::Bishop | Role::Knight);

        let player_after_eval = if is_white { w_after } else { -w_after };
        let is_winning_or_drawing = player_after_eval >= 0.8 || m_aft.is_some_and(|m| (is_white && m > 0) || (!is_white && m < 0));

        let is_sac = is_piece && (
            (i + 1 < n && captures[i + 1]) ||
            (role == Role::Queen && p_drop <= 0.15 && is_winning_or_drawing)
        );

        let classification = if i < 4 && p_drop <= 0.35 {
            MoveClassification::Book
        } else if is_sac && (is_best || p_drop <= 0.15) && is_winning_or_drawing {
            MoveClassification::Brilliant
        } else if is_best && (player_after_eval >= 3.0 || m_aft.is_some()) && p_drop <= 0.05 {
            MoveClassification::Great
        } else if is_best {
            MoveClassification::Best
        } else if p_drop <= 0.25 {
            MoveClassification::Excellent
        } else if p_drop <= 0.55 {
            MoveClassification::Good
        } else if p_drop <= 1.25 {
            MoveClassification::Inaccuracy
        } else if p_drop <= 2.20 {
            MoveClassification::Mistake
        } else if player_after_eval < 0.5 && p_drop > 2.50 {
            MoveClassification::Miss
        } else {
            MoveClassification::Blunder
        };

        let pct = if let Some(m) = m_aft {
            if m > 0 { 1.0 } else { 0.0 }
        } else {
            eval_to_white_pct(w_after)
        };

        plies.push(PositionEval {
            eval_score: w_after,
            eval_pct: pct,
            mate_in: m_aft,
            classification,
            best_move_uci: Some(bm.clone()),
        });
    }

    Ok(plies)
}

/// Gestor de caché y análisis en segundo plano para no consumir CPU en la simulación
pub struct EngineManager {
    stockfish_path: Option<PathBuf>,
    cache: Arc<Mutex<HashMap<usize, Vec<PositionEval>>>>,
}

impl EngineManager {
    pub fn new() -> Self {
        let stockfish_path = find_stockfish_binary();
        if let Some(ref p) = stockfish_path {
            println!("Motor Stockfish detectado: {}", p.display());
        } else {
            println!("Aviso: Stockfish no encontrado. Usando evaluación estática de respaldo.");
        }

        Self {
            stockfish_path,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_analysis(&self, game_idx: usize) -> Option<Vec<PositionEval>> {
        let cache = self.cache.lock().ok()?;
        cache.get(&game_idx).cloned()
    }

    pub fn request_analysis(&self, game_idx: usize, moves: Vec<String>) {
        if let Some(sf_path) = self.stockfish_path.clone() {
            let cache_ref = Arc::clone(&self.cache);
            {
                if let Ok(cache) = cache_ref.lock() {
                    if cache.contains_key(&game_idx) {
                        return; // Ya analizada previamente
                    }
                }
            }

            thread::spawn(move || {
                if let Ok(evals) = analyze_game_with_stockfish(&sf_path, &moves) {
                    if let Ok(mut cache) = cache_ref.lock() {
                        cache.insert(game_idx, evals);
                    }
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stockfish_binary_exists() {
        let sf = find_stockfish_binary();
        assert!(sf.is_some(), "Stockfish debe estar presente en el sistema o en ~/.local/share/chess-wallpaper/bin/stockfish");
    }

    #[test]
    fn test_analyze_opera_game_with_stockfish() {
        let sf = find_stockfish_binary().expect("Stockfish needed for test");
        let moves = vec![
            "e4".to_string(), "e5".to_string(), "Nf3".to_string(), "d6".to_string(),
            "d4".to_string(), "Bg4".to_string(), "dxe5".to_string(), "Bxf3".to_string(),
            "Qxf3".to_string(), "dxe5".to_string(), "Bc4".to_string(), "Nf6".to_string(),
            "Qb3".to_string(), "Qe7".to_string(), "Nc3".to_string(), "c6".to_string(),
            "Bg5".to_string(), "b5".to_string(), "Nxb5".to_string(), "cxb5".to_string(),
            "Bxb5+".to_string(), "Nbd7".to_string(), "O-O-O".to_string(), "Rd8".to_string(),
            "Rxd7".to_string(), "Rxd7".to_string(), "Rd1".to_string(), "Qe6".to_string(),
            "Bxd7+".to_string(), "Nxd7".to_string(), "Qb8+".to_string(), "Nxb8".to_string(),
            "Rd8#".to_string(),
        ];

        let analysis = analyze_game_with_stockfish(&sf, &moves).expect("Analysis failed");
        assert_eq!(analysis.len(), moves.len());

        // La jugada 31 es 16. Qb8+!! (sacrificio de dama)
        let q_sac = &analysis[30];
        assert_eq!(q_sac.classification, MoveClassification::Brilliant);
        assert_eq!(q_sac.mate_in, Some(1));
        assert_eq!(q_sac.eval_pct, 1.0);
    }

    #[test]
    fn test_render_brilliant_badge_snapshot() {
        use crate::assets::{AssetManager, BoardStyle, PieceStyle};
        use crate::chess_sim::ChessSimulator;
        use crate::renderer::render_frame;
        use tiny_skia::{Color, Pixmap};
        use std::time::Instant;

        let assets = AssetManager::load_embedded();
        let mut sim = ChessSimulator::new(260, 500, BoardStyle::Tournament, PieceStyle::Neo, false, vec![]);
        let opera_idx = sim.games.iter().position(|g| g.event.contains("Opera Game")).unwrap();
        sim.playlist_cursor = sim.playlist.iter().position(|&idx| idx == opera_idx).unwrap();
        sim.engine.request_analysis(opera_idx, sim.games[opera_idx].moves.clone());

        // Esperar a que el análisis de Stockfish en segundo plano se complete
        for _ in 0..50 {
            if sim.engine.get_analysis(opera_idx).is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        // Reproducir hasta la jugada 31: 16. Qb8+!! (sacrificio de dama)
        for _ in 0..31 {
            sim.perform_next_move(Instant::now());
        }

        assert_eq!(sim.current_classification, Some(MoveClassification::Brilliant));
        assert_eq!(sim.current_mate_in, Some(1));
        assert_eq!(sim.target_eval_pct, 1.0);

        let mut pixmap = Pixmap::new(1920, 1080).unwrap();
        let bg = Color::from_rgba8(20, 22, 34, 255);
        render_frame(&mut pixmap.as_mut(), &assets, &sim, bg, 640.0, Instant::now());
        pixmap.save_png("/tmp/chess_brilliant_snapshot.png").unwrap();
    }
}
