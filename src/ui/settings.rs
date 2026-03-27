use eframe::egui::{self};
use eframe::egui::text::{LayoutJob, TextFormat};

use crate::app::{visuals_for_theme, Oxide};
use crate::i18n::tr;
use crate::types::{AppTheme, Langue, OngletsSettings, QuirkPreset};
fn reset_all_button_text(ui: &egui::Ui, label: &str) -> LayoutJob {
    let mut job = LayoutJob::default();
    let body_color = ui.visuals().text_color();
    let plain = TextFormat {
        color: body_color,
        ..Default::default()
    };
    let underlined = TextFormat {
        color: body_color,
        underline: egui::Stroke::new(1.0, body_color),
        ..Default::default()
    };
    job.append("\u{26A0} ", 0.0, plain.clone());
    job.append(label, 0.0, underlined);
    job.append(" \u{26A0}", 0.0, plain);
    job
}


fn theme_label(theme: AppTheme, t: &crate::i18n::Translations) -> String {
    let raw = match theme {
        AppTheme::Kiwano => &t.terminal_kiwano,
        AppTheme::Dark => &t.terminal_dark,
        AppTheme::Light => &t.terminal_light,
    };
    let mut label = {
        let mut chars = raw.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    };
    if theme == AppTheme::Kiwano {
        label.push_str(" ");
        label.push_str(&t.theme_default_suffix);
    }
    label
}

