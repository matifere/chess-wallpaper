use std::time::{Duration, Instant};
use rand::seq::SliceRandom;
use shakmaty::san::San;
use shakmaty::{Chess, Move, Piece, Position, Role};
use crate::assets::{BoardStyle, PieceStyle};
use crate::games::{get_master_games, GameData};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimAction {
    Quiet,
    Capture,
    KnightJump,
    KnightCapture,
    Castling,
}

#[derive(Clone, Debug)]
pub struct SinglePieceAnim {
    pub from: (u8, u8), // (file 0..7, rank 0..7)
    pub to: (u8, u8),
    pub piece: Piece,
    pub action: AnimAction,
}

impl SinglePieceAnim {
    pub fn current_pos(&self, progress: f32) -> (f32, f32) {
        let p = progress.clamp(0.0, 1.0);
        let fx = self.from.0 as f32;
        let fy = self.from.1 as f32;
        let tx = self.to.0 as f32;
        let ty = self.to.1 as f32;
        let dx = tx - fx;
        let dy = ty - fy;
        let dist = (dx * dx + dy * dy).sqrt().max(0.001);
        let nx = dx / dist;
        let ny = dy / dist;

        match self.action {
            AnimAction::Quiet | AnimAction::Castling => {
                // Deslizamiento fluido con cubic ease-out
                let inv = 1.0 - p;
                let t = 1.0 - inv * inv * inv;
                (fx + dx * t, fy + dy * t)
            }
            AnimAction::KnightJump => {
                // Salto del caballo con trayectoria suavemente curvada
                let inv = 1.0 - p;
                let t = 1.0 - inv * inv;
                let curve = (p * std::f32::consts::PI).sin() * 0.16;
                let px = -ny;
                let py = nx;
                (fx + dx * t + px * curve, fy + dy * t + py * curve)
            }
            AnimAction::Capture => {
                if p < 0.18 {
                    // Anticipación estilo Duolingo: retroceso de preparación
                    let u = p / 0.18;
                    let windup = (u * std::f32::consts::PI).sin() * 0.10;
                    (fx - nx * windup, fy - ny * windup)
                } else if p < 0.78 {
                    // Embestida enérgica directa hacia la casilla enemiga
                    let u = (p - 0.18) / 0.60;
                    let strike_t = u * u * (3.0 - 2.0 * u);
                    (fx + dx * strike_t, fy + dy * strike_t)
                } else {
                    // Impacto firme en la casilla de destino
                    (tx, ty)
                }
            }
            AnimAction::KnightCapture => {
                if p < 0.18 {
                    let u = p / 0.18;
                    let windup = (u * std::f32::consts::PI).sin() * 0.08;
                    (fx - nx * windup, fy - ny * windup)
                } else if p < 0.78 {
                    let u = (p - 0.18) / 0.60;
                    let strike_t = u * u * (3.0 - 2.0 * u);
                    let curve = (u * std::f32::consts::PI).sin() * 0.18;
                    let px = -ny;
                    let py = nx;
                    (fx + dx * strike_t + px * curve, fy + dy * strike_t + py * curve)
                } else {
                    (tx, ty)
                }
            }
        }
    }

    pub fn jump_height(&self, progress: f32) -> f32 {
        let p = progress.clamp(0.0, 1.0);
        match self.action {
            AnimAction::Quiet | AnimAction::Castling => {
                // Elevación sutil y elegante de planeo
                (p * std::f32::consts::PI).sin() * 0.10
            }
            AnimAction::KnightJump => {
                // Salto alto del caballo sobre otras piezas
                (p * std::f32::consts::PI).sin() * 0.42
            }
            AnimAction::Capture => {
                if p < 0.18 {
                    0.0
                } else if p < 0.78 {
                    let u = (p - 0.18) / 0.60;
                    (u * std::f32::consts::PI).sin() * 0.28
                } else {
                    0.0
                }
            }
            AnimAction::KnightCapture => {
                if p < 0.18 {
                    0.0
                } else if p < 0.78 {
                    let u = (p - 0.18) / 0.60;
                    (u * std::f32::consts::PI).sin() * 0.45
                } else {
                    0.0
                }
            }
        }
    }

