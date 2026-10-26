# Chess Wallpaper ♟️✨

Un fondo de escritorio animado de ajedrez para **Wayland** (`zwlr_layer_shell_v1`) escrito en **Rust**, con análisis de **Stockfish 19**, insignias oficiales de revisión de partidas de **Chess.com**, barra de evaluación en vivo, captura de piezas en tiempo real y **auto-pausa reactiva con 0% de CPU** bajo el compositor **Niri**.

![Chess Wallpaper Preview](https://raw.githubusercontent.com/matifere/chess-wallpaper/main/assets/preview.png)

---

## Características Principales

- 🏆 **Reproductor de Partidas Maestras**: Incluye 29 partidas inmortales de la historia (Kasparov, Fischer, Morphy, Tal, Carlsen, Rubinstein, Capablanca, etc.) y rota automáticamente al finalizar cada una.
- 📂 **Soporte de Partidas Personalizadas (PGN)**: Carga tus propias partidas de Chess.com o Lichess guardándolas en `~/.config/chess-wallpaper/games/*.pgn` o mediante la opción `--pgn`.
- 🧠 **Motor Stockfish 19 UCI en Tiempo Real**: Análisis de jugadas en segundo plano con cálculo de ventaja posicional en peones (`+1.4`, `-0.8`) y detección de jaque mate forzado (`M1`, `M2`).
- 💎 **Insignias Oficiales de Chess.com**: Renderizado vectorial de alta definición de las insignias auténticas de Chess.com:
  - 💎 **Brillante (`!!`)**: Sacrificios tácticos ganadores (`#26c2a3`).
  - ⭐️ **Mejor jugada (`★`)**: Primera línea del motor (`#81b64c`).
  - 🔷 **Gran jugada (`!`)**: Jugadas críticas bajo presión (`#5c8bb0`).
  - 🟢 **Excelente (`✔`)**: Movimientos sólidos posicionales (`#96bc4b`).
  - 📖 **Libro (`📖`)**: Teoría de aperturas (`#a88b67`).
  - 🟡 **Imprecisión (`?!`)**, 🟠 **Error (`?`)**, 🔴 **Grave error (`??`)**, ✖ **Perdida (`✖`)**, ↷ **Forzada**.
  - Insignia en el ticker de jugadas y mini-insignia circular en la casilla de destino en el tablero.
- 📊 **Barra de Evaluación Dinámica**: Barra vertical integrada a la izquierda del marco del tablero, con animación suave, marcador central y etiqueta numérica.
- ⚔️ **Métricas en Vivo de Jugadores**:
  - Puntuación Elo de los jugadores (`Garry Kasparov (2851)`).
  - Piezas capturadas en tiempo real ordenadas por valor relativo (D, T, A, C, P).
  - Diferencial de material (`+3`, `+1`).
  - Código ECO y nombre de la apertura (`B07 · Defensa Pirc`).
- ⚡ **Auto-Pausa Reactiva (0% CPU)**:
  - Conexión al socket IPC de Niri (`NIRI_SOCKET`).
  - Se suspende de inmediato cuando las ventanas cubren el espacio de trabajo.
  - Se reanuda en **<1 ms** al cambiar a un espacio vacío o al abrir el Overview.
- 🌌 **Soporte de Telón de Fondo en Niri Overview (`Super + O`)**:
  - Compatible con `place-within-backdrop true`.
  - Cubre el 100% de la pantalla detrás del zoom-out de los espacios de trabajo.
- 🎨 **Múltiples Temas y Estilos**:
  - **Tableros**: Torneo (`tournament`), Nogal (`walnut`), Azul (`blue`), Neón (`neon`), Glaciar (`icy_sea`), Vidrio (`glass`), Chicle (`bubblegum`), 8-Bit (`8_bit`), Mármol (`marble`).
  - **Piezas**: `neo`, `wood`, `glass`, `classic`, `8_bit`, `bubblegum`.
  - Opción `--rotate` para alternar temas aleatoriamente en cada partida.
- 🚀 **Sincronización VSync por Hardware**: Animaciones a 60 FPS fluidas mediante `tiny-skia` y reposo absoluto entre jugadas.
- 🪟 **Modo Ventana de Prueba (`-w`)**: Pruébalo en una ventana interactiva antes de aplicarlo al fondo.

---

## Requisitos

- Sistema operativo **Linux** con sesión **Wayland** (optimizado para **Niri**, compatible con Hyprland, Sway, etc.).
- [Rust](https://rustup.rs/) (versión 1.75 o superior).
- Motor [Stockfish](https://stockfishchess.org/download/) (se detecta automáticamente en el sistema o en `~/.local/share/chess-wallpaper/bin/stockfish`).

---

## Instalación

1. **Clonar el repositorio**:
   ```bash
   git clone https://github.com/matifere/chess-wallpaper.git
   cd chess-wallpaper
   ```

2. **Compilar en modo release**:
   ```bash
   cargo build --release
   ```

3. **Instalar el binario**:
   ```bash
   mkdir -p ~/.local/bin
   cp target/release/chess-wallpaper ~/.local/bin/
   ```

---

## Configuración en Niri

Para disfrutar de la integración completa (fondo permanente fijo y telón de fondo durante el zoom out de `Super + O`):

### 1. Regla de capa en Niri (`~/.config/niri/modules/layer-rules.kdl`)
Añade la siguiente regla:

```kdl
layer-rule {
  match namespace="^chess_wallpaper$"
  place-within-backdrop true
}
```

### 2. Transparencia de espacios de trabajo (`~/.config/niri/config.kdl`)
Dentro del bloque `layout`, añade `background-color "transparent"`:

```kdl
layout {
  background-color "transparent"
  gaps 9
  // ... resto de tu configuración
}
```

---

## Opciones de Línea de Comandos (CLI)

```text
USO:
    chess-wallpaper [OPCIONES]

OPCIONES:
    -w, --window             Ejecutar en ventana de prueba interactiva
    -t, --theme <TEMA>       Tema visual (tournament, wood, blue, neon, icy, classic, etc.)
        --board <TABLERO>    Estilo específico de tablero
        --pieces <PIEZAS>    Estilo específico de piezas
        --rotate             Rotar estilo de tablero/piezas aleatoriamente entre partidas
    -b, --board-size <PX>    Tamaño del tablero en píxeles (default: 640)
        --bg <HEX>           Color de fondo en hexadecimal (default: #141622)
    -s, --speed <MS>         Duración de animación de cada jugada en ms (default: 260)
    -d, --delay <MS>         Pausa entre jugadas en ms (default: 500)
    -f, --fast               Modo rápido (speed: 200ms, delay: 320ms)
        --pgn <ARCHIVO>      Cargar archivo PGN específico con partidas
        --pgn-dir <DIR>      Cargar directorio de archivos .pgn
        --only-custom        Reproducir exclusivamente partidas personalizadas del usuario
        --no-auto-pause      Desactivar pausa automática por oclusión
        --layer <CAPA>       Capa Wayland: background (default) o bottom
        --snapshot <PATH>    Generar captura PNG y salir
    -h, --help               Mostrar esta ayuda
```

### Ejemplos

- **Probar en ventana flotante**:
  ```bash
  chess-wallpaper -w
  ```

- **Tema de madera clásico**:
  ```bash
  chess-wallpaper --theme wood
  ```

- **Rotación aleatoria de tableros y piezas en cada partida**:
  ```bash
  chess-wallpaper --rotate
  ```

- **Reproducir solo tus partidas guardadas**:
  ```bash
  chess-wallpaper --only-custom
  ```

---

## Cargar tus Partidas (PGN)

Copia cualquier archivo `.pgn` descargado de **Chess.com** o **Lichess** en la carpeta:

```bash
mkdir -p ~/.config/chess-wallpaper/games
cp mis_partidas.pgn ~/.config/chess-wallpaper/games/
```

Al iniciar `chess-wallpaper`, las detectará automáticamente, las validará y las mezclará en la rotación de partidas.

---

## Ejecución Automática (Systemd)

Crea el archivo de servicio en `~/.config/systemd/user/chess-wallpaper.service`:

```ini
[Unit]
Description=Chess Wallpaper - Fondo animado de ajedrez (Wayland)
PartOf=graphical-session.target
After=graphical-session.target

[Service]
Type=simple
ExecStart=%h/.local/bin/chess-wallpaper
Restart=on-failure
RestartSec=3

[Install]
WantedBy=graphical-session.target
```

Habilita y arranca el servicio:
```bash
systemctl --user daemon-reload
systemctl --user enable --now chess-wallpaper.service
```

---

## Script de Alternancia (Toggle)

Puedes guardar este script en `~/.local/bin/chess-wallpaper-toggle` y vincularlo a un atajo de teclado:

```bash
#!/usr/bin/env bash
if pgrep -x "chess-wallpaper" >/dev/null 2>&1; then
    pkill -x "chess-wallpaper"
    notify-send -a "Chess Wallpaper" "Chess Wallpaper detenido"
else
    nohup ~/.local/bin/chess-wallpaper "$@" >/dev/null 2>&1 &
    notify-send -a "Chess Wallpaper" "Chess Wallpaper en marcha"
fi
```

---

## Licencia

Distribuido bajo la licencia MIT.
Los assets gráficos de piezas, tableros e insignias pertenecen a sus respectivos autores y a Chess.com.
