use crate::debug;
use std::fs::{self, File, OpenOptions};
use std::path::PathBuf;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use chrono::Local;
use image::GenericImageView;

use eframe::egui::{self};

use crate::constants::VERSION;
use crate::cpu::{CPU, CpuQuirks};
use crate::display::Display;
use crate::gamepad;
use crate::i18n::tr;
use crate::keypad::Keypad;
use crate::types::{AppTheme, Langue, OngletsSettings, QuirkPreset, Raccourcis};
use crate::ui;
use crate::utils::{default_touches, key_from_label, mouse_from_label};
use crate::audio::AudioEngine;
use zip::write::FileOptions;
use zip::CompressionMethod;
// EN: Snapshot of CPU/display state used by save/load slots.
// FR: Snapshot etat CPU/affichage utilise par les slots de sauvegarde/chargement.
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct EmuSnapshot {
    pub cpu: CPU,
    pub display: Display,
    #[serde(default)]
    pub memory: Vec<u8>,
}
// EN: Metadata stored alongside each save-state slot (name + timestamp).
// FR: Metadonnees stockees avec chaque slot de sauvegarde (nom + horodatage).
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct SaveStateMeta {
    pub name: String,
    pub timestamp: String,
}

// EN: On-disk savestate container (one file per slot).
// FR: Conteneur de savestate sur disque (un fichier par slot).
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct PersistedSaveState {
    pub version: u32,
    pub rom_name: String,
    pub rom_hash: String,
    #[serde(default)]
    pub rom_bytes: Vec<u8>,
    #[serde(default)]
    pub rom_path: String,
    pub slot: usize,
    pub meta: SaveStateMeta,
    pub snapshot: EmuSnapshot,
}

// EN: Default empty save-state array.
// FR: Tableau par defaut des save-states vides.
fn default_savestates() -> [Option<EmuSnapshot>; 3] {
    [None, None, None]
}
// EN: Default empty savestate metadata array.
// FR: Tableau par defaut des metadonnees de savestates vides.
fn default_savestate_meta() -> [Option<SaveStateMeta>; 3] {
    [None, None, None]
}

// EN: Serializer helper for runtime Instant fields.
// FR: Helper de serialisation pour les champs Instant runtime.
fn default_instant() -> Instant {
    Instant::now()
}

// EN: Serializer helper for default Kiwano theme.
// FR: Helper de serialisation pour le theme Kiwano par defaut.
fn default_theme_kiwano() -> AppTheme {
    AppTheme::Kiwano
}

// EN: Serializer helper for default UI language (French).
// FR: Helper de serialisation pour la langue UI par defaut (francais).
fn default_langue_fr() -> Langue {
    Langue::Francais
}

// EN: Serializer helper for default focus request on startup.
// FR: Helper de serialisation pour demander le focus au demarrage.
fn default_focus_main() -> bool {
    true
}

// EN: Serializer helper for default sound volume.
// FR: Helper de serialisation pour le volume sonore par defaut.
fn default_sound_volume() -> u8 {
    100
}
// EN: Timestamp string for UI display (local time).
// FR: Chaine d horodatage pour affichage UI (heure locale).
fn timestamp_for_display() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

// EN: Timestamp string for filenames (local time, filesystem-safe).
// FR: Chaine d horodatage pour noms de fichiers (heure locale, safe).
fn timestamp_for_filename() -> String {
    Local::now().format("%Y-%m-%d_%H-%M-%S").to_string()
}

// EN: Stable, lightweight hash for ROM identification (FNV-1a 64-bit).
// FR: Hash stable et leger pour identifier une ROM (FNV-1a 64-bit).
fn rom_hash_hex(data: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in data {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}

// EN: Sanitize a string for use as a filename.
// FR: Nettoie une chaine pour l usage en nom de fichier.
fn sanitize_filename(input: &str) -> String {
    let mut out: String = input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if out.is_empty() {
        out = "rom".to_string();
    }
    out
}

const SAVE_STATE_VERSION: u32 = 1;

// EN: Rotate an existing latest log into a zip archive and open a fresh file.
// FR: Archive le dernier log en zip puis ouvre un fichier neuf.
fn rotate_log_file(dir: &str, prefix: &str) -> Option<File> {
    let _ = fs::create_dir_all(dir);
    let latest_path = PathBuf::from(dir).join("latest.logs");

    if latest_path.exists() {
        let zip_name = format!("logs-{}-{}.zip", prefix, timestamp_for_filename());
        let zip_path = PathBuf::from(dir).join(zip_name);
        if let Ok(zip_file) = File::create(&zip_path) {
            let mut zip = zip::ZipWriter::new(zip_file);
            let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
            if let Ok(mut src) = File::open(&latest_path) {
                if zip.start_file("latest.logs", options).is_ok() {
                    let _ = std::io::copy(&mut src, &mut zip);
                }
            }
            let _ = zip.finish();
        }
        let _ = fs::remove_file(&latest_path);
    }

    OpenOptions::new().create(true).append(true).open(latest_path).ok()
}

// EN: Initialize log files (latest.logs) and rotate previous ones to zip.
// FR: Initialise les fichiers de log (latest.logs) et archive les precedents.
fn init_log_files() -> (Option<Arc<Mutex<File>>>, Option<Arc<Mutex<File>>>) {
    let terminal = rotate_log_file("logs/app", "app");
    let emulator = rotate_log_file("logs/emulator", "emulator");
    (terminal.map(|f| Arc::new(Mutex::new(f))), emulator.map(|f| Arc::new(Mutex::new(f))))
}

// EN: Serializer helper for default video scale.
// FR: Helper de serialisation pour l echelle video par defaut.
fn default_video_scale() -> u8 {
    2
}
// EN: Custom Kiwano visuals (dark red application chrome).
// FR: Visuels personnalises Kiwano (chrome applicatif rouge fonce).
fn kiwano_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(egui::Color32::from_rgb(244, 233, 233));
    visuals.hyperlink_color = egui::Color32::from_rgb(255, 184, 160);
    visuals.faint_bg_color = egui::Color32::from_rgb(118, 36, 42);
    visuals.extreme_bg_color = egui::Color32::from_rgb(116, 34, 40);
    visuals.text_edit_bg_color = Some(egui::Color32::from_rgb(132, 40, 46));
    visuals.code_bg_color = egui::Color32::from_rgb(104, 30, 36);
    visuals.panel_fill = egui::Color32::from_rgb(108, 30, 36);
    visuals.window_fill = egui::Color32::from_rgb(126, 36, 42);
    visuals.selection.bg_fill = egui::Color32::from_rgb(170, 56, 64);
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 242, 242));
    visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(168, 74, 80));
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [8, 14],
        blur: 18,
        spread: 0,
        color: egui::Color32::from_rgba_unmultiplied(52, 10, 14, 72),
    };
    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [4, 8],
        blur: 12,
        spread: 0,
        color: egui::Color32::from_rgba_unmultiplied(52, 10, 14, 64),
    };
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(118, 34, 40);
    visuals.widgets.noninteractive.weak_bg_fill = egui::Color32::from_rgb(108, 30, 36);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(130, 42, 48));
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(240, 228, 228));
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(142, 46, 54);
    visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(126, 36, 42);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(156, 58, 66));
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(244, 233, 233));
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(148, 48, 56);
    visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(128, 40, 48);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(198, 88, 94));
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(174, 60, 68);
    visuals.widgets.active.weak_bg_fill = egui::Color32::from_rgb(150, 48, 56);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 120, 124));
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    visuals.widgets.open.bg_fill = egui::Color32::from_rgb(170, 58, 66);
    visuals.widgets.open.weak_bg_fill = egui::Color32::from_rgb(156, 50, 58);
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(216, 112, 118));
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 244, 244));
    visuals
}