    pub fn scale(&self, progress: f32) -> (f32, f32) {
        let p = progress.clamp(0.0, 1.0);
        match self.action {
            AnimAction::Quiet | AnimAction::Castling => {
                let jump = (p * std::f32::consts::PI).sin();
                let stretch = jump * 0.04;
                if p < 0.85 {
                    (1.0 - stretch * 0.4, 1.0 + stretch)
                } else {
                    let s = (p - 0.85) / 0.15;
                    let settle = (-6.0 * s).exp() * (s * std::f32::consts::PI * 2.0).sin() * 0.03;
                    (1.0 + settle, (1.0 - settle).max(0.92))
                }
            }
            AnimAction::KnightJump => {
                if p < 0.80 {
                    let jump = (p * std::f32::consts::PI).sin();
                    (1.0 - jump * 0.05, 1.0 + jump * 0.08)
                } else {
                    let s = (p - 0.80) / 0.20;
                    let bounce = (-5.0 * s).exp() * (s * std::f32::consts::PI * 2.5).sin() * 0.07;
                    (1.0 + bounce, (1.0 - bounce).max(0.88))
                }
            }
            AnimAction::Capture | AnimAction::KnightCapture => {
                if p < 0.18 {
                    // Carga / Squish de anticipación (se comprime hacia abajo y ensancha)
                    let u = p / 0.18;
                    let windup = (u * std::f32::consts::PI).sin() * 0.12;
                    (1.0 + windup, (1.0 - windup * 0.85).max(0.85))
                } else if p < 0.78 {
                    // En vuelo / Embestida: se estira verticalmente en la dirección del ataque
                    let u = (p - 0.18) / 0.60;
                    let stretch = (u * std::f32::consts::PI).sin() * 0.15;
                    (1.0 - stretch * 0.5, 1.0 + stretch)
                } else {
                    // Impacto físico contundente (Slam) y rebote elástico amortiguado
                    let u = (p - 0.78) / 0.22;
                    let decay = (-6.5 * u).exp();
                    let wave = (u * std::f32::consts::PI * 2.2).cos();
                    let impact = decay * wave * 0.22;
                    (1.0 + impact, (1.0 - impact * 0.85).max(0.72))
                }
            }
        }
    }

