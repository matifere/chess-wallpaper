mod assets;
mod chess_sim;
mod engine;
mod games;
mod occlusion;
mod pgn;
mod renderer;

use std::num::NonZeroU32;
use std::time::{Duration, Instant};

use assets::{AssetManager, BoardStyle, PieceStyle};
use chess_sim::{ChessSimulator, SimState};
use occlusion::OcclusionDetector;

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, FrameCallbackData},
    delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers},
        pointer::{PointerEvent, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        xdg::{
            window::{Window, WindowConfigure, WindowDecorations, WindowHandler},
            XdgShell,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use smithay_client_toolkit::reexports::calloop::{
    ping::make_ping,
    timer::{TimeoutAction, Timer},
    EventLoop,
};
use smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource;
use tiny_skia::{Color, PixmapMut};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};

enum SurfaceMode {
    Window(Window),
    Layer(LayerSurface),
}

impl SurfaceMode {
    fn wl_surface(&self) -> &wl_surface::WlSurface {
        match self {
            SurfaceMode::Window(w) => w.wl_surface(),
            SurfaceMode::Layer(l) => l.wl_surface(),
        }
    }

    fn commit(&self) {
        match self {
            SurfaceMode::Window(w) => w.commit(),
            SurfaceMode::Layer(l) => l.commit(),
        }
    }
}

pub struct AppState {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,
    pool: SlotPool,

    surface_mode: Option<SurfaceMode>,
    width: u32,
    height: u32,
    first_configure: bool,
    exit: bool,

    assets: AssetManager,
    sim: ChessSimulator,
    bg_color: Color,
    board_size: f32,
    occlusion: OcclusionDetector,
    qh: QueueHandle<AppState>,
}

impl AppState {
    pub fn draw(&mut self) {
        if self.width == 0 || self.height == 0 {
            return;
        }

        let width = self.width;
        let height = self.height;
        let stride = (width * 4) as i32;

        let (buffer, canvas) = match self.pool.create_buffer(
            width as i32,
            height as i32,
            stride,
            wl_shm::Format::Argb8888,
        ) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error creating buffer: {e}");
                return;
            }
        };

        // Renderizar frame con tiny-skia (formato nativo RGBA)
        if let Some(mut pixmap) = PixmapMut::from_bytes(canvas, width, height) {
            renderer::render_frame(
                &mut pixmap,
                &self.assets,
                &self.sim,
                self.bg_color,
                self.board_size,
                Instant::now(),
            );
        }

        // ¡IMPORTANTE! tiny-skia escribe en formato RGBA (bytes: R, G, B, A).
        // Sin embargo, wl_shm::Format::Argb8888 en arquitecturas little-endian (x86_64 / arm64)
        // almacena los componentes en memoria como B, G, R, A.
        // Sin este swap, los canales Rojo y Azul quedan invertidos en el compositor Wayland (Niri/Hyprland/Sway),
        // haciendo que el azul se vea naranja/marrón y el rojo se vea azul.
        for pixel in canvas.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }

        if let Some(ref surface_mode) = self.surface_mode {
            let surface = surface_mode.wl_surface();
            surface.damage_buffer(0, 0, width as i32, height as i32);

            // Sincronización VSync por hardware: solicitar frame callback solo mientras anima
            if matches!(self.sim.state, SimState::Animating(_)) {
                surface.frame(&self.qh, FrameCallbackData(surface.clone()));
            }

            if let Err(e) = buffer.attach_to(surface) {
                eprintln!("Error attaching buffer: {e}");
            }
            surface_mode.commit();
        }
    }
}

// SCTK Trait Implementations
impl CompositorHandler for AppState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    // Callback de frame sincronizado con el monitor (VSync)
    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        if self.occlusion.is_occluded() {
            return;
        }

        if matches!(self.sim.state, SimState::Animating(_)) {
            let now = Instant::now();
            self.sim.update(now);
            self.draw();
        }
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for AppState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl ShmHandler for AppState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl SeatHandler for AppState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn new_capability(&mut self, _: &Connection, qh: &QueueHandle<Self>, seat: wl_seat::WlSeat, cap: Capability) {
        if cap == Capability::Keyboard {
            let _ = self.seat_state.get_keyboard(qh, &seat, None);
        }
    }
    fn remove_capability(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat, _: Capability) {}
}

impl KeyboardHandler for AppState {
    fn enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32, _: &[u32], _: &[Keysym]) {}
    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32) {}
    fn press_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, event: KeyEvent) {
        if event.keysym == Keysym::Escape || event.keysym == Keysym::q {
            self.exit = true;
        }
    }
    fn repeat_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: KeyEvent) {}
    fn release_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: KeyEvent) {}
    fn update_modifiers(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: Modifiers, _: RawModifiers, _: u32) {}
}

