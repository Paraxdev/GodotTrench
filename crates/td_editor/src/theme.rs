//! Editor color scheme and fonts. Grays 0 to 7 run from the darkest surface to the brightest text, the rest are
//! accents. Every color lives in a [`Palette`], read through [`colors`].

use std::sync::Arc;

use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontTweak, Margin, Shadow, Stroke, Vec2, Visuals};

const RADIUS: u8 = 6;
const WINDOW_RADIUS: u8 = 10;
pub const DOCK_GAP: f32 = 8.0;

/// Every color the editor draws its own interface with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub fg: Color32,
    pub gray_0: Color32,
    pub gray_1: Color32,
    pub gray_2: Color32,
    pub gray_3: Color32,
    pub gray_4: Color32,
    pub gray_5: Color32,
    pub gray_6: Color32,
    pub gray_7: Color32,
    pub magenta: Color32,
    pub green: Color32,
    pub yellow: Color32,
    pub blue: Color32,
    pub cyan: Color32,
    pub pink: Color32,
    pub red: Color32,
    pub teal: Color32,
    /// Between red and yellow, like the orange of Okabe & Ito.
    pub orange: Color32,
    pub accent: Color32,
    /// Behind and between the docks, which sit on it as rounded cards.
    pub backdrop: Color32,
    /// The menu, toolbar and status bars.
    pub chrome: Color32,
    /// Collapsible section headers.
    pub header: Color32,
}

// Accents use a colorblind-safe palette derived from Okabe & Ito, so meaning carried by hue (axes,
// error/success/warning, entity and format tints) stays distinguishable under deuteranopia,
// protanopia and tritanopia. red is a vermillion and green a bluish green, the pair that keeps
// red-green viewers apart, and the cvd_pairs test guards the critical distinctions.
pub const DARK: Palette = Palette {
    fg: Color32::from_rgb(0xdd, 0xe0, 0xe6),
    gray_0: Color32::from_rgb(0x11, 0x12, 0x15),
    gray_1: Color32::from_rgb(0x1d, 0x1e, 0x23),
    gray_2: Color32::from_rgb(0x2b, 0x2d, 0x34),
    gray_3: Color32::from_rgb(0x3e, 0x41, 0x4a),
    gray_4: Color32::from_rgb(0x5b, 0x5f, 0x6c),
    gray_5: Color32::from_rgb(0x83, 0x88, 0x96),
    gray_6: Color32::from_rgb(0xad, 0xb2, 0xbf),
    gray_7: Color32::from_rgb(0xe7, 0xe9, 0xee),
    magenta: Color32::from_rgb(0xd2, 0x6f, 0xc0),
    green: Color32::from_rgb(0x1c, 0xb5, 0x8e),
    yellow: Color32::from_rgb(0xec, 0xc5, 0x31),
    blue: Color32::from_rgb(0x2e, 0x8b, 0xd6),
    cyan: Color32::from_rgb(0x63, 0xc3, 0xec),
    pink: Color32::from_rgb(0xe0, 0x69, 0x9f),
    red: Color32::from_rgb(0xe8, 0x61, 0x3c),
    teal: Color32::from_rgb(0x17, 0xb3, 0xc4),
    orange: Color32::from_rgb(0xe9, 0x93, 0x36),
    accent: Color32::from_rgb(0x6c, 0x8d, 0xff),
    backdrop: Color32::from_rgb(0x11, 0x12, 0x15),
    chrome: Color32::from_rgb(0x16, 0x17, 0x1b),
    header: Color32::from_rgb(0x26, 0x28, 0x2e),
};

/// The palette the editor draws with.
pub fn colors() -> &'static Palette {
    &DARK
}

/// Colors a Logic graph frame can take, the first unused one goes to a new frame. They are saved in the map, so
/// they do not follow the palette.
pub const FRAME_COLORS: [Color32; 6] = [DARK.blue, DARK.green, DARK.orange, DARK.magenta, DARK.teal, DARK.yellow];

impl Palette {
    pub fn axis(&self) -> [Color32; 3] {
        [self.red, self.green, self.blue]
    }

    pub fn error(&self) -> Color32 {
        self.red
    }

    pub fn warning(&self) -> Color32 {
        self.yellow
    }

    pub fn success(&self) -> Color32 {
        self.green
    }

    pub fn info(&self) -> Color32 {
        self.cyan
    }

    /// The walkable area overlay: the biggest connected area.
    pub fn walkable(&self) -> Color32 {
        self.green
    }