    pub fn tilt_degrees(&self, progress: f32) -> f32 {
        let p = progress.clamp(0.0, 1.0);
        let dx = (self.to.0 as f32) - (self.from.0 as f32);
        let sign = if dx > 0.0 { 1.0 } else if dx < 0.0 { -1.0 } else { 0.0 };

        match self.action {
            AnimAction::Quiet | AnimAction::Castling => {
                // Inclinación aerodinámica suave en la dirección del desplazamiento
                sign * (p * std::f32::consts::PI).sin() * 4.0
            }
            AnimAction::KnightJump => {
                // Cabeceo dinámico de galope
                sign * (p * std::f32::consts::PI * 1.5).sin() * 7.0
            }
            AnimAction::Capture | AnimAction::KnightCapture => {
                if p < 0.18 {
                    // Se inclina hacia atrás tensándose en la anticipación
                    let u = p / 0.18;
                    -sign * (u * std::f32::consts::PI).sin() * 6.0
                } else if p < 0.78 {
                    // Se inclina agresivamente hacia adelante durante la embestida
                    let u = (p - 0.18) / 0.60;
                    sign * (u * std::f32::consts::PI).sin() * 9.0
                } else {
                    // Se estabiliza verticalmente tras el impacto
                    let u = (p - 0.78) / 0.22;
                    sign * 9.0 * (-8.0 * u).exp()
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct MoveAnim {
    pub primary: SinglePieceAnim,
    pub secondary: Option<SinglePieceAnim>, // Torre en caso de enroque
    pub captured: Option<(Piece, (u8, u8))>, // Pieza capturada y casilla donde estaba
    pub start: Instant,
    pub duration: Duration,
}

impl MoveAnim {
    pub fn progress(&self, now: Instant) -> f32 {
        let elapsed = now.saturating_duration_since(self.start).as_secs_f32();
        let total = self.duration.as_secs_f32();
        if elapsed >= total {
            1.0
        } else {
            elapsed / total
        }
    }
}

pub enum SimState {
    Idle { next_move_at: Instant },
    Animating(MoveAnim),
    GameOver { restart_at: Instant },
}

pub struct ChessSimulator {
    pub pos: Chess,
    pub state: SimState,
    pub last_move: Option<((u8, u8), (u8, u8))>,
    pub board_style: BoardStyle,
    pub piece_style: PieceStyle,
    pub rotate_theme: bool,
    pub move_duration: Duration,
    pub pause_between_moves: Duration,
    pub pause_after_game: Duration,

    games: Vec<GameData>,
    playlist: Vec<usize>,
    playlist_cursor: usize,
    current_move_index: usize,
}

impl ChessSimulator {
    pub fn new(
        move_ms: u64,
        delay_ms: u64,
        board_style: BoardStyle,
        piece_style: PieceStyle,
        rotate_theme: bool,
    ) -> Self {
        let games = get_master_games();
        let mut playlist: Vec<usize> = (0..games.len()).collect();
        let mut rng = rand::rng();
        playlist.shuffle(&mut rng);

        let first_game = &games[playlist[0]];
        println!("Cargadas {} partidas maestras históricas.", games.len());
        println!("Tablero: {:?} | Piezas: {:?}", board_style, piece_style);
        println!(
            "Partida inicial [1/{}]: '{} vs {}' - {} ({})",
            games.len(),
            first_game.white,
            first_game.black,
            first_game.event,
            first_game.year
        );

        Self {
            pos: Chess::default(),
            state: SimState::Idle {
                next_move_at: Instant::now() + Duration::from_millis(400),
            },
            last_move: None,
            board_style,
            piece_style,
            rotate_theme,
            move_duration: Duration::from_millis(move_ms),
            pause_between_moves: Duration::from_millis(delay_ms),
            pause_after_game: Duration::from_millis(3600),
            games,
            playlist,
            playlist_cursor: 0,
            current_move_index: 0,
        }
    }

    pub fn update(&mut self, now: Instant) -> bool {
        match self.state {
            SimState::Animating(ref anim) => {
                if anim.progress(now) >= 1.0 {
                    let current_game_idx = self.playlist[self.playlist_cursor];
                    let current_game = &self.games[current_game_idx];
                    if self.pos.is_game_over() || self.current_move_index >= current_game.moves.len() {
                        self.state = SimState::GameOver {
                            restart_at: now + self.pause_after_game,
                        };
                    } else {
                        self.state = SimState::Idle {
                            next_move_at: now + self.pause_between_moves,
                        };
                    }
                    return true;
                }
                true
            }
            SimState::Idle { next_move_at } => {
                if now >= next_move_at {
                    self.perform_next_move(now);
                    return true;
                }
                false
            }
            SimState::GameOver { restart_at } => {
                if now >= restart_at {
                    self.reset_game();
                    return true;
                }
                false
            }
        }
    }

    pub fn perform_next_move(&mut self, now: Instant) {
        let current_game_idx = self.playlist[self.playlist_cursor];
        let current_game = &self.games[current_game_idx];

        if self.pos.is_game_over() || self.current_move_index >= current_game.moves.len() {
            println!(
                "¡Fin de partida: '{} vs {}' - {} ({})! Resultado: {} ({}). Próxima partida en {:.1}s...",
                current_game.white,
                current_game.black,
                current_game.event,
                current_game.year,
                current_game.result,
                current_game.termination,
                self.pause_after_game.as_secs_f32()
            );
            self.state = SimState::GameOver {
                restart_at: now + self.pause_after_game,
            };
            return;
        }

        let san_str = current_game.moves[self.current_move_index];
        let chosen_move = match san_str.parse::<San>().ok().and_then(|s| s.to_move(&self.pos).ok()) {
            Some(m) => {
                self.current_move_index += 1;
                m
            }
            None => {
                eprintln!(
                    "Error al aplicar jugada {} ('{}') en '{}'",
                    self.current_move_index + 1,
                    san_str,
                    current_game.event
                );
                self.state = SimState::GameOver {
                    restart_at: now + self.pause_after_game,
                };
                return;
            }
        };

        // Construir la animación de forma exacta según el tipo de jugada (Normal, Enroque, En Passant)
        let anim = match chosen_move {
            Move::Castle { king, rook } => {
                let color = self.pos.turn();
                let rank = king.rank() as u8;
                let is_kingside = rook.file() > king.file();

                // En ajedrez estándar:
                // Enroque corto (Kingside): Rey e1->g1 (4->6), Torre h1->f1 (7->5)
                // Enroque largo (Queenside): Rey e1->c1 (4->2), Torre a1->d1 (0->3)
                let (king_to_file, rook_to_file) = if is_kingside {
                    (6, 5)
                } else {
                    (2, 3)
                };

                let king_anim = SinglePieceAnim {
                    from: (king.file() as u8, rank),
                    to: (king_to_file, rank),
                    piece: Piece { color, role: Role::King },
                    action: AnimAction::Castling,
                };

                let rook_anim = SinglePieceAnim {
                    from: (rook.file() as u8, rank),
                    to: (rook_to_file, rank),
                    piece: Piece { color, role: Role::Rook },
                    action: AnimAction::Castling,
                };

                MoveAnim {
                    primary: king_anim,
                    secondary: Some(rook_anim),
                    captured: None,
                    start: now,
                    duration: self.move_duration,
                }
            }
            Move::EnPassant { from, to } => {
                let color = self.pos.turn();
                let pawn = Piece { color, role: Role::Pawn };
                let enemy_pawn = Piece { color: !color, role: Role::Pawn };
                let captured_sq = (to.file() as u8, from.rank() as u8);

                MoveAnim {
                    primary: SinglePieceAnim {
                        from: (from.file() as u8, from.rank() as u8),
                        to: (to.file() as u8, to.rank() as u8),
                        piece: pawn,
                        action: AnimAction::Capture,
                    },
                    secondary: None,
                    captured: Some((enemy_pawn, captured_sq)),
                    start: now,
                    duration: self.move_duration,
                }
            }
            Move::Normal { role, from, capture, to, promotion } => {
                let color = self.pos.turn();
                let piece = if let Some(prom_role) = promotion {
                    Piece { color, role: prom_role }
                } else {
                    Piece { color, role }
                };

                let captured_info = capture.map(|c_role| {
                    (Piece { color: !color, role: c_role }, (to.file() as u8, to.rank() as u8))
                });

                let action = match (role, capture.is_some()) {
                    (Role::Knight, true) => AnimAction::KnightCapture,
                    (Role::Knight, false) => AnimAction::KnightJump,
                    (_, true) => AnimAction::Capture,
                    (_, false) => AnimAction::Quiet,
                };

                MoveAnim {
                    primary: SinglePieceAnim {
                        from: (from.file() as u8, from.rank() as u8),
                        to: (to.file() as u8, to.rank() as u8),
                        piece,
                        action,
                    },
                    secondary: None,
                    captured: captured_info,
                    start: now,
                    duration: self.move_duration,
                }
            }
            Move::Put { role, to } => {
                let color = self.pos.turn();
                let piece = Piece { color, role };
                MoveAnim {
                    primary: SinglePieceAnim {
                        from: (to.file() as u8, to.rank() as u8),
                        to: (to.file() as u8, to.rank() as u8),
                        piece,
                        action: AnimAction::Quiet,
                    },
                    secondary: None,
                    captured: None,
                    start: now,
                    duration: self.move_duration,
                }
            }
        };

        let (from_sq, to_sq) = match chosen_move {
            Move::Castle { king, rook } => {
                let rank = king.rank() as u8;
                let is_kingside = rook.file() > king.file();
                let king_to_file = if is_kingside { 6 } else { 2 };
                ((king.file() as u8, rank), (king_to_file, rank))
            }
            Move::EnPassant { from, to } => {
                ((from.file() as u8, from.rank() as u8), (to.file() as u8, to.rank() as u8))
            }
            Move::Normal { from, to, .. } => {
                ((from.file() as u8, from.rank() as u8), (to.file() as u8, to.rank() as u8))
            }
            Move::Put { to, .. } => {
                ((to.file() as u8, to.rank() as u8), (to.file() as u8, to.rank() as u8))
            }
        };
        self.last_move = Some((from_sq, to_sq));

        self.pos.play_unchecked(chosen_move);
        self.state = SimState::Animating(anim);
    }

    fn reset_game(&mut self) {
        self.pos = Chess::default();
        self.current_move_index = 0;
        self.last_move = None;

        self.playlist_cursor += 1;
        if self.playlist_cursor >= self.playlist.len() {
            let last_game = self.playlist[self.playlist.len() - 1];
            let mut rng = rand::rng();
            self.playlist.shuffle(&mut rng);
            if self.playlist.len() > 1 && self.playlist[0] == last_game {
                self.playlist.swap(0, 1);
            }
            self.playlist_cursor = 0;
            println!("--- Playlist de partidas completada. Nueva mezcla aleatoria ---");
        }

        if self.rotate_theme {
            self.board_style = self.board_style.next();
            self.piece_style = self.piece_style.next();
        }

        let next_idx = self.playlist[self.playlist_cursor];
        let next_game = &self.games[next_idx];
        println!(
            "Nueva partida [{}/{}]: '{} vs {}' - {} ({}) (Tablero: {:?}, Piezas: {:?})",
            self.playlist_cursor + 1,
            self.playlist.len(),
            next_game.white,
            next_game.black,
            next_game.event,
            next_game.year,
            self.board_style,
            self.piece_style
        );

        self.state = SimState::Idle {
            next_move_at: Instant::now() + Duration::from_millis(500),
        };
    }

    pub fn current_game(&self) -> &GameData {
        &self.games[self.playlist[self.playlist_cursor]]
    }

    pub fn current_move_num(&self) -> usize {
        (self.current_move_index / 2) + 1
    }

    pub fn total_moves(&self) -> usize {
        (self.current_game().moves.len() + 1) / 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_castling_animation() {
        let mut sim = ChessSimulator::new(260, 500, BoardStyle::Tournament, PieceStyle::Neo, false);
        // Forzar a la Opera Game (índice donde esté en la playlist)
        let opera_idx = sim.games.iter().position(|g| g.event.contains("Opera Game")).unwrap();
        sim.playlist_cursor = sim.playlist.iter().position(|&idx| idx == opera_idx).unwrap();

        // 22 jugadas previas hasta O-O-O
        for _ in 0..22 {
            sim.perform_next_move(Instant::now());
        }
        // Jugada 23: O-O-O (Enroque largo de blancas)
        sim.perform_next_move(Instant::now());
        if let SimState::Animating(ref anim) = sim.state {
            // Rey: e1 (col 4, fila 0) -> c1 (col 2, fila 0)
            assert_eq!(anim.primary.from, (4, 0));
            assert_eq!(anim.primary.to, (2, 0));
            assert_eq!(anim.primary.piece.role, Role::King);

            // Torre: a1 (col 0, fila 0) -> d1 (col 3, fila 0)
            assert!(anim.secondary.is_some(), "Debe existir animación secundaria para la torre");
            let sec = anim.secondary.as_ref().unwrap();
            assert_eq!(sec.from, (0, 0));
            assert_eq!(sec.to, (3, 0));
            assert_eq!(sec.piece.role, Role::Rook);
        } else {
            panic!("Expected Animating state");
        }
    }

    #[test]
    fn test_render_snapshots() {
        use crate::assets::AssetManager;
        use crate::renderer::render_frame;
        use tiny_skia::{Color as SkColor, Pixmap};

        let assets = AssetManager::load_embedded();
        let mut sim = ChessSimulator::new(260, 500, BoardStyle::Tournament, PieceStyle::Neo, false);
        let opera_idx = sim.games.iter().position(|g| g.event.contains("Opera Game")).unwrap();
        sim.playlist_cursor = sim.playlist.iter().position(|&idx| idx == opera_idx).unwrap();

        // 1. Jugada 1: e4 terminada (resaltado amarillo de e2 y e4)
        let t0 = Instant::now();
        sim.perform_next_move(t0);
        sim.update(t0 + Duration::from_millis(300));
        assert!(matches!(sim.state, SimState::Idle { .. }));
        assert_eq!(sim.last_move, Some(((4, 1), (4, 3))));

        let mut pixmap = Pixmap::new(1920, 1080).unwrap();
        let bg = SkColor::from_rgba8(20, 22, 34, 255);
        render_frame(&mut pixmap.as_mut(), &assets, &sim, bg, 640.0, t0);
        pixmap.save_png("/tmp/chess_test_highlight.png").unwrap();

        // 2. Jugada 2: e5 animando en pleno vuelo (progress = 0.5)
        let t1 = Instant::now();
        sim.perform_next_move(t1);
        let mut pixmap_anim = Pixmap::new(1920, 1080).unwrap();
        render_frame(&mut pixmap_anim.as_mut(), &assets, &sim, bg, 640.0, t1 + Duration::from_millis(130));
        pixmap_anim.save_png("/tmp/chess_test_mid_anim.png").unwrap();

        // Completar hasta jugada 7 (dxe5 - captura!)
        sim.update(t1 + Duration::from_millis(300));
        for _ in 2..6 {
            let t = Instant::now();
            sim.perform_next_move(t);
            sim.update(t + Duration::from_millis(300));
        }

        // Jugada 7: 4. dxe5 (Captura con anticipación e impacto físico)
        let t_cap = Instant::now();
        sim.perform_next_move(t_cap);

        // Frame A: Anticipación (p = 0.10, retroceso de carga)
        let mut pix_windup = Pixmap::new(1920, 1080).unwrap();
        render_frame(&mut pix_windup.as_mut(), &assets, &sim, bg, 640.0, t_cap + Duration::from_millis(26));
        pix_windup.save_png("/tmp/chess_capture_windup.png").unwrap();

        // Frame B: Embestida aérea con estiramiento (p = 0.50)
        let mut pix_strike = Pixmap::new(1920, 1080).unwrap();
        render_frame(&mut pix_strike.as_mut(), &assets, &sim, bg, 640.0, t_cap + Duration::from_millis(130));
        pix_strike.save_png("/tmp/chess_capture_strike.png").unwrap();

        // Frame C: Impacto contundente y knockback (p = 0.85)
        let mut pix_impact = Pixmap::new(1920, 1080).unwrap();
        render_frame(&mut pix_impact.as_mut(), &assets, &sim, bg, 640.0, t_cap + Duration::from_millis(221));
        pix_impact.save_png("/tmp/chess_capture_impact.png").unwrap();

        // 3. Buscar una posición de jaque y verificar brillo carmesí
        let mut check_found = false;
        for _ in 0..40 {
            let t = Instant::now();
            sim.perform_next_move(t);
            sim.update(t + Duration::from_millis(300));
            if sim.pos.is_check() {
                check_found = true;
                let mut pixmap_check = Pixmap::new(1920, 1080).unwrap();
                render_frame(&mut pixmap_check.as_mut(), &assets, &sim, bg, 640.0, t);
                pixmap_check.save_png("/tmp/chess_test_check.png").unwrap();
                break;
            }
        }
        assert!(check_found, "Debe encontrarse una jugada con jaque en Opera Game");
    }
}
