use eframe::egui;
use eframe::egui::text::{LayoutJob, TextFormat};
use std::fs;
use std::sync::Arc;

use crate::app::{visuals_for_theme, Oxide};
use crate::i18n::tr;
use crate::utils::key_from_label;

// EN: Build rich text layout for log lines and color enabled/disabled tokens.
// FR: Construit le layout texte riche des logs et colore les tokens actif/inactif.
fn highlighted_log_job(
    text: &str,
    enabled_word: &str,
    disabled_word: &str,
    base_color: egui::Color32,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    let base = TextFormat {
        color: base_color,
        ..Default::default()
    };
    let enabled_fmt = TextFormat {
        color: egui::Color32::from_rgb(80, 200, 120),
        ..Default::default()
    };
    let disabled_fmt = TextFormat {
        color: egui::Color32::from_rgb(220, 80, 80),
        ..Default::default()
    };

    let lower = text.to_lowercase();
    let enabled_lower = enabled_word.to_lowercase();
    let disabled_lower = disabled_word.to_lowercase();
    let mut i = 0usize;

    while i < text.len() {
        let rem = &lower[i..];
        let next_enabled = rem.find(&enabled_lower).map(|p| (i + p, true));
        let next_disabled = rem.find(&disabled_lower).map(|p| (i + p, false));

        let next = match (next_enabled, next_disabled) {
            (Some(a), Some(b)) => {
                if a.0 <= b.0 {
                    Some(a)
                } else {
                    Some(b)
                }
            }
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };

        let Some((pos, is_enabled)) = next else {
            job.append(&text[i..], 0.0, base.clone());
            break;
        };

        if pos > i {
            job.append(&text[i..pos], 0.0, base.clone());
        }

        let token = if is_enabled {
            &text[pos..pos + enabled_word.len()]
        } else {
            &text[pos..pos + disabled_word.len()]
        };
        job.append(
            token,
            0.0,
            if is_enabled {
                enabled_fmt.clone()
            } else {
                disabled_fmt.clone()
            },
        );
        i = pos + token.len();
    }

    job
}

