// EN: Default framebuffer state (all pixels off).
// FR: Etat par defaut du framebuffer (tous les pixels eteints).
fn default_pixels() -> Vec<u8> {
    vec![0; 64 * 32]
}

// EN: CHIP-8 monochrome framebuffer (64x32).
// FR: Framebuffer monochrome CHIP-8 (64x32).
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Display {
    // EN: Raw pixel buffer stored in row-major order.
    // FR: Tampon de pixels brut stocke en ordre ligne par ligne.
    #[serde(default = "default_pixels")]
    pub pixels: Vec<u8>,
}

impl Display {
    // EN: Create a cleared display.
    // FR: Cree un affichage vide.
    pub fn new() -> Self {
        Self {
            pixels: vec![0; 64 * 32],
        }
    }

    // EN: Clear the full display.
    // FR: Efface completement l affichage.
    pub fn clear(&mut self) {
        self.pixels.fill(0);
    }

    // EN: Read one pixel value at (x, y).
    // FR: Lit la valeur d un pixel en (x, y).
    pub fn get_pixel(&self, x: usize, y: usize) -> bool {
        self.pixels[y * 64 + x] != 0
    }

    // EN: XOR one pixel and return collision flag (true if pixel was previously set).
    // FR: XOR un pixel et retourne le drapeau de collision (vrai si le pixel etait deja actif).
    pub fn set_pixel(&mut self, x: usize, y: usize) -> bool {
        let idx = y * 64 + x;
        let collision = self.pixels[idx] != 0;
        self.pixels[idx] ^= 1;
        collision
    }
}

