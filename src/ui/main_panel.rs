use eframe::egui;
use crate::i18n::tr;
use image::GenericImageView;
use std::time::Instant;

use crate::app::Oxide;
use crate::constants::{CHIP8_H, CHIP8_W, PIXEL_BASE};

// EN: Draw CHIP-8 framebuffer in the central panel, centered in window/fullscreen.
// FR: Dessine le framebuffer CHIP-8 dans le panneau central, centre en fenetre/plein ecran.
pub fn show(ui: &mut egui::Ui, app: &mut Oxide) {
    let avail = ui.available_size();
    let (pixel_size, screen_width, screen_height, offset_x, offset_y) = if app.fullscreen_active {
        let scale_x = (avail.x / CHIP8_W).floor();
        let scale_y = (avail.y / CHIP8_H).floor();
        let p = scale_x.min(scale_y).max(1.0);
        let screen_width = CHIP8_W * p;
        let screen_height = CHIP8_H * p;
        let offset_x = (avail.x - screen_width) / 2.0;
        let offset_y = (avail.y - screen_height) / 2.0;
        (p, screen_width, screen_height, offset_x, offset_y)
    } else {
        let target = app.video_scale as f32 * PIXEL_BASE;
        let max_w = avail.x / CHIP8_W;
        let max_h = avail.y / CHIP8_H;
        let p = target.min(max_w.min(max_h)).max(1.0);
        let screen_width = CHIP8_W * p;
        let screen_height = CHIP8_H * p;
        let offset_x = ((avail.x - screen_width) / 2.0).max(0.0);
        let offset_y = if app.window_maximized {
            ((avail.y - screen_height) / 2.0).max(0.0)
        } else {
            0.0
        };
        (p, screen_width, screen_height, offset_x, offset_y)
    };

    let (response, painter) = ui.allocate_painter(
        egui::vec2(ui.available_width(), ui.available_height()),
        egui::Sense::click(),
    );

    let origin = response.rect.min + egui::vec2(offset_x, offset_y);
    // Background color: grey only for windowed fullscreen, black otherwise.
    let bg_color = if app.window_maximized && !app.fullscreen_active {
        egui::Color32::from_gray(45)
    } else {
        egui::Color32::BLACK
    };
    painter.rect_filled(response.rect, 0.0, bg_color);
    let display_rect = egui::Rect::from_min_size(origin, egui::vec2(screen_width, screen_height));
    let hover_pos = ui.ctx().input(|i| i.pointer.hover_pos());
    let pointer_moved = ui.ctx().input(|i| i.pointer.delta().length_sq() > 0.0);
    let now = Instant::now();
    app.cursor_over_display = hover_pos.map(|p| display_rect.contains(p)).unwrap_or(false);
    if ui.ctx().wants_pointer_input() {
        app.cursor_over_display = false;
    }
    if pointer_moved {
        app.last_mouse_move = now;
        app.cursor_hidden = false;
    }
    if app.cursor_over_display {
        if now.duration_since(app.last_mouse_move).as_secs_f32() >= 2.0 {
            app.cursor_hidden = true;
        }
    } else {
        app.cursor_hidden = false;
    }

    if response.double_clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            if display_rect.contains(pos) {
                app.set_fullscreen(!app.fullscreen);
            }
        }
    }

    // EN: Paint each CHIP-8 pixel as a filled rectangle.
    // FR: Dessine chaque pixel CHIP-8 comme un rectangle rempli.
    for y in 0..32 {
        for x in 0..64 {
            let color = if app.display.get_pixel(x, y) {
                egui::Color32::WHITE
            } else {
                egui::Color32::BLACK
            };

            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(
                        origin.x + x as f32 * pixel_size,
                        origin.y + y as f32 * pixel_size,
                    ),
                    egui::vec2(pixel_size, pixel_size),
                ),
                0.0,
                color,
            );
        }
    }

    let t = tr(app.langue);
    let pause_active = app.en_pause;
    let show_overlay = pause_active
        || (!app.display_overlay_message.is_empty() && Instant::now() < app.display_overlay_until);
    if show_overlay {
        let overlay_text = if pause_active {
            t.status_paused.as_str()
        } else {
            app.display_overlay_message.as_str()
        };
        if pause_active {
            if app.pause_overlay_texture.is_none() {
                let png_bytes = include_bytes!("../assets/icons/Pause.png");
                if let Ok(image) = image::load_from_memory(png_bytes) {
                    let (w, h) = image.dimensions();
                    let rgba = image.to_rgba8();
                    let pixels = rgba.as_flat_samples();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        [w as usize, h as usize],
                        pixels.as_slice(),
                    );
                    let texture = ui.ctx().load_texture(
                        "pause_overlay",
                        color_image,
                        egui::TextureOptions::LINEAR,
                    );
                    app.pause_overlay_texture = Some(texture);
                    app.pause_overlay_size = egui::vec2(w as f32, h as f32);
                }
            }
            if let Some(texture) = &app.pause_overlay_texture {
                let mut size = app.pause_overlay_size;
                let max_w = (screen_width - 16.0).max(40.0);
                let max_h = (screen_width * 0.25).max(32.0);
                let scale = (max_w / size.x).min(max_h / size.y).min(1.0);
                size *= scale;
                let image_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        origin.x + (screen_width - size.x) / 2.0,
                        origin.y + (screen_height - size.y) / 2.0,
                    ),
                    size,
                );
                painter.image(
                    texture.id(),
                    image_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
        }
        let font_id = egui::FontId::proportional(14.0);
        let text_color = if app.theme == crate::types::AppTheme::Kiwano || app.theme == crate::types::AppTheme::Dark {
            egui::Color32::from_gray(240)
        } else {
            egui::Color32::from_gray(25)
        };
        let text_size = ui
            .painter()
            .layout_no_wrap(overlay_text.to_owned(), font_id.clone(), text_color)
            .size();
        let padding_x = 14.0;
        let padding_y = 6.0;
        let overlay_size = egui::vec2(
            (text_size.x + padding_x * 2.0).min(screen_width - 16.0).max(80.0),
            (text_size.y + padding_y * 2.0).max(22.0),
        );
        let overlay_rect = egui::Rect::from_min_size(
            egui::pos2(origin.x + (screen_width - overlay_size.x) / 2.0, origin.y + 8.0),
            overlay_size,
        );
        let (bg, fg) = if app.theme == crate::types::AppTheme::Kiwano || app.theme == crate::types::AppTheme::Dark {
            (
                egui::Color32::from_rgba_unmultiplied(48, 12, 16, 230),
                egui::Color32::from_gray(240),
            )
        } else {
            (
                egui::Color32::from_rgba_unmultiplied(245, 245, 245, 220),
                egui::Color32::from_gray(25),
            )
        };

        painter.rect_filled(overlay_rect, 4.0, bg);
        painter.rect_stroke(
            overlay_rect,
            4.0,
            egui::Stroke::new(1.0, egui::Color32::from_gray(110)),
            egui::StrokeKind::Outside,
        );
        painter.text(
            overlay_rect.center(),
            egui::Align2::CENTER_CENTER,
            overlay_text,
            font_id,
            fg,
        );
    }
}






