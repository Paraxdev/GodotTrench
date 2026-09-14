//! Lucide icons (ISC, see assets/icons/LICENSE-lucide.txt) and a simplified Godot logo (CC BY 4.0) embedded as SVG.
//!
//! They are drawn in white so buttons can tint them with the theme text color.

use egui::{Atom, Button, Image, ImageSource, Response, Ui, Vec2, Widget, WidgetText};

use crate::state::Shade;
use crate::tools::ToolKind;

/// Icon size in toolbars.
pub const TOOLBAR: f32 = 18.0;
/// Largest toolbar icon size when the toolbar is dragged taller.
pub const TOOLBAR_MAX: f32 = 40.0;
/// Icon size in menus, panels and the outliner.
pub const SMALL: f32 = 14.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Icon {
    uri: &'static str,
    bytes: &'static [u8],
}

macro_rules! icons {
    ($($name:ident = $file:literal,)*) => {
        $(pub const $name: Icon = Icon {
            uri: concat!("bytes://icons/", $file, ".svg"),
            bytes: include_bytes!(concat!("../assets/icons/", $file, ".svg")),
        };)*
        #[cfg(test)]
        const ALL: &[Icon] = &[$($name),*];
    };
}

icons! {
    SELECT = "mouse-pointer-2",
    CLIP = "scissors",
    VERTEX = "vector-square",
    ROTATE = "rotate-3d",
    SCALE = "scale-3d",
    MESH = "pyramid",
    SCULPT = "mountain-snow",
    BLEND = "blend",
    PAINT = "paintbrush",
    SCATTER = "trees",
    VOLUME = "square-dashed",
    PATH = "waypoints",
    MEASURE = "ruler",
    TEXTURE = "paint-roller",
    NEW = "file-plus",
    OPEN = "folder-open",
    SAVE = "save",
    UNDO = "undo-2",
    REDO = "redo-2",
    GRID = "grid-3x3",
    SNAP = "magnet",
    UV_LOCK = "lock-keyhole",
    CSG_SUBTRACT = "squares-subtract",
    CSG_MERGE = "squares-unite",
    CSG_INTERSECT = "squares-intersect",
    CSG_HOLLOW = "square-square",
    SHADE_TEXTURED = "brick-wall",
    SHADE_FLAT = "paint-bucket",
    SHADE_LIT = "sun",
    SHADE_WIREFRAME = "box",
    LAYER = "layers",
    GROUP = "group",
    ENTITY = "diamond",
    BRUSH = "cuboid",
    INSTANCE = "package",
    TERRAIN = "mountain",
    EYE = "eye",
    EYE_OFF = "eye-off",
    LOCK = "lock",
    UNLOCK = "lock-open",
    EXPAND = "chevron-right",
    COLLAPSE = "chevron-down",
    PLUS = "plus",
    CHECK = "check",
    IMPORT = "file-input",
    EXPORT = "file-output",
    RECENT = "history",
    PLAY = "play",
    SETTINGS = "settings",
    KEYBOARD = "keyboard",
    COMMAND = "command",
    REFERENCE = "book-open",
    HELP = "circle-help",
    LINK = "link",
    DOOR = "door-open",
    FOCUS = "scan-eye",
    COPY = "copy",
    PASTE = "clipboard-paste",
    DELETE = "trash-2",
    SHAPES = "shapes",
    GODOT = "godot",
}

impl Icon {
    pub fn image(self, size: f32) -> Image<'static> {
        Image::new(ImageSource::Bytes { uri: self.uri.into(), bytes: egui::load::Bytes::Static(self.bytes) }).fit_to_exact_size(Vec2::splat(size))
    }
}

pub fn install(ctx: &egui::Context) {
    egui_extras::install_image_loaders(ctx);
}

pub fn tool(tool: ToolKind) -> Icon {
    match tool {
        ToolKind::Select => SELECT,
        ToolKind::Clip => CLIP,
        ToolKind::Vertex => VERTEX,
        ToolKind::Rotate => ROTATE,
        ToolKind::Scale => SCALE,
        ToolKind::Mesh => MESH,
        ToolKind::Sculpt => SCULPT,
        ToolKind::Blend => BLEND,
        ToolKind::Paint => PAINT,
        ToolKind::Scatter => SCATTER,
        ToolKind::Volume => VOLUME,
        ToolKind::Path => PATH,
        ToolKind::Measure => MEASURE,
        ToolKind::Texture => TEXTURE,
    }
}

pub fn shade(shade: Shade) -> Icon {
    match shade {
        Shade::Textured => SHADE_TEXTURED,
        Shade::Flat => SHADE_FLAT,
        Shade::Lit => SHADE_LIT,
        Shade::Wireframe => SHADE_WIREFRAME,
    }
}

/// Empty atom the size of an icon, so menu labels without an icon line up with those that have one.
pub fn spacer(size: f32) -> Atom<'static> {
    Atom { size: Some(Vec2::splat(size)), ..Default::default() }
}

/// Icon atom for a menu row, or a spacer when there is no icon.
pub fn atom(icon: Option<Icon>, size: f32) -> Atom<'static> {
    match icon {
        Some(i) => i.image(size).into(),
        None => spacer(size),
    }
}

/// Toolbar icon button of `size` points. `label` is its accessible name, `tooltip` shows on hover.
pub fn button(ui: &mut Ui, icon: Icon, size: f32, label: &str, tooltip: impl Into<WidgetText>) -> Response {
    Button::image(icon.image(size).alt_text(label)).image_tint_follows_text_color(true).frame_when_inactive(false).ui(ui).on_hover_text(tooltip)
}

/// Toolbar icon toggle that stays highlighted while `selected`.
pub fn toggle(ui: &mut Ui, icon: Icon, size: f32, selected: bool, label: &str, tooltip: impl Into<WidgetText>) -> Response {
    Button::image(icon.image(size).alt_text(label))
        .selected(selected)
        .frame_when_inactive(selected)
        .image_tint_follows_text_color(true)
        .ui(ui)
        .on_hover_text(tooltip)
}

/// Frameless small icon button for panel rows.
pub fn small(ui: &mut Ui, icon: Icon, label: &str, tint: Option<egui::Color32>) -> Response {
    let mut image = icon.image(SMALL).alt_text(label);
    if let Some(color) = tint {
        image = image.tint(color);
    }
    Button::image(image).frame(false).image_tint_follows_text_color(tint.is_none()).ui(ui)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn svg_attr<'a>(svg_tag: &'a str, name: &str) -> Option<&'a str> {
        let start = svg_tag.find(&format!(" {name}=\""))? + name.len() + 3;
        svg_tag[start..].split('"').next()
    }

    #[test]
    fn icons_are_square_white_svgs() {
        for icon in ALL {
            let text = std::str::from_utf8(icon.bytes).unwrap();
            let tag = text.find("<svg").map(|i| &text[i..]).and_then(|t| t.split('>').next()).unwrap_or_else(|| panic!("{} is not an svg", icon.uri));
            let square = match svg_attr(tag, "viewBox") {
                Some(view_box) => {
                    let n: Vec<&str> = view_box.split_whitespace().collect();
                    n.len() == 4 && n[2] == n[3]
                }
                None => svg_attr(tag, "width").is_some() && svg_attr(tag, "width") == svg_attr(tag, "height"),
            };
            assert!(square, "{} must be square", icon.uri);
            assert!(!text.contains("currentColor"), "{} must be drawn in white so it can be tinted", icon.uri);
        }
    }
}