// EN: Render detached debug terminal viewport and synchronize inputs/focus with main app.
// FR: Affiche la viewport detachee du terminal de debug et synchronise les entrees/focus avec l application principale.
pub fn show(ctx: &egui::Context, app: &mut Oxide) {
    fn shortcut_pressed_in_terminal(input: &egui::InputState, label: &str) -> bool {
        crate::utils::shortcut_pressed(input, label)
    }

    let terminal_w = 520.0;
    let terminal_h = 420.0;
    let margin = 16.0;
    let terminal_x = app.fenetre_principale_pos.x + app.fenetre_principale_size.x + margin;
    let terminal_y = app.fenetre_principale_pos.y;

    let fps_title = if app.configured_game_fps.is_finite() {
        format!("{} : {:.0}", tr(app.langue).terminal_fps, app.configured_game_fps)
    } else {
        format!("{} : {}", tr(app.langue).terminal_fps, tr(app.langue).terminal_unlimited)
    };
    let terminal_title = format!("{} - {}", tr(app.langue).terminal_title, fps_title);

    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("debug_terminal_window"),
        egui::ViewportBuilder::default()
            .with_title(terminal_title)
            .with_icon(terminal_window_icon())
            .with_inner_size([terminal_w, terminal_h])
            .with_resizable(true),
        |ctx, _| {
            ctx.set_visuals(visuals_for_theme(app.theme));
            let t = tr(app.langue);
            let just_opened = !app.terminal_position_initialized;
            let search_id = egui::Id::new("terminal_search");
            if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::F)) {
                ctx.memory_mut(|m| m.request_focus(search_id));
            }
            if just_opened {
                app.terminal_view_session = app.terminal_view_session.wrapping_add(1);
            }
            let view_session = app.terminal_view_session;
            let (bg_color, text_color) = if app.theme == crate::types::AppTheme::Kiwano || app.theme == crate::types::AppTheme::Dark {
                (ctx.style().visuals.window_fill(), egui::Color32::from_gray(228))
            } else {
                (ctx.style().visuals.window_fill(), egui::Color32::BLACK)
            };
            let mut terminal_keys = [false; 16];
            ctx.input(|i| {
                for (idx, key_name) in app.touches.iter().enumerate() {
                    if let Some(key) = key_from_label(key_name) {
                        terminal_keys[idx] = i.key_down(key);
                    }
                }
            });
            app.terminal_keypad_states = terminal_keys;
            if !ctx.wants_keyboard_input() {
                let (
                    sc_pause,
                    sc_reset,
                    sc_stop,
                    sc_load_game,
                    sc_fullscreen,
                    sc_save_1,
                    sc_save_2,
                    sc_save_3,
                    sc_load_1,
                    sc_load_2,
                    sc_load_3,
                    sc_alt_enter,
                ) = {
                    let r = &app.raccourcis;
                    ctx.input(|i| {
                        (
                            shortcut_pressed_in_terminal(i, &r.pause),
                            shortcut_pressed_in_terminal(i, &r.reset),
                            shortcut_pressed_in_terminal(i, &r.stop),
                            shortcut_pressed_in_terminal(i, &r.charger_jeu),
                            shortcut_pressed_in_terminal(i, &r.fullscreen),
                            shortcut_pressed_in_terminal(i, &r.savestate_1),
                            shortcut_pressed_in_terminal(i, &r.savestate_2),
                            shortcut_pressed_in_terminal(i, &r.savestate_3),
                            shortcut_pressed_in_terminal(i, &r.loadstate_1),
                            shortcut_pressed_in_terminal(i, &r.loadstate_2),
                            shortcut_pressed_in_terminal(i, &r.loadstate_3),
                            i.key_pressed(egui::Key::Enter) && i.modifiers.alt,
                        )
                    })
                };
                if sc_pause {
                    if app.rom_chargee {
                        app.en_pause = !app.en_pause;
                    }
                }
                if sc_reset {
                    if app.rom_chargee {
                        app.reset_rom();
                    }
                }
                if sc_stop {
                    if app.rom_chargee {
                        app.stop_emulation();
                    }
                }
                if sc_load_game {
                    app.load_rom_dialog();
                }
                if sc_fullscreen || sc_alt_enter {
                    app.fullscreen = !app.fullscreen;
                }
                if sc_save_1 {
                    if app.rom_chargee {
                        app.save_state_slot_shortcut(0);
                    }
                }
                if sc_save_2 {
                    if app.rom_chargee {
                        app.save_state_slot_shortcut(1);
                    }
                }
                if sc_save_3 {
                    if app.rom_chargee {
                        app.save_state_slot_shortcut(2);
                    }
                }
                if sc_load_1 {
                    if app.rom_chargee {
                        app.load_state_slot(0);
                    }
                }
                if sc_load_2 {
                    if app.rom_chargee {
                        app.load_state_slot(1);
                    }
                }
                if sc_load_3 {
                    if app.rom_chargee {
                        app.load_state_slot(2);
                    }
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::F9)) {
                app.emit_test_report("F9");
            }

            if !app.terminal_position_initialized {
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                    terminal_x,
                    terminal_y,
                )));
                app.terminal_position_initialized = true;
            }
            if app.focus_terminal_requested {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                app.focus_terminal_requested = false;
            }
            egui::TopBottomPanel::top("terminal_header").show(ctx, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    if ui.button(&t.terminal_test_report_btn).clicked() {
                        app.emit_test_report("Button");
                    }
                    if ui.button(&t.terminal_export_logs).clicked() {
                        let export_text = if app.terminal_logs_text.is_empty() && !app.terminal_logs.is_empty() {
                            app.terminal_logs.join("\n")
                        } else {
                            app.terminal_logs_text.clone()
                        };
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Logs", &["log", "txt"])
                            .set_file_name("Oxide-logs.txt")
                            .save_file()
                        {
                            match fs::write(&path, export_text) {
                                Ok(_) => {
                                    app.status_message = format!(
                                        "{}: {}",
                                        &t.terminal_export_success,
                                        path.display()
                                    );
                                }
                                Err(err) => {
                                    app.status_message = format!(
                                        "{}: {}",
                                        &t.terminal_export_error,
                                        err
                                    );
                                }
                            }
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_sized(
                            [220.0, ui.spacing().interact_size.y],
                            egui::TextEdit::singleline(&mut app.terminal_search_query)
                                .id(search_id)
                                .hint_text(&t.terminal_search_hint),
                        );
                    });
                });
            });

            egui::CentralPanel::default()
                .frame(
                    egui::Frame::default()
                        .fill(bg_color)
                        .inner_margin(egui::Margin::symmetric(10, 6)),
                )
                .show(ctx, |ui| {
                    if app.terminal_logs_text.is_empty() && !app.terminal_logs.is_empty() {
                        app.terminal_logs_text = app.terminal_logs.join("\n");
                    }
                    let visible_rows = 24usize;
                    let mut displayed_text = app.terminal_logs_text.clone();
                    let q = app.terminal_search_query.trim().to_lowercase();
                    if !q.is_empty() {
                        displayed_text = app
                            .terminal_logs
                            .iter()
                            .filter(|line| line.to_lowercase().contains(&q))
                            .cloned()
                            .collect::<Vec<String>>()
                            .join("\n");
                    }
                    let total_rows = displayed_text.lines().count().max(1);
                    if just_opened {
                        app.terminal_rows_at_open = total_rows;
                    }
                    // EN: Always follow the latest logs unless user is searching.
                    // FR: Suit toujours les derniers logs sauf en mode recherche.
                    let follow_bottom = q.is_empty() && total_rows > visible_rows;
                    egui::ScrollArea::vertical()
                        .id_salt(("terminal_scroll_area", view_session))
                        .auto_shrink([false, false])
                        .stick_to_bottom(follow_bottom)
                        .show(ui, |ui| {
                            ui.scope(|ui| {
                                let mut style = ui.style().as_ref().clone();
                                style.visuals.override_text_color = Some(text_color);
                                ui.set_style(style);

                                let rows = total_rows.max(visible_rows);
                                let enabled_word = tr(app.langue).terminal_enabled.clone();
                                let disabled_word = tr(app.langue).terminal_disabled.clone();
                                let mut layouter =
                                    move |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                                        let mut job = highlighted_log_job(
                                            text.as_str(),
                                            &enabled_word,
                                            &disabled_word,
                                            text_color,
                                        );
                                        job.wrap.max_width = wrap_width;
                                        ui.fonts_mut(|f| f.layout_job(job))
                                    };
                                let response = ui.add(
                                    egui::TextEdit::multiline(&mut displayed_text)
                                        .id_source(("terminal_logs_editor", view_session))
                                        .cursor_at_end(false)
                                        .font(egui::TextStyle::Monospace)
                                        .layouter(&mut layouter)
                                        .frame(false)
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(rows),
                                );
                                response.context_menu(|ui| {
                                    if ui.button(&t.terminal_copy).clicked() {
                                        ctx.copy_text(displayed_text.clone());
                                        ui.close();
                                    }
                                });
                            });
                        });
                });

            if ctx.input(|i| i.viewport().close_requested()) {
                app.terminal_active = false;
                app.temp_terminal_active = false;
                app.terminal_keypad_states = [false; 16];
            }
        },
    );
}

