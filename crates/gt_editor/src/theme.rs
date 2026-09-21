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
// Accents use a colorblind-safe palette derived from Okabe & Ito, so meaning carried by hue (axes,
// error/success/warning, entity and format tints) stays distinguishable under deuteranopia,
// protanopia and tritanopia. RED is a vermillion and GREEN a bluish green, the pair that keeps
// red-green viewers apart, and the cvd_pairs test guards the critical distinctions.
pub const MAGENTA: Color32 = Color32::from_rgb(0xd2, 0x6f, 0xc0);
pub const GREEN: Color32 = Color32::from_rgb(0x1c, 0xb5, 0x8e);
pub const YELLOW: Color32 = Color32::from_rgb(0xec, 0xc5, 0x31);
pub const BLUE: Color32 = Color32::from_rgb(0x2e, 0x8b, 0xd6);
pub const CYAN: Color32 = Color32::from_rgb(0x63, 0xc3, 0xec);
pub const PINK: Color32 = Color32::from_rgb(0xe0, 0x69, 0x9f);
pub const RED: Color32 = Color32::from_rgb(0xe8, 0x61, 0x3c);
pub const TEAL: Color32 = Color32::from_rgb(0x17, 0xb3, 0xc4);

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

    fn srgb_to_lin(v: f32) -> f32 {
        if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    }
    fn lin_to_srgb(v: f32) -> f32 {
        let v = v.clamp(0.0, 1.0);
        if v <= 0.0031308 { v * 12.92 } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 }
    }
    fn mat3(m: [[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
        [m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2], m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2], m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2]]
    }

    /// Viénot 1999 dichromacy simulation: how a color looks to a protan (0), deutan (1) or tritan (2).
    fn simulate(c: Color32, kind: usize) -> Color32 {
        let rgb = [srgb_to_lin(c.r() as f32 / 255.0), srgb_to_lin(c.g() as f32 / 255.0), srgb_to_lin(c.b() as f32 / 255.0)];
        let to_lms = [[0.3139902, 0.63951294, 0.04649755], [0.15537241, 0.75789446, 0.08670142], [0.01775239, 0.10944209, 0.8725692]];
        let from_lms = [[5.472212, -4.64196, 0.16963708], [-1.1252419, 2.293171, -0.1678952], [0.02980165, -0.19318073, 1.1636479]];
        let sim = [
            [[0.0, 1.051183, -0.05116099], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            [[1.0, 0.0, 0.0], [0.9513092, 0.0, 0.04866992], [0.0, 0.0, 1.0]],
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [-0.8674474, 1.867271, 0.0]],
        ][kind];
        let out = mat3(from_lms, mat3(sim, mat3(to_lms, rgb)));
        let b = |v: f32| (lin_to_srgb(v) * 255.0).round().clamp(0.0, 255.0) as u8;
        Color32::from_rgb(b(out[0]), b(out[1]), b(out[2]))
    }

    /// Perceptual "redmean" distance, a cheap sRGB metric that tracks human color difference well.
    fn distance(a: Color32, b: Color32) -> f32 {
        let rbar = (a.r() as f32 + b.r() as f32) / 2.0;
        let (dr, dg, db) = (a.r() as f32 - b.r() as f32, a.g() as f32 - b.g() as f32, a.b() as f32 - b.b() as f32);
        ((2.0 + rbar / 256.0) * dr * dr + 4.0 * dg * dg + (2.0 + (255.0 - rbar) / 256.0) * db * db).sqrt()
    }

    #[test]
    fn cvd_pairs_stay_distinguishable() {
        // Role pairs whose only difference is hue, so they must survive every common color blindness.
        let pairs: [(Color32, Color32, &str); 6] = [
            (ERROR, SUCCESS, "error vs success"),
            (WARNING, SUCCESS, "warning vs success"),
            (WARNING, ERROR, "warning vs error"),
            (AXIS[0], AXIS[1], "axis x vs y"),
            (AXIS[0], AXIS[2], "axis x vs z"),
            (AXIS[1], AXIS[2], "axis y vs z"),
        ];
        // Redmean threshold below which two swatches read as the same color to that viewer.
        const MIN: f32 = 40.0;
        for kind in 0..3 {
            for (a, b, what) in pairs {
                let d = distance(simulate(a, kind), simulate(b, kind));
                assert!(d >= MIN, "cvd {kind}: {what} too close ({d:.0} < {MIN})");
            }
        }
    }
}
