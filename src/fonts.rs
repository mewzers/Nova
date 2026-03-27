use eframe::egui::{self, FontData, FontDefinitions, FontFamily};

// EN: Register bundled fonts plus optional Windows fallbacks.
// FR: Enregistre les polices embarquees plus les fallbacks Windows optionnels.
pub fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // EN: Primary bundled fonts shipped with the application.
    // FR: Polices principales embarquees avec l application.
    fonts.font_data.insert(
        "noto_sans".to_owned(),
        FontData::from_static(include_bytes!("assets/fonts/NotoSans-Regular.ttf")).into(),
    );
    fonts.font_data.insert(
        "noto_sans_jp".to_owned(),
        FontData::from_static(include_bytes!("assets/fonts/NotoSansJP-Regular.ttf")).into(),
    );
    fonts.font_data.insert(
        "noto_emoji".to_owned(),
        FontData::from_static(include_bytes!("assets/fonts/NotoEmoji-Regular.ttf")).into(),
    );

    // EN: Optional Windows fallback fonts for scripts not covered by bundled fonts.
    // FR: Polices de fallback Windows optionnelles pour les scripts non couverts par les polices embarquees.
    try_add_system_font(&mut fonts, "win_segoe_ui", r"C:\Windows\Fonts\segoeui.ttf");
    try_add_system_font(&mut fonts, "win_nirmala", r"C:\Windows\Fonts\Nirmala.ttf");
    try_add_system_font(&mut fonts, "win_malgun", r"C:\Windows\Fonts\malgun.ttf");
    try_add_system_font(&mut fonts, "win_meiryo", r"C:\Windows\Fonts\meiryo.ttc");
    try_add_system_font(&mut fonts, "win_yahei", r"C:\Windows\Fonts\msyh.ttc");
    try_add_system_font(&mut fonts, "win_segoe_symbol", r"C:\Windows\Fonts\seguisym.ttf");
    try_add_system_font(&mut fonts, "win_segoe_emoji", r"C:\Windows\Fonts\seguiemj.ttf");

    // EN: Compose proportional family fallback order.
    // FR: Construit l ordre de fallback de la famille proportionnelle.
    {
        let proportional = fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default();
        proportional.insert(0, "noto_sans".to_owned());
        proportional.insert(1, "noto_sans_jp".to_owned());
        if fonts.font_data.contains_key("win_segoe_symbol") {
            proportional.insert(2, "win_segoe_symbol".to_owned());
        }
        if fonts.font_data.contains_key("win_segoe_emoji") {
            proportional.insert(3, "win_segoe_emoji".to_owned());
        }
        proportional.insert(4, "noto_emoji".to_owned());
    }

    // EN: Compose monospace family fallback order.
    // FR: Construit l ordre de fallback de la famille monospace.
    {
        let monospace = fonts.families.entry(FontFamily::Monospace).or_default();
        monospace.insert(0, "noto_sans".to_owned());
        monospace.insert(1, "noto_sans_jp".to_owned());
        if fonts.font_data.contains_key("win_segoe_symbol") {
            monospace.insert(2, "win_segoe_symbol".to_owned());
        }
        if fonts.font_data.contains_key("win_segoe_emoji") {
            monospace.insert(3, "win_segoe_emoji".to_owned());
        }
        monospace.insert(4, "noto_emoji".to_owned());
    }

    // EN: Append system fallbacks to both families when available.
    // FR: Ajoute les fallbacks systeme aux deux familles quand disponibles.
    for name in [
        "win_segoe_ui",
        "win_nirmala",
        "win_malgun",
        "win_meiryo",
        "win_yahei",
        "win_segoe_symbol",
        "win_segoe_emoji",
    ] {
        if fonts.font_data.contains_key(name) {
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .push(name.to_owned());
            fonts
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .push(name.to_owned());
        }
    }

    // EN: Activate the final font configuration.
    // FR: Active la configuration finale des polices.
    ctx.set_fonts(fonts);
}

// EN: Try to load a font file from disk and register it under `name`.
// FR: Tente de charger une police depuis le disque et de l enregistrer sous `name`.
fn try_add_system_font(fonts: &mut FontDefinitions, name: &str, path: &str) {
    if let Ok(bytes) = std::fs::read(path) {
        fonts
            .font_data
            .insert(name.to_owned(), FontData::from_owned(bytes).into());
    }
}

