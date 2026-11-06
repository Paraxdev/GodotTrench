//! Editor color scheme and fonts. Grays 0 to 7 run from the darkest surface to the brightest text, the rest are
//! accents.

use std::sync::Arc;

use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontTweak, Margin, Shadow, Stroke, Vec2, Visuals};

pub const BG: Color32 = Color32::from_rgb(0x05, 0x06, 0x08);
pub const FG: Color32 = Color32::from_rgb(0xdd, 0xe0, 0xe6);
pub const GRAY_0: Color32 = Color32::from_rgb(0x11, 0x12, 0x15);
pub const GRAY_1: Color32 = Color32::from_rgb(0x1d, 0x1e, 0x23);
pub const GRAY_2: Color32 = Color32::from_rgb(0x2b, 0x2d, 0x34);
pub const GRAY_3: Color32 = Color32::from_rgb(0x3e, 0x41, 0x4a);
pub const GRAY_4: Color32 = Color32::from_rgb(0x5b, 0x5f, 0x6c);
pub const GRAY_5: Color32 = Color32::from_rgb(0x83, 0x88, 0x96);
pub const GRAY_6: Color32 = Color32::from_rgb(0xad, 0xb2, 0xbf);
pub const GRAY_7: Color32 = Color32::from_rgb(0xe7, 0xe9, 0xee);
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

pub const ACCENT: Color32 = Color32::from_rgb(0x6c, 0x8d, 0xff);
/// Behind and between the docks, which sit on it as rounded cards.
pub const BACKDROP: Color32 = GRAY_0;
/// The menu, toolbar and status bars.
pub const CHROME: Color32 = Color32::from_rgb(0x16, 0x17, 0x1b);
/// Collapsible section headers.
pub const HEADER: Color32 = Color32::from_rgb(0x26, 0x28, 0x2e);
const RADIUS: u8 = 6;
const WINDOW_RADIUS: u8 = 10;
pub const DOCK_GAP: f32 = 8.0;
pub const AXIS: [Color32; 3] = [RED, GREEN, BLUE];
pub const ERROR: Color32 = RED;
pub const WARNING: Color32 = YELLOW;
pub const SUCCESS: Color32 = GREEN;
pub const INFO: Color32 = CYAN;
/// The walkable area overlay: the biggest connected area, and the islands cut off from it.
pub const WALKABLE: Color32 = GREEN;
pub const WALKABLE_CUT_OFF: Color32 = RED;

/// Between RED and YELLOW, like the orange of Okabe & Ito.
pub const ORANGE: Color32 = Color32::from_rgb(0xe9, 0x93, 0x36);

/// Logic graph node headers by entity category. The node names its category too, so hue is never the only cue.
pub const CATEGORIES: [(&str, Color32); 10] = [
    ("logic", BLUE),
    ("math", ORANGE),
    ("trigger", GREEN),
    ("func", RED),
    ("light", YELLOW),
    ("env", TEAL),
    ("game", CYAN),
    ("prop", PINK),
    ("npc", MAGENTA),
    ("info", GRAY_5),
];

/// A category's color, and for one the editor does not know a stable pick from the accents.
pub fn category_color(name: &str) -> Color32 {
    const OTHERS: [Color32; 8] = [BLUE, MAGENTA, GREEN, RED, YELLOW, TEAL, CYAN, PINK];
    CATEGORIES.iter().find(|(n, _)| *n == name).map(|(_, c)| *c).unwrap_or_else(|| {
        let hash = name.bytes().fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32));
        OTHERS[hash as usize % OTHERS.len()]
    })
}

/// Logic graph pins and wires by the type of value they carry, Blueprint style. A pulse carries none.
pub const PIN_TYPES: [(&str, Color32); 9] = [
    ("pulse", GRAY_7),
    ("bool", RED),
    ("int", CYAN),
    ("float", GREEN),
    ("string", PINK),
    ("vector3", YELLOW),
    ("color", MAGENTA),
    ("node", BLUE),
    ("variant", GRAY_5),
];

/// Colors a Logic graph frame can take, the first unused one goes to a new frame.
pub const FRAME_COLORS: [Color32; 6] = [BLUE, GREEN, ORANGE, MAGENTA, TEAL, YELLOW];

