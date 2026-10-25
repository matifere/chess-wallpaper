use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use shakmaty::san::San;
use shakmaty::{Chess, Position};

use crate::games::GameData;

/// Limpia y normaliza un texto PGN extrayendo todas las partidas válidas.
pub fn parse_pgn_text(content: &str) -> Vec<GameData> {
    let mut games = Vec::new();
    let mut current_tags: HashMap<String, String> = HashMap::new();
    let mut move_tokens: Vec<String> = Vec::new();
    let mut in_header = true;

    // Estados para comentarios y variantes
    let mut in_comment = false;
    let mut in_line_comment = false;
    let mut variation_depth = 0;

    // Buffer para construir el texto de jugadas sin comentarios
    let mut clean_move_text = String::with_capacity(content.len());

    for line in content.lines() {
        let trimmed = line.trim();

        if in_line_comment {
            in_line_comment = false;
        }

        // Si estamos buscando cabeceras y encontramos una etiqueta PGN: [Key "Value"]
        if trimmed.starts_with('[') && trimmed.ends_with(']') && !in_comment {
            // Si ya teníamos jugadas acumuladas de una partida previa, cerramos esa partida
            if !move_tokens.is_empty() || !clean_move_text.trim().is_empty() {
                if let Some(game) = finalize_game(&current_tags, &clean_move_text) {
                    games.push(game);
                }
                current_tags.clear();
                clean_move_text.clear();
                move_tokens.clear();
            }

            in_header = true;
            if let Some((k, v)) = parse_tag(trimmed) {
                current_tags.insert(k, v);
            }
            continue;
        }

        // Si es una línea vacía en zona de cabeceras, cambiamos a modo jugadas
        if in_header && trimmed.is_empty() {
            in_header = false;
            continue;
        }

        // Procesar caracteres de texto de jugadas eliminando comentarios { ... } y variantes ( ... )
        for ch in line.chars() {
            if in_line_comment {
                break;
            }
            if ch == ';' {
                in_line_comment = true;
                continue;
            }
            if ch == '{' {
                in_comment = true;
                continue;
            }
            if ch == '}' {
                in_comment = false;
                continue;
            }
            if in_comment {
                continue;
            }

            if ch == '(' {
                variation_depth += 1;
                continue;
            }
            if ch == ')' {
                if variation_depth > 0 {
                    variation_depth -= 1;
                }
                continue;
            }
            if variation_depth > 0 {
                continue;
            }

            clean_move_text.push(ch);
        }
        clean_move_text.push(' ');
    }

    // Cerrar la última partida del archivo si quedó pendiente
    if !clean_move_text.trim().is_empty() || !current_tags.is_empty() {
        if let Some(game) = finalize_game(&current_tags, &clean_move_text) {
            games.push(game);
        }
    }

    games
}

fn parse_tag(line: &str) -> Option<(String, String)> {
    let inner = line.strip_prefix('[')?.strip_suffix(']')?.trim();
    let space_idx = inner.find(char::is_whitespace)?;
    let key = inner[..space_idx].trim().to_string();
    let val_part = inner[space_idx..].trim();
    let val = val_part.strip_prefix('"')?.strip_suffix('"')?.to_string();
    Some((key, val))
}