// EN: Render and manage the detached settings viewport.
// FR: Affiche et gere la viewport detachee des parametres.
pub fn show(ctx: &egui::Context, app: &mut Oxide) {
    let t = tr(app.langue);
    let text_color = if app.theme == AppTheme::Kiwano || app.theme == AppTheme::Dark {
        egui::Color32::from_gray(235)
    } else {
        egui::Color32::from_gray(20)
    };
    let center_x = app.fenetre_principale_pos.x + app.fenetre_principale_size.x / 2.0;
    let center_y = app.fenetre_principale_pos.y + app.fenetre_principale_size.y / 2.0;
    let settings_pos = egui::pos2(center_x - 280.0, center_y - 300.0);

    let mut viewport_builder = egui::ViewportBuilder::default()
        .with_title(t.settings_title.clone())
        .with_inner_size([620.0, 600.0]);
    if !app.settings_position_initialized {
        viewport_builder = viewport_builder.with_position([settings_pos.x, settings_pos.y]);
    }

    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("settings_window"),
        viewport_builder,
        |ctx, _| {
            ctx.set_visuals(visuals_for_theme(app.theme));
            if app.focus_settings_requested {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                app.focus_settings_requested = false;
            }
            if !app.settings_position_initialized {
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(settings_pos));
                // EN: Sync temp + snapshot values on first open to avoid pending-change warning.
                // FR: Synchronise temp + snapshot a l ouverture pour eviter un faux avertissement.
                app.snapshot_theme = app.theme;
                app.snapshot_langue = app.langue;
                app.snapshot_vsync = app.vsync;
                app.snapshot_video_scale = app.video_scale;
                app.snapshot_touches = app.touches.clone();
                app.snapshot_raccourcis = app.raccourcis.clone();
                app.snapshot_cycles_par_seconde = app.cycles_par_seconde;
                app.snapshot_son_active = app.son_active;
                app.snapshot_sound_volume = app.sound_volume;
                app.snapshot_quirks = app.quirks;
                app.snapshot_quirks_preset = app.quirks_preset;
                app.snapshot_terminal_active = app.terminal_active;

                app.temp_theme = app.theme;
                app.temp_langue = app.langue;
                app.temp_vsync = app.vsync;
                app.temp_video_scale = app.video_scale;
                app.temp_touches = app.touches.clone();
                app.temp_raccourcis = app.raccourcis.clone();
                app.temp_cycles_par_seconde = app.cycles_par_seconde;
                app.temp_son_active = app.son_active;
                app.temp_sound_volume = app.sound_volume;
                app.temp_quirks = app.quirks;
                app.temp_quirks_preset = app.quirks_preset;
                app.temp_terminal_active = app.terminal_active;
            }

            ctx.style_mut(|style| {
                style.wrap_mode = Some(egui::TextWrapMode::Extend);
                style.visuals.override_text_color = Some(text_color);
            });

            let has_pending_changes = app.temp_theme != app.theme
                || app.temp_langue != app.langue
                || app.temp_cycles_par_seconde != app.cycles_par_seconde
                || app.temp_vsync != app.vsync
                || app.temp_video_scale != app.video_scale
                || app.temp_son_active != app.son_active
                || app.temp_sound_volume != app.sound_volume
                || app.temp_touches != app.touches
                || app.temp_raccourcis != app.raccourcis
                || app.temp_terminal_active != app.terminal_active
                || app.temp_quirks != app.quirks
                || app.temp_quirks_preset != app.quirks_preset;

            egui::TopBottomPanel::bottom("settings_boutons").show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.button(&t.ok).clicked() {
                        app.apply_temp_values();
                        app.fenetre_settings = false;
                        app.settings_position_initialized = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button(&t.apply).clicked() {
                        app.apply_temp_values();
                    }
                    if ui.button(&t.defaults).clicked() {
                        app.reset_current_settings_tab_to_default();
                    }
                    if has_pending_changes {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(&t.settings_pending_apply)
                                .color(egui::Color32::from_rgb(220, 180, 60))
                                .strong(),
                        );
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(&t.cancel).clicked() {
                            app.restore_snapshots();
                            app.fenetre_settings = false;
                            app.settings_position_initialized = false;
                            app.confirm_reset_all = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
                ui.add_space(4.0);
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut app.onglet_settings,
                        OngletsSettings::Emulateur,
                        t.tab_emulator.clone(),
                    );
                    ui.selectable_value(
                        &mut app.onglet_settings,
                        OngletsSettings::Video,
                        t.tab_video.clone(),
                    );
                    ui.selectable_value(
                        &mut app.onglet_settings,
                        OngletsSettings::Audio,
                        t.tab_audio.clone(),
                    );
                    ui.selectable_value(
                        &mut app.onglet_settings,
                        OngletsSettings::Controles,
                        t.tab_controls.clone(),
                    );
                    ui.selectable_value(
                        &mut app.onglet_settings,
                        OngletsSettings::Raccourcis,
                        t.tab_shortcuts.clone(),
                    );
                    ui.selectable_value(
                        &mut app.onglet_settings,
                        OngletsSettings::Debug,
                        t.tab_debug.clone(),
                    );


                });

                ui.separator();

                match app.onglet_settings {
                    OngletsSettings::Emulateur => show_emulateur_settings(ui, app, t),
                    OngletsSettings::Video => show_video_settings(ui, app, t),
                    OngletsSettings::Audio => show_audio_settings(ui, app, t),
                    OngletsSettings::Controles => show_controles_settings(ui, app, t),
                    OngletsSettings::Raccourcis => show_raccourcis_settings(ui, app, t),
                    OngletsSettings::Debug => show_debug_settings(ui, app, t),
                }
            });

            if ctx.input(|i| i.viewport().close_requested()) {
                app.fenetre_settings = false;
                app.settings_position_initialized = false;
                app.confirm_reset_all = false;
            } else {
                app.settings_position_initialized = true;
            }
        },
    );
}

