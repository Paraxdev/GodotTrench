//! Inspector building blocks: label and value rows, axis colored vector fields and color fields.

use egui::{Align, Layout, RichText, Stroke, Ui, Vec2, WidgetText};
use gt_formats::PropertyType;

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
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, theme::GRAY_2);
    v.widgets.hovered.bg_fill = field;
    v.widgets.hovered.weak_bg_fill = field;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, theme::GRAY_4);
    v.widgets.active.bg_fill = field;
    v.widgets.active.weak_bg_fill = field;
    v.widgets.active.bg_stroke = Stroke::new(1.0, theme::ACCENT);
}

/// Speed that drags small values finely and large ones quickly.
pub fn drag_speed(values: &[f64]) -> f64 {
    let largest = values.iter().fold(0.0f64, |m, v| m.max(v.abs()));
    (largest * 0.01).clamp(0.01, 1.0)
}

/// X, Y and Z (or U and V with two values) fields sharing the width, each after its axis letter in the axis color.
pub fn vector_input(ui: &mut Ui, values: &mut [f64], width: f32, speed: f64) -> bool {
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
                0 => ("X", theme::AXIS[0]),
                1 => ("Y", theme::AXIS[1]),
                _ => ("Z", theme::AXIS[2]),
            };
            ui.add_sized([AXIS_LABEL, ROW_HEIGHT - 2.0], egui::Label::new(RichText::new(letter).color(color).strong()));
            let drag = egui::DragValue::new(value).speed(speed).custom_formatter(|n, _| format_number(n));
            changed |= ui.add_sized([field_w, ROW_HEIGHT - 2.0], drag).changed();
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
    let (mut rgb, unit) = parse_color(value);
    let swatch_w = 36.0f32.min(width * 0.35);
    let picked = ui
        .scope(|ui| {
            ui.spacing_mut().interact_size = Vec2::new(swatch_w, ROW_HEIGHT - 4.0);
            ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0, theme::GRAY_3);
            ui.color_edit_button_rgb(&mut rgb).changed()
        })
        .inner;
    if picked {
        *value = format_color(rgb, unit);
    }
    picked | text_field(ui, value, width - swatch_w - ui.spacing().item_spacing.x)
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
        rgb.iter().map(|c| format_number((*c as f64 * 1000.0).round() / 1000.0)).collect::<Vec<_>>().join(" ")
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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(format_color(rgb, unit), "1 0.5 0.25");
        assert_eq!(format_color([1.0, 0.5, 0.0], false), "255 128 0");
        assert_eq!(format_number(-35.0), "-35");
        assert_eq!(format_number(0.0002), "0.0002");
    }
}
