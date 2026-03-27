use eframe::egui::{self};

use crate::app::Oxide;
use crate::i18n::tr;
use crate::types::{AppTheme, Langue, OngletsSettings};

// EN: Replace regular spaces with non-breaking spaces for stable menu labels.
// FR: Remplace les espaces classiques par des espaces inseparables pour stabiliser les libelles de menu.
fn nw(text: &str) -> String {
    text.replace(' ', "\u{00A0}")
}

// EN: Build label with optional shortcut hint.
// FR: Construit un libelle avec un hint de raccourci optionnel.
fn with_shortcut(label: &str, shortcut: &str) -> String {
    if shortcut.is_empty() {
        label.to_string()
    } else {
        format!("{} ({})", label, shortcut)
    }
}

// EN: Combine label + shortcut into a single padded string (shortcut right-aligned).
// FR: Combine libelle + raccourci dans une seule chaine (raccourci aligne a droite).
fn label_with_right_shortcut(ui: &egui::Ui, left: &str, right: &str, total_width: f32) -> String {
    if right.is_empty() {
        return left.to_string();
    }
    let font_id = egui::TextStyle::Button.resolve(ui.style());
    let color = ui.visuals().text_color();
    let left_w = ui
        .painter()
        .layout_no_wrap(left.to_owned(), font_id.clone(), color)
        .size()
        .x;
    let right_w = ui
        .painter()
        .layout_no_wrap(right.to_owned(), font_id.clone(), color)
        .size()
        .x;
    let space_w = ui
        .painter()
        .layout_no_wrap("\u{00A0}".to_owned(), font_id, color)
        .size()
        .x
        .max(1.0);
    let padding = ui.spacing().button_padding.x * 2.0;
    let available = (total_width - padding - left_w - right_w).max(space_w);
    let spaces = (available / space_w).floor().max(1.0) as usize;
    let spacer = "\u{00A0}".repeat(spaces);
    format!("{}{}{}", left, spacer, right)
}

// EN: Build a savestate label using metadata when available.
// FR: Construit un libelle de savestate avec metadonnees si disponibles.
fn slot_label_text(app: &Oxide, slot: usize) -> String {
    let t = tr(app.langue);
    let slot_prefix = format!("{} - ", slot + 1);
    if let Some(meta) = app.savestate_meta.get(slot).and_then(|m| m.as_ref()) {
        if meta.name.is_empty() {
            format!("{}({})", slot_prefix, t.slot_empty)
        } else {
            format!("{}{} - {}", slot_prefix, meta.timestamp, meta.name)
        }
    } else {
        format!("{}({})", slot_prefix, t.slot_empty)
    }
}

// EN: Build a width probe label for menu sizing.
// FR: Construit un libelle de mesure pour le dimensionnement.
fn slot_width_label(left: &str, right: &str) -> String {
    if right.is_empty() {
        left.to_string()
    } else {
        format!("{}  {}", left, right)
    }
}

// EN: Compute menu width from the widest label.
// FR: Calcule la largeur du menu a partir du libelle le plus large.
fn menu_width(ui: &egui::Ui, labels: &[String]) -> f32 {
    let font_id = egui::TextStyle::Button.resolve(ui.style());
    let color = ui.visuals().text_color();
    let max_text_width = labels
        .iter()
        .map(|label| {
            ui.painter()
                .layout_no_wrap(label.clone(), font_id.clone(), color)
                .size()
                .x
        })
        .fold(0.0_f32, f32::max);
    let padding = ui.spacing().button_padding.x * 2.0;
    (max_text_width + padding).max(1.0)
}

// EN: Compute menu width including icon/checkmark reserved space.
// FR: Calcule la largeur du menu en incluant l espace reserve aux icones/coches.
fn menu_width_with_extras(
    ui: &egui::Ui,
    labels: &[String],
    has_leading_icon: bool,
    has_trailing_icon: bool,
) -> f32 {
    let mut width = menu_width(ui, labels);
    if has_leading_icon {
        width += ui.spacing().icon_width + ui.spacing().icon_spacing;
    }
    if has_trailing_icon {
        width += ui.spacing().icon_width + ui.spacing().item_spacing.x;
    }
    width
}

// EN: Force a fixed width for consistent dropdown layout.
// FR: Force une largeur fixe pour une mise en page coherente du menu deroulant.
fn set_menu_width(ui: &mut egui::Ui, width: f32) {
    ui.set_min_width(width);
    ui.set_max_width(width);
}

