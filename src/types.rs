// EN: Supported UI languages.
// FR: Langues UI supportees.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Copy)]
pub(crate) enum Langue {
    Francais,
    Anglais,
    Espagnol,
    Italien,
    Allemand,
    Portugais,
    Russe,
    Chinois,
    Japonais,
    Coreen,
    Arabe,
    Hindi,
}

impl Langue {
    // EN: Ordered list used to populate language menus.
    // FR: Liste ordonnee utilisee pour remplir les menus de langue.
    pub(crate) const ALL: [Langue; 12] = [
        Langue::Francais,
        Langue::Anglais,
        Langue::Espagnol,
        Langue::Italien,
        Langue::Allemand,
        Langue::Portugais,
        Langue::Russe,
        Langue::Chinois,
        Langue::Japonais,
        Langue::Coreen,
        Langue::Arabe,
        Langue::Hindi,
    ];

    // EN: Human-readable label shown in the UI.
    // FR: Libelle lisible affiche dans l interface.
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Langue::Francais => "Fran\u{00E7}ais",
            Langue::Anglais => "English",
            Langue::Espagnol => "Espa\u{00F1}ol",
            Langue::Italien => "Italiano",
            Langue::Allemand => "Deutsch",
            Langue::Portugais => "Portugu\u{00EA}s",
            Langue::Russe => "\u{0420}\u{0443}\u{0441}\u{0441}\u{043A}\u{0438}\u{0439}",
            Langue::Chinois => "\u{4E2D}\u{6587}",
            Langue::Japonais => "\u{65E5}\u{672C}\u{8A9E}",
            Langue::Coreen => "\u{D55C}\u{AD6D}\u{C5B4}",
            Langue::Arabe => "\u{0627}\u{0644}\u{0639}\u{0631}\u{0628}\u{064A}\u{0629}",
            Langue::Hindi => "\u{0939}\u{093F}\u{0928}\u{094D}\u{0926}\u{0940}",
        }
    }
}

// EN: Tabs shown in the settings window.
// FR: Onglets affiches dans la fenetre des parametres.

// EN: Application themes available in the UI.
// FR: Themes applicatifs disponibles dans l interface.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Copy)]
pub(crate) enum AppTheme {
    Kiwano,
    Dark,
    Light,
}
impl AppTheme {
    // EN: Ordered list used to populate theme menus.
    // FR: Liste ordonnee utilisee pour remplir les menus de themes.
    pub(crate) const ALL: [AppTheme; 3] = [AppTheme::Kiwano, AppTheme::Dark, AppTheme::Light];
}
// EN: Tabs shown in the settings window.
// FR: Onglets affiches dans la fenetre des parametres.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone)]
pub(crate) enum OngletsSettings {
    Emulateur,
    Video,
    Audio,
    Controles,
    Raccourcis,
    Debug,
}

// EN: Quirk presets used to quickly switch compatibility profiles.
// FR: Presets de quirks utilises pour changer rapidement de profil de compatibilite.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Copy, Default)]
pub(crate) enum QuirkPreset {
    #[default]
    Chip8,
    Chip48,
    SuperChip,
    Custom,
}

impl QuirkPreset {
    pub(crate) fn quirks(self) -> crate::cpu::CpuQuirks {
        match self {
            // EN: Baseline modern profile tuned for most CHIP-8 ROMs.
            QuirkPreset::Chip8 => crate::cpu::CpuQuirks {
                shift_uses_vy: false,
                jump_uses_vx: false,
                draw_clips: false,
                load_store_increment_i: false,
                logic_clears_vf: false,
            },
            QuirkPreset::Chip48 => crate::cpu::CpuQuirks {
                shift_uses_vy: true,
                jump_uses_vx: true,
                draw_clips: false,
                load_store_increment_i: false,
                logic_clears_vf: false,
            },
            QuirkPreset::SuperChip => crate::cpu::CpuQuirks {
                shift_uses_vy: true,
                jump_uses_vx: true,
                draw_clips: true,
                load_store_increment_i: false,
                logic_clears_vf: false,
            },
            QuirkPreset::Custom => crate::cpu::CpuQuirks::default(),
        }
    }

    pub(crate) fn from_quirks(quirks: crate::cpu::CpuQuirks) -> Self {
        if quirks == QuirkPreset::Chip8.quirks() {
            QuirkPreset::Chip8
        } else if quirks == QuirkPreset::Chip48.quirks() {
            QuirkPreset::Chip48
        } else if quirks == QuirkPreset::SuperChip.quirks() {
            QuirkPreset::SuperChip
        } else {
            QuirkPreset::Custom
        }
    }
}

// EN: User-configurable keyboard shortcuts.
// FR: Raccourcis clavier configurables par l utilisateur.
#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub(crate) struct Raccourcis {
    pub(crate) pause: String,
    pub(crate) reset: String,
    pub(crate) stop: String,
    pub(crate) charger_jeu: String,
    pub(crate) fullscreen: String,
    pub(crate) savestate_1: String,
    pub(crate) savestate_2: String,
    pub(crate) savestate_3: String,
    pub(crate) loadstate_1: String,
    pub(crate) loadstate_2: String,
    pub(crate) loadstate_3: String,
}

impl Default for Raccourcis {
    // EN: Default key mapping used on first launch and reset.
    // FR: Mapping de touches par defaut utilise au premier lancement et lors du reset.
    fn default() -> Self {
        Self {
            pause: "P".into(),
            reset: "R".into(),
            stop: "Echap".into(),
            charger_jeu: "O".into(),
            fullscreen: "F11".into(),
            savestate_1: "F1".into(),
            savestate_2: "F2".into(),
            savestate_3: "F3".into(),
            loadstate_1: "F5".into(),
            loadstate_2: "F6".into(),
            loadstate_3: "F7".into(),
        }
    }
}
