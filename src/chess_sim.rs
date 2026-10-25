use std::time::{Duration, Instant};
use rand::seq::{IndexedRandom, SliceRandom};
use shakmaty::san::San;
use shakmaty::{Chess, Move, Piece, Position, Role};
use crate::assets::{BoardStyle, PieceStyle};
use crate::games::{get_master_games, GameData};

#[derive(Clone, Debug)]
pub struct SinglePieceAnim {
    pub from: (u8, u8), // (file 0..7, rank 0..7)
    pub to: (u8, u8),
    pub piece: Piece,
}

impl SinglePieceAnim {
    pub fn current_pos(&self, progress: f32) -> (f32, f32) {
        let t = progress * progress * (3.0 - 2.0 * progress);
        let fx = self.from.0 as f32;
        let fy = self.from.1 as f32;
        let tx = self.to.0 as f32;
        let ty = self.to.1 as f32;
        (fx + (tx - fx) * t, fy + (ty - fy) * t)
    }

    pub fn jump_height(&self, progress: f32) -> f32 {
        (progress * std::f32::consts::PI).sin()
    }

    pub fn jelly_scale(&self, progress: f32) -> (f32, f32) {
        let p = progress;
        if p < 0.25 {
            let t = p / 0.25;
            let stretch = (t * std::f32::consts::PI).sin() * 0.08;
            (1.0 - stretch * 0.5, 1.0 + stretch)
        } else if p < 0.70 {
            (1.0, 1.0)
        } else {
            let t = (p - 0.70) / 0.30;
            let decay = (-5.0 * t).exp();
            let wave = (t * std::f32::consts::PI * 3.0).sin();
            let wobble = decay * wave * 0.12;
            (1.0 + wobble, (1.0 - wobble * 0.8).max(0.75))
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

        println!("Cargadas {} partidas maestras históricas.", games.len());
        println!("Tablero: {:?} | Piezas: {:?}", board_style, piece_style);
        println!("Partida inicial [1/{}]: '{}'", games.len(), games[playlist[0]].name);

        Self {
            pos: Chess::default(),
            state: SimState::Idle {
                next_move_at: Instant::now() + Duration::from_millis(400),
            },
            board_style,
            piece_style,
            rotate_theme,
            move_duration: Duration::from_millis(move_ms),
            pause_between_moves: Duration::from_millis(delay_ms),
            pause_after_game: Duration::from_millis(2200),
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
                    self.state = SimState::Idle {
                        next_move_at: now + self.pause_between_moves,
                    };
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
        if self.pos.is_game_over() {
            println!("¡Fin de la partida! Próxima partida en {:.1}s...", self.pause_after_game.as_secs_f32());
            self.state = SimState::GameOver {
                restart_at: now + self.pause_after_game,
            };
            return;
        }

        let current_game_idx = self.playlist[self.playlist_cursor];
        let current_game = &self.games[current_game_idx];

        let next_m = if self.current_move_index < current_game.moves.len() {
            let san_str = current_game.moves[self.current_move_index];
            if let Ok(san) = san_str.parse::<San>() {
                if let Ok(m) = san.to_move(&self.pos) {
                    self.current_move_index += 1;
                    Some(m)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let chosen_move = match next_m {
            Some(m) => m,
            None => {
                let legal = self.pos.legal_moves();
                if legal.is_empty() {
                    self.state = SimState::GameOver {
                        restart_at: now + self.pause_after_game,
                    };
                    return;
                }
                let mut rng = rand::rng();
                let captures: Vec<Move> = legal.iter().filter(|m| m.is_capture()).cloned().collect();
                if !captures.is_empty() && rand::random::<f32>() < 0.65 {
                    *captures.choose(&mut rng).unwrap()
                } else {
                    *legal.choose(&mut rng).unwrap()
                }
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
                };

                let rook_anim = SinglePieceAnim {
                    from: (rook.file() as u8, rank),
                    to: (rook_to_file, rank),
                    piece: Piece { color, role: Role::Rook },
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

                MoveAnim {
                    primary: SinglePieceAnim {
                        from: (from.file() as u8, from.rank() as u8),
                        to: (to.file() as u8, to.rank() as u8),
                        piece,
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
                    },
                    secondary: None,
                    captured: None,
                    start: now,
                    duration: self.move_duration,
                }
            }
        };

        self.pos.play_unchecked(chosen_move);
        self.state = SimState::Animating(anim);
    }

    fn reset_game(&mut self) {
        self.pos = Chess::default();
        self.current_move_index = 0;

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
        println!(
            "Nueva partida [{}/{}]: '{}' (Tablero: {:?}, Piezas: {:?})",
            self.playlist_cursor + 1,
            self.playlist.len(),
            self.games[next_idx].name,
            self.board_style,
            self.piece_style
        );

        self.state = SimState::Idle {
            next_move_at: Instant::now() + Duration::from_millis(500),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_castling_animation() {
        let mut sim = ChessSimulator::new(260, 500, BoardStyle::Blue, PieceStyle::Neo, false);
        // Forzar a la Opera Game (índice donde esté en la playlist)
        let opera_idx = sim.games.iter().position(|g| g.name.contains("Opera Game")).unwrap();
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
}
