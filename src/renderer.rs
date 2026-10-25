use std::time::Instant;
use shakmaty::{Color, Piece, Position, Role, Square};
use tiny_skia::{
    Color as SkColor, FilterQuality, GradientStop, Paint, PathBuilder, PixmapMut, PixmapPaint, Point,
    RadialGradient, Rect, SpreadMode, Transform,
};

use crate::assets::AssetManager;
use crate::chess_sim::{ChessSimulator, SimState, SinglePieceAnim};

fn add_rounded_rect(pb: &mut PathBuilder, x: f32, y: f32, w: f32, h: f32, r: f32) {
    let r = r.min(w / 2.0).min(h / 2.0);
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
}

fn draw_text(
    canvas: &mut PixmapMut,
    font: &fontdue::Font,
    text: &str,
    start_x: f32,
    baseline_y: f32,
    font_size: f32,
    color: SkColor,
) -> f32 {
    let mut cur_x = start_x;
    let cr = color.red() * 255.0;
    let cg = color.green() * 255.0;
    let cb = color.blue() * 255.0;
    let ca = color.alpha();

    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, font_size);
        if metrics.width > 0 && metrics.height > 0 {
            let gx = (cur_x + metrics.xmin as f32).round() as i32;
            let gy = (baseline_y - (metrics.height as f32 + metrics.ymin as f32)).round() as i32;

            for row in 0..metrics.height {
                let py = gy + row as i32;
                if py < 0 || py >= canvas.height() as i32 {
                    continue;
                }
                for col in 0..metrics.width {
                    let px = gx + col as i32;
                    if px < 0 || px >= canvas.width() as i32 {
                        continue;
                    }
                    let coverage = bitmap[row * metrics.width + col];
                    if coverage == 0 {
                        continue;
                    }

                    let alpha_f = (coverage as f32 / 255.0) * ca;
                    let dst_idx = (py as usize * canvas.width() as usize + px as usize) * 4;
                    let pixels = canvas.data_mut();

                    let dr = pixels[dst_idx] as f32;
                    let dg = pixels[dst_idx + 1] as f32;
                    let db = pixels[dst_idx + 2] as f32;
                    let da = pixels[dst_idx + 3] as f32;

                    let sr = cr * alpha_f;
                    let sg = cg * alpha_f;
                    let sb = cb * alpha_f;
                    let sa = 255.0 * alpha_f;

                    let inv_sa = 1.0 - alpha_f;
                    pixels[dst_idx] = (sr + dr * inv_sa).round().min(255.0) as u8;
                    pixels[dst_idx + 1] = (sg + dg * inv_sa).round().min(255.0) as u8;
                    pixels[dst_idx + 2] = (sb + db * inv_sa).round().min(255.0) as u8;
                    pixels[dst_idx + 3] = (sa + da * inv_sa).round().min(255.0) as u8;
                }
            }
        }
        cur_x += metrics.advance_width;
    }
    cur_x
}

fn measure_text(font: &fontdue::Font, text: &str, font_size: f32) -> f32 {
    let mut width = 0.0;
    for ch in text.chars() {
        let metrics = font.metrics(ch, font_size);
        width += metrics.advance_width;
    }
    width
}

