use std::time::Instant;
use shakmaty::{Piece, Position, Square};
use tiny_skia::{Color as SkColor, FilterQuality, Paint, PathBuilder, PixmapMut, PixmapPaint, Rect, Transform};
use crate::assets::AssetManager;
use crate::chess_sim::{ChessSimulator, SimState};

pub fn render_frame(
    canvas: &mut PixmapMut,
    assets: &AssetManager,
    sim: &ChessSimulator,
    now: Instant,
) {
    let width = canvas.width() as f32;
    let height = canvas.height() as f32;

    // 1. Limpiar fondo con color oscuro elegante (Catppuccin Crust / Deep Dark)
    canvas.fill(SkColor::from_rgba8(17, 17, 27, 255));

    // 2. Calcular dimensiones del tablero
    let max_board_dim = width.min(height) * 0.88;
    let tile_size = (max_board_dim / 8.0).floor().max(16.0);
    let board_total_w = tile_size * 8.0;
    let board_total_h = tile_size * 8.0;

    let board_x = ((width - board_total_w) / 2.0).floor();
    let board_y = ((height - board_total_h) / 2.0).floor();

    // 3. Marco decorativo / Sombra exterior del tablero
    let frame_pad = 14.0;
    let frame_x = board_x - frame_pad;
    let frame_y = board_y - frame_pad;
    let frame_w = board_total_w + frame_pad * 2.0;
    let frame_h = board_total_h + frame_pad * 2.0;

    // Sombra suave difusa
    let mut shadow_paint = Paint::default();
    shadow_paint.set_color(SkColor::from_rgba8(0, 0, 0, 95));
    if let Some(r) = Rect::from_xywh(frame_x - 4.0, frame_y + 4.0, frame_w + 8.0, frame_h + 10.0) {
        let mut pb = PathBuilder::new();
        pb.push_rect(r);
        if let Some(path) = pb.finish() {
            canvas.fill_path(&path, &shadow_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }
    }

    // Marco base del tablero con esquinas suavemente biseladas
    let mut frame_paint = Paint::default();
    frame_paint.set_color(SkColor::from_rgba8(26, 27, 38, 255));
    if let Some(r) = Rect::from_xywh(frame_x, frame_y, frame_w, frame_h) {
        canvas.fill_rect(r, &frame_paint, Transform::identity(), None);
    }

    // Borde fino interior para el marco
    let mut border_paint = Paint::default();
    border_paint.set_color(SkColor::from_rgba8(45, 48, 68, 200));
    let mut stroke = tiny_skia::Stroke::default();
    stroke.width = 1.5;
    if let Some(r) = Rect::from_xywh(frame_x, frame_y, frame_w, frame_h) {
        let mut pb = PathBuilder::new();
        pb.push_rect(r);
        if let Some(path) = pb.finish() {
            canvas.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
        }
    }

    // 4. Dibujar las 64 baldosas con filtrado Bicúbico (Anti-Aliased de máxima calidad)
    let mut tile_paint = PixmapPaint::default();
    tile_paint.quality = FilterQuality::Bicubic;

    for rank_idx in 0..8 {
        for file_idx in 0..8 {
            let is_dark = (file_idx + rank_idx) % 2 == 0;
            let tile_pix = assets.get_tile(sim.theme, is_dark);

            let tx = board_x + (file_idx as f32) * tile_size;
            let ty = board_y + ((7 - rank_idx) as f32) * tile_size;

            // Leve bleed de 0.6px para evitar líneas de sutura subpíxel entre baldosas
            let scale_x = (tile_size + 0.6) / (tile_pix.width() as f32);
            let scale_y = (tile_size + 0.6) / (tile_pix.height() as f32);

            let xform = Transform::from_scale(scale_x, scale_y)
                .post_translate(tx, ty);

            canvas.draw_pixmap(
                0,
                0,
                tile_pix.as_ref(),
                &tile_paint,
                xform,
                None,
            );
        }
    }

    // Identificar pieza que se está animando
    let animating_move = match sim.state {
        SimState::Animating(ref anim) => Some(anim),
        _ => None,
    };

    // 5. Dibujar piezas estáticas con proporciones naturales
    let board = sim.pos.board();
    for sq in Square::ALL {
        let file_idx = sq.file() as u8;
        let rank_idx = sq.rank() as u8;

        if let Some(anim) = animating_move {
            if anim.to == (file_idx, rank_idx) {
                // Pieza capturada desvaneciéndose suavemente
                if let Some(captured_piece) = anim.captured {
                    let progress = anim.progress(now);
                    let alpha = (1.0 - progress).clamp(0.0, 1.0);
                    if alpha > 0.02 {
                        draw_piece_at_square(
                            canvas,
                            assets,
                            sim,
                            captured_piece,
                            file_idx as f32,
                            rank_idx as f32,
                            board_x,
                            board_y,
                            tile_size,
                            1.0,
                            alpha,
                        );
                    }
                }
                continue;
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
                board_x,
                board_y,
                tile_size,
                1.0,
                1.0,
            );
        }
    }

    // 6. Dibujar pieza en movimiento (con rebote Jelly elástico y Bicubic)
    if let Some(anim) = animating_move {
        let (cur_f, cur_r) = anim.current_pos(now);
        let progress = anim.progress(now);

        // Squash & stretch físico en la fase de salto
        let squish = 1.0 + 0.14 * ((progress * std::f32::consts::PI).sin());

        draw_piece_at_square(
            canvas,
            assets,
            sim,
            anim.piece,
            cur_f,
            cur_r,
            board_x,
            board_y,
            tile_size,
            squish,
            1.0,
        );
    }
}

fn draw_piece_at_square(
    canvas: &mut PixmapMut,
    assets: &AssetManager,
    sim: &ChessSimulator,
    piece: Piece,
    file_f: f32,
    rank_f: f32,
    board_x: f32,
    board_y: f32,
    tile_size: f32,
    squish: f32,
    opacity: f32,
) {
    let pix = assets.get_piece(sim.theme, piece.color, piece.role);

    // Escala uniforme proporcional basada en la altura de referencia del Rey (124px)
    // De este modo, los peones son tiernos y proporcionados (~65% casilla),
    // y reyes/torres dominan el espacio (~80% casilla) sin estiramientos raros.
    let reference_king_h = 124.0;
    let base_scale = (tile_size * 0.80) / reference_king_h;

    let scale_x = base_scale * squish;
    let scale_y = base_scale * (2.0 - squish).max(0.5);

    let pw = (pix.width() as f32) * scale_x;
    let ph = (pix.height() as f32) * scale_y;

    // Centrado horizontal exacto y apoyado sobre la base de la casilla
    let tx = board_x + file_f * tile_size + (tile_size - pw) / 2.0;
    let ty = board_y + (7.0 - rank_f) * tile_size + (tile_size - ph) * 0.90;

    let xform = Transform::from_scale(scale_x, scale_y)
        .post_translate(tx, ty);

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