// EN: Return complete egui visuals for the selected app theme.
// FR: Retourne les visuels egui complets pour le theme applicatif selectionne.
pub(crate) fn visuals_for_theme(theme: AppTheme) -> egui::Visuals {
    match theme {
        AppTheme::Kiwano => kiwano_visuals(),
        AppTheme::Dark => egui::Visuals::dark(),
        AppTheme::Light => egui::Visuals::light(),
    }
}
// EN: Persistent + runtime application state for emulator, UI and debug terminal.
// FR: Etat applicatif persistant + runtime pour l emulateur, l UI et le terminal debug.
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Oxide {
    pub(crate) theme: AppTheme,
    pub(crate) cpu: CPU,
    pub(crate) display: Display,
    pub(crate) langue: Langue,
    pub(crate) video_scale: u8,
    pub(crate) vsync: bool,
    pub(crate) fullscreen: bool,
    pub(crate) en_pause: bool,
    pub(crate) rom_chargee: bool,
    pub(crate) touches: [String; 16],
    pub(crate) raccourcis: Raccourcis,
    pub(crate) fenetre_settings: bool,
    pub(crate) onglet_settings: OngletsSettings,
    pub(crate) temp_theme: AppTheme,
    pub(crate) temp_langue: Langue,
    pub(crate) temp_vsync: bool,
    #[serde(default = "default_video_scale")]
    pub(crate) temp_video_scale: u8,
    pub(crate) temp_touches: [String; 16],
    pub(crate) temp_raccourcis: Raccourcis,
    pub(crate) cycles_par_seconde: u16,
    pub(crate) son_active: bool,
    #[serde(default = "default_sound_volume")]
    pub(crate) sound_volume: u8,
    pub(crate) temp_cycles_par_seconde: u16,
    pub(crate) temp_son_active: bool,
    pub(crate) temp_sound_volume: u8,
    #[serde(default)]
    pub(crate) quirks: CpuQuirks,
    #[serde(default)]
    pub(crate) quirks_preset: QuirkPreset,
    #[serde(default)]
    pub(crate) temp_quirks: CpuQuirks,
    #[serde(default)]
    pub(crate) temp_quirks_preset: QuirkPreset,
    pub(crate) temp_terminal_active: bool,
    pub(crate) snapshot_theme: AppTheme,
    pub(crate) snapshot_langue: Langue,
    pub(crate) snapshot_vsync: bool,
    #[serde(default = "default_video_scale")]
    pub(crate) snapshot_video_scale: u8,
    pub(crate) snapshot_touches: [String; 16],
    pub(crate) snapshot_raccourcis: Raccourcis,
    pub(crate) snapshot_cycles_par_seconde: u16,
    pub(crate) snapshot_son_active: bool,
    pub(crate) snapshot_sound_volume: u8,
    #[serde(default)]
    pub(crate) snapshot_quirks: CpuQuirks,
    #[serde(default)]
    pub(crate) snapshot_quirks_preset: QuirkPreset,
    pub(crate) snapshot_terminal_active: bool,
    pub(crate) terminal_active: bool,
    #[serde(skip, default)]
    pub(crate) status_message: String,
    #[serde(skip, default)]
    pub(crate) display_overlay_message: String,
    #[serde(skip, default = "default_instant")]
    pub(crate) display_overlay_until: Instant,
    #[serde(skip, default)]
    pub(crate) terminal_logs: Vec<String>,
    #[serde(skip, default)]
    pub(crate) terminal_logs_text: String,
    #[serde(skip, default)]
    pub(crate) terminal_search_query: String,
    #[serde(skip, default)]
    pub(crate) last_status_logged: String,
    #[serde(skip, default)]
    pub(crate) focus_settings_requested: bool,
    #[serde(skip, default)]
    pub(crate) settings_position_initialized: bool,
    #[serde(skip, default)]
    pub(crate) focus_terminal_requested: bool,
    #[serde(skip, default = "default_focus_main")]
    pub(crate) focus_main_requested: bool,
    #[serde(skip, default)]
    pub(crate) terminal_position_initialized: bool,
    #[serde(skip, default)]
    pub(crate) terminal_rows_at_open: usize,
    #[serde(skip, default)]
    pub(crate) terminal_view_session: u64,
    #[serde(skip, default)]
    pub(crate) terminal_boot_logs_seeded: bool,
    #[serde(skip, default)]
    pub(crate) splash_launch_log_pending: bool,
    #[serde(skip, default)]
    pub(crate) configured_game_fps: f32,
    #[serde(skip, default = "default_theme_kiwano")]
    pub(crate) last_logged_theme: AppTheme,
    #[serde(skip, default = "default_langue_fr")]
    pub(crate) last_logged_langue: Langue,
    #[serde(skip)]
    pub(crate) last_logged_vsync: bool,
    #[serde(skip)]
    pub(crate) last_logged_fullscreen: bool,
    #[serde(skip)]
    pub(crate) last_logged_son_active: bool,
    #[serde(skip, default = "default_instant")]
    pub(crate) terminal_clock_origin: Instant,
    #[serde(skip)]
    pub(crate) fenetre_principale_pos: egui::Pos2,
    #[serde(skip)]
    pub(crate) fenetre_principale_size: egui::Vec2,
    #[serde(skip, default = "default_focus_main")]
    pub(crate) last_main_focused: bool,
    #[serde(skip)]
    pub(crate) video_scale_precedent: u8,
    #[serde(skip)]
    pub(crate) fullscreen_precedent: bool,
    #[serde(skip)]
    pub(crate) top_bar_height: f32,
    #[serde(skip)]
    pub(crate) bottom_bar_height: f32,
    #[serde(skip)]
    pub(crate) fullscreen_active: bool,
    #[serde(skip)]
    pub(crate) window_maximized: bool,
    #[serde(skip)]
    pub(crate) keypad: Keypad,
    #[serde(skip)]
    pub(crate) rom_data: Vec<u8>,
    #[serde(skip)]
    pub(crate) rom_path: Option<PathBuf>,
    pub(crate) last_rom_path: Option<PathBuf>,
    #[serde(skip)]
    pub(crate) cpu_step_accumulator: f32,
    #[serde(skip)]
    pub(crate) timer_accumulator: f32,
    #[serde(skip)]
    pub(crate) audio_engine: AudioEngine,
    #[serde(skip)]
    pub(crate) terminal_keypad_states: [bool; 16],
    #[serde(skip, default = "default_savestates")]
    pub(crate) savestates: [Option<EmuSnapshot>; 3],
    #[serde(skip, default = "default_savestate_meta")]
    pub(crate) savestate_meta: [Option<SaveStateMeta>; 3],
    #[serde(skip, default)]
    pub(crate) terminal_log_file: Option<Arc<Mutex<File>>>,
    #[serde(skip, default)]
    pub(crate) emulator_log_file: Option<Arc<Mutex<File>>>,
    #[serde(skip)]
    pub(crate) pending_open_path: Option<PathBuf>,
    #[serde(skip)]
    pub(crate) binding_key: Option<usize>,
    #[serde(skip)]
    pub(crate) binding_key_started: Option<std::time::Instant>,
    #[serde(skip)]
    pub(crate) binding_key_skip_first_click: bool,
    #[serde(skip)]
    pub(crate) binding_shortcut: Option<usize>,
    #[serde(skip)]
    pub(crate) binding_shortcut_started: Option<std::time::Instant>,
    #[serde(skip)]
    pub(crate) binding_shortcut_skip_first_click: bool,
    #[serde(skip)]
    pub(crate) confirm_reset_all: bool,
    #[serde(skip)]
    pub(crate) confirm_overwrite_slot: Option<usize>,
    #[serde(skip)]
    pub(crate) pause_overlay_texture: Option<egui::TextureHandle>,
    #[serde(skip)]
    pub(crate) pause_overlay_size: egui::Vec2,
    #[serde(skip)]
    pub(crate) splash_active: bool,
    #[serde(skip)]
    pub(crate) splash_started: Option<Instant>,
    #[serde(skip)]
    pub(crate) splash_texture: Option<egui::TextureHandle>,
    #[serde(skip)]
    pub(crate) splash_size: egui::Vec2,
    #[serde(skip)]
    pub(crate) cursor_over_display: bool,
    #[serde(skip)]
    pub(crate) cursor_hidden: bool,
    #[serde(skip, default = "default_instant")]
    pub(crate) last_mouse_move: Instant,
}

