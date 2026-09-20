//! The GodotTrench product logo (a paint brush), shared by the window icon and the menu bar.

/// The logo as a 256x256 RGBA PNG, embedded in the binary.
pub const LOGO_PNG: &[u8] = include_bytes!("../assets/logo.png");

/// Decode the embedded logo into RGBA pixels.
fn decode() -> image::RgbaImage {
    image::load_from_memory(LOGO_PNG).expect("embedded logo.png is a valid PNG").to_rgba8()
}

/// The window, taskbar and alt-tab icon built from the logo.
pub fn icon() -> egui::IconData {
    let img = decode();
    let (width, height) = img.dimensions();
    egui::IconData { rgba: img.into_raw(), width, height }
}

/// Load the logo as an egui texture so it can be drawn, scaled down, in the UI.
pub fn texture(ctx: &egui::Context) -> egui::TextureHandle {
    let img = decode();
    let (w, h) = img.dimensions();
    let color = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], img.as_raw());
    ctx.load_texture("brand-logo", color, egui::TextureOptions::LINEAR)
}
