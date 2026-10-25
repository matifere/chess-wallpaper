mod assets;
mod chess_sim;
mod games;
mod renderer;

use std::num::NonZeroU32;
use std::time::{Duration, Instant};

use assets::{AssetManager, BoardTheme};
use chess_sim::{ChessSimulator, SimState};

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

        // Renderizar frame con tiny-skia
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

    let theme_str: Option<String> = args.opt_value_from_str(["-t", "--theme"]).unwrap_or(None);
    let selected_theme = theme_str
        .as_deref()
        .and_then(BoardTheme::from_str_opt)
        .unwrap_or(BoardTheme::Blue); // Por defecto: Blue Glaciar (elegido por el usuario)

    let rotate_theme = args.contains("--rotate");

    let bg_str: Option<String> = args.opt_value_from_str(["--bg", "--background"]).unwrap_or(None);
    let bg_color = bg_str
        .as_deref()
        .and_then(parse_hex_color)
        .unwrap_or_else(|| Color::from_rgba8(20, 22, 34, 255)); // #141622 (fondo noche azul sobrio)

    let board_size: f32 = args
        .opt_value_from_str(["-b", "--board-size"])
        .unwrap_or(None)
        .unwrap_or(640.0); // Tamaño óptimo de 640px para 1080p (80px por casilla)

    if let Ok(Some(path)) = args.opt_value_from_str::<_, String>("--snapshot") {
        let assets = AssetManager::load_embedded();
        let sim = ChessSimulator::new(260, 500, selected_theme, false);
        let mut pixmap = tiny_skia::Pixmap::new(1920, 1080).unwrap();
        renderer::render_frame(&mut pixmap.as_mut(), &assets, &sim, bg_color, board_size, Instant::now());
        pixmap.save_png(&path).expect("Failed to save snapshot");
        println!("Snapshot guardado en {path}");
        return;
    }

    let is_window_mode = args.contains(["-w", "--window"]);
    let is_fast = args.contains(["-f", "--fast"]);

    let (default_speed, default_delay) = if is_fast {
        (200, 320)
    } else {
        (260, 500)
    };

    let move_speed: u64 = args.opt_value_from_str(["-s", "--speed"]).unwrap_or(None).unwrap_or(default_speed);
    let move_delay: u64 = args.opt_value_from_str(["-d", "--delay"]).unwrap_or(None).unwrap_or(default_delay);

    println!("Iniciando Chess Wallpaper (Rust)");
    println!("Modo: {}", if is_window_mode { "Ventana de prueba (-w)" } else { "Fondo de escritorio Layer-Shell" });
    println!("Tablero: {:?} (Fijo: {}) | Tamaño: {board_size}px", selected_theme, !rotate_theme);
    println!("Color de fondo: rgba({}, {}, {}, {})", bg_color.red(), bg_color.green(), bg_color.blue(), bg_color.alpha());
    println!("Velocidad de salto: {move_speed} ms | Pausa entre jugadas: {move_delay} ms");

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
            Layer::Background,
            Some("jellychess_background"),
            None,
        );
        layer_surface.set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
        layer_surface.set_exclusive_zone(0);
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer_surface.commit();
        (SurfaceMode::Layer(layer_surface), 1920, 1080)
    };

    let assets = AssetManager::load_embedded();
    let sim = ChessSimulator::new(move_speed, move_delay, selected_theme, rotate_theme);

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
        qh: qh.clone(),
    };

    // Temporizador reactivo: solo gestiona los descansos entre jugadas y reinicios
    let timer = Timer::from_duration(Duration::from_millis(50));
    loop_handle
        .insert_source(timer, move |_, _, state: &mut AppState| {
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