    /// The walkable area overlay: the islands cut off from the biggest area.
    pub fn walkable_cut_off(&self) -> Color32 {
        self.red
    }

    /// Logic graph node headers by entity category. The node names its category too, so hue is never the only cue.
    fn categories(&self) -> [(&'static str, Color32); 10] {
        [
            ("logic", self.blue),
            ("math", self.orange),
            ("trigger", self.green),
            ("func", self.red),
            ("light", self.yellow),
            ("env", self.teal),
            ("game", self.cyan),
            ("prop", self.pink),
            ("npc", self.magenta),
            ("info", self.gray_5),
        ]
    }

    /// A category's color, and for one the editor does not know a stable pick from the accents.
    pub fn category_color(&self, name: &str) -> Color32 {
        let others = [self.blue, self.magenta, self.green, self.red, self.yellow, self.teal, self.cyan, self.pink];
        self.categories().iter().find(|(n, _)| *n == name).map(|(_, c)| *c).unwrap_or_else(|| {
            let hash = name.bytes().fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32));
            others[hash as usize % others.len()]
        })
    }

    /// Logic graph pins and wires by the type of value they carry, Blueprint style. A pulse carries none.
    fn pin_types(&self) -> [(&'static str, Color32); 9] {
        [
            ("pulse", self.gray_7),
            ("bool", self.red),
            ("int", self.cyan),
            ("float", self.green),
            ("string", self.pink),
            ("vector3", self.yellow),
            ("color", self.magenta),
            ("node", self.blue),
            ("variant", self.gray_5),
        ]
    }

    pub fn pin_type_color(&self, name: &str) -> Color32 {
        self.pin_types().iter().find(|(n, _)| *n == name).map_or(self.gray_5, |(_, c)| *c)
    }

    /// Selected cards and rows: the accent sunk into the panel so text on top stays readable.
    pub fn selected_fill(&self) -> Color32 {
        blend(self.gray_1, self.accent, 0.45)
    }

    pub fn visuals(&self) -> Visuals {
        let mut v = Visuals::dark();
        let radius = CornerRadius::same(RADIUS);
        v.panel_fill = self.gray_1;
        v.window_fill = self.gray_1;
        v.window_stroke = Stroke::new(1.0, self.gray_3);
        v.extreme_bg_color = self.gray_0;
        v.code_bg_color = self.gray_0;
        v.faint_bg_color = blend(self.gray_1, self.gray_2, 0.5);
        v.text_edit_bg_color = Some(self.gray_0);
        v.hyperlink_color = self.cyan;
        v.warn_fg_color = self.warning();
        v.error_fg_color = self.error();
        // egui draws selected labels and selected text in selection.stroke on top of bg_fill.
        v.selection.bg_fill = blend(self.gray_2, self.accent, 0.6);
        v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
        v.window_corner_radius = CornerRadius::same(WINDOW_RADIUS);
        v.menu_corner_radius = CornerRadius::same(WINDOW_RADIUS - 1);
        let shadow = Shadow { offset: [0, 8], blur: 24, spread: 0, color: Color32::from_black_alpha(90) };
        v.window_shadow = shadow;
        v.popup_shadow = shadow;
        v.indent_has_left_vline = false;

        let w = &mut v.widgets;
        w.noninteractive.bg_fill = self.gray_1;
        w.noninteractive.weak_bg_fill = self.gray_1;
        w.noninteractive.bg_stroke = Stroke::new(1.0, self.gray_2);
        w.noninteractive.fg_stroke = Stroke::new(1.0, self.gray_6);
        w.inactive.bg_fill = self.gray_2;
        w.inactive.weak_bg_fill = self.gray_2;
        w.inactive.bg_stroke = Stroke::NONE;
        w.inactive.fg_stroke = Stroke::new(1.0, self.fg);
        w.hovered.bg_fill = self.gray_3;
        w.hovered.weak_bg_fill = self.gray_3;
        w.hovered.bg_stroke = Stroke::new(1.0, blend(self.gray_4, self.accent, 0.35));
        w.hovered.fg_stroke = Stroke::new(1.5, self.gray_7);
        w.active.bg_fill = self.gray_4;
        w.active.weak_bg_fill = self.gray_4;
        w.active.bg_stroke = Stroke::new(1.0, self.accent);
        w.active.fg_stroke = Stroke::new(2.0, self.gray_7);
        w.open.bg_fill = self.gray_2;
        w.open.weak_bg_fill = self.gray_2;
        w.open.bg_stroke = Stroke::new(1.0, self.gray_3);
        w.open.fg_stroke = Stroke::new(1.0, self.gray_7);
        for state in [&mut w.noninteractive, &mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
            state.corner_radius = radius;
        }

        v
    }
}

