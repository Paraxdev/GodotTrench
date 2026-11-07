//! Inspector building blocks: label and value rows, axis colored vector fields and color fields.

use egui::ecolor::{self, Hsva};
use egui::widgets::color_picker;
use egui::{Align, Color32, FontId, Layout, Pos2, Rect, RichText, Stroke, Ui, Vec2, WidgetText};
use td_formats::PropertyType;

use crate::theme;

pub const ROW_HEIGHT: f32 = 22.0;
/// Room kept at the end of a row for its reset or remove button.
pub const TRAILING: f32 = 30.0;
const AXIS_LABEL: f32 = 14.0;
const AXIS_GAP: f32 = 6.0;

/// Width of the label column, shared by every row of a panel so the values line up.
pub fn label_width(ui: &Ui) -> f32 {
    (ui.available_width() * 0.38).clamp(84.0, 170.0)
}

/// One inspector row: a label that truncates with its description on hover, then the value editor filling the rest.
/// Odd rows get a faint stripe.
pub fn row<R>(ui: &mut Ui, index: usize, label: impl Into<WidgetText>, hover: &str, label_w: f32, add: impl FnOnce(&mut Ui) -> R) -> R {
    let stripe = ui.painter().add(egui::Shape::Noop);
    let inner = ui.horizontal(|ui| {
        ui.set_min_height(ROW_HEIGHT);
        let label = ui
            .allocate_ui_with_layout(Vec2::new(label_w, ROW_HEIGHT), Layout::left_to_right(Align::Center), |ui| {
                ui.set_width(label_w);
                ui.add(egui::Label::new(label).truncate())
            })
            .inner;
        if !hover.is_empty() {
            label.on_hover_text(hover);
        }

        add(ui)
    });
    if index % 2 == 1 {
        let rect = egui::Rect::from_x_y_ranges(ui.max_rect().x_range(), inner.response.rect.y_range()).expand2(Vec2::new(2.0, 1.0));
        ui.painter().set(stripe, egui::Shape::rect_filled(rect, 2.0, ui.visuals().faint_bg_color));
    }

    inner.inner
}

/// Dark, rounded fields like text inputs for the drag values inside.
fn field_style(ui: &mut Ui) {
    let v = ui.visuals_mut();
    let field = v.extreme_bg_color;
    v.widgets.inactive.bg_fill = field;
    v.widgets.inactive.weak_bg_fill = field;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, theme::colors().gray_2);
    v.widgets.hovered.bg_fill = field;
    v.widgets.hovered.weak_bg_fill = field;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, theme::colors().gray_4);
    v.widgets.active.bg_fill = field;
    v.widgets.active.weak_bg_fill = field;
    v.widgets.active.bg_stroke = Stroke::new(1.0, theme::colors().accent);
}

/// Speed that drags small values finely and large ones quickly.
pub fn drag_speed(values: &[f64]) -> f64 {
    let largest = values.iter().fold(0.0f64, |m, v| m.max(v.abs()));
    (largest * 0.01).clamp(0.01, 1.0)
}

/// X, Y and Z (or U and V with two values) fields sharing the width, each after its axis letter in the axis color.
pub fn vector_input(ui: &mut Ui, values: &mut [f64], width: f32, speed: f64) -> bool {
    vector_input_snapped(ui, values, width, speed, 0.0)
}