impl PointerHandler for AppState {
    fn pointer_frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_pointer::WlPointer, _: &[PointerEvent]) {}
}

impl WindowHandler for AppState {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &Window) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _window: &Window,
        configure: WindowConfigure,
        _: u32,
    ) {
        let (w, h) = configure.new_size;
        self.width = w.map(NonZeroU32::get).unwrap_or(680);
        self.height = h.map(NonZeroU32::get).unwrap_or(680);

        if self.first_configure {
            self.first_configure = false;
        }
        self.draw();
    }
}

impl LayerShellHandler for AppState {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _: u32,
    ) {
        self.width = NonZeroU32::new(configure.new_size.0).map_or(1920, NonZeroU32::get);
        self.height = NonZeroU32::new(configure.new_size.1).map_or(1080, NonZeroU32::get);

        if self.first_configure {
            self.first_configure = false;
        }
        self.draw();
    }
}

delegate_registry!(AppState);

impl ProvidesRegistryState for AppState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

smithay_client_toolkit::delegate_dispatch2!(AppState);

fn parse_hex_color(s: &str) -> Option<Color> {
    let s = s.trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Color::from_rgba8(r, g, b, 255))
    } else if s.len() == 8 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        let a = u8::from_str_radix(&s[6..8], 16).ok()?;
        Some(Color::from_rgba8(r, g, b, a))
    } else {
        None
    }
}

