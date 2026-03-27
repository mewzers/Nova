use eframe::egui;

use crate::app::Oxide;
use crate::constants::VERSION;
use crate::i18n::tr;

// EN: Measure unwrapped text width in current UI style for fixed-size columns.
// FR: Mesure la largeur d un texte sans retour a la ligne pour des colonnes fixes.
fn text_width(ui: &egui::Ui, text: &str) -> f32 {
    let font_id = egui::TextStyle::Body.resolve(ui.style());
    let color = ui.visuals().text_color();
    ui.painter()
        .layout_no_wrap(text.to_owned(), font_id, color)
        .size()
        .x
}

// EN: Render a label constrained to a fixed width/height box.
// FR: Affiche un label contraint a une boite largeur/hauteur fixe.
fn fixed_label(ui: &mut egui::Ui, width: f32, height: f32, text: impl Into<egui::WidgetText>) {
    ui.add_sized([width, height], egui::Label::new(text));
}

// EN: Render bottom status bar (version, ROM state, runtime state, CPU/FPS, sound, status text).
// FR: Affiche la barre de statut du bas (version, etat ROM, etat runtime, CPU/FPS, son, statut).
pub fn show(ctx: &egui::Context, app: &mut Oxide) {
    let t = tr(app.langue);
    let text_color = if app.theme == crate::types::AppTheme::Kiwano || app.theme == crate::types::AppTheme::Dark {
        egui::Color32::from_gray(235)
    } else {
        egui::Color32::from_gray(20)
    };

    let response = egui::TopBottomPanel::bottom("barre_bas")
        .exact_height(crate::constants::BARRE_BAS)
        .show(ctx, |ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
        ui.visuals_mut().override_text_color = Some(text_color);

        let row_h = ui.spacing().interact_size.y;
        let rom_w = text_width(ui, &t.bottom_rom_loaded).max(text_width(ui, &t.bottom_rom_none));
        let state_w = text_width(ui, &t.bottom_state_paused).max(text_width(ui, &t.bottom_state_running));
        let cpu_w = text_width(ui, &format!("{}: 2000", t.bottom_cpu_hz));
        let fps_w = text_width(ui, &format!("{} : 9999", t.terminal_fps))
            .max(text_width(ui, &format!("{} : {}", t.terminal_fps, t.terminal_unlimited)));
        let sound_w = text_width(ui, &format!("{}: 100", t.settings_sound));

        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            fixed_label(ui, text_width(ui, &format!("Oxide v{}", VERSION)), row_h, 
                egui::RichText::new(format!("Oxide v{}", VERSION)).strong()
            );
            ui.separator();

            fixed_label(
                ui,
                rom_w,
                row_h,
                if app.rom_chargee {
                    &t.bottom_rom_loaded
                } else {
                    &t.bottom_rom_none
                },
            );
            ui.separator();

            fixed_label(
                ui,
                state_w,
                row_h,
                if app.en_pause {
                    &t.bottom_state_paused
                } else {
                    &t.bottom_state_running
                },
            );
            ui.separator();

            fixed_label(ui, cpu_w, row_h, format!("{}: {}", t.bottom_cpu_hz, app.cycles_par_seconde));
            ui.separator();

            if app.configured_game_fps.is_finite() {
                fixed_label(ui, fps_w, row_h, format!("{} : {:.0}", t.terminal_fps, app.configured_game_fps));
            } else {
                fixed_label(ui, fps_w, row_h, format!("{} : {}", t.terminal_fps, t.terminal_unlimited));
            }
            ui.separator();

            fixed_label(ui, sound_w, row_h, format!("{}: {}", t.settings_sound, app.sound_volume));
            ui.add_sized(
                [120.0, row_h],
                egui::Slider::new(&mut app.sound_volume, 0..=100).show_value(false),
            );

        });
    });
    app.bottom_bar_height = response.response.rect.height();
}
