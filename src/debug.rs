use once_cell::sync::Lazy;
// EN: Debug logging module - active only in debug builds.
// FR: Module de logging debug - actif uniquement en build debug.
pub mod i18n;
// EN: Detect system language on Windows using Win32 API, fallback to English on other platforms.
// FR: Detecte la langue systeme sur Windows via l API Win32, repli vers l anglais sur les autres plateformes.
#[cfg(target_os = "windows")]
fn detect_system_lang() -> &'static str {
    use windows_sys::Win32::Globalization::{GetUserDefaultUILanguage, LCIDToLocaleName};
    let lcid = unsafe { GetUserDefaultUILanguage() } as u32;
    let mut buf = [0u16; 85];
    let len = unsafe { LCIDToLocaleName(lcid, buf.as_mut_ptr(), buf.len() as i32, 0) };
    if len <= 0 {
        return "en";
    }
    let locale = String::from_utf16_lossy(&buf[..len as usize - 1]);
    match locale.get(..2).unwrap_or("") {
        "fr" => "fr",
        "es" => "es",
        "it" => "it",
        "de" => "de",
        "pt" => "pt",
        "ru" => "ru",
        "zh" => "zh",
        "ja" => "ja",
        "ko" => "ko",
        "ar" => "ar",
        "hi" => "hi",
        _    => "en",
    }
}
// EN: Fallback for non-Windows platforms.
// FR: Repli pour les plateformes non-Windows.
#[cfg(not(target_os = "windows"))]
fn detect_system_lang() -> &'static str {
    "en"
}
// EN: Global debug i18n instance loaded once at startup.
// FR: Instance i18n debug globale chargee une seule fois au demarrage.
static DEBUG_I18N: Lazy<i18n::I18n> = Lazy::new(|| i18n::I18n::new(detect_system_lang()));
// EN: Log a translated debug message to stdout in debug builds only.
// FR: Affiche un message debug traduit sur stdout uniquement en build debug.
pub fn log(key: &str) {
    if cfg!(debug_assertions) {
        let now = chrono::Local::now();
        println!("[DEBUG {}] {}", now.format("%H:%M:%S"), DEBUG_I18N.t(key));
    }
}