// EN: Emulation settings section (language, CPU speed, sound toggle).
// FR: Section des parametres d emulation (langue, vitesse CPU, activation du son).
fn show_emulateur_settings(ui: &mut egui::Ui, app: &mut Oxide, t: &crate::i18n::Translations) {
    ui.label(&t.theme);
    egui::ComboBox::from_id_salt("theme_settings")
        .selected_text(theme_label(app.temp_theme, t))
        .show_ui(ui, |ui| {
            for theme in AppTheme::ALL {
                ui.selectable_value(&mut app.temp_theme, theme, theme_label(theme, t));
            }
        });

    ui.add_space(8.0);
    ui.label(&t.language);

    let before = app.temp_langue;
    egui::ComboBox::from_id_salt("langue_settings")
        .selected_text(app.temp_langue.label())
        .show_ui(ui, |ui| {
            for language in Langue::ALL {
                ui.selectable_value(&mut app.temp_langue, language, language.label());
            }
        });
    if app.temp_langue != before {
        app.langue = app.temp_langue;
    }

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(&t.settings_cpu_hz);
        ui.add_enabled(
            !app.temp_vsync,
            egui::Slider::new(&mut app.temp_cycles_par_seconde, 60..=2000),
        );
    });
    if app.temp_vsync {
        ui.add_space(4.0);
        ui.label(egui::RichText::new(&t.settings_cpu_hz_note).color(egui::Color32::from_gray(140)));
    }
    ui.add_space(8.0);
    let mut warning_job = reset_all_button_text(ui, &t.defaults_all);
    for section in &mut warning_job.sections {
        section.format.font_id = egui::TextStyle::Button.resolve(ui.style());
    }
    if ui.add(egui::Button::new(warning_job)).clicked() {
        app.confirm_reset_all = true;
    }
    if app.confirm_reset_all {
        let modal = egui::Window::new(&t.defaults_all)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]);
        modal.show(ui.ctx(), |ui| {
            ui.label(&t.confirm_reset_all);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(&t.confirm_yes).clicked() {
                    app.reset_all_settings_to_default();
                    app.apply_temp_values();
                    ui.ctx().request_repaint();
                    app.confirm_reset_all = false;
                }
                if ui.button(&t.cancel).clicked() {
                    app.confirm_reset_all = false;
                }
            });
        });
    }
}

// EN: Video settings section (vsync, aspect and scale).
// FR: Section des parametres video (vsync, ratio et echelle).
fn show_video_settings(ui: &mut egui::Ui, app: &mut Oxide, t: &crate::i18n::Translations) {
    ui.label(&t.video_config);
    ui.checkbox(&mut app.temp_vsync, &t.vsync);
    ui.label(&t.render_size);
    egui::ComboBox::from_id_salt("video_scale")
        .selected_text(format!("{}x", app.temp_video_scale))
        .show_ui(ui, |ui| {
            for scale in [1u8, 2, 3, 4, 5] {
                ui.selectable_value(&mut app.temp_video_scale, scale, format!("{}x", scale));
            }
        });
}

