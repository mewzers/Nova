use eframe::egui;

use crate::constants::{BARRE_BAS, BARRE_HAUT, CHIP8_H, CHIP8_W, PIXEL_BASE};

// EN: Compute window size from the selected render scale.
// FR: Calcule la taille de fenetre selon l echelle de rendu choisie.
pub fn fenetre_size(scale: u8) -> [f32; 2] {
    let w = CHIP8_W * PIXEL_BASE * scale as f32;
    let h = CHIP8_H * PIXEL_BASE * scale as f32 + BARRE_HAUT + BARRE_BAS;
    [w, h]
}

// EN: Default CHIP-8 keypad mapping on keyboard.
// FR: Mapping clavier CHIP-8 par defaut.
pub fn default_touches() -> [String; 16] {
    [
        "X".into(), "1".into(), "2".into(), "3".into(),
        "Q".into(), "W".into(), "E".into(), "A".into(),
        "S".into(), "D".into(), "Z".into(), "C".into(),
        "4".into(), "R".into(), "F".into(), "V".into(),
    ]
}

// EN: Convert a user-facing key label into an egui key enum.
// FR: Convertit un libelle de touche utilisateur en enum de touche egui.
pub fn key_from_label(label: &str) -> Option<egui::Key> {
    let key = label.trim().to_uppercase();
    match key.as_str() {
        "0" | "NUM0" => Some(egui::Key::Num0),
        "1" | "NUM1" => Some(egui::Key::Num1),
        "2" | "NUM2" => Some(egui::Key::Num2),
        "3" | "NUM3" => Some(egui::Key::Num3),
        "4" | "NUM4" => Some(egui::Key::Num4),
        "5" | "NUM5" => Some(egui::Key::Num5),
        "6" | "NUM6" => Some(egui::Key::Num6),
        "7" | "NUM7" => Some(egui::Key::Num7),
        "8" | "NUM8" => Some(egui::Key::Num8),
        "9" | "NUM9" => Some(egui::Key::Num9),
        "A" => Some(egui::Key::A),
        "B" => Some(egui::Key::B),
        "C" => Some(egui::Key::C),
        "D" => Some(egui::Key::D),
        "E" => Some(egui::Key::E),
        "F" => Some(egui::Key::F),
        "G" => Some(egui::Key::G),
        "H" => Some(egui::Key::H),
        "I" => Some(egui::Key::I),
        "J" => Some(egui::Key::J),
        "K" => Some(egui::Key::K),
        "L" => Some(egui::Key::L),
        "M" => Some(egui::Key::M),
        "N" => Some(egui::Key::N),
        "O" => Some(egui::Key::O),
        "P" => Some(egui::Key::P),
        "Q" => Some(egui::Key::Q),
        "R" => Some(egui::Key::R),
        "S" => Some(egui::Key::S),
        "T" => Some(egui::Key::T),
        "U" => Some(egui::Key::U),
        "V" => Some(egui::Key::V),
        "W" => Some(egui::Key::W),
        "X" => Some(egui::Key::X),
        "Y" => Some(egui::Key::Y),
        "Z" => Some(egui::Key::Z),
        "F1" => Some(egui::Key::F1),
        "F2" => Some(egui::Key::F2),
        "F3" => Some(egui::Key::F3),
        "F4" => Some(egui::Key::F4),
        "F5" => Some(egui::Key::F5),
        "F6" => Some(egui::Key::F6),
        "F7" => Some(egui::Key::F7),
        "F8" => Some(egui::Key::F8),
        "F9" => Some(egui::Key::F9),
        "F10" => Some(egui::Key::F10),
        "F11" => Some(egui::Key::F11),
        "F12" => Some(egui::Key::F12),
        "ESC" | "ECHAP" | "ESCAPE" => Some(egui::Key::Escape),
        "ENTER" => Some(egui::Key::Enter),
        "SPACE" => Some(egui::Key::Space),
        "TAB" => Some(egui::Key::Tab),
        "PLUS" | "+" | "PLUS_EQUALS" => Some(egui::Key::Plus),
        "EQUALS" | "=" => Some(egui::Key::Equals),
        "MINUS" | "-" => Some(egui::Key::Minus),
        "LEFT" => Some(egui::Key::ArrowLeft),
        "RIGHT" => Some(egui::Key::ArrowRight),
        "UP" => Some(egui::Key::ArrowUp),
        "DOWN" => Some(egui::Key::ArrowDown),
        _ => None,
    }
}

// EN: Convert a mouse button label into an egui PointerButton.
// FR: Convertit un libelle de bouton souris en PointerButton egui.
pub fn mouse_from_label(label: &str) -> Option<egui::PointerButton> {
    match label.trim().to_uppercase().as_str() {
        "MOUSELEFT"   | "MOUSEGAUCHE" => Some(egui::PointerButton::Primary),
        "MOUSERIGHT"  | "MOUSEDROIT"  => Some(egui::PointerButton::Secondary),
        "MOUSEMIDDLE" | "MOUSEMILIEU" => Some(egui::PointerButton::Middle),
        "MOUSEEXTRA1" => Some(egui::PointerButton::Extra1),
        "MOUSEEXTRA2" => Some(egui::PointerButton::Extra2),
        _ => None,
    }
}
// EN: Parsed shortcut with optional modifiers.
// FR: Raccourci parse avec modificateurs optionnels.
#[derive(Clone, Copy)]
pub struct ShortcutSpec {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub key: egui::Key,
}

// EN: Parse a shortcut label like "Ctrl+Shift+F1".
// FR: Parse un libelle de raccourci du type "Ctrl+Shift+F1".
pub fn parse_shortcut(label: &str) -> Option<ShortcutSpec> {
    let mut ctrl = false;
    let mut shift = false;
    let mut alt = false;
    let mut key_label: Option<String> = None;

    for part in label.split('+') {
        let p = part.trim().to_uppercase();
        match p.as_str() {
            "CTRL" | "CONTROL" => ctrl = true,
            "SHIFT" => shift = true,
            "ALT" => alt = true,
            "" => {}
            _ => key_label = Some(p),
        }
    }

    if key_label.is_none() {
        if label.trim() == "+" || label.contains("++") {
            key_label = Some("+".to_string());
        }
    }

    let key = key_label.and_then(|k| key_from_label(&k))?;
    Some(ShortcutSpec { ctrl, shift, alt, key })
}

// EN: Check if an input state matches the shortcut label (including modifiers).
// FR: Verifie si un input correspond au libelle de raccourci (modificateurs inclus).
pub fn shortcut_pressed(input: &egui::InputState, label: &str) -> bool {
    let Some(spec) = parse_shortcut(label) else {
        return false;
    };
    let ctrl = input.modifiers.ctrl || input.modifiers.command;
    if ctrl != spec.ctrl || input.modifiers.shift != spec.shift || input.modifiers.alt != spec.alt {
        return false;
    }
    input.key_pressed(spec.key)
}





