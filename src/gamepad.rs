use std::sync::Mutex;

use gilrs::{Button, Gilrs};
use once_cell::sync::Lazy;

use crate::debug;
// EN: Shared gamepad context initialized once for the whole application.
// FR: Contexte manette partage initialise une seule fois pour toute l application.
static GILRS: Lazy<Mutex<Option<Gilrs>>> = Lazy::new(|| Mutex::new(Gilrs::new().ok()));

// EN: Return true if any button in the provided set is pressed.
// FR: Retourne vrai si au moins un bouton de l ensemble fourni est appuye.
fn any_pressed(gamepad: &gilrs::Gamepad<'_>, buttons: &[Button]) -> bool {
    buttons.iter().any(|b| gamepad.is_pressed(*b))
}

// EN: Poll controller state and map it to the 16 CHIP-8 keys.
// FR: Lit l etat du controleur et le mappe aux 16 touches CHIP-8.
pub fn poll_chip8_keys() -> [bool; 16] {
    let mut keys = [false; 16];
    let mut guard = match GILRS.lock() {
        Ok(g) => g,
        Err(_) => {
            debug::log("gamepad_mutex_error");
            return keys;
        }
    };

    let Some(gilrs) = guard.as_mut() else {
        return keys;
    };

    // EN: Drain pending events so button states stay up to date.
    // FR: Vide les evenements en attente pour garder les etats de boutons a jour.
    while gilrs.next_event().is_some() {}

    for (_, gamepad) in gilrs.gamepads() {
        if !gamepad.is_connected() {
            continue;
        }

        keys[0x0] |= any_pressed(&gamepad, &[Button::Select]);
        keys[0x1] |= any_pressed(&gamepad, &[Button::DPadUp]);
        keys[0x2] |= any_pressed(&gamepad, &[Button::North]);
        keys[0x3] |= any_pressed(&gamepad, &[Button::DPadRight]);
        keys[0x4] |= any_pressed(&gamepad, &[Button::DPadLeft]);
        keys[0x5] |= any_pressed(&gamepad, &[Button::South]);
        keys[0x6] |= any_pressed(&gamepad, &[Button::East]);
        keys[0x7] |= any_pressed(&gamepad, &[Button::LeftTrigger]);
        keys[0x8] |= any_pressed(&gamepad, &[Button::DPadDown]);
        keys[0x9] |= any_pressed(&gamepad, &[Button::West]);
        keys[0xA] |= any_pressed(&gamepad, &[Button::LeftThumb]);
        keys[0xB] |= any_pressed(&gamepad, &[Button::RightTrigger]);
        keys[0xC] |= any_pressed(&gamepad, &[Button::Start]);
        keys[0xD] |= any_pressed(&gamepad, &[Button::RightThumb]);
        keys[0xE] |= any_pressed(&gamepad, &[Button::LeftTrigger2]);
        keys[0xF] |= any_pressed(&gamepad, &[Button::RightTrigger2]);
    }

    keys
}