// EN: Audio settings section (sound toggle + volume).
// FR: Section des parametres audio (activation du son + volume).
fn show_audio_settings(ui: &mut egui::Ui, app: &mut Oxide, t: &crate::i18n::Translations) {
    ui.label(&t.audio_config);
    ui.checkbox(&mut app.temp_son_active, &t.settings_sound);
    ui.horizontal(|ui| {
        ui.label(&t.settings_volume);
        ui.add(egui::Slider::new(&mut app.temp_sound_volume, 0..=100));
    });
}// EN: Controls settings ? click a cell to bind next key/mouse input to that CHIP-8 key.
// FR: Parametres controles - cliquer une cellule pour binder la prochaine touche/souris a cette touche CHIP-8.
fn show_controles_settings(ui: &mut egui::Ui, app: &mut Oxide, t: &crate::i18n::Translations) {
    const CHIP8_VISUAL_ORDER: [[usize; 4]; 4] = [
        [0x1, 0x2, 0x3, 0xC],
        [0x4, 0x5, 0x6, 0xD],
        [0x7, 0x8, 0x9, 0xE],
        [0xA, 0x0, 0xB, 0xF],
    ];

    let cell_width = 100.0;
    let grid_width = cell_width * 4.0;
    let available_width = ui.available_width();
    let offset_x = (available_width - grid_width) / 2.0;

    // EN: If waiting for input, check timeout then capture next key/mouse press.
    // FR: Si en attente d'input, verifie le timeout puis capture la prochaine touche/souris.
    if let Some(binding_idx) = app.binding_key {
        if let Some(started) = app.binding_key_started {
            if started.elapsed().as_secs() >= 3 {
                app.binding_key = None;
                app.binding_key_started = None;
                app.binding_key_skip_first_click = false;
            }
        }

        if app.binding_key.is_some() {
            ui.input(|i| {
                for event in &i.events {
                    match event {
                        egui::Event::Key { key, pressed: true, .. } => {
                            let raw = format!("{:?}", key);
                            let label = if let Some(n) = raw.strip_prefix("Num") {
                                n.to_string()
                            } else {
                                raw
                            };
                            app.temp_touches[binding_idx] = label;
                            app.binding_key = None;
                            app.binding_key_started = None;
                            app.binding_key_skip_first_click = false;
                        }
                        egui::Event::PointerButton { button, pressed: true, .. } => {
                            // EN: Skip the first click that triggered the binding.
                            // FR: Ignore le premier clic qui a declenche le binding.
                            if app.binding_key_skip_first_click {
                                app.binding_key_skip_first_click = false;
                            } else {
                                let label = match button {
                                    egui::PointerButton::Primary   => "MouseLeft",
                                    egui::PointerButton::Secondary => "MouseRight",
                                    egui::PointerButton::Middle    => "MouseMiddle",
                                    egui::PointerButton::Extra1    => "MouseExtra1",
                                    egui::PointerButton::Extra2    => "MouseExtra2",
                                };
                                app.temp_touches[binding_idx] = label.to_string();
                                app.binding_key = None;
                                app.binding_key_started = None;
                            }
                        }
                        _ => {}
                    }
                }
            });
        }
    }

    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 8.0);
        ui.label(egui::RichText::new(&t.key_config).size(18.0));
        ui.add_space(8.0);
    });

    for row in &CHIP8_VISUAL_ORDER {
        ui.horizontal(|ui| {
            ui.add_space(offset_x);
            for &key in row {
                ui.vertical(|ui| {
                    ui.set_min_width(cell_width);
                    ui.label(egui::RichText::new(format!("{:X}", key)).size(16.0));

                    let is_binding = app.binding_key == Some(key);
                    let label = if is_binding {
                        "...".to_string()
                    } else {
                        app.temp_touches[key].clone()
                    };

                    let btn = egui::Button::new(label).min_size(egui::vec2(cell_width - 8.0, 32.0));
                    if ui.add(btn).clicked() {
                        app.binding_key = Some(key);
                        app.binding_key_started = Some(std::time::Instant::now());
                        app.binding_key_skip_first_click = true;
                    }
                });
            }
        });
        ui.add_space(4.0);
    }

    // EN: Request repaint while binding is active to update timeout check.
    // FR: Demande un repaint pendant le binding pour verifier le timeout.
    if app.binding_key.is_some() {
        ui.ctx().request_repaint();
    }
}

// EN: Keyboard shortcut mapping section.
// FR: Section de mapping des raccourcis clavier.
// EN: Build a standardized shortcut label from modifiers + key.
// FR: Construit un libelle de raccourci standardise a partir des modificateurs + touche.
fn format_shortcut(mods: &egui::Modifiers, key: egui::Key) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if mods.ctrl {
        parts.push("Ctrl");
    }
    if mods.shift {
        parts.push("Shift");
    }
    if mods.alt {
        parts.push("Alt");
    }
    parts.push(shortcut_key_label(key));
    parts.join("+")
}

