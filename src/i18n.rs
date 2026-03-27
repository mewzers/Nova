use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::Value;

use crate::types::Langue;
use crate::debug;
// EN: Aggregated UI and terminal localized strings.
// FR: Regroupe les chaines localisees de l interface et du terminal.
#[derive(Debug, Deserialize)]
pub struct Translations {
    pub game: String,
    pub load_game: String,
    pub load_recent_rom: String,
    pub resume: String,
    pub pause: String,
    pub reset: String,
    pub stop: String,
    pub save_state: String,
    pub load_state: String,
    pub load_state_file: String,
    pub slot_empty: String,
    pub fps_limiter: String,

    pub emulator: String,
    pub emulator_settings: String,
    pub debug_settings: String,
    pub languages: String,

    pub video: String,
    pub video_settings: String,
    pub render_size: String,
    pub vsync: String,
    pub fullscreen: String,

    pub controls: String,
    pub shortcuts: String,
    pub theme: String,
    pub theme_default_suffix: String,
    pub theme_kiwano_icon: String,
    pub theme_dark_icon: String,
    pub theme_light_icon: String,

    pub settings_title: String,
    pub ok: String,
    pub apply: String,
    pub defaults: String,
    pub defaults_all: String,
    pub confirm_reset_all: String,
    pub confirm_yes: String,
    pub confirm_overwrite_state: String,
    pub settings_pending_apply: String,
    pub cancel: String,

    pub tab_emulator: String,
    pub tab_video: String,
    pub tab_audio: String,
    pub tab_controls: String,
    pub tab_shortcuts: String,
    pub tab_debug: String,

    pub language: String,
    pub video_config: String,
    pub audio_config: String,
    pub key_config: String,

    pub keyboard_shortcuts: String,
    pub shortcut_pause: String,
    pub shortcut_reset: String,
    pub shortcut_stop: String,
    pub shortcut_load_game: String,
    pub shortcut_fullscreen: String,
    pub shortcut_save_slot_1: String,
    pub shortcut_save_slot_2: String,
    pub shortcut_save_slot_3: String,
    pub shortcut_load_slot_1: String,
    pub shortcut_load_slot_2: String,
    pub shortcut_load_slot_3: String,

    pub settings_cpu_hz: String,
    pub settings_cpu_hz_note: String,
    pub settings_sound: String,
    pub settings_volume: String,
    pub settings_debug_terminal: String,
    pub compat_quirks_title: String,
    pub quirks_preset_label: String,
    pub quirks_preset_chip8: String,
    pub quirks_preset_chip48: String,
    pub quirks_preset_superchip: String,
    pub quirks_preset_custom: String,
    pub quirks_shift_uses_vy: String,
    pub quirks_jump_uses_vx: String,
    pub quirks_draw_clips: String,
    pub quirks_load_store_increment_i: String,
    pub quirks_logic_clears_vf: String,
    pub terminal_title: String,
    pub terminal_legend: String,
    pub terminal_info_legend: String,
    pub terminal_runtime_ready: String,
    pub terminal_platform: String,
    pub terminal_theme: String,
    pub terminal_language: String,
    pub terminal_fps: String,
    pub terminal_vsync: String,
    pub terminal_fullscreen: String,
    pub terminal_sound: String,
    pub terminal_core_init: String,
    pub terminal_display_init: String,
    pub terminal_input_init: String,
    pub terminal_audio_init: String,
    pub terminal_kiwano: String,
    pub terminal_dark: String,
    pub terminal_light: String,
    pub terminal_enabled: String,
    pub terminal_disabled: String,
    pub terminal_unlimited: String,
    pub status_rom_loaded: String,
    pub status_rom_load_failed: String,
    pub status_no_rom_loaded: String,
    pub status_rom_reset: String,
    pub status_emulation_stopped: String,
    pub status_paused: String,
    pub status_resumed: String,
    pub status_fullscreen_on: String,
    pub status_fullscreen_off: String,
    pub status_state_saved_slot: String,
    pub status_state_loaded_slot: String,
    pub status_state_loaded_file: String,
    pub status_state_load_failed: String,
    pub status_slot_empty: String,

    pub bottom_rom_loaded: String,
    pub bottom_rom_none: String,
    pub bottom_state_paused: String,
    pub bottom_state_running: String,
    pub bottom_cpu_hz: String,
    pub bytes_unit: String,

