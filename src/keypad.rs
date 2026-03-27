// EN: Runtime state of the 16 CHIP-8 keys.
// FR: Etat runtime des 16 touches CHIP-8.
#[derive(serde::Deserialize, serde::Serialize, Clone, Default)]
pub struct Keypad {
    // EN: Pressed state for each CHIP-8 key index.
    // FR: Etat appuye pour chaque index de touche CHIP-8.
    pub keys: [bool; 16],
}

impl Keypad {
    // EN: Create a keypad with all keys released.
    // FR: Cree un clavier virtuel avec toutes les touches relachees.
    pub fn new() -> Self {
        Self { keys: [false; 16] }
    }

    // EN: Release all keys.
    // FR: Relache toutes les touches.
    pub fn clear(&mut self) {
        self.keys = [false; 16];
    }

    // EN: Overwrite all key states at once.
    // FR: Remplace tous les etats de touches en une fois.
    pub fn set_all(&mut self, keys: [bool; 16]) {
        self.keys = keys;
    }

    // EN: Return whether a key is currently pressed.
    // FR: Indique si une touche est actuellement appuyee.
    pub fn is_pressed(&self, key: usize) -> bool {
        key < 16 && self.keys[key]
    }

    // EN: Return the first pressed key index, if any.
    // FR: Retourne le premier index de touche appuyee, si present.
    pub fn first_pressed(&self) -> Option<u8> {
        self.keys
            .iter()
            .position(|pressed| *pressed)
            .map(|idx| idx as u8)
    }
}

