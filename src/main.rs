mod assets;
mod chess_sim;
mod renderer;

use std::num::NonZeroU32;
use std::time::{Duration, Instant};

use assets::AssetManager;
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
use tiny_skia::PixmapMut;
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

        // Renderizar con tiny-skia
        if let Some(mut pixmap) = PixmapMut::from_bytes(canvas, width, height) {
            renderer::render_frame(&mut pixmap, &self.assets, &self.sim, Instant::now());
        }

        if let Some(ref surface_mode) = self.surface_mode {
            let surface = surface_mode.wl_surface();
            surface.damage_buffer(0, 0, width as i32, height as i32);
            surface.frame(&self.qh, FrameCallbackData(surface.clone()));
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

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
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

// Window handler (para modo ventana)
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

// Layer shell handler (para modo fondo de pantalla)
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

fn main() {
    let mut args = pico_args::Arguments::from_env();
    let is_window_mode = args.contains(["-w", "--window"]);

    println!("Iniciando Chess Wallpaper (Rust)");
    println!("Modo: {}", if is_window_mode { "Ventana de prueba (-w)" } else { "Fondo de escritorio Layer-Shell" });

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
    let sim = ChessSimulator::new();

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
        qh: qh.clone(),
    };

    // Configurar temporizador reactivo ultra eficiente
    let timer = Timer::from_duration(Duration::from_millis(50));
    loop_handle
        .insert_source(timer, move |_, _, state: &mut AppState| {
            let now = Instant::now();
            let needs_redraw = state.sim.update(now);
            if needs_redraw {
                state.draw();
            }

            // Programar el siguiente despertar con precisión quirúrgica
            match state.sim.state {
                SimState::Animating(_) => {
                    // Animación activa: 60 FPS (~16ms)
                    TimeoutAction::ToDuration(Duration::from_millis(16))
                }
                SimState::Idle { next_move_at } => {
                    // Reposo absoluto: dormir hasta el próximo movimiento
                    let wait = next_move_at.saturating_duration_since(now).max(Duration::from_millis(15));
                    TimeoutAction::ToDuration(wait)
                }
                SimState::GameOver { restart_at } => {
                    // Partida terminada: dormir hasta el reinicio
                    let wait = restart_at.saturating_duration_since(now).max(Duration::from_millis(15));
                    TimeoutAction::ToDuration(wait)
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