pub fn category_color(name: &str) -> Color32 {
    colors().category_color(name)
}

pub fn pin_type_color(name: &str) -> Color32 {
    colors().pin_type_color(name)
}

pub fn selected_fill() -> Color32 {
    colors().selected_fill()
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

/// Colors and the roomier spacing the editor uses over egui's defaults.
pub fn style(style: &mut egui::Style) {
    style.visuals = colors().visuals();
    style.spacing.item_spacing = Vec2::new(8.0, 5.0);
    style.spacing.button_padding = Vec2::new(8.0, 3.0);
    style.spacing.interact_size.y = 24.0;
    style.spacing.indent = 18.0;
}

/// The frame of a top or bottom bar, the menu, the status line and the like.
pub fn bar_frame(style: &egui::Style) -> egui::Frame {
    egui::Frame::side_top_panel(style).fill(colors().chrome)
}

/// The docks as rounded cards on the backdrop, each with its tabs as pills along the top.
pub fn dock_style(base: &egui::Style) -> egui_dock::Style {
    let c = colors();
    let mut s = egui_dock::Style::from_egui(base);
    let none = Color32::TRANSPARENT;
    let pill = CornerRadius::same(RADIUS + 2);
    let tab = |bg: Color32, text: Color32| egui_dock::TabInteractionStyle { outline_color: none, corner_radius: pill, bg_fill: bg, text_color: text };
    s.main_surface_border_stroke = Stroke::NONE;
    s.dock_area_padding = Some(Margin::same(DOCK_GAP as i8));
    s.separator.width = DOCK_GAP;
    s.separator.color_idle = none;
    s.separator.color_hovered = c.accent;
    s.separator.color_dragged = c.accent;
    s.buttons.close_tab_color = c.gray_5;
    s.buttons.close_tab_active_color = c.gray_7;
    s.buttons.close_tab_bg_fill = none;
    s.tab_bar.bg_fill = c.gray_1;
    s.tab_bar.hline_color = none;
    s.tab_bar.height = 34.0;
    s.tab_bar.inner_margin = Margin::symmetric(6, 3);
    s.tab_bar.corner_radius = CornerRadius { nw: WINDOW_RADIUS, ne: WINDOW_RADIUS, sw: 0, se: 0 };
    s.tab.active = tab(c.gray_2, c.gray_7);
    s.tab.focused = tab(c.gray_2, c.gray_7);
    s.tab.inactive = tab(none, c.gray_5);
    s.tab.hovered = tab(blend(c.gray_1, c.gray_2, 0.6), c.gray_7);
    s.tab.tab_body.bg_fill = c.gray_1;
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

    fn hex(rgb: u32) -> Color32 {
        Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
    }

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
        assert!(linear(DARK.gray_1)[2] > linear(DARK.gray_0)[2]);
    }

    /// The colors before the palette existed. The editor must keep drawing exactly these in the dark theme.
    #[test]
    fn dark_palette_keeps_its_colors() {
        let p = DARK;
        let old = [
            ("fg", p.fg, 0xdde0e6),
            ("gray_0", p.gray_0, 0x111215),
            ("gray_1", p.gray_1, 0x1d1e23),
            ("gray_2", p.gray_2, 0x2b2d34),
            ("gray_3", p.gray_3, 0x3e414a),
            ("gray_4", p.gray_4, 0x5b5f6c),
            ("gray_5", p.gray_5, 0x838896),
            ("gray_6", p.gray_6, 0xadb2bf),
            ("gray_7", p.gray_7, 0xe7e9ee),
            ("magenta", p.magenta, 0xd26fc0),
            ("green", p.green, 0x1cb58e),
            ("yellow", p.yellow, 0xecc531),
            ("blue", p.blue, 0x2e8bd6),
            ("cyan", p.cyan, 0x63c3ec),
            ("pink", p.pink, 0xe0699f),
            ("red", p.red, 0xe8613c),
            ("teal", p.teal, 0x17b3c4),
            ("orange", p.orange, 0xe99336),
            ("accent", p.accent, 0x6c8dff),
            ("backdrop", p.backdrop, 0x111215),
            ("chrome", p.chrome, 0x16171b),
            ("header", p.header, 0x26282e),
        ];
        for (name, got, rgb) in old {
            assert_eq!(got, hex(rgb), "{name}");
        }

        assert_eq!(p.axis(), [p.red, p.green, p.blue]);
        assert_eq!(
            [p.error(), p.warning(), p.success(), p.info(), p.walkable(), p.walkable_cut_off()],
            [hex(0xe8613c), hex(0xecc531), hex(0x1cb58e), hex(0x63c3ec), hex(0x1cb58e), hex(0xe8613c)]
        );
        assert_eq!(FRAME_COLORS, [hex(0x2e8bd6), hex(0x1cb58e), hex(0xe99336), hex(0xd26fc0), hex(0x17b3c4), hex(0xecc531)]);
        assert_eq!(colors(), &DARK);
    }

    #[test]
    fn dark_tables_keep_their_colors() {
        let p = DARK;
        let categories = [
            ("logic", 0x2e8bd6),
            ("math", 0xe99336),
            ("trigger", 0x1cb58e),
            ("func", 0xe8613c),
            ("light", 0xecc531),
            ("env", 0x17b3c4),
            ("game", 0x63c3ec),
            ("prop", 0xe0699f),
            ("npc", 0xd26fc0),
            ("info", 0x838896),
        ];
        for (name, rgb) in categories {
            assert_eq!(p.category_color(name), hex(rgb), "category {name}");
        }

        // Unknown names hash onto the accents, the empty name onto the first and "xyz" onto the second.
        assert_eq!(p.category_color(""), hex(0x2e8bd6));
        assert_eq!(p.category_color("xyz"), hex(0xd26fc0));
        assert_eq!(p.category_color("audio"), hex(0x63c3ec));
        assert_eq!(p.category_color("Logic"), hex(0x63c3ec), "names are case sensitive");

        let pins = [
            ("pulse", 0xe7e9ee),
            ("bool", 0xe8613c),
            ("int", 0x63c3ec),
            ("float", 0x1cb58e),
            ("string", 0xe0699f),
            ("vector3", 0xecc531),
            ("color", 0xd26fc0),
            ("node", 0x2e8bd6),
            ("variant", 0x838896),
            ("unknown type", 0x838896),
        ];
        for (name, rgb) in pins {
            assert_eq!(p.pin_type_color(name), hex(rgb), "pin {name}");
        }

        assert_eq!(p.selected_fill(), blend(hex(0x1d1e23), hex(0x6c8dff), 0.45));
        assert_eq!(selected_fill(), p.selected_fill());
    }

    /// `Palette::visuals` as it was written before the palette, with the hex values spelled out.
    fn visuals_at_45708219() -> Visuals {
        let mut v = Visuals::dark();
        let radius = CornerRadius::same(6);
        let (gray_0, gray_1, gray_2, gray_3, gray_4, gray_6, gray_7) =
            (hex(0x111215), hex(0x1d1e23), hex(0x2b2d34), hex(0x3e414a), hex(0x5b5f6c), hex(0xadb2bf), hex(0xe7e9ee));
        let (fg, cyan, yellow, red, accent) = (hex(0xdde0e6), hex(0x63c3ec), hex(0xecc531), hex(0xe8613c), hex(0x6c8dff));
        v.panel_fill = gray_1;
        v.window_fill = gray_1;
        v.window_stroke = Stroke::new(1.0, gray_3);
        v.extreme_bg_color = gray_0;
        v.code_bg_color = gray_0;
        v.faint_bg_color = blend(gray_1, gray_2, 0.5);
        v.text_edit_bg_color = Some(gray_0);
        v.hyperlink_color = cyan;
        v.warn_fg_color = yellow;
        v.error_fg_color = red;
        v.selection.bg_fill = blend(gray_2, accent, 0.6);
        v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
        v.window_corner_radius = CornerRadius::same(10);
        v.menu_corner_radius = CornerRadius::same(9);
        let shadow = Shadow { offset: [0, 8], blur: 24, spread: 0, color: Color32::from_black_alpha(90) };
        v.window_shadow = shadow;
        v.popup_shadow = shadow;
        v.indent_has_left_vline = false;

        let w = &mut v.widgets;
        w.noninteractive.bg_fill = gray_1;
        w.noninteractive.weak_bg_fill = gray_1;
        w.noninteractive.bg_stroke = Stroke::new(1.0, gray_2);
        w.noninteractive.fg_stroke = Stroke::new(1.0, gray_6);
        w.inactive.bg_fill = gray_2;
        w.inactive.weak_bg_fill = gray_2;
        w.inactive.bg_stroke = Stroke::NONE;
        w.inactive.fg_stroke = Stroke::new(1.0, fg);
        w.hovered.bg_fill = gray_3;
        w.hovered.weak_bg_fill = gray_3;
        w.hovered.bg_stroke = Stroke::new(1.0, blend(gray_4, accent, 0.35));
        w.hovered.fg_stroke = Stroke::new(1.5, gray_7);
        w.active.bg_fill = gray_4;
        w.active.weak_bg_fill = gray_4;
        w.active.bg_stroke = Stroke::new(1.0, accent);
        w.active.fg_stroke = Stroke::new(2.0, gray_7);
        w.open.bg_fill = gray_2;
        w.open.weak_bg_fill = gray_2;
        w.open.bg_stroke = Stroke::new(1.0, gray_3);
        w.open.fg_stroke = Stroke::new(1.0, gray_7);
        for state in [&mut w.noninteractive, &mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
            state.corner_radius = radius;
        }

        v
    }

    #[test]
    fn dark_visuals_are_unchanged() {
        assert_eq!(DARK.visuals(), visuals_at_45708219());

        let mut style = egui::Style::default();
        super::style(&mut style);
        assert_eq!(style.visuals, visuals_at_45708219());
        assert_eq!((style.spacing.indent, style.spacing.interact_size.y), (18.0, 24.0));
    }

    #[test]
    fn frames_and_docks_use_the_palette() {
        let style = egui::Style::default();
        assert_eq!(bar_frame(&style).fill, DARK.chrome);
        let dock = dock_style(&style);
        assert_eq!((dock.tab.active.bg_fill, dock.tab.active.text_color), (DARK.gray_2, DARK.gray_7));
        assert_eq!((dock.tab.inactive.bg_fill, dock.tab.inactive.text_color), (Color32::TRANSPARENT, DARK.gray_5));
        assert_eq!(dock.tab_bar.bg_fill, DARK.gray_1);
        assert_eq!(dock.separator.color_hovered, DARK.accent);
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
        let v = DARK.visuals();
        for (text, fill, what) in [(DARK.fg, DARK.selected_fill(), "selected rows"), (v.selection.stroke.color, v.selection.bg_fill, "selected widgets")] {
            let ratio = contrast(text, fill);
            assert!(ratio >= 4.5, "{what}: text contrast {ratio:.2} < 4.5");
        }
    }

    #[test]
    fn selection_fill_stands_out_from_panels() {
        // Slider and progress fills, the view grid corner dot and the prefab card outline use it as a mark.
        let v = DARK.visuals();
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
        let p = DARK;
        // Role pairs whose only difference is hue, so they must survive every common color blindness.
        let category = |name: &str| p.category_color(name);
        let pin = |name: &str| p.pin_type_color(name);
        let axis = p.axis();
        let pairs: [(Color32, Color32, &str); 15] = [
            (category("logic"), category("math"), "logic vs math nodes"),
            (category("logic"), category("trigger"), "logic vs trigger nodes"),
            (category("trigger"), category("func"), "trigger vs func nodes"),
            (category("math"), category("func"), "math vs func nodes"),
            (pin("int"), pin("float"), "int vs float pins"),
            (pin("bool"), pin("float"), "bool vs float pins"),
            (pin("string"), pin("vector3"), "string vs vector3 pins"),
            (pin("node"), pin("int"), "node vs int pins"),
            (p.error(), p.success(), "error vs success"),
            (p.warning(), p.success(), "warning vs success"),
            (p.warning(), p.error(), "warning vs error"),
            (axis[0], axis[1], "axis x vs y"),
            (axis[0], axis[2], "axis x vs z"),
            (axis[1], axis[2], "axis y vs z"),
            (p.walkable(), p.walkable_cut_off(), "walkable vs cut off"),
        ];
        // Redmean threshold below which two swatches read as the same color to that viewer.
        const MIN: f32 = 40.0;
        for kind in 0..3 {
            for (a, b, what) in pairs {
                let d = distance(simulate(a, kind), simulate(b, kind));
                assert!(d >= MIN, "cvd {kind}: {what} too close ({d:.0} < {MIN})");
            }

            for (c, what) in [(p.error(), "error"), (p.success(), "success"), (p.warning(), "warning"), (axis[0], "axis x"), (axis[1], "axis y")] {
                let d = distance(simulate(p.accent, kind), simulate(c, kind));
                assert!(d >= MIN, "cvd {kind}: accent vs {what} too close ({d:.0} < {MIN})");
            }
        }
    }
}