// EN: Render a full-width submenu row inside a menu popup.
// FR: Affiche une ligne de sous-menu pleine largeur dans un menu popup.
fn full_width_submenu_button<R>(
    ui: &mut egui::Ui,
    label: String,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) {
    let width = ui.available_width();
    let response = ui.add_sized(
        [width, 0.0],
        egui::Button::new(label)
            .right_text("⏵")
            .min_size(egui::vec2(width, 0.0)),
    );
    let _ = egui::containers::menu::SubMenu::new().show(ui, &response, add_contents);
}

// EN: Render a selectable row inside top-bar menus with stronger Kiwano hover.
// FR: Affiche une ligne selectable dans les menus du haut avec un hover Kiwano plus lisible.
fn menu_selectable_row(ui: &mut egui::Ui, theme: AppTheme, selected: bool, label: String) -> egui::Response {
    ui.scope(|ui| {
        if theme == AppTheme::Kiwano {
            let style = ui.style_mut();
            style.visuals.selection.bg_fill = egui::Color32::from_rgb(176, 66, 74);
            style.visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 236, 236));
            style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(206, 86, 92);
            style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(186, 72, 80);
            style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        }
        ui.add_sized([ui.available_width(), 0.0], egui::Button::selectable(selected, label))
    }).inner
}

// EN: Render a top-level menu button with stronger Kiwano hover/open feedback.
// FR: Affiche un bouton de menu principal avec un hover/open Kiwano plus lisible.
fn top_menu_button<R>(ui: &mut egui::Ui, theme: AppTheme, label: String, add_contents: impl FnOnce(&mut egui::Ui) -> R) {
    ui.scope(|ui| {
        if theme == AppTheme::Kiwano {
            let style = ui.style_mut();
            style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(206, 86, 92);
            style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(186, 72, 80);
            style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(244, 150, 154));
            style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
            style.visuals.widgets.open = style.visuals.widgets.hovered;
        }
        let _ = egui::containers::menu::MenuButton::new(label).ui(ui, add_contents);
    });
}

// EN: Render a first-level row inside a top-bar menu with stronger Kiwano hover.
// FR: Affiche une ligne de premier niveau dans un menu du haut avec un hover Kiwano plus lisible.
fn first_level_menu_row(ui: &mut egui::Ui, theme: AppTheme, label: String) -> egui::Response {
    ui.scope(|ui| {
        if theme == AppTheme::Kiwano {
            let style = ui.style_mut();
            style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(206, 86, 92);
            style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(186, 72, 80);
            style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(244, 150, 154));
            style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        }
        ui.add_sized([ui.available_width(), 0.0], egui::Button::new(label))
    }).inner
}