fn draw_player_badge(
    canvas: &mut PixmapMut,
    assets: &AssetManager,
    sim: &ChessSimulator,
    color: Color,
    player_name: &str,
    meta_text: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    is_active_turn: bool,
) {
    let corner_radius = 12.0;

    // 1. Sombra suave del badge
    let mut shadow_paint = Paint::default();
    shadow_paint.set_color(SkColor::from_rgba8(0, 0, 0, 75));
    let mut pb_shadow = PathBuilder::new();
    add_rounded_rect(&mut pb_shadow, x - 2.0, y + 4.0, w + 4.0, h + 6.0, corner_radius + 2.0);
    if let Some(path) = pb_shadow.finish() {
        canvas.fill_path(&path, &shadow_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }

    // 2. Fondo del badge (sutilmente más luminoso si es su turno activo)
    let mut bg_paint = Paint::default();
    if is_active_turn {
        bg_paint.set_color(SkColor::from_rgba8(30, 34, 52, 255));
    } else {
        bg_paint.set_color(SkColor::from_rgba8(24, 27, 40, 255));
    }
    let mut pb_bg = PathBuilder::new();
    add_rounded_rect(&mut pb_bg, x, y, w, h, corner_radius);
    if let Some(path) = pb_bg.finish() {
        canvas.fill_path(&path, &bg_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }

    // 3. Borde fino elegante
    let mut border_paint = Paint::default();
    if is_active_turn {
        border_paint.set_color(SkColor::from_rgba8(82, 108, 160, 210));
    } else {
        border_paint.set_color(SkColor::from_rgba8(48, 54, 78, 160));
    }
    let mut stroke = tiny_skia::Stroke::default();
    stroke.width = 1.2;
    let mut pb_border = PathBuilder::new();
    add_rounded_rect(&mut pb_border, x, y, w, h, corner_radius);
    if let Some(path) = pb_border.finish() {
        canvas.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
    }

    // 4. Mini icono / Avatar de la pieza (Rey)
    let avatar_pix = assets.get_piece(sim.piece_style, color, Role::King);
    let avatar_dim = (h * 0.68).round().clamp(22.0, 30.0);
    let scale = avatar_dim / (avatar_pix.width() as f32);
    let mut avatar_paint = PixmapPaint::default();
    avatar_paint.quality = FilterQuality::Bicubic;
    let ax = x + 12.0;
    let ay = y + (h - avatar_dim) / 2.0;
    let xform = Transform::from_scale(scale, scale).post_translate(ax, ay);
    canvas.draw_pixmap(0, 0, avatar_pix.as_ref(), &avatar_paint, xform, None);

    // 5. Nombre del jugador
    let name_font_size = (h * 0.38).round().clamp(13.0, 16.0);
    let name_x = ax + avatar_dim + 10.0;
    let baseline_y = y + h * 0.64;
    let name_color = if is_active_turn {
        SkColor::from_rgba8(250, 252, 255, 255)
    } else {
        SkColor::from_rgba8(220, 226, 238, 255)
    };
    draw_text(canvas, &assets.ui_font, player_name, name_x, baseline_y, name_font_size, name_color);

    // 6. Metadatos a la derecha (Evento / Jugada)
    let meta_font_size = (h * 0.30).round().clamp(11.0, 13.0);
    let meta_w = measure_text(&assets.ui_font, meta_text, meta_font_size);
    let meta_x = (x + w - meta_w - 14.0).max(name_x + 120.0);
    let meta_color = SkColor::from_rgba8(138, 152, 178, 255);
    draw_text(canvas, &assets.ui_font, meta_text, meta_x, baseline_y, meta_font_size, meta_color);
}

pub fn render_frame(
    canvas: &mut PixmapMut,
    assets: &AssetManager,
    sim: &ChessSimulator,
    bg_color: SkColor,
    desired_board_size: f32,
    now: Instant,
) {
    let width = canvas.width() as f32;
    let height = canvas.height() as f32;

    // 1. Rellenar toda la pantalla con el color de fondo
    canvas.fill(bg_color);

    // 2. Calcular tamaño fijo del tablero
    let max_avail = (width * 0.90).min(height * 0.76); // Reserva vertical para badges
    let target_board_dim = desired_board_size.min(max_avail);
    let tile_size = (target_board_dim / 8.0).floor().max(16.0);
    let board_total_w = tile_size * 8.0;
    let board_total_h = tile_size * 8.0;

    let board_x = ((width - board_total_w) / 2.0).floor();
    let board_y = ((height - board_total_h) / 2.0).floor();

    // 3. Marco flotante del tablero con bordes redondeados y sombra suave
    let frame_pad = (tile_size * 0.28).floor().max(20.0);
    let frame_x = board_x - frame_pad;
    let frame_y = board_y - frame_pad;
    let frame_w = board_total_w + frame_pad * 2.0;
    let frame_h = board_total_h + frame_pad * 2.0;
    let corner_radius = 20.0;

    // Sombra exterior amplia y difusa
    let mut shadow_paint = Paint::default();
    shadow_paint.set_color(SkColor::from_rgba8(0, 0, 0, 90));
    let mut pb_shadow = PathBuilder::new();
    add_rounded_rect(&mut pb_shadow, frame_x - 6.0, frame_y + 8.0, frame_w + 12.0, frame_h + 14.0, corner_radius + 4.0);
    if let Some(path) = pb_shadow.finish() {
        canvas.fill_path(&path, &shadow_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }

    // Marco base sólido oscuro elegante (superficie flotante)
    let mut frame_paint = Paint::default();
    frame_paint.set_color(SkColor::from_rgba8(28, 31, 46, 255));
    let mut pb_frame = PathBuilder::new();
    add_rounded_rect(&mut pb_frame, frame_x, frame_y, frame_w, frame_h, corner_radius);
    if let Some(path) = pb_frame.finish() {
        canvas.fill_path(&path, &frame_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }

    // Borde fino sutil del marco
    let mut border_paint = Paint::default();
    border_paint.set_color(SkColor::from_rgba8(60, 68, 98, 160));
    let mut stroke = tiny_skia::Stroke::default();
    stroke.width = 1.5;
    let mut pb_border = PathBuilder::new();
    add_rounded_rect(&mut pb_border, frame_x, frame_y, frame_w, frame_h, corner_radius);
    if let Some(path) = pb_border.finish() {
        canvas.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
    }

    // 4. Dibujar el tablero oficial de Chess.com escalado con filtrado Bicúbico
    let board_pixmap = assets.get_board(sim.board_style);
    let scale = board_total_w / (board_pixmap.width() as f32);
    let mut board_paint = PixmapPaint::default();
    board_paint.quality = FilterQuality::Bicubic;

    let board_xform = Transform::from_scale(scale, scale).post_translate(board_x, board_y);
    canvas.draw_pixmap(0, 0, board_pixmap.as_ref(), &board_paint, board_xform, None);

    // 4.1. Resaltado de Rey en Jaque (Alerta carmesí Chess.com)
    if sim.pos.is_check() {
        let in_check_color = sim.pos.turn();
        if let Some(king_sq) = sim.pos.board().king_of(in_check_color) {
            let kf = king_sq.file() as u8 as f32;
            let kr = king_sq.rank() as u8 as f32;
            let sx = board_x + kf * tile_size;
            let sy = board_y + (7.0 - kr) * tile_size;

            let cx = sx + tile_size * 0.5;
            let cy = sy + tile_size * 0.5;

            // Degradado radial cónico carmesí al estilo Chess.com
            let stops = vec![
                GradientStop::new(0.0, SkColor::from_rgba8(235, 55, 55, 175)),
                GradientStop::new(0.55, SkColor::from_rgba8(235, 55, 55, 120)),
                GradientStop::new(1.0, SkColor::from_rgba8(235, 55, 55, 0)),
            ];
            if let Some(shader) = RadialGradient::new(
                Point::from_xy(cx, cy),
                0.0,
                Point::from_xy(cx, cy),
                tile_size * 0.72,
                stops,
                SpreadMode::Pad,
                Transform::identity(),
            ) {
                let mut check_paint = Paint::default();
                check_paint.shader = shader;
                if let Some(rect) = Rect::from_xywh(sx, sy, tile_size, tile_size) {
                    let mut pb = PathBuilder::new();
                    pb.push_rect(rect);
                    if let Some(path) = pb.finish() {
                        canvas.fill_path(&path, &check_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                    }
                }
            } else {
                let mut check_paint = Paint::default();
                check_paint.set_color(SkColor::from_rgba8(235, 55, 55, 130));
                if let Some(rect) = Rect::from_xywh(sx, sy, tile_size, tile_size) {
                    let mut pb = PathBuilder::new();
                    pb.push_rect(rect);
                    if let Some(path) = pb.finish() {
                        canvas.fill_path(&path, &check_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                    }
                }
            }
        }
    }

    // Borde fino interior alrededor de las casillas
    let mut inner_border_paint = Paint::default();
    inner_border_paint.set_color(SkColor::from_rgba8(0, 0, 0, 80));
    let mut inner_stroke = tiny_skia::Stroke::default();
    inner_stroke.width = 1.0;
    if let Some(r) = Rect::from_xywh(board_x, board_y, board_total_w, board_total_h) {
        let mut pb_in = PathBuilder::new();
        pb_in.push_rect(r);
        if let Some(path) = pb_in.finish() {
            canvas.stroke_path(&path, &inner_border_paint, &inner_stroke, Transform::identity(), None);
        }
    }

    // 5. Identificar piezas animadas
    let animating_move = match sim.state {
        SimState::Animating(ref anim) => Some(anim),
        _ => None,
    };

    // 6. Dibujar sombras de suelo de piezas en salto
    if let Some(anim) = animating_move {
        let progress = anim.progress(now);
        draw_ground_shadow(canvas, &anim.primary, progress, board_x, board_y, tile_size);
        if let Some(ref sec) = anim.secondary {
            draw_ground_shadow(canvas, sec, progress, board_x, board_y, tile_size);
        }
    }

    // 7. Dibujar piezas estáticas
    let board = sim.pos.board();
    for sq in Square::ALL {
        let file_idx = sq.file() as u8;
        let rank_idx = sq.rank() as u8;
        let coords = (file_idx, rank_idx);

        if let Some(anim) = animating_move {
            if coords == anim.primary.from || coords == anim.primary.to {
                continue;
            }
            if let Some(ref sec) = anim.secondary {
                if coords == sec.from || coords == sec.to {
                    continue;
                }
            }
            if let Some((_, cap_coords)) = anim.captured {
                if coords == cap_coords {
                    continue;
                }
            }
        }

        if let Some(piece) = board.piece_at(sq) {
            draw_piece_at_square(
                canvas,
                assets,
                sim,
                piece,
                file_idx as f32,
                rank_idx as f32,
                0.0,
                0.0,
                board_x,
                board_y,
                tile_size,
                1.0,
                1.0,
                1.0,
            );
        }
    }

    // 8. Pieza capturada con impacto físico, retroceso (knockback), giro y desvanecimiento
    if let Some(anim) = animating_move {
        if let Some((captured_piece, (cap_f, cap_r))) = anim.captured {
            let progress = anim.progress(now);
            let dx = (anim.primary.to.0 as f32) - (anim.primary.from.0 as f32);
            let dy = (anim.primary.to.1 as f32) - (anim.primary.from.1 as f32);
            let dist = (dx * dx + dy * dy).sqrt().max(0.001);
            let nx = dx / dist;
            let ny = dy / dist;

            if progress < 0.74 {
                // Antes del impacto: la pieza sigue firme en su casilla
                draw_piece_at_square(
                    canvas,
                    assets,
                    sim,
                    captured_piece,
                    cap_f as f32,
                    cap_r as f32,
                    0.0,
                    0.0,
                    board_x,
                    board_y,
                    tile_size,
                    1.0,
                    1.0,
                    1.0,
                );
            } else {
                // Momento del impacto: retroceso físico en la dirección del golpe, giro y desvanecimiento
                let u = (progress - 0.74) / 0.26;
                let knock_dist = u * 0.42;
                let kf = (cap_f as f32) + nx * knock_dist;
                let kr = (cap_r as f32) + ny * knock_dist;

                let tumble_tilt = nx * u * 24.0;
                let scale = (1.0 - u * 0.85).max(0.0);
                let opacity = (1.0 - u).clamp(0.0, 1.0);

                draw_piece_at_square(
                    canvas,
                    assets,
                    sim,
                    captured_piece,
                    kf,
                    kr,
                    0.0,
                    tumble_tilt,
                    board_x,
                    board_y,
                    tile_size,
                    scale,
                    scale,
                    opacity,
                );
            }
        }
    }

    // 9. Dibujar las piezas en animación (Rey y Torre en enroque)
    if let Some(anim) = animating_move {
        let progress = anim.progress(now);
        draw_animated_piece(canvas, assets, sim, &anim.primary, progress, board_x, board_y, tile_size);
        if let Some(ref sec) = anim.secondary {
            draw_animated_piece(canvas, assets, sim, sec, progress, board_x, board_y, tile_size);
        }
    }

    // 10. Badges de jugadores estilo retransmisión (Opción A)
    let badge_h = (tile_size * 0.52).floor().clamp(38.0, 44.0);
    let badge_pad = 12.0;
    let badge_w = frame_w;
    let badge_x = frame_x;

    let top_badge_y = frame_y - badge_h - badge_pad;
    let bot_badge_y = frame_y + frame_h + badge_pad;

    // Solo dibujar si la ventana tiene altura suficiente
    if top_badge_y >= 6.0 && bot_badge_y + badge_h <= height - 6.0 {
        let game = sim.current_game();
        let current_turn = sim.pos.turn();

        let (is_black_turn, is_white_turn) = match sim.state {
            SimState::GameOver { .. } => (false, false),
            _ => (current_turn == Color::Black, current_turn == Color::White),
        };

        // Badge superior: Jugador de Negras
        let top_meta = format!("{} ({})", game.event, game.year);
        draw_player_badge(
            canvas,
            assets,
            sim,
            Color::Black,
            game.black,
            &top_meta,
            badge_x,
            top_badge_y,
            badge_w,
            badge_h,
            is_black_turn,
        );

        // Badge inferior: Jugador de Blancas
        let bot_meta = if matches!(sim.state, SimState::GameOver { .. }) {
            format!("Fin: {} ({})", game.result, game.termination)
        } else {
            format!("Jugada {}/{}", sim.current_move_num(), sim.total_moves())
        };
        draw_player_badge(
            canvas,
            assets,
            sim,
            Color::White,
            game.white,
            &bot_meta,
            badge_x,
            bot_badge_y,
            badge_w,
            badge_h,
            is_white_turn,
        );
    }
}

fn draw_ground_shadow(
    canvas: &mut PixmapMut,
    anim: &SinglePieceAnim,
    progress: f32,
    board_x: f32,
    board_y: f32,
    tile_size: f32,
) {
    let (cur_f, cur_r) = anim.current_pos(progress);
    let jump_norm = (anim.jump_height(progress) / 0.45).clamp(0.0, 1.0);

    let sx = board_x + cur_f * tile_size + tile_size * 0.5;
    let sy = board_y + (7.0 - cur_r) * tile_size + tile_size * 0.85;

    let shadow_w = (tile_size * 0.50) * (1.0 - jump_norm * 0.35);
    let shadow_h = (tile_size * 0.18) * (1.0 - jump_norm * 0.35);
    let shadow_alpha = (85.0 * (1.0 - jump_norm * 0.55)) as u8;

    let mut oval_paint = Paint::default();
    oval_paint.set_color_rgba8(0, 0, 0, shadow_alpha);

    let mut pb = PathBuilder::new();
    if let Some(r) = Rect::from_xywh(sx - shadow_w / 2.0, sy - shadow_h / 2.0, shadow_w, shadow_h) {
        pb.push_oval(r);
        if let Some(path) = pb.finish() {
            canvas.fill_path(&path, &oval_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }
    }
}

fn draw_animated_piece(
    canvas: &mut PixmapMut,
    assets: &AssetManager,
    sim: &ChessSimulator,
    anim: &SinglePieceAnim,
    progress: f32,
    board_x: f32,
    board_y: f32,
    tile_size: f32,
) {
    let (cur_f, cur_r) = anim.current_pos(progress);
    let jump_h = anim.jump_height(progress);
    let arc_y = jump_h * tile_size;
    let (scale_x, scale_y) = anim.scale(progress);
    let tilt = anim.tilt_degrees(progress);

    draw_piece_at_square(
        canvas,
        assets,
        sim,
        anim.piece,
        cur_f,
        cur_r,
        arc_y,
        tilt,
        board_x,
        board_y,
        tile_size,
        scale_x,
        scale_y,
        1.0,
    );
}

fn draw_piece_at_square(
    canvas: &mut PixmapMut,
    assets: &AssetManager,
    sim: &ChessSimulator,
    piece: Piece,
    file_f: f32,
    rank_f: f32,
    arc_offset_y: f32,
    tilt_deg: f32,
    board_x: f32,
    board_y: f32,
    tile_size: f32,
    scale_factor_x: f32,
    scale_factor_y: f32,
    opacity: f32,
) {
    let pix = assets.get_piece(sim.piece_style, piece.color, piece.role);

    let scale_base = tile_size / (pix.width() as f32);
    let scale_x = scale_base * scale_factor_x;
    let scale_y = scale_base * scale_factor_y;

    let pw = (pix.width() as f32) * scale_x;
    let ph = (pix.height() as f32) * scale_y;

    let tx = board_x + file_f * tile_size + (tile_size - pw) / 2.0;
    let ty = board_y + (7.0 - rank_f) * tile_size + (tile_size - ph) / 2.0 - arc_offset_y;

    let mut xform = Transform::from_scale(scale_x, scale_y).post_translate(tx, ty);
    if tilt_deg.abs() > 0.01 {
        let pivot_x = tx + pw * 0.5;
        let pivot_y = ty + ph * 0.85; // Base inferior donde apoya la pieza
        xform = xform.post_rotate_at(tilt_deg, pivot_x, pivot_y);
    }

    let mut paint = PixmapPaint::default();
    paint.quality = FilterQuality::Bicubic;
    paint.opacity = opacity.clamp(0.0, 1.0);

    canvas.draw_pixmap(
        0,
        0,
        pix.as_ref(),
        &paint,
        xform,
        None,
    );
}
