// EN: Launch as a GUI app on Windows (no attached console window).
// FR: Lance l'application en mode GUI sur Windows (sans fenetre console attachee).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio;
mod constants;
mod cpu;
mod debug;
mod display;
mod fonts;
mod gamepad;
mod i18n;
mod keypad;
mod memory;
mod types;
mod ui;
mod utils;

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui;

use crate::app::Oxide;
use crate::fonts::setup_custom_fonts;
use image::GenericImageView;

// EN: Load the main window icon from the bundled ICO asset.
// FR: Charge l'icone de la fenetre principale depuis l'ICO embarque.
fn main_window_icon() -> Option<Arc<egui::IconData>> {
    let ico_bytes = include_bytes!("assets/icons/32x32.ico");
    let image = image::load_from_memory_with_format(ico_bytes, image::ImageFormat::Ico)
        .ok()?
        .to_rgba8();
    let (width, height) = image.dimensions();
    Some(Arc::new(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }))
}

// EN: Compute a compact splash window size from the bundled logo.
// FR: Calcule une taille compacte de fenetre splash a partir du logo embarque.
fn splash_window_size() -> [f32; 2] {
    let png_bytes = include_bytes!("assets/logo/logo.png");
    if let Ok(image) = image::load_from_memory(png_bytes) {
        let (width, height) = image.dimensions();
        let width = width as f32;
        let height = height as f32;
        let max_width = 520.0;
        let max_height = 360.0;
        let scale = (max_width / width).min(max_height / height).min(1.0);
        [width * scale + 48.0, height * scale + 48.0]
    } else {
        [640.0, 360.0]
    }
}

// EN: Acquire a process-wide named mutex to enforce single instance mode.
// FR: Acquiert un mutex nomme global au processus pour imposer une seule instance.
#[cfg(target_os = "windows")]
fn single_instance_guard() -> Option<windows_sys::Win32::Foundation::HANDLE> {
    use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
    use windows_sys::Win32::System::Threading::CreateMutexW;

    let name: Vec<u16> = "Local\\OxideSingleInstanceMutex"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let handle: HANDLE = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
    if handle.is_null() {
        return None;
    }
    let already_running = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
    if already_running {
        None
    } else {
        Some(handle)
    }
}

// EN: Application entry point.
// FR: Point d'entree de l'application.
fn main() {
    #[cfg(target_os = "windows")]
    let _single_instance = match single_instance_guard() {
        Some(handle) => handle,
        None => return,
    };

    debug::log("starting_Oxide");

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size(splash_window_size())
        .with_resizable(false)
        .with_decorations(false)
        .with_title_shown(false)
        .with_titlebar_shown(false)
        .with_transparent(true);
    if let Some(icon) = main_window_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions { centered: true, viewport, ..Default::default() };

    let initial_path = std::env::args().nth(1).map(PathBuf::from);

    let _ = eframe::run_native(
        "Oxide",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            let app: Oxide = if let Some(storage) = cc.storage {
                eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
            } else {
                Oxide::default()
            };
            let mut app = app;
            // Re-arm splash runtime state on every launch, even when loading saved settings.
            app.splash_active = true;
            app.splash_started = None;
            app.splash_texture = None;
            app.splash_size = egui::Vec2::ZERO;
            app.reset_runtime_on_startup();
            app.video_scale_precedent = 0;
            if let Some(path) = initial_path.clone() {
                app.pending_open_path = Some(path);
            }

            debug::log("terminal_ready");

            Ok(Box::new(app))
        }),
    );
    debug::log("oxide_exited");
}