impl Default for Oxide {
    // EN: Build the full default app state used on first launch.
    // FR: Construit l etat complet par defaut de l application pour le premier lancement.
    fn default() -> Self {
        let (terminal_log_file, emulator_log_file) = init_log_files();
Self {
            theme: AppTheme::Kiwano,
            cpu: CPU::new(),
            display: Display::new(),
            langue: Langue::Francais,
            video_scale: 2,
            vsync: true,
            fullscreen: false,
            en_pause: false,
            rom_chargee: false,
            touches: default_touches(),
            raccourcis: Raccourcis::default(),
            fenetre_settings: false,
            onglet_settings: OngletsSettings::Emulateur,
            temp_theme: AppTheme::Kiwano,
            temp_langue: Langue::Francais,
            temp_vsync: true,
            temp_video_scale: 2,
            temp_touches: default_touches(),
            temp_raccourcis: Raccourcis::default(),
            cycles_par_seconde: 700,
            son_active: true,
            sound_volume: 100,
            temp_cycles_par_seconde: 700,
            temp_son_active: true,
            temp_sound_volume: 100,
quirks: QuirkPreset::Chip8.quirks(),
            quirks_preset: QuirkPreset::Chip8,
            temp_quirks: QuirkPreset::Chip8.quirks(),
            temp_quirks_preset: QuirkPreset::Chip8,
            temp_terminal_active: false,
            snapshot_theme: AppTheme::Kiwano,
            snapshot_langue: Langue::Francais,
            snapshot_vsync: true,
            snapshot_video_scale: 2,
            snapshot_touches: default_touches(),
            snapshot_raccourcis: Raccourcis::default(),
            snapshot_cycles_par_seconde: 700,
            snapshot_son_active: true,
            snapshot_sound_volume: 100,
snapshot_quirks: QuirkPreset::Chip8.quirks(),
            snapshot_quirks_preset: QuirkPreset::Chip8,
            snapshot_terminal_active: false,
            terminal_active: false,
            status_message: String::new(),
            display_overlay_message: String::new(),
            display_overlay_until: Instant::now(),
            terminal_logs: Vec::new(),
            terminal_logs_text: String::new(),
            terminal_search_query: String::new(),
            last_status_logged: String::new(),
            focus_settings_requested: false,
            settings_position_initialized: false,
            focus_terminal_requested: false,
            focus_main_requested: true,
            terminal_position_initialized: false,
            terminal_rows_at_open: 0,
            terminal_view_session: 0,
            terminal_boot_logs_seeded: false,
            splash_launch_log_pending: false,
            configured_game_fps: 60.0,
            last_logged_theme: AppTheme::Kiwano,
            last_logged_langue: Langue::Francais,
            last_logged_vsync: true,
            last_logged_fullscreen: false,
            last_logged_son_active: true,
            terminal_clock_origin: Instant::now(),
            fenetre_principale_pos: egui::Pos2::ZERO,
            fenetre_principale_size: egui::vec2(1280.0, 720.0),
            last_main_focused: true,
            video_scale_precedent: 2,
            fullscreen_precedent: false,
            top_bar_height: crate::constants::BARRE_HAUT,
            bottom_bar_height: crate::constants::BARRE_BAS,
            fullscreen_active: false,
            window_maximized: false,
            keypad: Keypad::new(),
            rom_data: Vec::new(),
            rom_path: None,
            last_rom_path: None,
            cpu_step_accumulator: 0.0,
            timer_accumulator: 0.0,
            audio_engine: AudioEngine::default(),
            terminal_keypad_states: [false; 16],
            savestates: [None, None, None],
            savestate_meta: [None, None, None],
            terminal_log_file,
            emulator_log_file,
            pending_open_path: None,
            binding_key: None,
            binding_key_started: None,
            binding_key_skip_first_click: false,
            binding_shortcut: None,
            binding_shortcut_started: None,
            binding_shortcut_skip_first_click: false,
            confirm_reset_all: false,
            confirm_overwrite_slot: None,
            pause_overlay_texture: None,
            pause_overlay_size: egui::Vec2::ZERO,
            splash_active: true,
            splash_started: None,
            splash_texture: None,
            splash_size: egui::Vec2::ZERO,
            cursor_over_display: false,
            cursor_hidden: false,
            last_mouse_move: Instant::now(),
        }
    }
}

impl Oxide {
    // EN: Clear any loaded ROM/runtime state at startup (keep user settings).
    // FR: Nettoie l etat ROM/runtime au demarrage (conserve les parametres).
    pub(crate) fn reset_runtime_on_startup(&mut self) {
        self.cpu.hard_reset();
        self.display.clear();
        self.keypad.clear();
        self.rom_data.clear();
        self.rom_path = None;
        self.rom_chargee = false;
        self.en_pause = false;
        self.cpu_step_accumulator = 0.0;
        self.timer_accumulator = 0.0;
        self.display_overlay_message.clear();
        self.display_overlay_until = Instant::now();
    }

    // EN: True when a ROM-dependent input/action is allowed.
    // FR: Vrai si une action dependante d une ROM est autorisee.
    fn can_interact_with_rom(&self) -> bool {
        self.rom_chargee
    }

    // EN: Return true when the shortcut label matches a key press on this frame.
    // FR: Retourne vrai quand le libelle du raccourci correspond a une touche appuyee sur cette frame.
    fn shortcut_pressed(ctx: &egui::Context, label: &str) -> bool {
        ctx.input(|i| crate::utils::shortcut_pressed(i, label))
    }

    // EN: Process global keyboard shortcuts bound in settings.
    // FR: Traite les raccourcis clavier globaux definis dans les parametres.
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        // EN: Ignore shortcuts while text fields are being edited.
        // FR: Ignore les raccourcis pendant l edition des champs texte.
        if ctx.wants_keyboard_input() {
            return;
        }

        if self.can_interact_with_rom() {
            if Self::shortcut_pressed(ctx, &self.raccourcis.pause) {
                self.toggle_pause();
            }
            if Self::shortcut_pressed(ctx, &self.raccourcis.reset) {
                self.reset_rom();
            }
            if Self::shortcut_pressed(ctx, &self.raccourcis.stop) {
                self.stop_emulation();
            }
        }
        if Self::shortcut_pressed(ctx, &self.raccourcis.charger_jeu) {
            self.load_rom_dialog();
        }
        if Self::shortcut_pressed(ctx, &self.raccourcis.fullscreen) {
            self.toggle_fullscreen();
        }
        if self.can_interact_with_rom() {
            if Self::shortcut_pressed(ctx, &self.raccourcis.savestate_1) {
                self.save_state_slot_shortcut(0);
            }
            if Self::shortcut_pressed(ctx, &self.raccourcis.savestate_2) {
                self.save_state_slot_shortcut(1);
            }
            if Self::shortcut_pressed(ctx, &self.raccourcis.savestate_3) {
                self.save_state_slot_shortcut(2);
            }
            if Self::shortcut_pressed(ctx, &self.raccourcis.loadstate_1) {
                self.load_state_slot(0);
            }
            if Self::shortcut_pressed(ctx, &self.raccourcis.loadstate_2) {
                self.load_state_slot(1);
            }
            if Self::shortcut_pressed(ctx, &self.raccourcis.loadstate_3) {
                self.load_state_slot(2);
            }
        }