// EN: Build a tiny terminal-like icon for the debug viewport.
// FR: Construit une petite icone type terminal pour la viewport de debug.
fn terminal_window_icon() -> Arc<egui::IconData> {
    const W: u32 = 32;
    const H: u32 = 32;
    let mut rgba = vec![0u8; (W * H * 4) as usize];

    for y in 0..H {
        for x in 0..W {
            let i = ((y * W + x) * 4) as usize;
            let border = x == 0 || y == 0 || x == W - 1 || y == H - 1;
            let header = y < 6;
            let bg = if border {
                [30, 30, 30, 255]
            } else if header {
                [50, 50, 50, 255]
            } else {
                [12, 12, 12, 255]
            };
            rgba[i..i + 4].copy_from_slice(&bg);
        }
    }

    // EN: Draw a minimal prompt glyph (>_) in the terminal icon.
    // FR: Dessine un glyphe d invite minimal (>_) dans l icone du terminal.
    for (x, y) in [(8, 14), (10, 15), (8, 16), (11, 15), (12, 15), (13, 15), (14, 15)] {
        let i = (((y as u32) * W + (x as u32)) * 4) as usize;
        rgba[i..i + 4].copy_from_slice(&[120, 220, 120, 255]);
    }

    Arc::new(egui::IconData {
        rgba,
        width: W,
        height: H,
    })
}