// EN: Render the top menu bar and wire actions to app state changes.
// FR: Affiche la barre de menu du haut et relie les actions aux changements d etat.
pub fn show(ctx: &egui::Context, app: &mut Oxide) {
    let t = tr(app.langue);
    let text_color = if app.theme == AppTheme::Kiwano || app.theme == AppTheme::Dark {
        egui::Color32::from_gray(235)
    } else {
        egui::Color32::from_gray(20)
    };

    let response = egui::TopBottomPanel::top("barre").show(ctx, |ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
        ui.visuals_mut().override_text_color = Some(text_color);
        if app.theme == AppTheme::Kiwano {
            let hovered = &mut ui.visuals_mut().widgets.hovered;
            hovered.bg_fill = egui::Color32::from_rgb(192, 72, 80);
            hovered.weak_bg_fill = egui::Color32::from_rgb(170, 60, 68);
            hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(244, 150, 154));
        }

        let raccourcis = app.raccourcis.clone();
        let game_w_plain = menu_width(ui, &[
            nw(&with_shortcut(&t.load_game, &raccourcis.charger_jeu)),
            nw(&t.load_recent_rom),
            nw(&with_shortcut(&t.resume, &raccourcis.pause)),
            nw(&with_shortcut(&t.pause, &raccourcis.pause)),
            nw(&with_shortcut(&t.reset, &raccourcis.reset)),
            nw(&with_shortcut(&t.stop, &raccourcis.stop)),
            nw(&t.save_state),
            nw(&t.load_state),
        ]);
        let game_w_checkbox = menu_width_with_extras(ui, &[nw(&t.fps_limiter)], true, false);
        let game_w_submenu = menu_width_with_extras(
            ui,
            &[nw(&t.save_state), nw(&t.load_state)],
            false,
            true,
        );
        let game_w = game_w_plain.max(game_w_checkbox).max(game_w_submenu);
        let state_w = menu_width(ui, &[
            nw(&slot_width_label(&slot_label_text(app, 0), &raccourcis.savestate_1)),
            nw(&slot_width_label(&slot_label_text(app, 1), &raccourcis.savestate_2)),
            nw(&slot_width_label(&slot_label_text(app, 2), &raccourcis.savestate_3)),
            nw(&slot_width_label(&slot_label_text(app, 0), &raccourcis.loadstate_1)),
            nw(&slot_width_label(&slot_label_text(app, 1), &raccourcis.loadstate_2)),
            nw(&slot_width_label(&slot_label_text(app, 2), &raccourcis.loadstate_3)),
            nw(&t.load_state_file),
        ]);
        let emulator_w = menu_width(ui, &[nw(&t.emulator_settings), nw(&t.languages)]);
        let language_w = menu_width(
            ui,
            &Langue::ALL
                .iter()
                .map(|l| nw(l.label()))
                .collect::<Vec<String>>(),
        );
        let video_w_plain = menu_width(ui, &[
            nw(&t.video_settings),
            nw(&t.render_size),
        ]);
        let video_w_checkbox = menu_width_with_extras(
            ui,
            &[nw(&t.vsync), nw(&with_shortcut(&t.fullscreen, &raccourcis.fullscreen))],
            true,
            false,
        );
        let video_w_submenu = menu_width_with_extras(ui, &[nw(&t.render_size)], false, true);
        let video_w = video_w_plain.max(video_w_checkbox).max(video_w_submenu);
        let render_w = menu_width(
            ui,
            &[
                "1x".to_owned(),
                "2x".to_owned(),
                "3x".to_owned(),
                "4x".to_owned(),
                "5x".to_owned(),
            ],
        );
        let debug_w_plain = menu_width(ui, &[nw(&t.debug_settings)]);
        let debug_w_checkbox =
            menu_width_with_extras(ui, &[nw(&t.settings_debug_terminal)], true, false);
        let debug_w = debug_w_plain.max(debug_w_checkbox);

        ui.horizontal(|ui| {
            top_menu_button(ui, app.theme, nw(&t.game), |ui| {
                set_menu_width(ui, game_w);
                if ui.button(nw(&t.load_game)).clicked() {
                    app.load_rom_dialog();
                    ui.close();
                }
                if ui
                    .add_enabled(app.last_rom_path.is_some(), egui::Button::new(nw(&t.load_recent_rom)))
                    .clicked()
                {
                    app.load_recent_rom();
                    ui.close();
                }
                ui.separator();
                let pause_label = if app.en_pause {
                    nw(&with_shortcut(&t.resume, &raccourcis.pause))
                } else {
                    nw(&with_shortcut(&t.pause, &raccourcis.pause))
                };
                if ui
                    .add_enabled(app.rom_chargee, egui::Button::new(pause_label))
                    .clicked()
                {
                    app.toggle_pause();
                    ui.close();
                }
                if ui
                    .add_enabled(
                        app.rom_chargee,
                        egui::Button::new(nw(&with_shortcut(&t.reset, &raccourcis.reset))),
                    )
                    .clicked()
                {
                    app.reset_rom();
                    ui.close();
                }
                if ui
                    .add_enabled(
                        app.rom_chargee,
                        egui::Button::new(nw(&with_shortcut(&t.stop, &raccourcis.stop))),
                    )
                    .clicked()
                {
                    app.stop_emulation();
                    ui.close();
                }
                ui.separator();
                ui.add_enabled_ui(app.rom_chargee, |ui| {
                    full_width_submenu_button(ui, nw(&t.save_state), |ui| {
                        set_menu_width(ui, state_w);
                        let s1 = label_with_right_shortcut(ui, &slot_label_text(app, 0), &raccourcis.savestate_1, state_w);
                        let s2 = label_with_right_shortcut(ui, &slot_label_text(app, 1), &raccourcis.savestate_2, state_w);
                        let s3 = label_with_right_shortcut(ui, &slot_label_text(app, 2), &raccourcis.savestate_3, state_w);
                        if ui.button(nw(&s1)).clicked() { app.save_state_slot_manual(0); ui.close(); }
                        if ui.button(nw(&s2)).clicked() { app.save_state_slot_manual(1); ui.close(); }
                        if ui.button(nw(&s3)).clicked() { app.save_state_slot_manual(2); ui.close(); }
                    });
                    full_width_submenu_button(ui, nw(&t.load_state), |ui| {
                        set_menu_width(ui, state_w);
                        let l1 = label_with_right_shortcut(ui, &slot_label_text(app, 0), &raccourcis.loadstate_1, state_w);
                        let l2 = label_with_right_shortcut(ui, &slot_label_text(app, 1), &raccourcis.loadstate_2, state_w);
                        let l3 = label_with_right_shortcut(ui, &slot_label_text(app, 2), &raccourcis.loadstate_3, state_w);
                        if ui.button(nw(&l1)).clicked() { app.load_state_slot(0); ui.close(); }
                        if ui.button(nw(&l2)).clicked() { app.load_state_slot(1); ui.close(); }
                        if ui.button(nw(&l3)).clicked() { app.load_state_slot(2); ui.close(); }
                        ui.separator();
                        if ui.button(nw(&t.load_state_file)).clicked() { app.load_state_file_dialog(); ui.close(); }
                    });
                });
                ui.separator();
                ui.checkbox(&mut app.vsync, nw(&t.fps_limiter));
            });

            top_menu_button(ui, app.theme, nw(&t.emulator), |ui| {
                set_menu_width(ui, emulator_w);
                if first_level_menu_row(ui, app.theme, nw(&t.emulator_settings)).clicked() {
                    app.snapshot_theme = app.theme;
                    app.temp_theme = app.theme;
                    app.snapshot_langue = app.langue;
                    app.temp_langue = app.langue;
                    app.snapshot_cycles_par_seconde = app.cycles_par_seconde;
                    app.snapshot_son_active = app.son_active;
                    app.snapshot_sound_volume = app.sound_volume;
                    app.snapshot_quirks = app.quirks;
                    app.snapshot_quirks_preset = app.quirks_preset;
                    app.snapshot_terminal_active = app.terminal_active;
                    app.snapshot_sound_volume = app.sound_volume;
                    app.temp_cycles_par_seconde = app.cycles_par_seconde;
                    app.temp_son_active = app.son_active;
                    app.temp_sound_volume = app.sound_volume;
                    app.temp_quirks = app.quirks;
                    app.temp_quirks_preset = app.quirks_preset;
                    app.temp_terminal_active = app.terminal_active;
                    app.temp_sound_volume = app.sound_volume;
                    app.onglet_settings = OngletsSettings::Emulateur;
                    app.settings_position_initialized = false;
                    app.fenetre_settings = true;
                    app.focus_settings_requested = true;
                    ui.close();
                }
                ui.separator();
                full_width_submenu_button(ui, nw(&t.languages), |ui| {
                    set_menu_width(ui, language_w);
                    for language in Langue::ALL {
                        show_language_item(ui, app, language);
                    }
                });
            });

            top_menu_button(ui, app.theme, nw(&t.video), |ui| {
                set_menu_width(ui, video_w);
                if first_level_menu_row(ui, app.theme, nw(&t.video_settings)).clicked() {
                    app.snapshot_vsync = app.vsync;
                    app.snapshot_video_scale = app.video_scale;
                    app.snapshot_quirks = app.quirks;
                    app.snapshot_quirks_preset = app.quirks_preset;
                    app.snapshot_terminal_active = app.terminal_active;
                    app.snapshot_sound_volume = app.sound_volume;
                    app.temp_vsync = app.vsync;
                    app.temp_video_scale = app.video_scale;
                    app.temp_quirks = app.quirks;
                    app.temp_quirks_preset = app.quirks_preset;
                    app.temp_terminal_active = app.terminal_active;
                    app.temp_sound_volume = app.sound_volume;
                    app.onglet_settings = OngletsSettings::Video;
                    app.settings_position_initialized = false;
                    app.fenetre_settings = true;
                    app.focus_settings_requested = true;
                    ui.close();
                }
                ui.separator();
                full_width_submenu_button(ui, nw(&t.render_size), |ui| {
                    set_menu_width(ui, render_w);
                    for scale in [1u8, 2, 3, 4, 5] {
                        if menu_selectable_row(ui, app.theme, app.video_scale == scale, format!("{}x", scale)).clicked() {
                            app.video_scale = scale;
                            ui.close();
                        }
                    }
                });
                ui.checkbox(&mut app.vsync, nw(&t.vsync));
                let mut fullscreen = app.fullscreen;
                if ui.checkbox(&mut fullscreen, nw(&with_shortcut(&t.fullscreen, &raccourcis.fullscreen))).clicked() {
                    app.set_fullscreen(fullscreen);
                }
            });

            if ui.button(nw(&t.controls)).clicked() {
                app.snapshot_touches = app.touches.clone();
                app.snapshot_raccourcis = app.raccourcis.clone();
                app.snapshot_quirks = app.quirks;
                app.snapshot_quirks_preset = app.quirks_preset;
                app.snapshot_terminal_active = app.terminal_active;
                app.snapshot_sound_volume = app.sound_volume;
                app.temp_touches = app.touches.clone();
                app.temp_raccourcis = app.raccourcis.clone();
                app.temp_quirks = app.quirks;
                app.temp_quirks_preset = app.quirks_preset;
                app.temp_terminal_active = app.terminal_active;
                app.temp_sound_volume = app.sound_volume;
                app.onglet_settings = OngletsSettings::Controles;
                app.settings_position_initialized = false;
                app.fenetre_settings = true;
                app.focus_settings_requested = true;
            }

            if ui.button(nw(&t.shortcuts)).clicked() {
                app.snapshot_raccourcis = app.raccourcis.clone();
                app.snapshot_quirks = app.quirks;
                app.snapshot_quirks_preset = app.quirks_preset;
                app.snapshot_terminal_active = app.terminal_active;
                app.snapshot_sound_volume = app.sound_volume;
                app.temp_raccourcis = app.raccourcis.clone();
                app.temp_quirks = app.quirks;
                app.temp_quirks_preset = app.quirks_preset;
                app.temp_terminal_active = app.terminal_active;
                app.temp_sound_volume = app.sound_volume;
                app.onglet_settings = OngletsSettings::Raccourcis;
                app.settings_position_initialized = false;
                app.fenetre_settings = true;
                app.focus_settings_requested = true;
            }

            top_menu_button(ui, app.theme, nw(&t.tab_debug), |ui| {
                set_menu_width(ui, debug_w);
                if first_level_menu_row(ui, app.theme, nw(&t.debug_settings)).clicked() {
                    app.snapshot_terminal_active = app.terminal_active;
                    app.snapshot_quirks = app.quirks;
                    app.snapshot_quirks_preset = app.quirks_preset;
                    app.temp_terminal_active = app.terminal_active;
                    app.temp_quirks = app.quirks;
                    app.temp_quirks_preset = app.quirks_preset;
                    app.onglet_settings = OngletsSettings::Debug;
                    app.settings_position_initialized = false;
                    app.fenetre_settings = true;
                    app.focus_settings_requested = true;
                    ui.close();
                }
                ui.separator();
                let resp = ui.checkbox(&mut app.terminal_active, nw(&t.settings_debug_terminal));
                if resp.changed() && app.terminal_active {
                    app.focus_terminal_requested = true;
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (mode_icon, icon_color) = match app.theme {
                    AppTheme::Kiwano => (
                        t.theme_kiwano_icon.as_str(),
                        egui::Color32::from_rgb(230, 205, 70),
                    ),
                    AppTheme::Dark => (t.theme_dark_icon.as_str(), text_color),
                    AppTheme::Light => (t.theme_light_icon.as_str(), text_color),
                };
                let icon = egui::RichText::new(mode_icon).color(icon_color);
                if ui.button(icon).clicked() {
                    app.theme = match app.theme {
                        AppTheme::Kiwano => AppTheme::Dark,
                        AppTheme::Dark => AppTheme::Light,
                        AppTheme::Light => AppTheme::Kiwano,
                    };
                }
                ui.label(nw(&format!("{} :", t.theme)));
            });
        });
    });
    app.top_bar_height = response.response.rect.height();
}

// EN: Render one language selection item.
// FR: Affiche un element de selection de langue.
fn show_language_item(ui: &mut egui::Ui, app: &mut Oxide, language: Langue) {
    let selected = app.langue == language;
    let label = nw(language.label());
    if menu_selectable_row(ui, app.theme, selected, label).clicked() {
        app.langue = language;
        ui.close();
    }
}