        // EN: Keep Alt+Enter as an additional fullscreen shortcut.
        // FR: Conserve Alt+Entree comme raccourci plein ecran supplementaire.
        if ctx.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.alt) {
            self.toggle_fullscreen();
        }
    }

    // EN: Apply temporary settings values into live runtime state.
    // FR: Applique les valeurs temporaires des parametres a l etat runtime en direct.
    pub(crate) fn apply_temp_values(&mut self) {
        let terminal_was_active = self.terminal_active;
        self.theme = self.temp_theme;
        self.langue = self.temp_langue;
        self.vsync = self.temp_vsync;
        self.video_scale = self.temp_video_scale;
        self.touches = self.temp_touches.clone();
        self.raccourcis = self.temp_raccourcis.clone();
        self.cycles_par_seconde = self.temp_cycles_par_seconde;
        self.son_active = self.temp_son_active;
        self.sound_volume = self.temp_sound_volume;
        self.temp_quirks_preset = QuirkPreset::from_quirks(self.temp_quirks);
        self.quirks = self.temp_quirks;
        self.quirks_preset = self.temp_quirks_preset;
        self.terminal_active = self.temp_terminal_active;
        if !terminal_was_active && self.terminal_active {
            self.focus_terminal_requested = true;
            self.terminal_position_initialized = false;
        }
    }

    // EN: Restore snapshots captured before opening settings.
    // FR: Restaure les snapshots captures avant l ouverture des parametres.
    pub(crate) fn restore_snapshots(&mut self) {
        self.theme = self.snapshot_theme;
        self.temp_theme = self.snapshot_theme;
        self.langue = self.snapshot_langue;
        self.vsync = self.snapshot_vsync;
        self.video_scale = self.snapshot_video_scale;
        self.temp_video_scale = self.snapshot_video_scale;
        self.touches = self.snapshot_touches.clone();
        self.raccourcis = self.snapshot_raccourcis.clone();
        self.cycles_par_seconde = self.snapshot_cycles_par_seconde;
        self.son_active = self.snapshot_son_active;
        self.sound_volume = self.snapshot_sound_volume;
        self.quirks = self.snapshot_quirks;
        self.quirks_preset = self.snapshot_quirks_preset;
        self.temp_quirks = self.snapshot_quirks;
        self.temp_quirks_preset = self.snapshot_quirks_preset;
        self.terminal_active = self.snapshot_terminal_active;
    }

    // EN: Reset only the currently selected settings tab to defaults.
    // FR: Reinitialise uniquement l onglet de parametres actuellement selectionne.
    pub(crate) fn reset_current_settings_tab_to_default(&mut self) {
        let defaut = Oxide::default();
        match self.onglet_settings {
            OngletsSettings::Emulateur => {
                self.temp_theme = defaut.theme;
                self.temp_langue = defaut.langue;
                self.temp_cycles_par_seconde = defaut.cycles_par_seconde;
            }
            OngletsSettings::Video => {
                self.temp_vsync = defaut.vsync;                self.temp_video_scale = defaut.video_scale;
            }
            OngletsSettings::Audio => {
                self.temp_son_active = defaut.son_active;
                self.temp_sound_volume = defaut.sound_volume;
            }
            OngletsSettings::Controles => {
                self.temp_touches = defaut.touches.clone();
            }
            OngletsSettings::Raccourcis => {
                self.temp_raccourcis = defaut.raccourcis.clone();
            }
            OngletsSettings::Debug => {
                self.temp_terminal_active = defaut.terminal_active;
                self.temp_quirks = defaut.quirks;
                self.temp_quirks_preset = defaut.quirks_preset;
            }
        }
    }

    // EN: Apply selected quirk preset into temporary settings values.
    // FR: Applique le preset de quirks selectionne aux valeurs temporaires.
    pub(crate) fn set_temp_quirks_preset(&mut self, preset: QuirkPreset) {
        self.temp_quirks_preset = preset;
        if preset != QuirkPreset::Custom {
            self.temp_quirks = preset.quirks();
        }
    }

    // EN: Refresh preset label based on current temporary quirks.
    // FR: Met a jour le label du preset selon les quirks temporaires actuels.
    pub(crate) fn sync_temp_quirks_preset_from_values(&mut self) {
        self.temp_quirks_preset = QuirkPreset::from_quirks(self.temp_quirks);
    }

    // EN: Reset runtime accumulators/timers not persisted across ROM changes.
    // FR: Reinitialise les accumulateurs/timers runtime non persistes entre changements de ROM.
    pub(crate) fn reset_runtime_clocks(&mut self) {
        self.cpu_step_accumulator = 0.0;
        self.timer_accumulator = 0.0;
        self.audio_engine.stop();
    }

    // EN: Open file dialog and load selected ROM.
    // FR: Ouvre le selecteur de fichiers et charge la ROM choisie.
    pub(crate) fn load_rom_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("CHIP-8", &["ch8", "rom", "bin"])
            .pick_file()
        {
            let t = tr(self.langue);
            let rom_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("rom");
            let path = path.clone();
            match self.load_rom_path(path) {
                Ok(loaded) => {
                    self.status_message = format!(
                        "{}: {} ({} {})",
                        t.status_rom_loaded, rom_name, loaded, t.bytes_unit
                    );
                    self.show_display_overlay(self.status_message.clone());
                }
                Err(err) => {
                    self.status_message =
                        format!("{}: {} ({})", t.status_rom_load_failed, rom_name, err);
                    self.show_display_overlay(self.status_message.clone());
                }
            }
        }
    }

    // EN: Reset all settings tabs to defaults (temporary values).
    // FR: Reinitialise tous les onglets de parametres a leurs valeurs par defaut (temporaires).
    pub(crate) fn reset_all_settings_to_default(&mut self) {
        let defaut = Oxide::default();
        self.temp_theme = defaut.theme;
        self.temp_langue = defaut.langue;
        self.temp_cycles_par_seconde = defaut.cycles_par_seconde;
        self.temp_vsync = defaut.vsync;
        self.temp_video_scale = defaut.video_scale;
        self.temp_son_active = defaut.son_active;
        self.temp_sound_volume = defaut.sound_volume;
        self.temp_touches = defaut.touches.clone();
        self.temp_raccourcis = defaut.raccourcis.clone();
        self.temp_terminal_active = defaut.terminal_active;
        self.temp_quirks = defaut.quirks;
        self.temp_quirks_preset = defaut.quirks_preset;
    }

    // EN: Load a ROM from a known path and reset runtime state.
    // FR: Charge une ROM depuis un chemin connu et reinitialise l etat runtime.
    pub(crate) fn load_rom_path(&mut self, path: PathBuf) -> Result<usize, String> {
        let rom = fs::read(&path).map_err(|e| e.to_string())?;
        self.rom_data = rom;
        self.last_rom_path = Some(path.clone());
        self.rom_path = Some(path);
        self.cpu.hard_reset();
        debug::log("cpu_hard_reset");
        self.display.clear();
        self.keypad.clear();
        self.rom_chargee = true;
        self.en_pause = false;
        self.reset_runtime_clocks();
        let loaded = self.cpu.load_program(&self.rom_data);
        self.load_savestates_for_rom();
        Ok(loaded)
    }

    // EN: Reload the most recently loaded ROM path.
    // FR: Recharge la ROM du chemin charge le plus recent.
    pub(crate) fn load_recent_rom(&mut self) {
        let t = tr(self.langue);
        let Some(path) = self.last_rom_path.clone() else {
            self.status_message = t.status_no_rom_loaded.clone();
            debug::log("no_recent_rom");
            return;
        };
        let rom_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("rom");
        let path = path.clone();
        match self.load_rom_path(path) {
            Ok(loaded) => {
                self.status_message = format!(
                    "{}: {} ({} {})",
                    t.status_rom_loaded, rom_name, loaded, t.bytes_unit
                );
                self.show_display_overlay(self.status_message.clone());
            }
            Err(err) => {
                self.status_message =
                    format!("{}: {} ({})", t.status_rom_load_failed, rom_name, err);
                self.show_display_overlay(self.status_message.clone());
            }
        }
    }

    // EN: Reset currently loaded ROM while keeping path/data.
    // FR: Reinitialise la ROM actuellement chargee en conservant chemin/donnees.
    pub(crate) fn reset_rom(&mut self) {
        let t = tr(self.langue);
        if self.rom_data.is_empty() {
            self.status_message = t.status_no_rom_loaded.clone();
            self.rom_chargee = false;
            return;
        }
        self.cpu.hard_reset();
        debug::log("cpu_hard_reset");
        self.display.clear();
        self.keypad.clear();
        let loaded = self.cpu.load_program(&self.rom_data);
        self.en_pause = false;
        self.rom_chargee = true;
        self.reset_runtime_clocks();
        self.status_message = format!("{} ({} {})", t.status_rom_reset, loaded, t.bytes_unit);
        self.show_display_overlay(self.status_message.clone());
    }

    // EN: Stop emulation and clear loaded ROM state.
    // FR: Arrete l emulation et nettoie l etat de ROM chargee.
    pub(crate) fn stop_emulation(&mut self) {
        let t = tr(self.langue);
        self.cpu.hard_reset();
        debug::log("cpu_hard_reset");
        self.display.clear();
        self.keypad.clear();
        self.rom_data.clear();
        self.rom_path = None;
        self.en_pause = false;
        self.rom_chargee = false;
        self.savestates = [None, None, None];
        self.savestate_meta = [None, None, None];
        self.reset_runtime_clocks();
        self.status_message = t.status_emulation_stopped.clone();
        self.show_display_overlay(self.status_message.clone());
        debug::log("emulation_stopped");
    }

    // EN: Compute savestate folder for the current ROM.
    // FR: Calcule le dossier des savestates pour la ROM courante.
    fn savestate_dir_for_rom(&self) -> Option<PathBuf> {
        if self.rom_data.is_empty() {
            return None;
        }
        let rom_name = self
            .rom_path
            .as_ref()
            .or(self.last_rom_path.as_ref())
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .unwrap_or("rom");
        let rom_id = format!("{}-{}", sanitize_filename(rom_name), rom_hash_hex(&self.rom_data));
        Some(PathBuf::from("savestates").join(rom_id))
    }

    // EN: Build savestate file path for a slot (timestamped).
    // FR: Construit le chemin du fichier de savestate pour un slot (horodate).
    fn savestate_file_path(&self, slot: usize, rom_name: &str) -> Option<PathBuf> {
        let dir = self.savestate_dir_for_rom()?;
        let safe_rom = sanitize_filename(rom_name);
        let name = format!(
            "{}_{:02}_{}.state",
            safe_rom,
            slot + 1,
            timestamp_for_filename()
        );
        Some(dir.join(name))
    }

    // EN: Find the most recent savestate file for a slot.
    // FR: Trouve le fichier de savestate le plus recent pour un slot.
    fn latest_savestate_file(dir: &PathBuf, slot: usize, rom_name: &str) -> Option<PathBuf> {
        let prefix = format!("{}_{:02}_", sanitize_filename(rom_name), slot + 1);
        let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
        let entries = fs::read_dir(dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("state") {
                continue;
            }
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if !name.starts_with(&prefix) {
                continue;
            }
            let mtime = entry.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            if best.as_ref().map(|(t, _)| mtime > *t).unwrap_or(true) {
                best = Some((mtime, path));
            }
        }
        best.map(|(_, p)| p)
    }

    // EN: Persist current slot to disk.
    // FR: Persiste le slot courant sur disque.
    fn persist_savestate(&self, slot: usize) {
        let Some(snapshot) = self.savestates.get(slot).and_then(|s| s.clone()) else {
            return;
        };
        let Some(meta) = self.savestate_meta.get(slot).and_then(|m| m.clone()) else {
            return;
        };
        let rom_name = meta.name.clone();
        let Some(path) = self.savestate_file_path(slot, &rom_name) else {
            return;
        };
        let Some(dir) = path.parent() else {
            return;
        };
        let _ = fs::create_dir_all(dir);
        if let Ok(entries) = fs::read_dir(dir) {
            let prefix = format!("{}_{:02}_", sanitize_filename(&rom_name), slot + 1);
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) != Some("state") {
                    continue;
                }
                let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if name.starts_with(&prefix) {
                    let _ = fs::remove_file(p);
                }
            }
        }
        let rom_hash = rom_hash_hex(&self.rom_data);
        let rom_path = self
            .rom_path
            .as_ref()
            .or(self.last_rom_path.as_ref())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let payload = PersistedSaveState {
            version: SAVE_STATE_VERSION,
            rom_name,
            rom_hash,
            rom_bytes: Vec::new(),
            rom_path,
            slot,
            meta,
            snapshot,
        };
        if let Ok(bytes) = serde_json::to_vec(&payload) {
            let _ = fs::write(path, bytes);
        }
    }

    // EN: Load savestates from disk for the current ROM.
    // FR: Charge les savestates depuis le disque pour la ROM courante.
    fn load_savestates_for_rom(&mut self) {
        self.savestates = [None, None, None];
        self.savestate_meta = [None, None, None];
        let Some(dir) = self.savestate_dir_for_rom() else {
            return;
        };
        let rom_hash = rom_hash_hex(&self.rom_data);
        let rom_name = self
            .rom_path
            .as_ref()
            .or(self.last_rom_path.as_ref())
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .unwrap_or("rom");
        for slot in 0..3 {
            let Some(path) = Self::latest_savestate_file(&dir, slot, rom_name) else {
                continue;
            };
            let Ok(data) = fs::read(&path) else {
                continue;
            };
            let Ok(payload) = serde_json::from_slice::<PersistedSaveState>(&data) else {
                continue;
            };
            if payload.rom_hash != rom_hash || payload.slot != slot {
                continue;
            }
            self.savestates[slot] = Some(payload.snapshot);
            self.savestate_meta[slot] = Some(payload.meta);
        }
    }

    // EN: Save full emulator snapshot into selected slot.
    // FR: Sauvegarde un snapshot complet de l emulateur dans le slot selectionne.
    // EN: Save full emulator snapshot into selected slot (commit).
    // FR: Sauvegarde un snapshot complet de l emulateur dans le slot selectionne (commit).
    fn save_state_slot_commit(&mut self, slot: usize, manual: bool) {
        let t = tr(self.langue);
        if slot >= self.savestates.len() {
            return;
        }
        self.savestates[slot] = Some(EmuSnapshot {
            cpu: self.cpu.clone(),
            display: self.display.clone(),
            memory: self.cpu.memory.to_vec(),
        });
        let timestamp = timestamp_for_display();
        let rom_name = self
            .rom_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("no-rom")
            .to_owned();
        self.savestate_meta[slot] = Some(SaveStateMeta {
            name: rom_name.clone(),
            timestamp: timestamp.clone(),
        });

        let base = format!("{} {}", t.status_state_saved_slot, slot + 1);
        if manual {
            self.status_message = format!("{} - {} - {}", base, timestamp, rom_name);
        } else {
            self.status_message = base;
        }
        self.show_display_overlay(self.status_message.clone());
        self.persist_savestate(slot);
    }

    // EN: Save full emulator snapshot into selected slot.
    // FR: Sauvegarde un snapshot complet de l emulateur dans le slot selectionne.
    fn save_state_slot_internal(&mut self, slot: usize, manual: bool) {
        if manual && self.savestate_meta.get(slot).and_then(|m| m.as_ref()).is_some() {
            self.confirm_overwrite_slot = Some(slot);
            return;
        }
        self.save_state_slot_commit(slot, manual);
    }

    // EN: Save state invoked via UI menu (adds timestamp/name).
    // FR: Sauvegarde d etat via menu UI (ajoute horodatage/nom).
    pub(crate) fn save_state_slot_manual(&mut self, slot: usize) {
        self.save_state_slot_internal(slot, true);
    }

    // EN: Save state invoked via shortcut (simple message).
    // FR: Sauvegarde d etat via raccourci (message simple).
    pub(crate) fn save_state_slot_shortcut(&mut self, slot: usize) {
        self.save_state_slot_internal(slot, false);
    }

    // EN: Load emulator snapshot from selected slot.
    // FR: Charge un snapshot d emulateur depuis le slot selectionne.
    pub(crate) fn load_state_slot(&mut self, slot: usize) {
        let t = tr(self.langue);
        if slot >= self.savestates.len() {
            return;
        }
        if let Some(snapshot) = &self.savestates[slot] {
            self.cpu = snapshot.cpu.clone();
            self.display = snapshot.display.clone();
            if snapshot.memory.len() == self.cpu.memory.len() {
                self.cpu.memory.copy_from_slice(&snapshot.memory);
            } else if !self.rom_data.is_empty() {
                let _ = self.cpu.load_program(&self.rom_data);
            }
            self.rom_chargee = true;
            self.en_pause = false;
            self.reset_runtime_clocks();
            self.status_message = format!("{} {}", t.status_state_loaded_slot, slot + 1);
            self.show_display_overlay(self.status_message.clone());
            debug::log("savestate_loaded");
        } else {
            self.status_message = format!("{} {}", t.status_slot_empty, slot + 1);
            self.show_display_overlay(self.status_message.clone());
            debug::log("savestate_empty");
        }
    }

    // EN: Load savestate from a file selected by the user.
    // FR: Charge un savestate depuis un fichier selectionne.
    pub(crate) fn load_state_file_dialog(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Oxide State", &["state", "json"])
            .pick_file()
        else {
            return;
        };
        self.load_state_file_path(&path);
    }

    // EN: Load savestate from a provided path (CLI or picker).
    // FR: Charge un savestate depuis un chemin fourni (CLI ou selection).
    pub(crate) fn load_state_file_path(&mut self, path: &PathBuf) {
        let t = tr(self.langue);
        if self.rom_data.is_empty() {
            return;
        }
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("state");
        match fs::read(path) {
            Ok(bytes) => match serde_json::from_slice::<PersistedSaveState>(&bytes) {
                Ok(payload) => {
                    let current_hash = rom_hash_hex(&self.rom_data);
                    if payload.rom_hash != current_hash {
                        return;
                    }
                    self.cpu = payload.snapshot.cpu;
                    self.display = payload.snapshot.display;
                    if payload.snapshot.memory.len() == self.cpu.memory.len() {
                        self.cpu.memory.copy_from_slice(&payload.snapshot.memory);
                    } else if !self.rom_data.is_empty() {
                        let _ = self.cpu.load_program(&self.rom_data);
                    }
                    if !payload.rom_bytes.is_empty() {
                        self.rom_data = payload.rom_bytes;
                    }
                    self.rom_chargee = true;
                    self.en_pause = false;
                    self.reset_runtime_clocks();
                    self.status_message = format!("{}: {}", t.status_state_loaded_file, file_name);
                    self.show_display_overlay(self.status_message.clone());
                }
                Err(err) => {
                    self.status_message =
                        format!("{}: {} ({})", t.status_state_load_failed, file_name, err);
                    self.show_display_overlay(self.status_message.clone());
                }
            },
            Err(err) => {
                self.status_message =
                    format!("{}: {} ({})", t.status_state_load_failed, file_name, err);
                self.show_display_overlay(self.status_message.clone());
            }
        }
    }

    // EN: Show a short-lived message overlay inside the main display area.
    // FR: Affiche un message temporaire superpose dans la zone de display principale.
    fn show_display_overlay(&mut self, message: String) {
        self.display_overlay_message = message;
        self.display_overlay_until = Instant::now() + Duration::from_secs(1);
    }

    // EN: Toggle pause state and show a localized overlay.
    // FR: Bascule l etat pause et affiche un overlay localise.
    pub(crate) fn toggle_pause(&mut self) {
        let t = tr(self.langue);
        self.en_pause = !self.en_pause;
        self.status_message = if self.en_pause {
            t.status_paused.clone()
        } else {
            t.status_resumed.clone()
        };
        self.show_display_overlay(self.status_message.clone());
    }

    // EN: Set fullscreen state and show a localized overlay.
    // FR: Defini l etat plein ecran et affiche un overlay localise.
    pub(crate) fn set_fullscreen(&mut self, fullscreen: bool) {
        if self.fullscreen == fullscreen {
            return;
        }
        let t = tr(self.langue);
        self.fullscreen = fullscreen;
        self.status_message = if self.fullscreen {
            t.status_fullscreen_on.clone()
        } else {
            t.status_fullscreen_off.clone()
        };
        self.show_display_overlay(self.status_message.clone());
    }

    // EN: Toggle fullscreen state and show a localized overlay.
    // FR: Bascule le plein ecran et affiche un overlay localise.
    fn toggle_fullscreen(&mut self) {
        let target = !self.fullscreen;
        self.set_fullscreen(target);
    }

    // EN: Merge keyboard/gamepad/terminal-key states into the CHIP-8 keypad.
    // FR: Fusionne les etats clavier/manette/terminal dans le clavier CHIP-8.
    fn update_keypad_from_input(&mut self, ctx: &egui::Context) {
        if !self.can_interact_with_rom() {
            self.keypad.set_all([false; 16]);
            return;
        }
        let mut states = [false; 16];
        ctx.input(|i| {
            for (idx, key_name) in self.touches.iter().enumerate() {
                // EN: Check keyboard key.
                // FR: Verifie la touche clavier.
                if let Some(key) = key_from_label(key_name) {
                    states[idx] = i.key_down(key);
                }
                // EN: Check mouse button.
                // FR: Verifie le bouton souris.
                if let Some(btn) = mouse_from_label(key_name) {
                    states[idx] = states[idx] || i.pointer.button_down(btn);
                }
            }
        });
        let gamepad_states = gamepad::poll_chip8_keys();
        for idx in 0..16 {
            states[idx] = states[idx] || self.terminal_keypad_states[idx] || gamepad_states[idx];
        }
        self.keypad.set_all(states);
    }

    // EN: Run emulation clocks (CPU cycles + 60 Hz timers) for this frame.
    // FR: Execute les horloges d emulation (cycles CPU + timers 60 Hz) pour cette frame.
    fn run_emulator_step(&mut self, dt: f32) {
        if !self.rom_chargee || self.en_pause {
            self.audio_engine.stop();
            return;
        }

        self.cpu_step_accumulator += dt * self.cycles_par_seconde as f32;
        // EN: Cap catch-up cycles to avoid UI stalls during move/resize.
        // FR: Limite les cycles de rattrapage pour eviter les freezes UI pendant deplacement/redimensionnement.
        const MAX_CYCLES_PER_FRAME: usize = 2_000;
        let mut executed_cycles = 0usize;
        while self.cpu_step_accumulator >= 1.0 && executed_cycles < MAX_CYCLES_PER_FRAME {
            self.cpu.cycle(&mut self.display, &self.keypad, self.quirks);
            self.cpu_step_accumulator -= 1.0;
            executed_cycles += 1;
        }
        if self.cpu_step_accumulator > MAX_CYCLES_PER_FRAME as f32 {
            // EN: Trim backlog when the emulator falls too far behind.
            // FR: Reduit l arriere-log quand l emulateur prend trop de retard.
            self.cpu_step_accumulator = MAX_CYCLES_PER_FRAME as f32;
        }

        self.timer_accumulator += dt * 60.0;
        while self.timer_accumulator >= 1.0 {
            self.cpu.tick_timers();
            self.timer_accumulator -= 1.0;
        }

        let buzzer_active = self.son_active && self.sound_volume > 0 && self.cpu.sound_timer > 0;
        self.audio_engine.set_buzzer(buzzer_active, self.sound_volume);
    }    // EN: Append user-facing status into terminal logs when it changes.
    // FR: Ajoute le statut utilisateur dans les logs terminal quand il change.
    fn update_terminal_log(&mut self) {
        if self.status_message.is_empty() || self.status_message == self.last_status_logged {
            return;
        }
        let line = format!("{} |I| {}", self.terminal_timestamp(), self.status_message);
        self.append_emulator_log(&line);
        self.push_terminal_plain_line(line);
        self.last_status_logged = self.status_message.clone();
    }

    // EN: Ensure log files are initialized for this session.
    // FR: Verifie que les fichiers de log sont initialises pour cette session.
    fn ensure_log_files(&mut self) {
        if self.terminal_log_file.is_none() || self.emulator_log_file.is_none() {
            let (terminal, emulator) = init_log_files();
            if self.terminal_log_file.is_none() {
                self.terminal_log_file = terminal;
            }
            if self.emulator_log_file.is_none() {
                self.emulator_log_file = emulator;
            }
        }
    }

    // EN: Append one line to the terminal log file.
    // FR: Ajoute une ligne au fichier de log du terminal.
    fn append_terminal_log(&mut self, line: &str) {
        self.ensure_log_files();
        if let Some(file) = &self.terminal_log_file {
            if let Ok(mut file) = file.lock() {
                let _ = writeln!(file, "{}", line);
                let _ = file.flush();
            }
        }
    }

    // EN: Append one line to the emulator log file.
    // FR: Ajoute une ligne au fichier de log de l emulation.
    fn append_emulator_log(&mut self, line: &str) {
        self.ensure_log_files();
        if let Some(file) = &self.emulator_log_file {
            if let Ok(mut file) = file.lock() {
                let _ = writeln!(file, "{}", line);
                let _ = file.flush();
            }
        }
    }

    // EN: Push one raw terminal line without timestamp.
    // FR: Ajoute une ligne terminal brute sans horodatage.
    fn push_terminal_plain_line(&mut self, message: String) {
        self.terminal_logs.push(message.clone());
        if !self.terminal_logs_text.is_empty() {
            self.terminal_logs_text.push('\n');
        }
        self.terminal_logs_text.push_str(&message);
        self.append_terminal_log(&message);
    }

    // EN: Format elapsed terminal uptime as HH:MM:SS.mmm.
    // FR: Formate le temps terminal ecoule en HH:MM:SS.mmm.
    fn terminal_timestamp(&self) -> String {
        let elapsed = self.terminal_clock_origin.elapsed();
        let total_secs = elapsed.as_secs();
        let h = total_secs / 3600;
        let m = (total_secs % 3600) / 60;
        let s = total_secs % 60;
        let ms = elapsed.subsec_millis();
        format!("{:02}:{:02}:{:02}.{:03}", h, m, s, ms)
    }

    // EN: Push one timestamped terminal log line with level marker.
    // FR: Ajoute une ligne de log terminal horodatee avec marqueur de niveau.
    fn push_terminal_log_line(&mut self, level: &str, message: String) {
        let line = format!("{} |{}| {}", self.terminal_timestamp(), level, message);
        self.push_terminal_plain_line(line);
    }

    // EN: Push one raw terminal line without timestamp.
    // FR: Ajoute une ligne terminal brute sans horodatage.

    // EN: Emit a compact diagnostics block meant for ROM test analysis.
    // FR: Emet un bloc de diagnostic compact destine a l analyse des ROMs de test.
    pub(crate) fn emit_test_report(&mut self, trigger: &str) {
        let t = tr(self.langue);
        let preset = match self.quirks_preset {
            QuirkPreset::Chip8 => "CHIP-8",
            QuirkPreset::Chip48 => "CHIP-48",
            QuirkPreset::SuperChip => "SUPER-CHIP",
            QuirkPreset::Custom => "Custom",
        };
        let rom_name = self
            .rom_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("<none>")
            .to_owned();

        let pc = self.cpu.pc as usize;
        let opcode = if pc + 1 < self.cpu.memory.len() {
            ((self.cpu.memory[pc] as u16) << 8) | self.cpu.memory[pc + 1] as u16
        } else {
            0
        };
        let lit_pixels = self.display.pixels.iter().filter(|v| **v != 0).count();
        let pressed_keys = self
            .terminal_keypad_states
            .iter()
            .enumerate()
            .filter_map(|(i, v)| if *v { Some(format!("{:X}", i)) } else { None })
            .collect::<Vec<String>>()
            .join(",");
        let keys_view = if pressed_keys.is_empty() {
            "-".to_owned()
        } else {
            pressed_keys
        };
        let regs = self
            .cpu
            .v
            .iter()
            .enumerate()
            .map(|(i, v)| format!("V{:X}={:02X}", i, v))
            .collect::<Vec<String>>()
            .join(" ");

        self.push_terminal_plain_line(String::new());
        self.push_terminal_plain_line(t.test_report_header.to_owned());
        self.push_terminal_log_line("I", format!("{}: {}", t.test_report_trigger, trigger));
        self.push_terminal_log_line("I", format!("{}: {}", t.test_report_rom_label, rom_name));
        self.push_terminal_log_line(
            "I",
            format!(
                "{}: {}={} {}={} {}={}",
                t.test_report_state,
                t.test_report_state_running,
                self.state_label_for_langue(!self.en_pause && self.rom_chargee),
                t.test_report_state_paused,
                self.state_label_for_langue(self.en_pause),
                t.test_report_state_rom_loaded,
                self.state_label_for_langue(self.rom_chargee)
            ),
        );
        self.push_terminal_log_line(
            "I",
            format!(
                "{}: PC={:04X} OP={:04X} I={:04X} SP={:02X} DT={} ST={}",
                t.test_report_cpu_label,
                self.cpu.pc, opcode, self.cpu.i, self.cpu.sp, self.cpu.delay_timer, self.cpu.sound_timer
            ),
        );
        self.push_terminal_log_line(
            "I",
            format!("{}: lit_pixels={} keypad={}", t.test_report_video, lit_pixels, keys_view),
        );
        self.push_terminal_log_line(
            "I",
            format!(
                "{}: preset={} shift_vy={} jump_vx={} draw_clips={} inc_i={} logic_vf0={}",
                t.test_report_quirks_label,
                preset,
                self.quirks.shift_uses_vy,
                self.quirks.jump_uses_vx,
                self.quirks.draw_clips,
                self.quirks.load_store_increment_i,
                self.quirks.logic_clears_vf
            ),
        );
        self.push_terminal_plain_line(regs);
        self.push_terminal_plain_line(t.test_report_header.to_owned());
    }

    // EN: Return localized state word for enabled/disabled values.
    // FR: Retourne le mot localise pour les etats active/desactive.
    fn state_label_for_langue(&self, value: bool) -> String {
        let t = tr(self.langue);
        if value {
            t.terminal_enabled.clone()
        } else {
            t.terminal_disabled.clone()
        }
    }

    // EN: Return localized theme label.
    // FR: Retourne le libelle localise du theme.
    fn theme_label_for_langue(&self, theme: AppTheme) -> String {
        let t = tr(self.langue);
        match theme {
            AppTheme::Kiwano => t.terminal_kiwano.clone(),
            AppTheme::Dark => t.terminal_dark.clone(),
            AppTheme::Light => t.terminal_light.clone(),
        }
    }
    // EN: Seed startup/debug context lines once per app run.
    // FR: Initialise les lignes de contexte demarrage/debug une seule fois par execution.
    fn seed_terminal_boot_logs(&mut self) {
        if self.terminal_boot_logs_seeded {
            return;
        }
        self.terminal_clock_origin = Instant::now();
        let t = tr(self.langue);
        self.push_terminal_plain_line(t.terminal_legend.clone());
        self.push_terminal_plain_line(t.terminal_info_legend.clone());
        self.push_terminal_plain_line(String::new());
        self.push_terminal_log_line("I", format!("Oxide v{}", VERSION));
        self.push_terminal_log_line(
            "I",
            format!("{}: {} ({})", t.terminal_platform, std::env::consts::OS, std::env::consts::ARCH),
        );
        self.push_terminal_log_line("I", t.terminal_runtime_ready.clone());
        self.push_terminal_log_line("I", format!("{}: {}", t.terminal_theme, self.theme_label_for_langue(self.theme)));
        self.push_terminal_log_line("I", format!("{}: {}", t.terminal_language, self.langue.label()));
        self.push_terminal_log_line(
            "I",
            format!(
                "{} : {}",
                t.terminal_fps,
                if self.vsync { "60".to_owned() } else { t.terminal_unlimited.clone() }
            ),
        );
        self.push_terminal_log_line("I", format!("{}: {}", t.terminal_vsync, self.state_label_for_langue(self.vsync)));
        self.push_terminal_log_line(
            "I",
            format!("{}: {}", t.terminal_fullscreen, self.state_label_for_langue(self.fullscreen)),
        );
        self.push_terminal_log_line("I", format!("{}: {}", t.terminal_sound, self.state_label_for_langue(self.son_active)));
        self.push_terminal_log_line("I", t.terminal_core_init.clone());
        self.push_terminal_log_line("I", t.terminal_display_init.clone());
        self.push_terminal_log_line("I", t.terminal_input_init.clone());
        self.push_terminal_log_line(
            "I",
            format!(
                "{} ({})",
                t.terminal_audio_init,
                if self.son_active {
                    t.terminal_enabled.clone()
                } else {
                    t.terminal_disabled.clone()
                }
            ),
        );
        self.last_logged_theme = self.theme;
        self.last_logged_langue = self.langue;
        self.last_logged_vsync = self.vsync;
        self.last_logged_fullscreen = self.fullscreen;
        self.last_logged_son_active = self.son_active;
        self.terminal_boot_logs_seeded = true;
    }

    // EN: Log runtime configuration changes when values differ from last logged state.
    // FR: Journalise les changements de configuration runtime quand les valeurs different du dernier etat loggue.
    fn log_config_changes(&mut self) {
        let t = tr(self.langue);
        if self.theme != self.last_logged_theme {
            self.push_terminal_log_line("I", format!("{}: {}", t.terminal_theme, self.theme_label_for_langue(self.theme)));
            self.last_logged_theme = self.theme;
        }
        if self.langue != self.last_logged_langue {
            self.push_terminal_log_line("I", format!("{}: {}", t.terminal_language, self.langue.label()));
            self.last_logged_langue = self.langue;
        }
        if self.vsync != self.last_logged_vsync {
            self.push_terminal_log_line("I", format!("{}: {}", t.terminal_vsync, self.state_label_for_langue(self.vsync)));
            self.push_terminal_log_line(
                "I",
                format!(
                    "{} : {}",
                    t.terminal_fps,
                    if self.vsync { "60".to_owned() } else { t.terminal_unlimited.clone() }
                ),
            );
            self.last_logged_vsync = self.vsync;
        }
        if self.fullscreen != self.last_logged_fullscreen {
            self.push_terminal_log_line(
                "I",
                format!("{}: {}", t.terminal_fullscreen, self.state_label_for_langue(self.fullscreen)),
            );
            self.last_logged_fullscreen = self.fullscreen;
        }
        if self.son_active != self.last_logged_son_active {
            self.push_terminal_log_line("I", format!("{}: {}", t.terminal_sound, self.state_label_for_langue(self.son_active)));
            self.last_logged_son_active = self.son_active;
        }
    }
}