pub fn pin_type_color(name: &str) -> Color32 {
    PIN_TYPES.iter().find(|(n, _)| *n == name).map_or(GRAY_5, |(_, c)| *c)
}

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
    let radius = CornerRadius::same(RADIUS);
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
    // egui draws selected labels and selected text in selection.stroke on top of bg_fill.
    v.selection.bg_fill = blend(GRAY_2, ACCENT, 0.6);
    v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    v.window_corner_radius = CornerRadius::same(WINDOW_RADIUS);
    v.menu_corner_radius = CornerRadius::same(WINDOW_RADIUS - 1);
    let shadow = Shadow { offset: [0, 8], blur: 24, spread: 0, color: Color32::from_black_alpha(90) };
    v.window_shadow = shadow;
    v.popup_shadow = shadow;
    v.indent_has_left_vline = false;

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
    w.hovered.bg_stroke = Stroke::new(1.0, blend(GRAY_4, ACCENT, 0.35));
    w.hovered.fg_stroke = Stroke::new(1.5, GRAY_7);
    w.active.bg_fill = GRAY_4;
    w.active.weak_bg_fill = GRAY_4;
    w.active.bg_stroke = Stroke::new(1.0, ACCENT);
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

/// Colors and the roomier spacing the editor uses over egui's defaults.
pub fn style(style: &mut egui::Style) {
    style.visuals = visuals();
    style.spacing.item_spacing = Vec2::new(8.0, 5.0);
    style.spacing.button_padding = Vec2::new(8.0, 3.0);
    style.spacing.interact_size.y = 24.0;
    style.spacing.indent = 18.0;
}

/// The frame of a top or bottom bar, the menu, the status line and the like.
pub fn bar_frame(style: &egui::Style) -> egui::Frame {
    egui::Frame::side_top_panel(style).fill(CHROME)
}

/// The docks as rounded cards on the backdrop, each with its tabs as pills along the top.
pub fn dock_style(base: &egui::Style) -> egui_dock::Style {
    let mut s = egui_dock::Style::from_egui(base);
    let none = Color32::TRANSPARENT;
    let pill = CornerRadius::same(RADIUS + 2);
    let tab = |bg: Color32, text: Color32| egui_dock::TabInteractionStyle { outline_color: none, corner_radius: pill, bg_fill: bg, text_color: text };
    s.main_surface_border_stroke = Stroke::NONE;
    s.dock_area_padding = Some(Margin::same(DOCK_GAP as i8));
    s.separator.width = DOCK_GAP;
    s.separator.color_idle = none;
    s.separator.color_hovered = ACCENT;
    s.separator.color_dragged = ACCENT;
    s.buttons.close_tab_color = GRAY_5;
    s.buttons.close_tab_active_color = GRAY_7;
    s.buttons.close_tab_bg_fill = none;
    s.tab_bar.bg_fill = GRAY_1;
    s.tab_bar.hline_color = none;
    s.tab_bar.height = 34.0;
    s.tab_bar.inner_margin = Margin::symmetric(6, 3);
    s.tab_bar.corner_radius = CornerRadius { nw: WINDOW_RADIUS, ne: WINDOW_RADIUS, sw: 0, se: 0 };
    s.tab.active = tab(GRAY_2, GRAY_7);
    s.tab.focused = tab(GRAY_2, GRAY_7);
    s.tab.inactive = tab(none, GRAY_5);
    s.tab.hovered = tab(blend(GRAY_1, GRAY_2, 0.6), GRAY_7);
    s.tab.tab_body.bg_fill = GRAY_1;
    s.tab.tab_body.stroke = Stroke::NONE;
    s.tab.tab_body.corner_radius = CornerRadius { nw: 0, ne: 0, sw: WINDOW_RADIUS, se: WINDOW_RADIUS };
    s
}

const UI_FONT: &str = "IBM Plex Sans";
const MONO_FONT: &str = "IBM Plex Mono";