    pub test_report_header: String,
    pub test_report_trigger: String,
    pub test_report_state: String,
    pub test_report_state_running: String,
    pub test_report_state_paused: String,
    pub test_report_state_rom_loaded: String,
    pub test_report_video: String,
    pub test_report_rom_label: String,
    pub test_report_cpu_label: String,
    pub test_report_quirks_label: String,
    pub terminal_search_hint: String,
    pub terminal_test_report_btn: String,
    pub terminal_copy: String,
    pub terminal_export_logs: String,
    pub terminal_export_success: String,
    pub terminal_export_error: String,
    pub terminal_app_launched: String,
}

// EN: Parse one translation JSON merged with common shared keys.
// FR: Parse un JSON de traduction fusionne avec des cles communes partagees.
fn parse_translation(common_raw: &str, raw: &str) -> Translations {
    fn parse_object(raw: &str) -> Option<serde_json::Map<String, Value>> {
        let cleaned = raw.trim_start_matches('\u{feff}');
        serde_json::from_str::<Value>(cleaned).ok()?.as_object().cloned()
    }

    let build = || {
        let mut merged = parse_object(common_raw)?;
        let overrides = parse_object(raw)?;
        merged.extend(overrides);
        serde_json::from_value::<Translations>(Value::Object(merged)).ok()
    };

    match build() {
        Some(tr) => tr,
        None => {
            debug::log("translation_parse_failed");
            let fallback_common = include_str!("i18n/common.json");
            let fallback_lang = include_str!("i18n/en.json");
            let mut merged = parse_object(fallback_common).expect("Invalid fallback common translation JSON");
            let overrides = parse_object(fallback_lang).expect("Invalid fallback translation JSON");
            merged.extend(overrides);
            serde_json::from_value(Value::Object(merged)).expect("Invalid merged fallback translation JSON")
        }
    }
}

// EN: Static translation tables loaded once at startup.
// FR: Tables de traduction statiques chargees une seule fois au demarrage.
static FR: Lazy<Translations> = Lazy::new(|| parse_translation(include_str!("i18n/common.json"), include_str!("i18n/fr.json")));
static EN: Lazy<Translations> = Lazy::new(|| parse_translation(include_str!("i18n/common.json"), include_str!("i18n/en.json")));
static ES: Lazy<Translations> = Lazy::new(|| parse_translation(include_str!("i18n/common.json"), include_str!("i18n/es.json")));
static IT: Lazy<Translations> = Lazy::new(|| parse_translation(include_str!("i18n/common.json"), include_str!("i18n/it.json")));
static DE: Lazy<Translations> = Lazy::new(|| parse_translation(include_str!("i18n/common.json"), include_str!("i18n/de.json")));
static PT: Lazy<Translations> = Lazy::new(|| parse_translation(include_str!("i18n/common.json"), include_str!("i18n/pt.json")));
static RU: Lazy<Translations> = Lazy::new(|| parse_translation(include_str!("i18n/common.json"), include_str!("i18n/ru.json")));
static ZH: Lazy<Translations> = Lazy::new(|| parse_translation(include_str!("i18n/common.json"), include_str!("i18n/zh.json")));
static JA: Lazy<Translations> = Lazy::new(|| parse_translation(include_str!("i18n/common.json"), include_str!("i18n/ja.json")));
static KO: Lazy<Translations> = Lazy::new(|| parse_translation(include_str!("i18n/common.json"), include_str!("i18n/ko.json")));
static AR: Lazy<Translations> = Lazy::new(|| parse_translation(include_str!("i18n/common.json"), include_str!("i18n/ar.json")));
static HI: Lazy<Translations> = Lazy::new(|| parse_translation(include_str!("i18n/common.json"), include_str!("i18n/hi.json")));

// EN: Return the translation table for the selected UI language.
// FR: Retourne la table de traduction pour la langue UI selectionnee.
pub fn tr(langue: Langue) -> &'static Translations {
    match langue {
        Langue::Francais => &FR,
        Langue::Anglais => &EN,
        Langue::Espagnol => &ES,
        Langue::Italien => &IT,
        Langue::Allemand => &DE,
        Langue::Portugais => &PT,
        Langue::Russe => &RU,
        Langue::Chinois => &ZH,
        Langue::Japonais => &JA,
        Langue::Coreen => &KO,
        Langue::Arabe => &AR,
        Langue::Hindi => &HI,
    }
}