/// As [`vector_input`], but a value dragged with the mouse lands on multiples of `grid` when that is above zero. A
/// typed value is kept exactly and applies once on Enter or when the field loses focus, so typing 128 never passes
/// through 1 and 12 on the way. Escape keeps the old value.
pub fn vector_input_snapped(ui: &mut Ui, values: &mut [f64], width: f32, speed: f64, grid: f64) -> bool {
    let n = values.len().max(1) as f32;
    let field_w = ((width - AXIS_GAP * (n - 1.0)) / n - AXIS_LABEL - 2.0).max(28.0);
    let mut changed = false;
    ui.scope(|ui| {
        field_style(ui);
        ui.spacing_mut().item_spacing.x = 2.0;
        for (i, value) in values.iter_mut().enumerate() {
            if i > 0 {
                ui.add_space(AXIS_GAP - 2.0);
            }

            let (letter, color) = match i {
                0 => ("X", theme::colors().axis()[0]),
                1 => ("Y", theme::colors().axis()[1]),
                _ => ("Z", theme::colors().axis()[2]),
            };
            ui.add_sized([AXIS_LABEL, ROW_HEIGHT - 2.0], egui::Label::new(RichText::new(letter).color(color).strong()));
            let before = *value;
            let drag = egui::DragValue::new(value).speed(speed).update_while_editing(false).custom_formatter(|n, _| format_number(n));
            let response = ui.add_sized([field_w, ROW_HEIGHT - 2.0], drag);
            // egui still sees the lost focus one frame after Escape and applies the typed text then, so undo that.
            let escaped = response.id.with("escaped");
            if ui.data_mut(|d| d.remove_temp::<bool>(escaped)).is_some() && response.changed() {
                *value = before;
                continue;
            }

            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                ui.data_mut(|d| d.insert_temp(escaped, true));
            }

            if response.changed() && response.dragged() && grid > 0.0 {
                *value = td_core::snap_to_grid(*value, grid);
            }

            changed |= response.changed();
        }
    });
    changed
}

pub fn drag_field(ui: &mut Ui, value: &mut f64, width: f32, speed: f64, integer: bool) -> bool {
    ui.scope(|ui| {
        field_style(ui);
        let drag = egui::DragValue::new(value).speed(if integer { speed.max(1.0) } else { speed }).custom_formatter(|n, _| format_number(n));
        ui.add_sized([width.max(40.0), ROW_HEIGHT - 2.0], drag).changed()
    })
    .inner
}

pub fn text_field(ui: &mut Ui, value: &mut String, width: f32) -> bool {
    ui.add_sized([width.max(40.0), ROW_HEIGHT - 2.0], egui::TextEdit::singleline(value)).changed()
}

/// Swatch that opens a picker, next to the raw value so it can also be typed.
pub fn color_input(ui: &mut Ui, value: &mut String, width: f32) -> bool {
    let mut hsva = picker_hsva(value);
    let swatch_w = 36.0f32.min(width * 0.35);
    let picked = ui
        .scope(|ui| {
            ui.spacing_mut().interact_size = Vec2::new(swatch_w, ROW_HEIGHT - 4.0);
            ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0, theme::colors().gray_3);
            color_picker::color_edit_button_hsva(ui, &mut hsva, color_picker::Alpha::Opaque).changed()
        })
        .inner;
    if picked {
        apply_pick(value, hsva);
    }

    picked | text_field(ui, value, width - swatch_w - ui.spacing().item_spacing.x)
}

// Map colors are sRGB like Godot's, the picker's own rgb buttons would read them as linear.
fn picker_hsva(value: &str) -> Hsva {
    Hsva::from_rgb(parse_color(value).0.map(ecolor::linear_from_gamma))
}

fn apply_pick(value: &mut String, hsva: Hsva) {
    let unit = parse_color(value).1;
    *value = format_color(hsva.to_rgb().map(ecolor::gamma_from_linear), unit);
}

/// Colors are "r g b" in 0 to 255, or 0 to 1 when every part is at most 1 and one has a decimal point.
pub fn parse_color(value: &str) -> ([f32; 3], bool) {
    let parts: Vec<&str> = value.split_whitespace().collect();
    let nums: Vec<f32> = parts.iter().filter_map(|p| p.parse().ok()).collect();
    let unit = !nums.is_empty() && nums.iter().all(|n| *n <= 1.0) && parts.iter().any(|p| p.contains('.'));
    let scale = if unit { 1.0 } else { 255.0 };
    let get = |i: usize| (nums.get(i).copied().unwrap_or(scale) / scale).clamp(0.0, 1.0);
    ([get(0), get(1), get(2)], unit)
}

