use std::time::Instant;
use shakmaty::{Piece, Position, Square};
use tiny_skia::{Color as SkColor, Paint, PixmapMut, PixmapPaint, Rect, Transform};
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

    // 1. Limpiar fondo con color oscuro elegante
    canvas.fill(SkColor::from_rgba8(20, 21, 30, 255));

    // 2. Calcular dimensiones del tablero
    let max_board_dim = width.min(height) * 0.88;
    let tile_size = (max_board_dim / 8.0).floor().max(16.0);
    let board_total_w = tile_size * 8.0;
    let board_total_h = tile_size * 8.0;

    let board_x = ((width - board_total_w) / 2.0).floor();
    let board_y = ((height - board_total_h) / 2.0).floor();

    // Dibujar sombra suave detrás del tablero
    let mut shadow_paint = Paint::default();
    shadow_paint.set_color(SkColor::from_rgba8(0, 0, 0, 110));
    let shadow_rect = Rect::from_xywh(board_x - 8.0, board_y - 4.0, board_total_w + 16.0, board_total_h + 18.0);
    if let Some(r) = shadow_rect {
        canvas.fill_rect(r, &shadow_paint, Transform::identity(), None);
    }

    // 3. Dibujar las 64 baldosas
    let tile_paint = PixmapPaint::default();
    for rank_idx in 0..8 {
        for file_idx in 0..8 {
            let is_dark = (file_idx + rank_idx) % 2 == 0;
            let tile_pix = assets.get_tile(sim.theme, is_dark);

            let tx = board_x + (file_idx as f32) * tile_size;
            let ty = board_y + ((7 - rank_idx) as f32) * tile_size;

            let scale_x = tile_size / (tile_pix.width() as f32);
            let scale_y = tile_size / (tile_pix.height() as f32);

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

    // Identificar pieza que se está animando (si la hay)
    let animating_move = match sim.state {
        SimState::Animating(ref anim) => Some(anim),
        _ => None,
    };

    // 4. Dibujar piezas estáticas
    let board = sim.pos.board();
    for sq in Square::ALL {
        let file_idx = sq.file() as u8;
        let rank_idx = sq.rank() as u8;

        // Si esta casilla es el destino o origen del movimiento en curso, se maneja especialmente
        if let Some(anim) = animating_move {
            if anim.to == (file_idx, rank_idx) {
                // Si hubo una captura, dibujar la pieza capturada desvaneciéndose
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

    // 5. Dibujar la pieza en movimiento con física "Jelly"
    if let Some(anim) = animating_move {
        let (cur_f, cur_r) = anim.current_pos(now);
        let progress = anim.progress(now);

        // Squash & stretch al moverse y rebotar
        let squish = 1.0 + 0.16 * ((progress * std::f32::consts::PI).sin());

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

    // Escalar la pieza para que encaje proporcionalmente en la casilla (~84% de altura)
    let target_h = tile_size * 0.84;
    let base_scale = target_h / (pix.height() as f32);

    let scale_x = base_scale * squish;
    let scale_y = base_scale * (2.0 - squish).max(0.5);

    let pw = (pix.width() as f32) * scale_x;
    let ph = (pix.height() as f32) * scale_y;

    // Centro horizontal y apoyado en la base de la casilla
    let tx = board_x + file_f * tile_size + (tile_size - pw) / 2.0;
    let ty = board_y + (7.0 - rank_f) * tile_size + (tile_size - ph) * 0.88;

    let xform = Transform::from_scale(scale_x, scale_y)
        .post_translate(tx, ty);

    let mut paint = PixmapPaint::default();
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
