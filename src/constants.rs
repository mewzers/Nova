// EN: Build/version string injected from Cargo metadata.
// FR: Chaine de version injectee depuis les metadonnees Cargo.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// EN: Base size for one CHIP-8 pixel in UI coordinates.
// FR: Taille de base d un pixel CHIP-8 dans les coordonnees UI.
pub const PIXEL_BASE: f32 = 10.0;
// EN: CHIP-8 display width in pixels.
// FR: Largeur de l ecran CHIP-8 en pixels.
pub const CHIP8_W: f32 = 64.0;
// EN: CHIP-8 display height in pixels.
// FR: Hauteur de l ecran CHIP-8 en pixels.
pub const CHIP8_H: f32 = 32.0;
// EN: Top bar reserved height.
// FR: Hauteur reservee pour la barre du haut.
pub const BARRE_HAUT: f32 = 30.0;
// EN: Bottom bar reserved height.
// FR: Hauteur reservee pour la barre du bas.
pub const BARRE_BAS: f32 = 30.0;

