#[cfg(target_os = "windows")]
fn main() {
    use std::fs::File;
    use std::path::PathBuf;

    use ico::{IconDir, IconDirEntry, IconImage, ResourceType};

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR missing"));
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR missing"));
    let icons_dir = manifest_dir.join("src/assets/icons");

    let icon_files = [
        "16x16.ico",
        "32x32.ico",
        "48x48.ico",
        "64x64.ico",
        "128x128.ico",
        "256x256.ico",
    ];

    let mut icon_dir = IconDir::new(ResourceType::Icon);
    for file in icon_files {
        let path = icons_dir.join(file);
        let image = image::open(&path)
            .unwrap_or_else(|e| panic!("Failed to open {}: {e}", path.display()))
            .to_rgba8();
        let (w, h) = image.dimensions();
        let icon_image = IconImage::from_rgba_data(w, h, image.into_raw());
        let entry = IconDirEntry::encode(&icon_image)
            .unwrap_or_else(|e| panic!("Failed to encode {}: {e}", file));
        icon_dir.add_entry(entry);
        println!("cargo:rerun-if-changed=src/assets/icons/{file}");
    }

    let ico_path = out_dir.join("Oxide-icon.ico");
    let mut file = File::create(&ico_path).expect("Failed to create generated icon file");
    icon_dir
        .write(&mut file)
        .expect("Failed to write generated icon file");

    winres::WindowsResource::new()
        .set_icon(ico_path.to_string_lossy().as_ref())
        .compile()
        .expect("Failed to compile Windows resources");
}

#[cfg(not(target_os = "windows"))]
fn main() {}