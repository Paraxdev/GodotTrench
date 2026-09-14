//! Editor color scheme. Grays 0 to 7 run from the darkest surface to the brightest text, the rest are accents.

use egui::{Color32, CornerRadius, Stroke, Visuals};

pub const BG: Color32 = Color32::from_rgb(0x05, 0x06, 0x08);
pub const FG: Color32 = Color32::from_rgb(0xd6, 0xda, 0xe2);
pub const GRAY_0: Color32 = Color32::from_rgb(0x09, 0x0b, 0x0f);
pub const GRAY_1: Color32 = Color32::from_rgb(0x14, 0x17, 0x1c);
pub const GRAY_2: Color32 = Color32::from_rgb(0x24, 0x29, 0x30);
pub const GRAY_3: Color32 = Color32::from_rgb(0x3b, 0x41, 0x4c);
pub const GRAY_4: Color32 = Color32::from_rgb(0x57, 0x60, 0x70);
pub const GRAY_5: Color32 = Color32::from_rgb(0x7a, 0x86, 0x9b);
pub const GRAY_6: Color32 = Color32::from_rgb(0xa7, 0xb1, 0xc3);
pub const GRAY_7: Color32 = Color32::from_rgb(0xdf, 0xe2, 0xe9);
pub const MAGENTA: Color32 = Color32::from_rgb(0xdf, 0x29, 0xe3);
pub const GREEN: Color32 = Color32::from_rgb(0x46, 0xe5, 0x47);
pub const YELLOW: Color32 = Color32::from_rgb(0xe7, 0xb9, 0x23);
pub const BLUE: Color32 = Color32::from_rgb(0x1a, 0x8a, 0xe9);
pub const CYAN: Color32 = Color32::from_rgb(0x46, 0xd2, 0xdf);
pub const PINK: Color32 = Color32::from_rgb(0xdb, 0x37, 0xc3);
pub const RED: Color32 = Color32::from_rgb(0xee, 0x34, 0x4a);
pub const TEAL: Color32 = Color32::from_rgb(0x2b, 0xd0, 0xb3);

pub const ACCENT: Color32 = BLUE;
pub const AXIS: [Color32; 3] = [RED, GREEN, BLUE];
pub const ERROR: Color32 = RED;
pub const WARNING: Color32 = YELLOW;
pub const SUCCESS: Color32 = GREEN;
pub const INFO: Color32 = CYAN;

/// Selected cards and rows: the accent sunk into the panel so text on top stays readable.
pub fn selected_fill() -> Color32 {
    blend(GRAY_1, ACCENT, 0.45)
}

pub fn blend(a: Color32, b: Color32, t: f32) -> Color32 {
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(mix(a.r(), b.r()), mix(a.g(), b.g()), mix(a.b(), b.b()))
}

/// Linear RGBA for GPU clear colors.
pub fn linear(c: Color32) -> [f64; 4] {
    let lin = |v: u8| {
        let s = v as f64 / 255.0;
        if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    };
    [lin(c.r()), lin(c.g()), lin(c.b()), 1.0]
}

pub fn visuals() -> Visuals {
    let mut v = Visuals::dark();
    let radius = CornerRadius::same(3);
    v.panel_fill = GRAY_1;
    v.window_fill = GRAY_1;
    v.window_stroke = Stroke::new(1.0, GRAY_3);
    v.extreme_bg_color = GRAY_0;
    v.code_bg_color = GRAY_0;
    v.faint_bg_color = blend(GRAY_1, GRAY_2, 0.5);
    v.text_edit_bg_color = Some(GRAY_0);
    v.hyperlink_color = CYAN;
    v.warn_fg_color = WARNING;
    v.error_fg_color = ERROR;
    v.selection.bg_fill = blend(GRAY_2, ACCENT, 0.7);
    v.selection.stroke = Stroke::new(1.0, GRAY_7);
    v.window_corner_radius = CornerRadius::same(5);
    v.menu_corner_radius = CornerRadius::same(4);

    let w = &mut v.widgets;
    w.noninteractive.bg_fill = GRAY_1;
    w.noninteractive.weak_bg_fill = GRAY_1;
    w.noninteractive.bg_stroke = Stroke::new(1.0, GRAY_2);
    w.noninteractive.fg_stroke = Stroke::new(1.0, GRAY_6);
    w.inactive.bg_fill = GRAY_2;
    w.inactive.weak_bg_fill = GRAY_2;
    w.inactive.bg_stroke = Stroke::NONE;
    w.inactive.fg_stroke = Stroke::new(1.0, FG);
    w.hovered.bg_fill = GRAY_3;
    w.hovered.weak_bg_fill = GRAY_3;
    w.hovered.bg_stroke = Stroke::new(1.0, GRAY_4);
    w.hovered.fg_stroke = Stroke::new(1.5, GRAY_7);
    w.active.bg_fill = GRAY_4;
    w.active.weak_bg_fill = GRAY_4;
    w.active.bg_stroke = Stroke::new(1.0, GRAY_5);
    w.active.fg_stroke = Stroke::new(2.0, GRAY_7);
    w.open.bg_fill = GRAY_2;
    w.open.weak_bg_fill = GRAY_2;
    w.open.bg_stroke = Stroke::new(1.0, GRAY_3);
    w.open.fg_stroke = Stroke::new(1.0, GRAY_7);
    for state in [&mut w.noninteractive, &mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
        state.corner_radius = radius;
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_matches_srgb_endpoints() {
        assert_eq!(linear(Color32::BLACK), [0.0, 0.0, 0.0, 1.0]);
        assert!((linear(Color32::WHITE)[0] - 1.0).abs() < 1e-9);
        assert!(linear(GRAY_1)[2] > linear(GRAY_0)[2]);
    }

    #[test]
    fn text_stays_readable_on_selected_fills() {
        let luma = |c: Color32| 0.2126 * c.r() as f32 + 0.7152 * c.g() as f32 + 0.0722 * c.b() as f32;
        assert!(luma(FG) - luma(selected_fill()) > 90.0);
        assert!(luma(FG) - luma(visuals().selection.bg_fill) > 70.0);
    }
}