pub fn format_color(rgb: [f32; 3], unit: bool) -> String {
    if unit {
        // Whole parts keep a decimal point, "1 0 0" would read back on the 0 to 255 scale.
        let part = |c: &f32| {
            let s = format_number((*c as f64 * 1000.0).round() / 1000.0);
            if s.contains('.') { s } else { s + ".0" }
        };
        rgb.iter().map(part).collect::<Vec<_>>().join(" ")
    } else {
        rgb.iter().map(|c| ((c * 255.0).round() as i32).to_string()).collect::<Vec<_>>().join(" ")
    }
}

pub fn format_number(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        let s = format!("{v:.6}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

pub fn parse_numbers(value: &str) -> Option<Vec<f64>> {
    value.split_whitespace().map(|p| p.parse().ok()).collect()
}

/// Editor type for a property: the definition's type, or a guess from the name and value when there is no definition.
/// Defined string properties only become colors, so free text is never forced into a number field.
pub fn effective_type(defined: Option<PropertyType>, name: &str, value: &str) -> PropertyType {
    let lower = name.to_ascii_lowercase();
    let numbers = parse_numbers(value).unwrap_or_default();
    let color_name = lower.contains("color") || lower.contains("colour");
    if color_name && (value.trim().is_empty() || matches!(numbers.len(), 3 | 4)) {
        return PropertyType::Color;
    }

    match defined {
        Some(ty) => ty,
        None => match numbers.len() {
            _ if matches!(value.trim(), "true" | "false") => PropertyType::Bool,
            3 => PropertyType::Vector3,
            2 => PropertyType::Vector2,
            1 if value.contains('.') || value.contains('e') => PropertyType::Float,
            1 => PropertyType::Int,
            _ => PropertyType::String,
        },
    }
}

/// A window titled in the editor's language under the id egui gives its English title, so positions, `open_windows`
/// and the MCP window list stay the same in every language.
pub fn window(title: &str) -> egui::Window<'_> {
    egui::Window::new(crate::i18n::tr(title)).id(egui::Id::new(Some(crate::i18n::english(title))))
}

/// Where a tool's hint goes at the bottom left of a viewport, and its text. It wraps at the viewport width and grows
/// upward, so a long translation stays inside the view.
fn corner_hint_galley(painter: &egui::Painter, rect: Rect, text: String, color: Color32) -> (Pos2, std::sync::Arc<egui::Galley>) {
    let galley = painter.layout(text, FontId::proportional(12.0), color, (rect.width() - 16.0).max(40.0));
    (rect.left_bottom() + Vec2::new(8.0, -8.0 - galley.size().y), galley)
}

pub fn corner_hint(painter: &egui::Painter, rect: Rect, text: String, color: Color32) {
    let (pos, galley) = corner_hint_galley(painter, rect, text, color);
    painter.galley(pos, galley, color);
}

/// A collapsible section on a raised header card.
pub struct Collapsing {
    title: RichText,
    salt: Option<String>,
    open: bool,
}

pub fn collapsing(title: impl Into<RichText>) -> Collapsing {
    Collapsing { title: title.into(), salt: None, open: false }
}

impl Collapsing {
    pub fn id_salt(mut self, salt: &str) -> Self {
        self.salt = Some(salt.to_owned());
        self
    }

    pub fn default_open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> egui::CollapsingResponse<R> {
        let salt = self.salt.unwrap_or_else(|| self.title.text().to_owned());
        let outer = ui.style().clone();
        ui.scope(|ui| {
            let label = RichText::new(self.title.text()).size(12.5).color(theme::colors().gray_7);
            ui.spacing_mut().button_padding.y = 5.0;
            let v = ui.visuals_mut();
            v.collapsing_header_frame = true;
            for state in [&mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active] {
                state.expansion = 0.0;
                state.bg_stroke = Stroke::NONE;
            }

            let hover = theme::blend(theme::colors().header, theme::colors().gray_3, 0.5);
            v.widgets.inactive.weak_bg_fill = theme::colors().header;
            v.widgets.hovered.weak_bg_fill = hover;
            v.widgets.active.weak_bg_fill = hover;
            egui::CollapsingHeader::new(label).id_salt(salt).default_open(self.open).icon(chevron).show(ui, |ui| {
                ui.set_style(outer);
                body(ui)
            })
        })
        .inner
    }
}

fn chevron(ui: &mut Ui, openness: f32, response: &egui::Response) {
    let color = if response.hovered() { theme::colors().gray_7 } else { theme::colors().accent };
    let c = response.rect.center();
    let turn = egui::emath::Rot2::from_angle(openness * std::f32::consts::FRAC_PI_2);
    let points = [egui::vec2(-2.0, -3.5), egui::vec2(2.0, 0.0), egui::vec2(-2.0, 3.5)].map(|p| c + turn * p);
    ui.painter().add(egui::Shape::line(points.to_vec(), Stroke::new(1.5, color)));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window_id(ctx: &egui::Context, window: egui::Window<'_>) -> egui::Id {
        ctx.begin_pass(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0))), ..Default::default() });
        let id = window.show(ctx, |_| {}).expect("the window is visible").response.layer_id.id;
        ctx.end_pass().drop_without_applying_deltas();
        id
    }

    #[test]
    fn a_window_shows_its_translation_under_the_id_of_its_english_title() {
        use crate::i18n::{Lang, scope, test_catalogue};
        let ctx = egui::Context::default();
        let english = window_id(&ctx, egui::Window::new("Shape Generator"));
        assert_eq!(window_id(&ctx, window("Shape Generator")), english, "English");
        let _de = test_catalogue(Lang::De, r#"{ "Shape Generator": "Formgenerator", "Create Terrain": "Formgenerator" }"#);
        let _scope = scope(Lang::De);
        assert_eq!(window_id(&ctx, window("Shape Generator")), english, "German");
        let terrain = window_id(&ctx, window("Create Terrain"));
        assert_eq!(terrain, window_id(&ctx, egui::Window::new("Create Terrain")));
        assert_ne!(terrain, english, "two keys that translate to one word keep two ids");
    }

    #[test]
    fn a_long_hint_wraps_inside_the_view_and_ends_at_its_bottom_left() {
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput::default());
        let painter = ctx.layer_painter(egui::LayerId::background());
        let text = "Scale: drag handles, Alt scales symmetrically around the center of the selection".to_string();
        let view = |w: f32| Rect::from_min_size(Pos2::new(40.0, 30.0), egui::vec2(w, 200.0));
        let (wide_pos, wide) = corner_hint_galley(&painter, view(900.0), text.clone(), Color32::WHITE);
        let (narrow_pos, narrow) = corner_hint_galley(&painter, view(220.0), text.clone(), Color32::WHITE);
        let (tiny_pos, tiny) = corner_hint_galley(&painter, view(10.0), text, Color32::WHITE);
        ctx.end_pass().drop_without_applying_deltas();
        assert_eq!(wide.rows.len(), 1);
        assert!(narrow.rows.len() > 1 && narrow.size().x <= 220.0 - 16.0, "{:?}", narrow.size());
        assert!(tiny.rows.len() > narrow.rows.len(), "a sliver of a view still wraps");
        for (pos, galley) in [(wide_pos, &wide), (narrow_pos, &narrow), (tiny_pos, &tiny)] {
            assert_eq!(pos.x, 48.0);
            assert!((pos.y + galley.size().y - 222.0).abs() < 0.01, "the last row sits 8 above the bottom edge");
        }
    }

    #[test]
    fn every_tool_hint_fits_a_small_view_in_each_language() {
        use crate::i18n::{Lang, scope, tr};
        const HINTS: [&str; 15] = [
            "Clip: click to place points, keep {side} (Tab), Enter to apply",
            "Vertex: drag a corner or a blue dot, double click a corner for a gizmo, Del removes",
            "Scale: drag handles, Alt scales symmetrically",
            "Rotate: drag a ring, snaps {degrees}° (Shift for 1°)",
            "Path: click to place path corners linked by target, Enter finishes the chain",
            "Sculpt: no terrain or displacements. Terrain > Create Terrain, or select quad faces and use Brush > Displacement",
            "Vertex paint: drag over brush faces (the selected brushes, or every brush when nothing is selected)",
            "Measure: {units} units, {meters} m   dx {dx}  dy {dy}  dz {dz}   horizontal {horizontal}",
            "Transform: X/Y/Z axis, Shift+X/Y/Z plane, type a value, click or Enter confirms, Esc cancels",
            "Loop cut: hover an edge, wheel or +/- changes cuts ({cuts}), click to cut",
            "Mesh edit: select a mesh (or brushes and press Tab to convert them)",
            "Mesh edit ({mode}): 1/2/3 mode, G/R/S transform, E extrude, I inset, Ctrl+R loop cut, double click for a gizmo",
            "Gizmo: arrows and squares move, rings rotate, boxes scale. Esc hides it",
            "Bevel: move the mouse to set the width, click to confirm",
            "Sculpt {mode}: drag to apply, Shift inverts, Ctrl smooths",
        ];
        let view = Rect::from_min_size(Pos2::new(40.0, 30.0), egui::vec2(480.0, 300.0));
        for lang in [Lang::De, Lang::Ru, Lang::Ja] {
            let _scope = scope(lang);
            let ctx = egui::Context::default();
            ctx.set_fonts(crate::fonts::definitions(lang));
            ctx.begin_pass(egui::RawInput::default());
            ctx.end_pass().drop_without_applying_deltas();
            ctx.begin_pass(egui::RawInput::default());
            let painter = ctx.layer_painter(egui::LayerId::background());
            for key in HINTS {
                let shown = tr(key);
                let (pos, galley) = corner_hint_galley(&painter, view, shown.to_string(), Color32::WHITE);
                assert!(galley.rows.len() <= 3, "{lang:?} {key}: {} rows", galley.rows.len());
                assert!(galley.size().x <= view.width() - 16.0 + 0.01, "{lang:?} {key}");
                assert!(pos.y >= view.top() + 150.0, "{lang:?} {key}: the hint climbs past the middle of the view");
            }

            ctx.end_pass().drop_without_applying_deltas();
        }
    }

    #[test]
    fn guesses_editors_for_undefined_properties() {
        assert_eq!(effective_type(None, "ambient_color", "76 86 104"), PropertyType::Color);
        assert_eq!(effective_type(None, "sun_angles", "-42 -35"), PropertyType::Vector2);
        assert_eq!(effective_type(None, "hinge", "0 0 0"), PropertyType::Vector3);
        assert_eq!(effective_type(None, "fog_density", "0.0002"), PropertyType::Float);
        assert_eq!(effective_type(None, "message", "Church and School"), PropertyType::String);
        assert_eq!(effective_type(Some(PropertyType::String), "targetname", "12"), PropertyType::String);
        assert_eq!(effective_type(Some(PropertyType::String), "light_color", "255 244 225"), PropertyType::Color);
        assert_eq!(effective_type(Some(PropertyType::Float), "speed", "90"), PropertyType::Float);
    }

    #[test]
    fn colors_keep_their_number_format() {
        assert_eq!(parse_color("255 128 0"), ([1.0, 128.0 / 255.0, 0.0], false));
        let (rgb, unit) = parse_color("1 0.5 0.25");
        assert!(unit);
        assert_eq!(format_color(rgb, unit), "1.0 0.5 0.25");
        assert_eq!(format_color([1.0, 0.5, 0.0], false), "255 128 0");
        for rgb in [[1.0, 1.0, 1.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 1.0]] {
            let text = format_color(rgb, true);
            assert_eq!(parse_color(&text), (rgb, true), "{text}");
        }

        assert_eq!(format_color([1.0, 0.0, 0.0], true), "1.0 0.0 0.0");
        assert_eq!(format_number(-35.0), "-35");
        assert_eq!(format_number(0.0002), "0.0002");
    }

    #[test]
    fn color_picker_reads_values_as_srgb() {
        assert_eq!(picker_hsva("89 23 72").to_srgb(), [89, 23, 72]);
        let mut value = "89 23 72".to_string();
        apply_pick(&mut value, Hsva::from_srgb([200, 100, 50]));
        assert_eq!(value, "200 100 50");
    }
}