fn main() {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        println!("Chess Wallpaper - Fondo de escritorio animado de ajedrez (Wayland)");
        println!();
        println!("USO:");
        println!("    chess-wallpaper [OPCIONES]");
        println!();
        println!("OPCIONES:");
        println!("    -w, --window             Ejecutar en ventana de prueba interactiva");
        println!("    -t, --theme <TEMA>       Tema visual (tournament, wood, blue, neon, icy, classic, etc.)");
        println!("        --board <TABLERO>    Estilo específico de tablero (tournament, walnut, blue, neon, etc.)");
        println!("        --pieces <PIEZAS>    Estilo específico de piezas (neo, wood, glass, 8bit, etc.)");
        println!("        --rotate             Rotar tema tablero/piezas aleatoriamente entre partidas");
        println!("    -b, --board-size <PX>    Tamaño del tablero en píxeles (default: 640)");
        println!("        --bg <HEX>           Color de fondo en hexadecimal (ej. #141622)");
        println!("    -s, --speed <MS>         Duración de animación de cada jugada en ms (default: 260)");
        println!("    -d, --delay <MS>         Pausa entre jugadas en ms (default: 500)");
        println!("    -f, --fast               Modo rápido (speed: 200ms, delay: 320ms)");
        println!("        --pgn <ARCHIVO>      Cargar archivo PGN con partidas del usuario");
        println!("        --pgn-dir <DIR>      Cargar directorio de archivos .pgn");
        println!("        --only-custom        Reproducir exclusivamente partidas personalizadas del usuario");
        println!("        --no-auto-pause      Desactivar pausa automática cuando la pantalla está ocupada por ventanas");
        println!("        --layer <CAPA>       Capa Wayland: bottom (default, visible sobre fondo) o background");
        println!("        --snapshot <PATH>    Generar captura PNG y salir");
        println!("    -h, --help               Mostrar esta ayuda");
        println!();
        println!("DIRECTORIO PGN POR DEFECTO:");
        println!("    ~/.config/chess-wallpaper/games/*.pgn");
        return;
    }

    let pgn_file: Option<String> = args.opt_value_from_str("--pgn").unwrap_or(None);
    let pgn_dir: Option<String> = args.opt_value_from_str("--pgn-dir").unwrap_or(None);
    let only_custom = args.contains("--only-custom") || args.contains("--custom-only");

    let custom_games = pgn::load_all_user_games(pgn_file.as_deref(), pgn_dir.as_deref());
    let games = if only_custom {
        if custom_games.is_empty() {
            eprintln!("Aviso: Se especificó --only-custom pero no se encontraron partidas personalizadas.");
            eprintln!("Cargando las 29 partidas maestras históricas por defecto.");
            games::get_master_games()
        } else {
            println!("Usando exclusivamente {} partidas de usuario (--only-custom)", custom_games.len());
            custom_games
        }
    } else if !custom_games.is_empty() {
        let mut all = custom_games;
        let master = games::get_master_games();
        println!("Combinando {} partidas de usuario con {} partidas maestras.", all.len(), master.len());
        all.extend(master);
        all
    } else {
        games::get_master_games()
    };

    let theme_str: Option<String> = args.opt_value_from_str(["-t", "--theme"]).unwrap_or(None);
    let (mut selected_board, mut selected_pieces) = match theme_str
        .as_deref()
        .unwrap_or("tournament")
        .to_lowercase()
        .as_str()
    {
        "tournament" | "torneo" => (BoardStyle::Tournament, PieceStyle::Neo),
        "blue" | "azul" => (BoardStyle::Blue, PieceStyle::Neo),
        "green" | "verde" => (BoardStyle::Green, PieceStyle::Neo),
        "wood" | "walnut" | "madera" => (BoardStyle::Walnut, PieceStyle::Wood),
        "glass" | "vidrio" => (BoardStyle::Glass, PieceStyle::Glass),
        "icy" | "icy_sea" | "glaciar" | "hielo" => (BoardStyle::IcySea, PieceStyle::IcySea),
        "neon" => (BoardStyle::Neon, PieceStyle::Neon),
        "classic" | "clasico" => (BoardStyle::Tournament, PieceStyle::Classic),
        "dark_wood" => (BoardStyle::DarkWood, PieceStyle::Wood),
        "marble" | "marmol" => (BoardStyle::Marble, PieceStyle::Classic),
        "bubblegum" | "rosa" | "pink" => (BoardStyle::Bubblegum, PieceStyle::Bubblegum),
        "8_bit" | "8bit" | "retro" => (BoardStyle::EightBit, PieceStyle::EightBit),
        _ => (BoardStyle::Tournament, PieceStyle::Neo), // Por defecto: Tablero de Torneo + Piezas Neo
    };

    let board_str: Option<String> = args.opt_value_from_str("--board").unwrap_or(None);
    if let Some(b) = board_str.as_deref().and_then(BoardStyle::from_str_opt) {
        selected_board = b;
    }

    let piece_str: Option<String> = args.opt_value_from_str("--pieces").unwrap_or(None);
    if let Some(p) = piece_str.as_deref().and_then(PieceStyle::from_str_opt) {
        selected_pieces = p;
    }

    let rotate_theme = args.contains("--rotate");

    let bg_str: Option<String> = args
        .opt_value_from_str("--bg")
        .unwrap_or(None)
        .or_else(|| args.opt_value_from_str("--background").unwrap_or(None));
    let bg_color = bg_str
        .as_deref()
        .and_then(parse_hex_color)
        .unwrap_or_else(|| Color::from_rgba8(20, 22, 34, 255)); // #141622 (fondo noche azul sobrio)

    let board_size: f32 = args
        .opt_value_from_str(["-b", "--board-size"])
        .unwrap_or(None)
        .unwrap_or(640.0); // Tamaño óptimo de 640px para 1080p (80px por casilla)

    let layer_str: Option<String> = args.opt_value_from_str("--layer").unwrap_or(None);
    let chosen_layer = match layer_str.as_deref().unwrap_or("bottom") {
        "background" | "bg" => Layer::Background,
        _ => Layer::Bottom,
    };

    if let Ok(Some(path)) = args.opt_value_from_str::<_, String>("--snapshot") {
        let assets = AssetManager::load_embedded();
        let sim = ChessSimulator::new(260, 500, selected_board, selected_pieces, false, games.clone());
        let mut pixmap = tiny_skia::Pixmap::new(1920, 1080).unwrap();
        renderer::render_frame(&mut pixmap.as_mut(), &assets, &sim, bg_color, board_size, Instant::now());
        pixmap.save_png(&path).expect("Failed to save snapshot");
        println!("Snapshot guardado en {path}");
        return;
    }

    let is_window_mode = args.contains(["-w", "--window"]);
    let is_fast = args.contains(["-f", "--fast"]);
    let no_auto_pause = args.contains("--no-auto-pause");
    let auto_pause_enabled = !is_window_mode && !no_auto_pause;

    let (default_speed, default_delay) = if is_fast {
        (200, 320)
    } else {
        (260, 500)
    };

    let move_speed: u64 = args.opt_value_from_str(["-s", "--speed"]).unwrap_or(None).unwrap_or(default_speed);
    let move_delay: u64 = args.opt_value_from_str(["-d", "--delay"]).unwrap_or(None).unwrap_or(default_delay);

    println!("Iniciando Chess Wallpaper (Rust - Chess.com Assets)");
    println!("Modo: {}", if is_window_mode { "Ventana de prueba (-w)" } else { "Fondo de escritorio Layer-Shell" });
    println!("Tablero: {:?} | Piezas: {:?} (Rotar: {}) | Tamaño: {board_size}px", selected_board, selected_pieces, rotate_theme);
    println!("Color de fondo: rgba({}, {}, {}, {})", bg_color.red(), bg_color.green(), bg_color.blue(), bg_color.alpha());
    println!("Velocidad de salto: {move_speed} ms | Pausa entre jugadas: {move_delay} ms");
    println!("Auto-pausa por oclusión: {}", if auto_pause_enabled { "Activada (0% CPU cuando hay ventanas abiertas)" } else { "Desactivada" });

    let conn = Connection::connect_to_env().expect("No se pudo conectar al compositor Wayland");
    let (globals, event_queue) = registry_queue_init(&conn).expect("Error al inicializar registro Wayland");
    let qh: QueueHandle<AppState> = event_queue.handle();

    let mut event_loop: EventLoop<AppState> = EventLoop::try_new().expect("Error al crear calloop EventLoop");
    let loop_handle = event_loop.handle();
    WaylandSource::new(conn.clone(), event_queue)
        .insert(loop_handle.clone())
        .expect("Error al insertar WaylandSource en EventLoop");

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor no disponible");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm no disponible");
    let pool = SlotPool::new(1920 * 1080 * 4, &shm).expect("Error al crear SlotPool");

    let surface = compositor.create_surface(&qh);

    let (surface_mode, initial_w, initial_h) = if is_window_mode {
        let xdg_shell = XdgShell::bind(&globals, &qh).expect("xdg_shell no disponible");
        let window = xdg_shell.create_window(surface, WindowDecorations::ServerDefault, &qh);
        window.set_title("JellyChess Simulation");
        window.set_app_id("jellychess-wallpaper");
        window.set_min_size(Some((300, 300)));
        window.commit();
        (SurfaceMode::Window(window), 680, 680)
    } else {
        let layer_shell = LayerShell::bind(&globals, &qh).expect("wlr_layer_shell no disponible");
        let layer_surface = layer_shell.create_layer_surface(
            &qh,
            surface,
            chosen_layer,
            Some("chess_wallpaper"),
            None,
        );
        layer_surface.set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
        layer_surface.set_exclusive_zone(-1); // -1: Ignorar zonas de exclusión (barras/paneles) y ocupar el 100% de la pantalla
        layer_surface.set_margin(0, 0, 0, 0);
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer_surface.commit();
        (SurfaceMode::Layer(layer_surface), 1920, 1080)
    };

    let (ping, ping_source) = make_ping().expect("Error al crear calloop ping");
    loop_handle
        .insert_source(ping_source, |_, _, state: &mut AppState| {
            if !state.occlusion.is_occluded() {
                state.draw();
            }
        })
        .expect("Error al registrar calloop ping_source");

    let occlusion = OcclusionDetector::new(auto_pause_enabled, None, Some(ping));

    let assets = AssetManager::load_embedded();
    let sim = ChessSimulator::new(move_speed, move_delay, selected_board, selected_pieces, rotate_theme, games);

    let mut app_state = AppState {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        pool,
        surface_mode: Some(surface_mode),
        width: initial_w,
        height: initial_h,
        first_configure: true,
        exit: false,
        assets,
        sim,
        bg_color,
        board_size,
        occlusion,
        qh: qh.clone(),
    };

    // Temporizador reactivo: solo gestiona los descansos entre jugadas y reinicios
    let timer = Timer::from_duration(Duration::from_millis(50));
    loop_handle
        .insert_source(timer, move |_, _, state: &mut AppState| {
            // Pausa reactiva: si la pantalla está tapada por ventanas, dormir y ahorrar 100% de CPU
            if state.occlusion.is_occluded() {
                return TimeoutAction::ToDuration(Duration::from_millis(300));
            }

            let now = Instant::now();
            match state.sim.state {
                SimState::Idle { next_move_at } => {
                    if now >= next_move_at {
                        state.sim.perform_next_move(now);
                        state.draw(); // Dispara el primer frame y engancha CompositorHandler::frame (VSync)
                        TimeoutAction::ToDuration(state.sim.move_duration + Duration::from_millis(40))
                    } else {
                        let wait = next_move_at.saturating_duration_since(now).max(Duration::from_millis(15));
                        TimeoutAction::ToDuration(wait)
                    }
                }
                SimState::GameOver { restart_at } => {
                    if now >= restart_at {
                        state.sim.update(now);
                        state.draw();
                    }
                    let wait = restart_at.saturating_duration_since(now).max(Duration::from_millis(20));
                    TimeoutAction::ToDuration(wait)
                }
                SimState::Animating(_) => {
                    TimeoutAction::ToDuration(Duration::from_millis(80))
                }
            }
        })
        .expect("Error al insertar Timer en event loop");

    println!("Loop de eventos iniciado. Presiona 'q' o 'Esc' en modo ventana para cerrar.");
    while !app_state.exit {
        event_loop
            .dispatch(None, &mut app_state)
            .expect("Error en dispatch de calloop");
    }

    println!("Chess Wallpaper finalizado.");
}
