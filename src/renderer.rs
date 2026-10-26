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

fn draw_eval_bar(
    canvas: &mut PixmapMut,
    assets: &AssetManager,
    sim: &ChessSimulator,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    let r = 6.0;

    // Sombra sutil de la barra de evaluación
    let mut shadow_paint = Paint::default();
    shadow_paint.set_color(SkColor::from_rgba8(0, 0, 0, 70));
    let mut pb_shadow = PathBuilder::new();
    add_rounded_rect(&mut pb_shadow, x - 1.0, y + 2.0, w + 2.0, h + 3.0, r + 1.0);
    if let Some(path) = pb_shadow.finish() {
        canvas.fill_path(&path, &shadow_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }

    // Fondo: Zona de Negras (Antracita oscuro Chess.com #262421)
    let mut black_paint = Paint::default();
    black_paint.set_color(SkColor::from_rgba8(38, 36, 33, 255));
    let mut pb_bg = PathBuilder::new();
    add_rounded_rect(&mut pb_bg, x, y, w, h, r);
    if let Some(path) = pb_bg.finish() {
        canvas.fill_path(&path, &black_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }

    // Zona de Blancas: Llena desde abajo hacia arriba
    let white_pct = sim.current_eval_pct.clamp(0.0, 1.0);
    let white_h = (h * white_pct).round();
    let white_y = y + h - white_h;

    if white_h > 0.0 {
        let mut white_paint = Paint::default();
        white_paint.set_color(SkColor::from_rgba8(245, 245, 248, 255));

        let mut pb_white = PathBuilder::new();
        if white_pct >= 0.98 {
            add_rounded_rect(&mut pb_white, x, y, w, h, r);
        } else if white_pct > 0.01 {
            let wr = r.min(white_h).min(w / 2.0);
            pb_white.move_to(x, white_y);
            pb_white.line_to(x + w, white_y);
            pb_white.line_to(x + w, y + h - wr);
            pb_white.quad_to(x + w, y + h, x + w - wr, y + h);
            pb_white.line_to(x + wr, y + h);
            pb_white.quad_to(x, y + h, x, y + h - wr);
            pb_white.line_to(x, white_y);
            pb_white.close();
        }
        if let Some(path) = pb_white.finish() {
            canvas.fill_path(&path, &white_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }
    }

    // Línea central (50% - igualdad absoluta)
    let mid_y = (y + h * 0.5).round();
    let mut mid_paint = Paint::default();
    mid_paint.set_color(SkColor::from_rgba8(110, 120, 140, 150));
    let mut pb_mid = PathBuilder::new();
    pb_mid.move_to(x + 2.0, mid_y);
    pb_mid.line_to(x + w - 2.0, mid_y);
    let mut mid_stroke = tiny_skia::Stroke::default();
    mid_stroke.width = 1.0;
    if let Some(path) = pb_mid.finish() {
        canvas.stroke_path(&path, &mid_paint, &mid_stroke, Transform::identity(), None);
    }

    // Borde exterior
    let mut border_paint = Paint::default();
    border_paint.set_color(SkColor::from_rgba8(65, 75, 105, 170));
    let mut stroke = tiny_skia::Stroke::default();
    stroke.width = 1.2;
    let mut pb_border = PathBuilder::new();
    add_rounded_rect(&mut pb_border, x, y, w, h, r);
    if let Some(path) = pb_border.finish() {
        canvas.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
    }

    // Puntuación numérica en la barra (e.g. +1.4 o M1)
    let score = sim.current_eval_score;
    let font_size = 9.5;
    if let Some(mate) = sim.current_mate_in {
        let (mate_str, is_white_win) = if mate > 0 {
            (format!("M{}", mate), true)
        } else {
            (format!("-M{}", -mate), false)
        };
        let tw = measure_text(&assets.ui_font, &mate_str, font_size);
        let tx = x + (w - tw) / 2.0;
        if is_white_win {
            draw_text(canvas, &assets.ui_font, &mate_str, tx, y + h - 6.0, font_size, SkColor::from_rgba8(20, 22, 28, 255));
        } else {
            draw_text(canvas, &assets.ui_font, &mate_str, tx, y + 14.0, font_size, SkColor::from_rgba8(240, 240, 245, 255));
        }
    } else if sim.pos.is_checkmate() {
        let (mate_str, is_white_win) = if score > 0.0 { ("M1", true) } else { ("-M1", false) };
        let tw = measure_text(&assets.ui_font, mate_str, font_size);
        let tx = x + (w - tw) / 2.0;
        if is_white_win {
            draw_text(canvas, &assets.ui_font, mate_str, tx, y + h - 6.0, font_size, SkColor::from_rgba8(20, 22, 28, 255));
        } else {
            draw_text(canvas, &assets.ui_font, mate_str, tx, y + 14.0, font_size, SkColor::from_rgba8(240, 240, 245, 255));
        }
    } else {
        let abs_score = score.abs();
        let score_str = if abs_score < 0.1 {
            "0.0".to_string()
        } else {
            format!("{:.1}", abs_score)
        };
        let tw = measure_text(&assets.ui_font, &score_str, font_size);
        let tx = x + (w - tw) / 2.0;

        if score >= 0.0 {
            // Ventaja blanca: abajo dentro de la barra blanca
            let ty = (y + h - 6.0).min(white_y + white_h - 4.0).max(y + 14.0);
            draw_text(canvas, &assets.ui_font, &score_str, tx, ty, font_size, SkColor::from_rgba8(25, 27, 34, 255));
        } else {
            // Ventaja negra: arriba dentro de la barra negra
            let ty = y + 14.0;
            draw_text(canvas, &assets.ui_font, &score_str, tx, ty, font_size, SkColor::from_rgba8(230, 235, 245, 255));
        }
    }
}

fn draw_player_badge(
    canvas: &mut PixmapMut,
    assets: &AssetManager,
    sim: &ChessSimulator,
    color: Color,
    player_name: &str,
    elo: Option<&str>,
    top_meta: &str,
    sub_meta: Option<&str>,
    classification: Option<crate::engine::MoveClassification>,
    captured: &crate::chess_sim::CapturedPieces,
    material_advantage: i32,
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
    let avatar_dim = (h * 0.68).round().clamp(24.0, 32.0);
    let scale = avatar_dim / (avatar_pix.width() as f32);
    let mut avatar_paint = PixmapPaint::default();
    avatar_paint.quality = FilterQuality::Bicubic;
    let ax = x + 12.0;
    let ay = y + (h - avatar_dim) / 2.0;
    let xform = Transform::from_scale(scale, scale).post_translate(ax, ay);
    canvas.draw_pixmap(0, 0, avatar_pix.as_ref(), &avatar_paint, xform, None);

    // 5. Nombre del jugador y Elo
    let name_font_size = (h * 0.33).round().clamp(13.0, 16.0);
    let name_x = ax + avatar_dim + 10.0;
    let baseline_name_y = y + h * 0.44;
    let name_color = if is_active_turn {
        SkColor::from_rgba8(250, 252, 255, 255)
    } else {
        SkColor::from_rgba8(220, 226, 238, 255)
    };
    let cur_name_end = draw_text(canvas, &assets.ui_font, player_name, name_x, baseline_name_y, name_font_size, name_color);

    if let Some(elo_str) = elo {
        let elo_text = format!(" ({})", elo_str);
        let elo_color = SkColor::from_rgba8(145, 158, 185, 255);
        draw_text(canvas, &assets.ui_font, &elo_text, cur_name_end, baseline_name_y, name_font_size - 1.5, elo_color);
    }

    // 6. Sub-fila izquierda: Piezas capturadas (en orden de valor: Q, R, B, N, P) y ventaja material (+N)
    let enemy_color = !color;
    let mini_icon_dim = (h * 0.32).round().clamp(13.0, 16.0);
    let mini_spacing = (mini_icon_dim * 0.62).round();
    let mini_y = y + h * 0.56;
    let mut cur_icon_x = name_x;

    let roles_in_order = [
        (Role::Queen, captured.queens),
        (Role::Rook, captured.rooks),
        (Role::Bishop, captured.bishops),
        (Role::Knight, captured.knights),
        (Role::Pawn, captured.pawns),
    ];

    for (role, count) in roles_in_order {
        if count == 0 {
            continue;
        }
        let piece_pix = assets.get_piece(sim.piece_style, enemy_color, role);
        let p_scale = mini_icon_dim / (piece_pix.width() as f32);
        let mut p_paint = PixmapPaint::default();
        p_paint.quality = FilterQuality::Bicubic;

        for _ in 0..count {
            let pxform = Transform::from_scale(p_scale, p_scale).post_translate(cur_icon_x, mini_y);
            canvas.draw_pixmap(0, 0, piece_pix.as_ref(), &p_paint, pxform, None);
            cur_icon_x += mini_spacing;
        }
        cur_icon_x += 2.0;
    }

    // Ventaja material pill badge (+N)
    if material_advantage > 0 {
        let pill_text = format!("+{}", material_advantage);
        let pill_font_size = 10.5;
        let pill_tw = measure_text(&assets.ui_font, &pill_text, pill_font_size);
        let pill_pad = 5.0;
        let pill_w = pill_tw + pill_pad * 2.0;
        let pill_h = mini_icon_dim * 0.88;
        let pill_x = cur_icon_x + 4.0;
        let pill_y = mini_y + (mini_icon_dim - pill_h) / 2.0;

        let mut pill_bg = Paint::default();
        pill_bg.set_color(SkColor::from_rgba8(255, 255, 255, 28));
        let mut pb_pill = PathBuilder::new();
        add_rounded_rect(&mut pb_pill, pill_x, pill_y, pill_w, pill_h, 4.0);
        if let Some(path) = pb_pill.finish() {
            canvas.fill_path(&path, &pill_bg, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }

        let pill_color = SkColor::from_rgba8(240, 242, 248, 255);
        let pill_base_y = pill_y + pill_h * 0.78;
        draw_text(canvas, &assets.ui_font, &pill_text, pill_x + pill_pad, pill_base_y, pill_font_size, pill_color);
    }

    // 7. Metadatos a la derecha (Fila 1: Apertura/Jugada, Fila 2: Evento/SAN + Insignia Chess.com)
    let meta_font_size = (h * 0.28).round().clamp(11.0, 13.0);
    let top_w = measure_text(&assets.ui_font, top_meta, meta_font_size);
    let right_pad = 14.0;
    let top_x = x + w - top_w - right_pad;
    let top_baseline = y + h * 0.44;
    let top_color = SkColor::from_rgba8(195, 205, 225, 255);
    draw_text(canvas, &assets.ui_font, top_meta, top_x, top_baseline, meta_font_size, top_color);

    if let Some(sub_text) = sub_meta {
        let sub_font_size = meta_font_size - 1.0;
        let sub_w = measure_text(&assets.ui_font, sub_text, sub_font_size);

        if let Some(cls) = classification {
            let label = cls.label();
            let badge_font_size = 10.0;
            let label_tw = measure_text(&assets.ui_font, label, badge_font_size);

            let icon_size = 15.0;
            let badge_pad_x = 6.0;
            let icon_text_gap = 5.0;
            let badge_w = badge_pad_x + icon_size + icon_text_gap + label_tw + badge_pad_x;
            let badge_h = 19.0;

            let badge_x = x + w - badge_w - right_pad;
            let badge_y = y + h * 0.58;

            let mut badge_bg = Paint::default();
            badge_bg.set_color(cls.bg_color());
            let mut pb_badge = PathBuilder::new();
            add_rounded_rect(&mut pb_badge, badge_x, badge_y, badge_w, badge_h, 4.5);
            if let Some(path) = pb_badge.finish() {
                canvas.fill_path(&path, &badge_bg, tiny_skia::FillRule::Winding, Transform::identity(), None);
            }

            // Dibujar el icono oficial Chess.com
            let badge_pixmap = assets.get_badge(cls);
            let scale = icon_size / badge_pixmap.width() as f32;
            let icon_x = badge_x + badge_pad_x;
            let icon_y = badge_y + (badge_h - icon_size) / 2.0;
            let transform = Transform::from_scale(scale, scale).post_translate(icon_x, icon_y);

            canvas.draw_pixmap(
                0,
                0,
                badge_pixmap.as_ref(),
                &PixmapPaint {
                    opacity: 1.0,
                    blend_mode: tiny_skia::BlendMode::SourceOver,
                    quality: tiny_skia::FilterQuality::Bilinear,
                },
                transform,
                None,
            );

            let badge_base_y = badge_y + badge_h * 0.74;
            draw_text(
                canvas,
                &assets.ui_font,
                label,
                icon_x + icon_size + icon_text_gap,
                badge_base_y,
                badge_font_size,
                cls.text_color(),
            );

            let sub_x = badge_x - sub_w - 8.0;
            let sub_baseline = y + h * 0.78;
            let sub_color = SkColor::from_rgba8(130, 142, 168, 255);
            draw_text(canvas, &assets.ui_font, sub_text, sub_x, sub_baseline, sub_font_size, sub_color);
        } else {
            let sub_x = x + w - sub_w - right_pad;
            let sub_baseline = y + h * 0.78;
            let sub_color = SkColor::from_rgba8(130, 142, 168, 255);
            draw_text(canvas, &assets.ui_font, sub_text, sub_x, sub_baseline, sub_font_size, sub_color);
        }
    }
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

    // 2. Calcular tamaño fijo del tablero y de la barra de evaluación
    let max_avail = (width * 0.90).min(height * 0.74); // Reserva vertical para badges
    let target_board_dim = desired_board_size.min(max_avail);
    let tile_size = (target_board_dim / 8.0).floor().max(16.0);
    let board_total_w = tile_size * 8.0;
    let board_total_h = tile_size * 8.0;

    let eval_bar_w = (tile_size * 0.28).clamp(15.0, 22.0);
    let eval_bar_gap = 10.0;
    let total_content_w = eval_bar_w + eval_bar_gap + board_total_w;

    let content_x = ((width - total_content_w) / 2.0).floor();
    let eval_bar_x = content_x;
    let board_x = content_x + eval_bar_w + eval_bar_gap;
    let board_y = ((height - board_total_h) / 2.0).floor();

    // 3. Marco flotante del tablero con bordes redondeados y sombra suave
    let frame_pad = (tile_size * 0.26).floor().max(18.0);
    let frame_x = content_x - frame_pad;
    let frame_y = board_y - frame_pad;
    let frame_w = total_content_w + frame_pad * 2.0;
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

    // 3.1. Dibujar la Barra de Evaluación dinámica animada
    draw_eval_bar(
        canvas,
        assets,
        sim,
        eval_bar_x,
        board_y,
        eval_bar_w,
        board_total_h,
    );

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

    // 9.1. Badge de jugada oficial de Chess.com sobre la casilla de destino
    if let (Some((_, to)), Some(cls)) = (sim.last_move, sim.current_classification) {
        if cls.shows_on_board() {
            let to_f = to.0 as f32;
            let to_r = to.1 as f32;

            // En Chess.com el badge mide ~32% del ancho de la casilla
            let badge_size = (tile_size * 0.32).round().clamp(18.0, 32.0);
            let bx = board_x + to_f * tile_size + tile_size - badge_size - 1.0;
            let by = board_y + (7.0 - to_r) * tile_size + 1.0;

            let badge_pixmap = assets.get_badge(cls);
            let scale_x = badge_size / badge_pixmap.width() as f32;
            let scale_y = badge_size / badge_pixmap.height() as f32;

            let transform = Transform::from_scale(scale_x, scale_y)
                .post_translate(bx, by);

            canvas.draw_pixmap(
                0,
                0,
                badge_pixmap.as_ref(),
                &PixmapPaint {
                    opacity: 1.0,
                    blend_mode: tiny_skia::BlendMode::SourceOver,
                    quality: tiny_skia::FilterQuality::Bilinear,
                },
                transform,
                None,
            );
        }
    }

    // 10. Badges de jugadores estilo retransmisión
    let badge_h = (tile_size * 0.58).floor().clamp(44.0, 50.0);
    let badge_pad = 12.0;
    let badge_w = frame_w;
    let badge_x = frame_x;

    let top_badge_y = frame_y - badge_h - badge_pad;
    let bot_badge_y = frame_y + frame_h + badge_pad;

    // Solo dibujar si la ventana tiene altura suficiente
    if top_badge_y >= 4.0 && bot_badge_y + badge_h <= height - 4.0 {
        let game = sim.current_game();
        let current_turn = sim.pos.turn();

        let (is_black_turn, is_white_turn) = match sim.state {
            SimState::GameOver { .. } => (false, false),
            _ => (current_turn == Color::Black, current_turn == Color::White),
        };

        let last_move_was_white = current_turn == Color::Black;
        let white_cls = if last_move_was_white { sim.current_classification } else { None };
        let black_cls = if !last_move_was_white { sim.current_classification } else { None };

        let (white_cap, black_cap, diff) = sim.captured_pieces();
        let white_advantage = diff.max(0);
        let black_advantage = (-diff).max(0);

        // Badge superior: Jugador de Negras
        let top_meta = match (&game.eco, &game.opening) {
            (Some(eco), Some(op)) => format!("{} · {}", eco, op),
            (Some(eco), None) => format!("ECO {}", eco),
            (None, Some(op)) => op.clone(),
            (None, None) => format!("{} ({})", game.event, game.year),
        };
        let top_sub_meta = if game.eco.is_some() || game.opening.is_some() {
            Some(format!("{} ({})", game.event, game.year))
        } else {
            None
        };

        draw_player_badge(
            canvas,
            assets,
            sim,
            Color::Black,
            &game.black,
            game.black_elo.as_deref(),
            &top_meta,
            top_sub_meta.as_deref(),
            black_cls,
            &black_cap,
            black_advantage,
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
        let bot_sub_meta = if let Some(last_san) = sim.current_move_san() {
            Some(format!("Última: {}", last_san))
        } else {
            Some("Comienzo de partida".to_string())
        };

        draw_player_badge(
            canvas,
            assets,
            sim,
            Color::White,
            &game.white,
            game.white_elo.as_deref(),
            &bot_meta,
            bot_sub_meta.as_deref(),
            white_cls,
            &white_cap,
            white_advantage,
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