impl eframe::App for Oxide {
    // EN: Main frame update loop (input, emulation tick, UI rendering, viewport sync).
    // FR: Boucle principale de mise a jour de frame (entrees, tick emulation, rendu UI, sync viewport).
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            if self.splash_active {
            let started = self.splash_started.get_or_insert_with(Instant::now);
            if started.elapsed().as_secs_f32() >= 5.0 {
                self.splash_active = false;
                self.splash_started = None;
                self.splash_launch_log_pending = true;
                debug::log("oxide_launched");
                self.focus_main_requested = true;
                self.video_scale_precedent = 0;
                let window_size = crate::utils::fenetre_size(self.video_scale);
                ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Title("Oxide".to_owned()));
                ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Transparent(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                    window_size[0],
                    window_size[1],
                )));
            } else {
                let splash_time = started.elapsed().as_secs_f32();
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE.inner_margin(egui::Margin::ZERO))
                    .show(ctx, |ui| {
                        let rect = ui.max_rect();
                        let painter = ui.painter();

                        if self.splash_texture.is_none() {
                            let png_bytes = include_bytes!("assets/logo/logo.png");
                            if let Ok(image) = image::load_from_memory(png_bytes) {
                                let (w, h) = image.dimensions();
                                let rgba = image.to_rgba8();
                                let pixels = rgba.as_flat_samples();
                                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                    [w as usize, h as usize],
                                    pixels.as_slice(),
                                );
                                let texture = ctx.load_texture(
                                    "splash_logo",
                                    color_image,
                                    egui::TextureOptions::LINEAR,
                                );
                                self.splash_texture = Some(texture);
                                self.splash_size = egui::vec2(w as f32, h as f32);
                            }
                        }

                        if let Some(texture) = &self.splash_texture {
                            let available = rect.shrink(24.0);
                            let version_gap = 4.0;
                            let version_band = 26.0;
                            let image_area = egui::Rect::from_min_max(
                                available.min,
                                egui::pos2(available.max.x, available.max.y - version_gap - version_band),
                            );
                            let scale = (image_area.width() / self.splash_size.x)
                                .min(image_area.height() / self.splash_size.y)
                                .min(1.0);
                            let base_size = self.splash_size * scale;
                            let base_image_rect = egui::Rect::from_center_size(image_area.center(), base_size);
                            let pulse_scale = 1.0 + 0.035 * (splash_time * 1.6).sin();
                            let image_rect = egui::Rect::from_center_size(image_area.center(), base_size * pulse_scale);
                            painter.image(
                                texture.id(),
                                image_rect,
                                egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                ),
                                egui::Color32::WHITE,
                            );

                            let version_text = format!("v{}", VERSION);
                            let version_pos = egui::pos2(
                                base_image_rect.min.x + base_image_rect.width() * 0.14,
                                base_image_rect.max.y - base_image_rect.height() * 0.15,
                            );
                            let version_font_size = 28.0;
                            let version_font = egui::FontId::proportional(version_font_size);
                            let version_layer = ctx.layer_painter(egui::LayerId::new(
                                egui::Order::Foreground,
                                egui::Id::new("splash_version_layer"),
                            ));
                            let pulse_phase = ((pulse_scale - 1.0) / 0.035).clamp(-1.0, 1.0);
                            let glow_alpha = (10.0 + (pulse_phase + 1.0) * 5.0).round() as u8;
                            let text_alpha = (40.0 + (pulse_phase + 1.0) * 4.0).round() as u8;
                            let glow_offsets = [
                                egui::vec2(-0.6, 0.0),
                                egui::vec2(0.6, 0.0),
                                egui::vec2(0.0, -0.6),
                                egui::vec2(0.0, 0.6),
                                egui::vec2(-0.45, -0.45),
                                egui::vec2(0.45, -0.45),
                                egui::vec2(-0.45, 0.45),
                                egui::vec2(0.45, 0.45),
                            ];
                            for offset in glow_offsets {
                                version_layer.text(
                                    version_pos + offset,
                                    egui::Align2::LEFT_BOTTOM,
                                    &version_text,
                                    version_font.clone(),
                                    egui::Color32::from_rgba_unmultiplied(255, 238, 238, glow_alpha),
                                );
                            }
                            version_layer.text(
                                version_pos,
                                egui::Align2::LEFT_BOTTOM,
                                version_text,
                                version_font,
                                egui::Color32::from_rgba_unmultiplied(255, 255, 255, text_alpha),
                            );
                        }
                    });
                ctx.request_repaint();
                return;
            }
        }
        if let Some(path) = self.pending_open_path.take() {
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            if ext == "state" {
                self.load_state_file_path(&path);
            } else if ["ch8", "rom", "bin"].contains(&ext.as_str()) {
                let _ = self.load_rom_path(path);
            }
        }
        self.fenetre_principale_size = ctx.content_rect().size();
        if let Some(rect) = ctx.input(|i| i.viewport().outer_rect) {
            self.fenetre_principale_pos = rect.min;
        }
        self.fullscreen_active = ctx.input(|i| i.viewport().fullscreen.unwrap_or(self.fullscreen));
        self.window_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        if self.rom_data.is_empty() {
            self.rom_chargee = false;
        }
        let main_focused = ctx.input(|i| i.viewport().focused.unwrap_or(true));
        if main_focused && !self.last_main_focused && self.fenetre_settings {
            self.focus_settings_requested = true;
        }
        self.last_main_focused = main_focused;
        if self.focus_main_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            self.focus_main_requested = false;
        }
        if ctx.input(|i| i.pointer.any_pressed()) {
            if self.fenetre_settings {
                self.focus_settings_requested = true;
            }
        }

        if !self.fullscreen_active && !self.window_maximized {
            let display_w = crate::constants::CHIP8_W * crate::constants::PIXEL_BASE * self.video_scale as f32;
            let display_h = crate::constants::CHIP8_H * crate::constants::PIXEL_BASE * self.video_scale as f32;
            let size = [
                display_w,
                display_h + self.top_bar_height + self.bottom_bar_height,
            ];
            let current = ctx.input(|i| i.viewport().inner_rect.map(|r| r.size()));
            let needs_resize = match current {
                Some(sz) => (sz.x - size[0]).abs() > 0.5 || (sz.y - size[1]).abs() > 0.5,
                None => true,
            };
            if needs_resize {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(size[0], size[1])));
            }
        }
        if self.video_scale != self.video_scale_precedent {
            self.video_scale_precedent = self.video_scale;
        }

        self.handle_shortcuts(ctx);

        if self.fullscreen != self.fullscreen_precedent {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
            self.fullscreen_precedent = self.fullscreen;
        }

        if self.cursor_hidden {
            ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(false));
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(true));
        }

        self.update_keypad_from_input(ctx);
        let dt = ctx.input(|i| i.stable_dt).max(0.0);
        self.configured_game_fps = if self.vsync { 60.0 } else { f32::INFINITY };
        self.run_emulator_step(dt);
        self.seed_terminal_boot_logs();
        if self.splash_launch_log_pending {
            self.push_terminal_log_line("I", tr(self.langue).terminal_app_launched.clone());
            self.splash_launch_log_pending = false;
        }
        self.log_config_changes();
        self.update_terminal_log();

        ctx.set_visuals(visuals_for_theme(self.theme));
        if !self.fullscreen_active {
            ui::top_bar::show(ctx, self);
            ui::bottom_bar::show(ctx, self);
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.inner_margin(egui::Margin::ZERO))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                ui::main_panel::show(ui, self);
            });

        if self.fenetre_settings {
            ui::settings::show(ctx, self);
        }
        if self.terminal_active {
            ui::debug_terminal::show(ctx, self);
        } else {
            self.terminal_position_initialized = false;
            self.terminal_keypad_states = [false; 16];
        }

        if let Some(slot) = self.confirm_overwrite_slot {
            let t = tr(self.langue);
            let mut keep_open = true;
            egui::Window::new(t.confirm_overwrite_state.clone())
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .open(&mut keep_open)
                .show(ctx, |ui| {
                    ui.label(format!("{} {}", t.confirm_overwrite_state, slot + 1));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(t.confirm_yes.clone()).clicked() {
                            self.confirm_overwrite_slot = None;
                            self.save_state_slot_commit(slot, true);
                        }
                        if ui.button(t.cancel.clone()).clicked() {
                            self.confirm_overwrite_slot = None;
                        }
                    });
                });
            if !keep_open {
                self.confirm_overwrite_slot = None;
            }
        }

        if self.rom_chargee && !self.en_pause {
            ctx.request_repaint();
        }
        if Instant::now() < self.display_overlay_until {
            ctx.request_repaint();
        }
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        if self.splash_active {
            egui::Color32::TRANSPARENT.to_normalized_gamma_f32()
        } else {
            visuals.panel_fill.to_normalized_gamma_f32()
        }
    }

    // EN: Persist user-configurable state while stripping runtime-only transient data.
    // FR: Persiste l etat configurable utilisateur en retirant les donnees runtime transitoires.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let mut clone = self.clone();
        clone.fenetre_settings = false;
        clone.keypad = Keypad::new();
        clone.rom_data.clear();
        clone.rom_path = None;
        clone.rom_chargee = false;
        clone.en_pause = false;
        clone.display = Display::new();
        clone.cpu_step_accumulator = 0.0;
        clone.timer_accumulator = 0.0;
        clone.audio_engine = AudioEngine::default();
        clone.terminal_keypad_states = [false; 16];
        clone.terminal_logs.clear();
        clone.terminal_log_file = None;
        clone.terminal_logs_text.clear();
        clone.terminal_search_query.clear();
        clone.last_status_logged.clear();
        clone.pending_open_path = None;
        clone.emulator_log_file = None;
        clone.display_overlay_message.clear();
        clone.display_overlay_until = Instant::now();
        clone.focus_settings_requested = false;
        clone.settings_position_initialized = false;
        clone.focus_terminal_requested = false;
        clone.focus_main_requested = false;
        clone.last_main_focused = true;
        clone.terminal_position_initialized = false;
        clone.terminal_rows_at_open = 0;
        clone.terminal_view_session = 0;
        clone.terminal_boot_logs_seeded = false;
        clone.splash_launch_log_pending = false;
        clone.last_logged_theme = clone.theme;
        clone.last_logged_langue = clone.langue;
        clone.last_logged_vsync = clone.vsync;
        clone.last_logged_fullscreen = clone.fullscreen;
        clone.last_logged_son_active = clone.son_active;
        clone.configured_game_fps = if clone.vsync { 60.0 } else { f32::INFINITY };
        clone.savestates = [None, None, None];
        eframe::set_value(storage, eframe::APP_KEY, &clone);
    }
}



















































