use std::time::{Duration, Instant};
use rand::seq::IndexedRandom;
use shakmaty::san::San;
use shakmaty::{Chess, Move, Piece, Position};
use crate::assets::BoardTheme;

pub fn ease_out_back(t: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

#[derive(Clone, Debug)]
pub struct MoveAnim {
    pub from: (u8, u8), // (file 0..7, rank 0..7)
    pub to: (u8, u8),
    pub piece: Piece,
    pub captured: Option<Piece>,
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

    pub fn current_pos(&self, now: Instant) -> (f32, f32) {
        let p = self.progress(now);
        let t = ease_out_back(p);
        let fx = self.from.0 as f32;
        let fy = self.from.1 as f32;
        let tx = self.to.0 as f32;
        let ty = self.to.1 as f32;
        (fx + (tx - fx) * t, fy + (ty - fy) * t)
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
    pub theme: BoardTheme,
    pub move_duration: Duration,
    pub pause_between_moves: Duration,
    current_game_index: usize,
    current_move_index: usize,
    games: Vec<Vec<&'static str>>,
}

impl ChessSimulator {
    pub fn new() -> Self {
        let games = vec![
            // 1. The Opera Game: Morphy vs Duke of Brunswick (1858)
            vec![
                "e4", "e5", "Nf3", "d6", "d4", "Bg4", "dxe5", "Bxf3", "Qxf3", "dxe5",
                "Bc4", "Nf6", "Qb3", "Qe7", "Nc3", "c6", "Bg5", "b5", "Nxb5", "cxb5",
                "Bxb5+", "Nbd7", "O-O-O", "Rd8", "Rxd7", "Rxd7", "Rd1", "Qe6", "Bxd7+", "Nxd7",
                "Qb8+", "Nxb8", "Rd8#"
            ],
            // 2. The Immortal Game: Adolf Anderssen vs Lionel Kieseritzky (1851)
            vec![
                "e4", "e5", "f4", "exf4", "Bc4", "Qh4+", "Kf1", "b5", "Bxb5", "Nf6",
                "Nf3", "Qh6", "d3", "Nh5", "Nh4", "Qg5", "Nf5", "c6", "g4", "Nf6",
                "Rg1", "cxb5", "h4", "Qg6", "h5", "Qg5", "Qf3", "Ng8", "Bxf4", "Qf6",
                "Nc3", "Bc5", "Nd5", "Qxb2", "Bd6", "Bxg1", "e5", "Qxa1+", "Ke2", "Na6",
                "Nxg7+", "Kd8", "Qf6+", "Nxf6", "Be7#"
            ],
            // 3. The Game of the Century: Donald Byrne vs Bobby Fischer (1956)
            vec![
                "Nf3", "Nf6", "c4", "g6", "Nc3", "Bg7", "d4", "O-O", "Bf4", "d5",
                "Qb3", "dxc4", "Qxc4", "c6", "e4", "Nbd7", "Rd1", "Nb6", "Qc5", "Bg4",
                "Bg5", "Na4", "Qa3", "Nxc3", "bxc3", "Nxe4", "Bxe7", "Qb6", "Bc4", "Nxc3",
                "Bc5", "Rfe8+", "Kf1", "Be6", "Bxb6", "Bxc4+", "Kg1", "Ne2+", "Kf1", "Nxd4+",
                "Kg1", "Ne2+", "Kf1", "Nc3+", "Kg1", "axb6", "Qb4", "Ra4", "Qxb6", "Nxd1",
                "h3", "Rxa2", "Kh2", "Nxf2", "Re1", "Rxe1", "Qd8+", "Bf8", "Nxe1", "Bd5",
                "Nf3", "Ne4", "Qb8", "b5", "h4", "h5", "Ne5", "Kg7", "Kg1", "Bc5+",
                "Kf1", "Ng3+", "Ke1", "Bb4+", "Kd1", "Bb3+", "Kc1", "Ne2+", "Kb1", "Nc3+",
                "Kc1", "Rc2#"
            ]
        ];

        Self {
            pos: Chess::default(),
            state: SimState::Idle {
                next_move_at: Instant::now() + Duration::from_millis(600),
            },
            theme: BoardTheme::PinkBlue,
            move_duration: Duration::from_millis(320),
            pause_between_moves: Duration::from_millis(1600),
            current_game_index: 0,
            current_move_index: 0,
            games,
        }
    }

    pub fn update(&mut self, now: Instant) -> bool {
        match self.state {
            SimState::Animating(ref anim) => {
                if anim.progress(now) >= 1.0 {
                    // Animación terminada
                    self.state = SimState::Idle {
                        next_move_at: now + self.pause_between_moves,
                    };
                    return true;
                }
                true // todavía animando, requiere redraw
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

    fn perform_next_move(&mut self, now: Instant) {
        if self.pos.is_game_over() {
            self.state = SimState::GameOver {
                restart_at: now + Duration::from_secs(5),
            };
            return;
        }

        // 1. Intentar jugar el movimiento de la partida PGN actual
        let current_game = &self.games[self.current_game_index];
        let next_m = if self.current_move_index < current_game.len() {
            let san_str = current_game[self.current_move_index];
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

        // 2. Si terminó la partida o falló el SAN, elegir movimiento legal inteligente
        let chosen_move = match next_m {
            Some(m) => m,
            None => {
                let legal = self.pos.legal_moves();
                if legal.is_empty() {
                    self.state = SimState::GameOver {
                        restart_at: now + Duration::from_secs(5),
                    };
                    return;
                }
                // Preferir capturas o jaques
                let mut rng = rand::rng();
                let captures: Vec<Move> = legal.iter().filter(|m| m.is_capture()).cloned().collect();
                if !captures.is_empty() && rand::random::<f32>() < 0.6 {
                    *captures.choose(&mut rng).unwrap()
                } else {
                    *legal.choose(&mut rng).unwrap()
                }
            }
        };

        // Guardar datos para la animación
        let from_sq = chosen_move.from().unwrap();
        let to_sq = chosen_move.to();
        let piece = self.pos.board().piece_at(from_sq).unwrap();
        let captured = self.pos.board().piece_at(to_sq);

        let anim = MoveAnim {
            from: (from_sq.file() as u8, from_sq.rank() as u8),
            to: (to_sq.file() as u8, to_sq.rank() as u8),
            piece,
            captured,
            start: now,
            duration: self.move_duration,
        };

        // Aplicar movimiento en la posición interna
        self.pos.play_unchecked(chosen_move);

        self.state = SimState::Animating(anim);
    }

    fn reset_game(&mut self) {
        self.pos = Chess::default();
        self.current_move_index = 0;
        self.current_game_index = (self.current_game_index + 1) % self.games.len();
        self.theme = self.theme.next();
        self.state = SimState::Idle {
            next_move_at: Instant::now() + Duration::from_millis(800),
        };
    }
}
