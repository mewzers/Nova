// EN: Debug i18n - loads developer-facing log messages from JSON.
// FR: i18n debug - charge les messages de log destines au developpeur depuis JSON.
use std::collections::HashMap;
// EN: Key-value store for debug log messages in the selected language.
// FR: Dictionnaire cle-valeur des messages de log debug dans la langue choisie.
pub struct I18n {
    pub messages: HashMap<String, String>,
}
impl I18n {
    // EN: Load debug messages for the given language code, merged with common shared keys.
    // FR: Charge les messages debug pour le code langue donne, fusionnes avec les cles communes.
    pub fn new(lang: &str) -> Self {
        fn parse_map(raw: &str) -> Option<HashMap<String, String>> {
            let cleaned = raw.trim_start_matches('\u{feff}');
            serde_json::from_str::<HashMap<String, String>>(cleaned).ok()
        }

        let data = match lang {
            "en" => include_str!("i18n/en.json"),
            "fr" => include_str!("i18n/fr.json"),
            "es" => include_str!("i18n/es.json"),
            "it" => include_str!("i18n/it.json"),
            "de" => include_str!("i18n/de.json"),
            "pt" => include_str!("i18n/pt.json"),
            "ru" => include_str!("i18n/ru.json"),
            "zh" => include_str!("i18n/zh.json"),
            "ja" => include_str!("i18n/ja.json"),
            "ko" => include_str!("i18n/ko.json"),
            "ar" => include_str!("i18n/ar.json"),
            "hi" => include_str!("i18n/hi.json"),
            _ => include_str!("i18n/en.json"),
        };
        let common = include_str!("i18n/common.json");
        let messages = match (parse_map(common), parse_map(data)) {
            (Some(mut common), Some(lang_map)) => {
                common.extend(lang_map);
                common
            }
            _ => {
                let mut common = parse_map(include_str!("i18n/common.json"))
                    .expect("Failed to parse debug common i18n JSON");
                let fallback = parse_map(include_str!("i18n/en.json"))
                    .expect("Failed to parse fallback debug i18n JSON");
                common.extend(fallback);
                common
            }
        };
        I18n { messages }
    }
    // EN: Resolve a key to its translated message, return the key itself if missing.
    // FR: Resout une cle vers son message traduit, retourne la cle si absente.
    pub fn t(&self, key: &str) -> String {
        self.messages
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }
}