// EN: Convert an egui::Key into a human-readable label.
// FR: Convertit une egui::Key en libelle lisible.
fn shortcut_key_label(key: egui::Key) -> &'static str {
    match key {
        egui::Key::ArrowUp => "Up",
        egui::Key::ArrowDown => "Down",
        egui::Key::ArrowLeft => "Left",
        egui::Key::ArrowRight => "Right",
        egui::Key::Escape => "Esc",
        egui::Key::Enter => "Enter",
        egui::Key::Tab => "Tab",
        egui::Key::Space => "Space",
        egui::Key::Backspace => "Backspace",
        egui::Key::Insert => "Insert",
        egui::Key::Delete => "Delete",
        egui::Key::Home => "Home",
        egui::Key::End => "End",
        egui::Key::PageUp => "PageUp",
        egui::Key::PageDown => "PageDown",
        egui::Key::Minus => "-",
        egui::Key::Plus => "+",
        egui::Key::Equals => "=",
        egui::Key::A => "A",
        egui::Key::B => "B",
        egui::Key::C => "C",
        egui::Key::D => "D",
        egui::Key::E => "E",
        egui::Key::F => "F",
        egui::Key::G => "G",
        egui::Key::H => "H",
        egui::Key::I => "I",
        egui::Key::J => "J",
        egui::Key::K => "K",
        egui::Key::L => "L",
        egui::Key::M => "M",
        egui::Key::N => "N",
        egui::Key::O => "O",
        egui::Key::P => "P",
        egui::Key::Q => "Q",
        egui::Key::R => "R",
        egui::Key::S => "S",
        egui::Key::T => "T",
        egui::Key::U => "U",
        egui::Key::V => "V",
        egui::Key::W => "W",
        egui::Key::X => "X",
        egui::Key::Y => "Y",
        egui::Key::Z => "Z",
        egui::Key::Num0 => "0",
        egui::Key::Num1 => "1",
        egui::Key::Num2 => "2",
        egui::Key::Num3 => "3",
        egui::Key::Num4 => "4",
        egui::Key::Num5 => "5",
        egui::Key::Num6 => "6",
        egui::Key::Num7 => "7",
        egui::Key::Num8 => "8",
        egui::Key::Num9 => "9",
        egui::Key::F1 => "F1",
        egui::Key::F2 => "F2",
        egui::Key::F3 => "F3",
        egui::Key::F4 => "F4",
        egui::Key::F5 => "F5",
        egui::Key::F6 => "F6",
        egui::Key::F7 => "F7",
        egui::Key::F8 => "F8",
        egui::Key::F9 => "F9",
        egui::Key::F10 => "F10",
        egui::Key::F11 => "F11",
        egui::Key::F12 => "F12",
        _ => "?",
    }
}

// EN: Map shortcut index to mutable field.
// FR: Mappe un index de raccourci vers le champ mutable.
fn shortcut_value_mut<'a>(r: &'a mut crate::types::Raccourcis, idx: usize) -> Option<&'a mut String> {
    match idx {
        0 => Some(&mut r.pause),
        1 => Some(&mut r.reset),
        2 => Some(&mut r.stop),
        3 => Some(&mut r.charger_jeu),
        4 => Some(&mut r.fullscreen),
        5 => Some(&mut r.savestate_1),
        6 => Some(&mut r.savestate_2),
        7 => Some(&mut r.savestate_3),
        8 => Some(&mut r.loadstate_1),
        9 => Some(&mut r.loadstate_2),
        10 => Some(&mut r.loadstate_3),
        _ => None,
    }
}

