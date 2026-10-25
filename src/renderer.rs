use std::time::Instant;
use shakmaty::{Piece, Position, Square};
use tiny_skia::{Color as SkColor, FilterQuality, Paint, PathBuilder, PixmapMut, PixmapPaint, Rect, Transform};

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
    let max_avail = (width * 0.92).min(height * 0.92);
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
                board_x,
                board_y,
                tile_size,
                1.0,
                1.0,
                1.0,
            );
        }
    }

    // 8. Pieza capturada desvaneciéndose en su casilla
    if let Some(anim) = animating_move {
        if let Some((captured_piece, (cap_f, cap_r))) = anim.captured {
            let progress = anim.progress(now);
            let opacity = (1.0 - progress).clamp(0.0, 1.0);
            draw_piece_at_square(
                canvas,
                assets,
                sim,
                captured_piece,
                cap_f as f32,
                cap_r as f32,
                0.0,
                board_x,
                board_y,
                tile_size,
                1.0,
                1.0,
                opacity,
            );
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
    let jump = anim.jump_height(progress);

    let sx = board_x + cur_f * tile_size + tile_size * 0.5;
    let sy = board_y + (7.0 - cur_r) * tile_size + tile_size * 0.85;

    let shadow_w = (tile_size * 0.52) * (1.0 - jump * 0.22);
    let shadow_h = (tile_size * 0.20) * (1.0 - jump * 0.22);
    let shadow_alpha = (90.0 * (1.0 - jump * 0.45)) as u8;

    let mut oval_paint = Paint::default();
    oval_paint.set_color_rgba8(0, 0, 0, shadow_alpha);

    let mut pb = PathBuilder::new();
    pb.push_oval(Rect::from_xywh(sx - shadow_w / 2.0, sy - shadow_h / 2.0, shadow_w, shadow_h).unwrap());
    if let Some(path) = pb.finish() {
        canvas.fill_path(&path, &oval_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
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
    let jump = anim.jump_height(progress);
    let arc_y = jump * (tile_size * 0.35);
    let (squish_x, squish_y) = anim.jelly_scale(progress);

    draw_piece_at_square(
        canvas,
        assets,
        sim,
        anim.piece,
        cur_f,
        cur_r,
        arc_y,
        board_x,
        board_y,
        tile_size,
        squish_x,
        squish_y,
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

    let xform = Transform::from_scale(scale_x, scale_y).post_translate(tx, ty);

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