fn finalize_game(tags: &HashMap<String, String>, move_text: &str) -> Option<GameData> {
    let mut pos = Chess::default();
    let mut valid_moves = Vec::new();

    for token in move_text.split_whitespace() {
        let t = token.trim();
        if t.is_empty() {
            continue;
        }

        // Fin de partida marcado en el texto
        if matches!(t, "1-0" | "0-1" | "1/2-1/2" | "*") {
            break;
        }

        // Ignorar sufijos NAG como $1, $2, etc.
        if t.starts_with('$') {
            continue;
        }

        // Extraer la jugada si viene precedida de número de jugada (ej. "1.e4" o "1...e5")
        let move_str = if let Some(dot_pos) = t.rfind('.') {
            t[dot_pos + 1..].trim()
        } else {
            t
        };

        if move_str.is_empty() {
            continue;
        }

        // Limpiar símbolos de evaluación/anotación como !, ?, !!, !?, etc.
        let clean_san = move_str.trim_end_matches(['!', '?']);
        if clean_san.is_empty() {
            continue;
        }

        // Validar con shakmaty que sea una jugada legal en la posición actual
        let san: San = match clean_san.parse() {
            Ok(s) => s,
            Err(_) => continue,
        };

        let m = match san.to_move(&pos) {
            Ok(m) => m,
            Err(_) => break, // Jugada ilegal, cortamos hasta aquí
        };

        valid_moves.push(clean_san.to_string());
        pos.play_unchecked(m);
    }

    // Descartar partidas que no tengan al menos 2 jugadas
    if valid_moves.len() < 2 {
        return None;
    }

    let white = tags
        .get("White")
        .cloned()
        .unwrap_or_else(|| "Blancas".to_string());
    let black = tags
        .get("Black")
        .cloned()
        .unwrap_or_else(|| "Negras".to_string());
    let white_elo = tags.get("WhiteElo").cloned();
    let black_elo = tags.get("BlackElo").cloned();
    let event = tags
        .get("Event")
        .cloned()
        .unwrap_or_else(|| "Partida PGN".to_string());
    let site = tags.get("Site").cloned();
    let date = tags.get("Date").cloned();
    let eco = tags.get("ECO").cloned();
    let opening = tags.get("Opening").cloned();

    let year = date
        .as_deref()
        .and_then(|d| d.split('.').next())
        .and_then(|y| y.parse::<u16>().ok())
        .unwrap_or(2024);

    let result = tags
        .get("Result")
        .cloned()
        .unwrap_or_else(|| {
            if pos.is_checkmate() {
                match pos.turn() {
                    shakmaty::Color::White => "0-1".to_string(),
                    shakmaty::Color::Black => "1-0".to_string(),
                }
            } else if pos.is_stalemate() || pos.is_insufficient_material() {
                "1/2-1/2".to_string()
            } else {
                "*".to_string()
            }
        });

    let termination = tags
        .get("Termination")
        .cloned()
        .unwrap_or_else(|| {
            if pos.is_checkmate() {
                "Jaque mate".to_string()
            } else if result == "1-0" {
                "Victoria de blancas".to_string()
            } else if result == "0-1" {
                "Victoria de negras".to_string()
            } else if result == "1/2-1/2" {
                "Tablas".to_string()
            } else {
                "Partida concluida".to_string()
            }
        });

    Some(GameData {
        white,
        black,
        white_elo,
        black_elo,
        event,
        site,
        date,
        year,
        eco,
        opening,
        result,
        termination,
        moves: valid_moves,
    })
}

/// Carga partidas de un archivo PGN específico
pub fn load_pgn_file(path: &Path) -> Result<Vec<GameData>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Error al leer {}: {}", path.display(), e))?;
    let games = parse_pgn_text(&content);
    if games.is_empty() {
        Err(format!("No se encontraron partidas válidas en {}", path.display()))
    } else {
        Ok(games)
    }
}

/// Carga todas las partidas de los archivos .pgn de un directorio
pub fn load_pgn_dir(dir: &Path) -> Vec<GameData> {
    let mut games = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() && p.extension().is_some_and(|ext| ext == "pgn") {
                if let Ok(file_games) = load_pgn_file(&p) {
                    println!("  [+] {} partidas cargadas de {}", file_games.len(), p.file_name().unwrap_or_default().to_string_lossy());
                    games.extend(file_games);
                }
            }
        }
    }
    games
}

/// Obtiene la ruta por defecto del usuario: ~/.config/chess-wallpaper/games/
pub fn get_default_user_games_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".config").join("chess-wallpaper").join("games"))
}