// EN: Keyboard shortcut mapping section.
// FR: Section de mapping des raccourcis clavier.
fn show_raccourcis_settings(ui: &mut egui::Ui, app: &mut Oxide, t: &crate::i18n::Translations) {
    ui.label(&t.keyboard_shortcuts);
    let raccourcis = &mut app.temp_raccourcis;
    let cell_width = 220.0;

    // EN: If waiting for shortcut input, check timeout then capture next key press.
    // FR: Si en attente d'input, verifie le timeout puis capture la prochaine touche.
    if let Some(binding_idx) = app.binding_shortcut {
        if let Some(started) = app.binding_shortcut_started {
            if started.elapsed().as_secs() >= 3 {
                app.binding_shortcut = None;
                app.binding_shortcut_started = None;
                app.binding_shortcut_skip_first_click = false;
            }
        }

        if app.binding_shortcut.is_some() {
            ui.input(|i| {
                for event in &i.events {
                    if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                        let label = format_shortcut(modifiers, *key);
                        if let Some(target) = shortcut_value_mut(raccourcis, binding_idx) {
                            *target = label;
                        }
                        app.binding_shortcut = None;
                        app.binding_shortcut_started = None;
                        app.binding_shortcut_skip_first_click = false;
                        break;
                    }
                }
            });
        }
    }

    let rows = [
        (&t.shortcut_pause, 0usize),
        (&t.shortcut_reset, 1),
        (&t.shortcut_stop, 2),
        (&t.shortcut_load_game, 3),
        (&t.shortcut_fullscreen, 4),
        (&t.shortcut_save_slot_1, 5),
        (&t.shortcut_save_slot_2, 6),
        (&t.shortcut_save_slot_3, 7),
        (&t.shortcut_load_slot_1, 8),
        (&t.shortcut_load_slot_2, 9),
        (&t.shortcut_load_slot_3, 10),
    ];

    egui::Grid::new("raccourcis_grid")
        .num_columns(2)
        .spacing([24.0, 6.0])
        .show(ui, |ui| {
            for (label, idx) in rows {
                ui.label(label);
                let is_binding = app.binding_shortcut == Some(idx);
                let current = shortcut_value_mut(raccourcis, idx).map(|v| v.clone()).unwrap_or_default();
                let button_label = if is_binding { "..." } else { current.as_str() };
                let btn = egui::Button::new(button_label).min_size(egui::vec2(cell_width, 32.0));
                if ui.add(btn).clicked() {
                    app.binding_key = None;
                    app.binding_key_started = None;
                    app.binding_key_skip_first_click = false;
                    app.binding_shortcut = Some(idx);
                    app.binding_shortcut_started = Some(std::time::Instant::now());
                    app.binding_shortcut_skip_first_click = true;
                }
                ui.end_row();
            }
        });

    if app.binding_shortcut.is_some() {
        ui.ctx().request_repaint();
    }
}

// EN: Debug-specific settings section.
// FR: Section des parametres specifiques au debug.
fn show_debug_settings(ui: &mut egui::Ui, app: &mut Oxide, t: &crate::i18n::Translations) {
    ui.checkbox(&mut app.temp_terminal_active, &t.settings_debug_terminal);
    ui.label(&t.compat_quirks_title);
    ui.horizontal(|ui| {
        ui.label(&t.quirks_preset_label);
        let before_preset = app.temp_quirks_preset;
        egui::ComboBox::from_id_salt("quirks_preset")
            .selected_text(match app.temp_quirks_preset {
                QuirkPreset::Chip8 => t.quirks_preset_chip8.as_str(),
                QuirkPreset::Chip48 => t.quirks_preset_chip48.as_str(),
                QuirkPreset::SuperChip => t.quirks_preset_superchip.as_str(),
                QuirkPreset::Custom => t.quirks_preset_custom.as_str(),
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut app.temp_quirks_preset, QuirkPreset::Chip8, &t.quirks_preset_chip8);
                ui.selectable_value(&mut app.temp_quirks_preset, QuirkPreset::Chip48, &t.quirks_preset_chip48);
                ui.selectable_value(
                    &mut app.temp_quirks_preset,
                    QuirkPreset::SuperChip,
                    &t.quirks_preset_superchip,
                );
                ui.selectable_value(&mut app.temp_quirks_preset, QuirkPreset::Custom, &t.quirks_preset_custom);
            });
        if app.temp_quirks_preset != before_preset {
            app.set_temp_quirks_preset(app.temp_quirks_preset);
        }
    });
    let before_quirks = app.temp_quirks;
    ui.checkbox(&mut app.temp_quirks.shift_uses_vy, &t.quirks_shift_uses_vy);
    ui.checkbox(&mut app.temp_quirks.jump_uses_vx, &t.quirks_jump_uses_vx);
    ui.checkbox(&mut app.temp_quirks.draw_clips, &t.quirks_draw_clips);
    ui.checkbox(&mut app.temp_quirks.load_store_increment_i, &t.quirks_load_store_increment_i);
    ui.checkbox(&mut app.temp_quirks.logic_clears_vf, &t.quirks_logic_clears_vf);
    if app.temp_quirks != before_quirks {
        app.sync_temp_quirks_preset_from_values();
    }
}
