/// IBM Plex in front of egui's default fonts, which stay behind it as fallbacks: egui's own widgets draw their
/// submenu arrows and warning signs from the emoji fonts.
pub fn fonts() -> FontDefinitions {
    // Plex caps sit a little below the row centre. This lifts them by a pixel from 150% scale, below that epaint
    // rounds glyphs to whole pixels and it has no effect.
    let tweak = FontTweak { y_offset_factor: -0.026, ..Default::default() };
    let mut fonts = FontDefinitions::default();
    for (name, data) in
        [(UI_FONT, &include_bytes!("../assets/fonts/IBMPlexSans-Regular.ttf")[..]), (MONO_FONT, &include_bytes!("../assets/fonts/IBMPlexMono-Regular.ttf")[..])]
    {
        fonts.font_data.insert(name.to_owned(), Arc::new(FontData::from_static(data).tweak(tweak.clone())));
    }

    let proportional = fonts.families.entry(FontFamily::Proportional).or_default();
    proportional.insert(0, UI_FONT.to_owned());
    // Hack is the only bundled font with ⇄ and ▾. Last, so the box drawn for a missing glyph still comes from Noto Emoji.
    proportional.push("Hack".to_owned());
    fonts.families.entry(FontFamily::Monospace).or_default().insert(0, MONO_FONT.to_owned());
    fonts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plex_leads_and_ui_glyphs_have_a_font() {
        let defs = fonts();
        assert_eq!(defs.families[&FontFamily::Proportional][0], UI_FONT);
        assert_eq!(defs.families[&FontFamily::Monospace][0], MONO_FONT);
        let mut fonts = egui::epaint::text::Fonts::new(Default::default(), defs);
        assert!(fonts.has_glyphs(&egui::FontId::proportional(13.0), "Inspector 1482 ⇄ ▾ × ⏵"));
        assert!(fonts.has_glyphs(&egui::FontId::monospace(13.0), "x = -202.5"));
    }

    #[test]
    fn linear_matches_srgb_endpoints() {
        assert_eq!(linear(Color32::BLACK), [0.0, 0.0, 0.0, 1.0]);
        assert!((linear(Color32::WHITE)[0] - 1.0).abs() < 1e-9);
        assert!(linear(GRAY_1)[2] > linear(GRAY_0)[2]);
    }

    /// WCAG 2 contrast ratio. AA asks 4.5 for text and 3 for the parts of a control.
    fn contrast(a: Color32, b: Color32) -> f64 {
        let luminance = |c: Color32| {
            let [r, g, b, _] = linear(c);
            0.2126 * r + 0.7152 * g + 0.0722 * b
        };
        let (hi, lo) = (luminance(a).max(luminance(b)), luminance(a).min(luminance(b)));
        (hi + 0.05) / (lo + 0.05)
    }

    #[test]
    fn text_stays_readable_on_selected_fills() {
        let v = visuals();
        for (text, fill, what) in [(FG, selected_fill(), "selected rows"), (v.selection.stroke.color, v.selection.bg_fill, "selected widgets")] {
            let ratio = contrast(text, fill);
            assert!(ratio >= 4.5, "{what}: text contrast {ratio:.2} < 4.5");
        }
    }

    #[test]
    fn selection_fill_stands_out_from_panels() {
        // Slider and progress fills, the view grid corner dot and the prefab card outline use it as a mark.
        let v = visuals();
        for (bg, what) in [(v.panel_fill, "panels"), (v.extreme_bg_color, "fields and cards")] {
            let ratio = contrast(v.selection.bg_fill, bg);
            assert!(ratio >= 3.0, "selection fill on {what}: contrast {ratio:.2} < 3");
        }
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
        let category = |name: &str| category_color(name);
        let pin = |name: &str| pin_type_color(name);
        let pairs: [(Color32, Color32, &str); 15] = [
            (category("logic"), category("math"), "logic vs math nodes"),
            (category("logic"), category("trigger"), "logic vs trigger nodes"),
            (category("trigger"), category("func"), "trigger vs func nodes"),
            (category("math"), category("func"), "math vs func nodes"),
            (pin("int"), pin("float"), "int vs float pins"),
            (pin("bool"), pin("float"), "bool vs float pins"),
            (pin("string"), pin("vector3"), "string vs vector3 pins"),
            (pin("node"), pin("int"), "node vs int pins"),
            (ERROR, SUCCESS, "error vs success"),
            (WARNING, SUCCESS, "warning vs success"),
            (WARNING, ERROR, "warning vs error"),
            (AXIS[0], AXIS[1], "axis x vs y"),
            (AXIS[0], AXIS[2], "axis x vs z"),
            (AXIS[1], AXIS[2], "axis y vs z"),
            (WALKABLE, WALKABLE_CUT_OFF, "walkable vs cut off"),
        ];
        // Redmean threshold below which two swatches read as the same color to that viewer.
        const MIN: f32 = 40.0;
        for kind in 0..3 {
            for (a, b, what) in pairs {
                let d = distance(simulate(a, kind), simulate(b, kind));
                assert!(d >= MIN, "cvd {kind}: {what} too close ({d:.0} < {MIN})");
            }

            for (c, what) in [(ERROR, "error"), (SUCCESS, "success"), (WARNING, "warning"), (AXIS[0], "axis x"), (AXIS[1], "axis y")] {
                let d = distance(simulate(ACCENT, kind), simulate(c, kind));
                assert!(d >= MIN, "cvd {kind}: accent vs {what} too close ({d:.0} < {MIN})");
            }
        }
    }
}