/// Carga todas las partidas de usuario disponibles (archivo individual, directorio especificado o directorio por defecto)
pub fn load_all_user_games(
    cli_file: Option<&str>,
    cli_dir: Option<&str>,
) -> Vec<GameData> {
    let mut user_games = Vec::new();

    // 1. Archivo específico pasado por CLI (--pgn)
    if let Some(f) = cli_file {
        let p = Path::new(f);
        match load_pgn_file(p) {
            Ok(g) => {
                println!("Cargadas {} partidas desde archivo PGN: {}", g.len(), p.display());
                user_games.extend(g);
            }
            Err(e) => {
                eprintln!("Advertencia: {}", e);
            }
        }
    }

    // 2. Directorio específico pasado por CLI (--pgn-dir)
    if let Some(d) = cli_dir {
        let p = Path::new(d);
        println!("Buscando partidas PGN en: {}", p.display());
        let g = load_pgn_dir(p);
        println!("Total en directorio {}: {} partidas.", p.display(), g.len());
        user_games.extend(g);
    }

    // 3. Carpeta por defecto del usuario ~/.config/chess-wallpaper/games/
    if let Some(default_dir) = get_default_user_games_dir() {
        if default_dir.is_dir() {
            println!("Explorando carpeta de partidas de usuario: {}", default_dir.display());
            let g = load_pgn_dir(&default_dir);
            if !g.is_empty() {
                println!("Total en carpeta de usuario: {} partidas.", g.len());
                user_games.extend(g);
            }
        } else {
            // Crear el directorio silenciosamente si no existe, para conveniencia del usuario
            let _ = fs::create_dir_all(&default_dir);
        }
    }

    user_games
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_pgn() {
        let pgn_text = r#"
[Event "Mati Championship"]
[Site "Buenos Aires, ARG"]
[Date "2024.10.25"]
[Round "1"]
[White "Mati"]
[Black "Deep Thought"]
[Result "1-0"]
[WhiteElo "1800"]
[BlackElo "1750"]
[ECO "C41"]
[Opening "Philidor Defense"]

1. e4 e5 2. Nf3 d6 3. d4 exd4 4. Qxd4 Nc6 5. Bb5 Bd7 6. Bxc6 Bxc6 7. Nc3 Nf6 8. Bg5 Be7 9. O-O-O O-O 1-0
"#;

        let games = parse_pgn_text(pgn_text);
        assert_eq!(games.len(), 1);
        let g = &games[0];
        assert_eq!(g.white, "Mati");
        assert_eq!(g.black, "Deep Thought");
        assert_eq!(g.white_elo.as_deref(), Some("1800"));
        assert_eq!(g.black_elo.as_deref(), Some("1750"));
        assert_eq!(g.eco.as_deref(), Some("C41"));
        assert_eq!(g.opening.as_deref(), Some("Philidor Defense"));
        assert_eq!(g.moves.len(), 18);
        assert_eq!(g.moves[0], "e4");
        assert_eq!(g.moves[1], "e5");
    }

    #[test]
    fn test_parse_pgn_with_comments_and_annotations() {
        let pgn_text = r#"
[Event "Rated Blitz game"]
[Site "https://lichess.org/test1234"]
[Date "2024.08.15"]
[White "PlayerOne"]
[Black "PlayerTwo"]
[Result "0-1"]

1. e4 {[%clk 0:03:00]} 1... c5 {[%clk 0:02:59]} 2. Nf3 {[%clk 0:02:58]} 2... d6 {[%clk 0:02:57]}
3. d4 cxd4 4. Nxd4 Nf6 5. Nc3 a6 { Najdorf variation } 6. Be3 e5! 7. Nb3 Be6 0-1
"#;

        let games = parse_pgn_text(pgn_text);
        assert_eq!(games.len(), 1);
        let g = &games[0];
        assert_eq!(g.white, "PlayerOne");
        assert_eq!(g.black, "PlayerTwo");
        assert_eq!(g.result, "0-1");
        assert_eq!(g.moves.len(), 14);
        assert_eq!(g.moves[11], "e5"); // "e5!" limpio como "e5"
    }

    #[test]
    fn test_parse_multiple_games() {
        let pgn_text = r#"
[Event "Game 1"]
[White "Alpha"]
[Black "Beta"]
[Result "1-0"]

1. e4 e5 2. Nf3 Nc6 1-0

[Event "Game 2"]
[White "Gamma"]
[Black "Delta"]
[Result "0-1"]

1. d4 d5 2. c4 c6 0-1
"#;

        let games = parse_pgn_text(pgn_text);
        assert_eq!(games.len(), 2);
        assert_eq!(games[0].white, "Alpha");
        assert_eq!(games[1].white, "Gamma");
    }
}
