use std::time::{Duration, Instant};
use rand::seq::SliceRandom;
use rand::seq::IndexedRandom;
use shakmaty::san::San;
use shakmaty::{Chess, Move, Piece, Position};
use crate::assets::BoardTheme;
use crate::games::{get_master_games, GameData};

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

    /// Posición X/Y con aceleración y desaceleración fluida
    pub fn current_pos(&self, now: Instant) -> (f32, f32) {
        let p = self.progress(now);
        // Suavizado cubic smoothstep: 3p^2 - 2p^3
        let t = p * p * (3.0 - 2.0 * p);

        let fx = self.from.0 as f32;
        let fy = self.from.1 as f32;
        let tx = self.to.0 as f32;
        let ty = self.to.1 as f32;
        (fx + (tx - fx) * t, fy + (ty - fy) * t)
    }

    /// Altura del salto parabólico (arco 3D)
    pub fn jump_height(&self, now: Instant) -> f32 {
        let p = self.progress(now);
        (p * std::f32::consts::PI).sin()
    }

    /// Física Jelly: Rebote elástico amortiguado al tocar tierra
    pub fn jelly_scale(&self, now: Instant) -> (f32, f32) {
        let p = self.progress(now);
        if p < 0.20 {
            // Despegue elástico
            let t = p / 0.20;
            let stretch = (t * std::f32::consts::PI).sin() * 0.12;
            (1.0 - stretch * 0.5, 1.0 + stretch)
        } else if p < 0.70 {
            // Vuelo en el aire
            (1.0, 1.0)
        } else {
            // Aterrizaje: squish y wobble amortiguado
            let t = (p - 0.70) / 0.30;
            let decay = (-4.0 * t).exp();
            let wave = (t * std::f32::consts::PI * 3.5).sin();
            let wobble = decay * wave * 0.24;

            (1.0 + wobble, (1.0 - wobble * 0.85).max(0.6))
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
    pub theme: BoardTheme,
    pub move_duration: Duration,
    pub pause_between_moves: Duration,
    pub pause_after_game: Duration,

    games: Vec<GameData>,
    playlist: Vec<usize>,
    playlist_cursor: usize,
    current_move_index: usize,
}

impl ChessSimulator {
    pub fn new(move_ms: u64, delay_ms: u64) -> Self {
        let games = get_master_games();
        let mut playlist: Vec<usize> = (0..games.len()).collect();
        let mut rng = rand::rng();
        playlist.shuffle(&mut rng);

        let initial_theme = match rand::random::<u8>() % 3 {
            0 => BoardTheme::PinkBlue,
            1 => BoardTheme::LimeOchre,
            _ => BoardTheme::PurpleCyan,
        };

        println!("Cargadas {} partidas maestras históricas.", games.len());
        println!("Partida inicial [1/{}]: '{}'", games.len(), games[playlist[0]].name);

        Self {
            pos: Chess::default(),
            state: SimState::Idle {
                next_move_at: Instant::now() + Duration::from_millis(400),
            },
            theme: initial_theme,
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

        // Si se terminaron las jugadas del PGN o fue jaque mate
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

        self.pos.play_unchecked(chosen_move);
        self.state = SimState::Animating(anim);
    }

    fn reset_game(&mut self) {
        self.pos = Chess::default();
        self.current_move_index = 0;

        // Avanzar en la playlist aleatoria
        self.playlist_cursor += 1;
        if self.playlist_cursor >= self.playlist.len() {
            let last_game = self.playlist[self.playlist.len() - 1];
            let mut rng = rand::rng();
            self.playlist.shuffle(&mut rng);
            // Evitar que la primera de la nueva tanda sea idéntica a la última
            if self.playlist.len() > 1 && self.playlist[0] == last_game {
                self.playlist.swap(0, 1);
            }
            self.playlist_cursor = 0;
            println!("--- Playlist de partidas completada. Nueva mezcla aleatoria ---");
        }

        // Cambiar paleta de colores aleatoriamente
        self.theme = self.theme.next();

        let next_idx = self.playlist[self.playlist_cursor];
        println!(
            "Nueva partida [{}/{}]: '{}' (Tema: {:?})",
            self.playlist_cursor + 1,
            self.playlist.len(),
            self.games[next_idx].name,
            self.theme
        );

        self.state = SimState::Idle {
            next_move_at: Instant::now() + Duration::from_millis(500),
        };
    }
}
