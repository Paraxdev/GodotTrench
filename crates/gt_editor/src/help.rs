//! Help inside the editor: the short guidance lines, and the manual page behind each tool, panel, menu and Inspector
//! section. F1 over one of them opens its page, and so does a Ctrl+click (Cmd+click on macOS) on a tool in the toolbar.

use egui::{Context, Key, Modifiers, Response, RichText, Ui, WidgetText};

use crate::tools::ToolKind;

/// The published manual, built from `docs/`.
pub const SITE: &str = "https://paraxdev.github.io/GodotTrench/";
/// Ends the tooltip of anything F1 opens the manual for.
pub const F1_HINT: &str = "F1: manual";
/// The click on a toolbar tool that opens its page. egui's command modifier is Cmd on macOS, where Ctrl+click is a right
/// click.
pub const MANUAL_CLICK: &str = if cfg!(target_os = "macos") { "Cmd+click" } else { "Ctrl+click" };
/// Ends the tooltip of a toolbar tool.
pub const TOOL_HINT: &str = if cfg!(target_os = "macos") { "F1 or Cmd+click: manual" } else { "F1 or Ctrl+click: manual" };

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Topic<'a> {
    Tool(ToolKind),
    /// A dock tab by its title, "View" for any of the views.
    Panel(&'a str),
    /// A menu, submenu or menu item by its label.
    Menu(&'a str),
    /// An Inspector section or heading by its title, without the count some of them show.
    Section(&'a str),
}

use Topic::{Menu, Panel, Section, Tool};

/// Every topic with its page, the path under `docs/` without `.md`, and the anchor of a heading on that page or "" for
/// its top. Anchors are HonKit's: the heading in lower case, spaces as hyphens, punctuation and underscores dropped.
pub const PAGES: &[(Topic<'static>, &str, &str)] = &[
    (Tool(ToolKind::Select), "editor/brushes", "draw-and-move"),
    (Tool(ToolKind::Clip), "editor/brushes", "reshape"),
    (Tool(ToolKind::Vertex), "editor/brushes", "reshape"),
    (Tool(ToolKind::Rotate), "editor/brushes", "rotate-scale-and-measure"),
    (Tool(ToolKind::Scale), "editor/brushes", "rotate-scale-and-measure"),
    (Tool(ToolKind::Mesh), "editor/meshes", ""),
    (Tool(ToolKind::Texture), "editor/texturing", "the-texture-tool"),
    (Tool(ToolKind::Paint), "editor/texturing", "vertex-paint"),
    (Tool(ToolKind::Sculpt), "editor/terrain", "sculpting"),
    (Tool(ToolKind::Blend), "editor/terrain-painting", "the-blend-tool"),
    (Tool(ToolKind::Scatter), "editor/scatter", "painting"),
    (Tool(ToolKind::Volume), "gameplay/entities/triggers", ""),
    (Tool(ToolKind::Path), "gameplay/entities/actors", "pathcorner"),
    (Tool(ToolKind::Measure), "editor/brushes", "rotate-scale-and-measure"),
    (Panel("View"), "editor/interface", "camera"),
    (Panel("Outliner"), "editor/organizing", "layers"),
    (Panel("Inspector"), "editor/interface", "the-window"),
    (Panel("Materials"), "editor/texturing", "applying-materials"),
    (Panel("Models"), "editor/models", "placing-a-model"),
    (Panel("Entities"), "gameplay/placing", "placing-entities"),
    (Panel("History"), "editor/organizing", "history"),
    (Panel("Issues"), "editor/organizing", "issues"),
    (Panel("Logic"), "gameplay/placing", "checking-wiring-without-godot"),
    (Panel("UV Editor"), "editor/uv-editor", ""),
    (Panel("Reference"), "gameplay/placing", "code-for-any-entity"),
    (Panel("Scatter"), "editor/scatter", "scatter-sets"),
    (Menu("File"), "editor/saving", ""),
    (Menu("Open Recent"), "editor/saving", "recovering-a-map"),
    (Menu("Import"), "editor/importing", ""),
    (Menu("Convert Textures (VTF/VMT, WAD, WAL)…"), "editor/importing-textures", ""),
    (Menu("Edit"), "editor/shortcuts", "editing"),
    (Menu("Groups"), "editor/organizing", "groups"),
    (Menu("Prefabs"), "editor/organizing", "prefabs"),
    (Menu("Layers"), "editor/organizing", "layers"),
    (Menu("Brush"), "editor/brushes", ""),
    (Menu("Shape Generator…"), "editor/brushes", "shape-generator"),
    (Menu("CSG"), "editor/brushes", "csg"),
    (Menu("Displacement"), "editor/brushes", "displacements"),
    (Menu("Mesh"), "editor/meshes", ""),
    (Menu("Texture"), "editor/texturing", ""),
    (Menu("UV Editor"), "editor/uv-editor", ""),
    (Menu("Hotspot Editor…"), "editor/texturing", "hotspots"),
    (Menu("Hotspot Fit"), "editor/texturing", "hotspots"),
    (Menu("Justify"), "editor/texturing", "the-texture-tool"),
    (Menu("Mesh UVs"), "editor/uv-editor", "mesh-uv-projections"),
    (Menu("Terrain"), "editor/terrain", ""),
    (Menu("Create Terrain…"), "editor/terrain", "creating-a-terrain"),
    (Menu("Sculpt"), "editor/terrain", "sculpting"),
    (Menu("Blend"), "editor/terrain-painting", "the-blend-tool"),
    (Menu("Auto Paint Layers"), "editor/terrain-painting", "auto-paint"),
    (Menu("Scatter"), "editor/scatter", ""),
    (Menu("Gameplay"), "gameplay/placing", ""),
    (Menu("Doors and Movers"), "gameplay/entities/movers", ""),
    (Menu("Triggers"), "gameplay/entities/triggers", ""),
    (Menu("Logic"), "gameplay/entities/logic", ""),
    (Menu("Link Two Selected Entities…"), "gameplay/placing", "wiring"),
    (Menu("Spawning"), "gameplay/entities/actors", ""),
    (Menu("Paths"), "gameplay/entities/actors", "pathcorner"),
    (Menu("Tools"), "editor/shortcuts", "tools"),
    (Menu("View"), "editor/interface", "views"),
    (Menu("Godot Overlays"), "godot/overlays", "what-the-level-designer-sees"),
    (Menu("Walkable Area"), "godot/navigation", ""),
    (Menu("Views"), "editor/interface", "views"),
    (Menu("Shading"), "editor/interface", "shading"),
    (Menu("Interface Scale"), "editor/interface", "interface-scale"),
    (Menu("Cordon"), "editor/organizing", "cordon"),
    (Menu("Camera Bookmarks"), "editor/interface", "camera"),
    (Menu("Godot"), "godot/running-godot", ""),
    (Menu("Run Project"), "godot/running-godot", ""),
    (Menu("Build in Godot"), "godot/building", "three-ways-to-build"),
    (Menu("Live Mode"), "godot/live", "live-mode"),
    (Menu("Command Palette"), "editor/interface", "the-window"),
    (Menu("Keyboard Shortcuts…"), "editor/shortcuts", ""),
    (Menu("Entity and Code Reference"), "gameplay/placing", "code-for-any-entity"),
    (Menu("Getting Started Guide"), "getting-started", ""),
    (Menu("Manual"), "README", ""),
    (Menu("Mouse and Keys"), "editor/shortcuts", "mouse-in-the-views"),
    (Section("Map (worldspawn)"), "godot/building", "environment-and-sun"),
    (Section("Selection"), "editor/brushes", "exact-position-and-size"),
    (Section("Transform"), "editor/brushes", "draw-and-move"),
    (Section("Gameplay"), "gameplay/placing", "placing-entities"),
    (Section("Make Brush Entity"), "gameplay/placing", "placing-entities"),
    (Section("Mesh Operations"), "editor/meshes", ""),
    (Section("Terrain"), "editor/terrain-layers", ""),
    (Section("Scatter set"), "editor/scatter", "scatter-sets"),
    (Section("Properties"), "gameplay/entities/README", "reading-the-entries"),
    (Section("Outputs"), "gameplay/placing", "wiring"),
    (Section("Alignment"), "editor/texturing", "the-texture-tool"),
    (Section("Texture Tools"), "editor/texturing", ""),
];

pub fn page(topic: Topic) -> Option<(&'static str, &'static str)> {
    PAGES.iter().find(|(t, _, _)| *t == topic).map(|(_, page, anchor)| (*page, *anchor))
}

pub fn url(topic: Topic) -> Option<String> {
    let (page, anchor) = page(topic)?;
    // HonKit publishes every README.md as the index.html of its folder.
    let page = page.strip_suffix("README").map_or_else(|| page.to_string(), |folder| format!("{folder}index"));
    Some(if anchor.is_empty() { format!("{SITE}{page}.html") } else { format!("{SITE}{page}.html#{anchor}") })
}

fn hover_id() -> egui::Id {
    egui::Id::new("gt_manual_under_pointer")
}

/// Lets F1 open the page of `topic` while the pointer is over `response`. Topics without a page are left out.
pub fn register(response: &Response, topic: Topic) {
    if response.hovered()
        && let Some(url) = url(topic)
    {
        response.ctx.data_mut(|d| d.insert_temp(hover_id(), url));
    }
}

/// The page registered under the pointer this frame. Widgets run after the keys are read, so the app takes it at the
/// end of each frame and hands it to [`open_on_f1`] in the next.
pub fn take_hovered(ctx: &Context) -> Option<String> {
    ctx.data_mut(|d| d.remove_temp::<String>(hover_id()))
}

/// F1 over something with a page opens it and takes the key. Anywhere else F1 keeps its binding, the command palette
/// unless the keymap says otherwise.
pub fn open_on_f1(ctx: &Context, hovered: Option<&str>) -> bool {
    let Some(url) = hovered else { return false };
    if !ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F1)) {
        return false;
    }

    ctx.open_url(egui::OpenUrl::new_tab(url));
    true
}

/// Opens the page of `topic` in the browser, through the app's link handling.
pub fn open(ctx: &Context, topic: Topic) {
    if let Some(url) = url(topic) {
        ctx.open_url(egui::OpenUrl::new_tab(url));
    }
}

/// A Ctrl+click, Cmd+click on macOS, on a toolbar tool opens its page instead of picking the tool.
pub fn open_on_ctrl_click(response: &Response, topic: Topic) -> bool {
    if !response.clicked() || !response.ctx.input(|i| i.modifiers.command) {
        return false;
    }

    open(&response.ctx, topic);
    true
}

/// Tooltip text ending with a faint `hint` line.
pub fn tooltip(ui: &Ui, text: &str, hint: &str) -> WidgetText {
    let mut job = egui::text::LayoutJob::default();
    let style = ui.style();
    RichText::new(format!("{text}\n")).append_to(&mut job, style, egui::FontSelection::Default, egui::Align::Min);
    RichText::new(hint).weak().append_to(&mut job, style, egui::FontSelection::Default, egui::Align::Min);
    job.into()
}

/// The faint hint line at the end of a tooltip built with `on_hover_ui`.
pub fn hint(ui: &mut Ui, hint: &str) {
    ui.label(RichText::new(hint).weak());
}

/// One line of guidance for a tool without options, shown in the tool options bar. The toolbar tooltip has the rest,
/// see [`crate::panels::TOOL_HELP`].
pub fn tool_line(tool: ToolKind) -> &'static str {
    match tool {
        ToolKind::Select => "Click to pick, drag empty space to draw a brush, drag the selection to move it",
        ToolKind::Clip => "Click two or three points, Tab picks the side to keep, Enter clips",
        ToolKind::Vertex => "Drag corners and midpoints, Delete removes vertices",
        ToolKind::Rotate => "Drag a ring to rotate, Shift for 1° steps",
        ToolKind::Scale => "Drag a handle, Alt scales from the center",
        ToolKind::Path => "Click to place path corners, Enter finishes",
        ToolKind::Measure => "Drag or click two points",
        _ => "",
    }
}

/// A faint line of guidance, left out when the Help text preference is off. `details` shows on hover.
pub fn line(ui: &mut Ui, show: bool, text: &str, details: &str) {
    if !show {
        return;
    }

    let label = ui.label(RichText::new(text).weak());
    if !details.is_empty() {
        label.on_hover_text(details);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// HonKit's heading id, from the `github-slugid` package it uses.
    fn slug(heading: &str) -> String {
        const SYMBOLS: &str = "[]!\"'#$%&()*+,./:;<=>?@\\^_`{|}~©∑®†“”‘’∂ƒ™℠…œŒ˚ºª•∆∞♥";
        let kept: String = heading.chars().filter(|c| !SYMBOLS.contains(*c)).collect();
        let slug = kept.replace(' ', "-").to_lowercase();
        slug.strip_prefix('-').map(str::to_string).unwrap_or(slug)
    }

    /// The text of the headings in a markdown page, as the HTML shows it.
    fn headings(markdown: &str) -> Vec<String> {
        let mut fenced = false;
        let mut out = Vec::new();
        for line in markdown.lines() {
            if line.trim_start().starts_with("```") {
                fenced = !fenced;
            }

            if fenced || !line.starts_with('#') {
                continue;
            }

            let mut text = line.trim_start_matches('#').trim().to_string();
            // A link shows only its text.
            while let Some(start) = text.find("](") {
                let end = text[start..].find(')').map_or(text.len(), |e| start + e + 1);
                text.replace_range(start + 1..end, "");
            }

            out.push(text);
        }

        out
    }

    #[test]
    fn slugs_match_honkit() {
        assert_eq!(slug("Hello World"), "hello-world");
        assert_eq!(slug("Cool !!"), "cool-");
        assert_eq!(slug("4. Build a room"), "4-build-a-room");
        assert_eq!(slug("path_corner"), "pathcorner");
        assert_eq!(slug("Rotate, scale and measure"), "rotate-scale-and-measure");
        assert_eq!(headings("# Title\n```\n# not a heading\n```\n## See [Targets](x.md#y) here"), ["Title", "See [Targets] here"]);
    }

    #[test]
    fn every_page_and_anchor_exists() {
        let docs = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
        for (topic, page, anchor) in PAGES {
            let path = docs.join(format!("{page}.md"));
            let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{topic:?}: {} does not exist: {e}", path.display()));
            let summary = std::fs::read_to_string(docs.join("SUMMARY.md")).unwrap();
            assert!(summary.contains(&format!("({page}.md)")), "{topic:?}: {page}.md is not in SUMMARY.md, so it is not published");
            if !anchor.is_empty() {
                let slugs: Vec<String> = headings(&text).iter().map(|h| slug(h)).collect();
                assert!(slugs.iter().any(|s| s == anchor), "{topic:?}: no heading on {page}.md has the anchor #{anchor}, it has {slugs:?}");
            }
        }
    }

    #[test]
    fn every_tool_has_a_page_and_one_line() {
        for tool in ToolKind::all() {
            assert!(url(Tool(tool)).is_some(), "{tool:?}");
        }

        let with_options = [ToolKind::Mesh, ToolKind::Texture, ToolKind::Paint, ToolKind::Sculpt, ToolKind::Blend, ToolKind::Scatter, ToolKind::Volume];
        for tool in ToolKind::all().into_iter().filter(|t| !with_options.contains(t)) {
            assert!(!tool_line(tool).is_empty(), "{tool:?} shows a line where other tools show options");
            assert!(tool_line(tool).len() < 90, "{tool:?}: one short line, the tooltip has the details");
        }

        assert_eq!(url(Tool(ToolKind::Clip)).unwrap(), format!("{SITE}editor/brushes.html#reshape"));
        assert_eq!(url(Menu("Mesh")).unwrap(), format!("{SITE}editor/meshes.html"));
        assert_eq!(url(Menu("Manual")).unwrap(), format!("{SITE}index.html"));
        assert_eq!(url(Section("Properties")).unwrap(), format!("{SITE}gameplay/entities/index.html#reading-the-entries"));
        assert_eq!(url(Menu("Getting Started Guide")).unwrap(), crate::welcome::GETTING_STARTED_URL);
        assert_eq!(url(Menu("Not a menu")), None);
    }

    /// A button with a page and one without, driven through egui's real input the way the app handles F1: the page
    /// under the pointer is taken at the end of a frame and checked when the next frame reads its keys.
    #[test]
    fn f1_opens_the_page_under_the_pointer_and_nothing_else() {
        use egui_kittest::kittest::Queryable;

        #[derive(Default)]
        struct Probe {
            under_pointer: Option<String>,
            opened: Vec<String>,
            f1_left: usize,
            picked: usize,
        }

        let mut harness = egui_kittest::Harness::builder().with_size(egui::vec2(400.0, 200.0)).build_ui_state(
            |ui, p: &mut Probe| {
                open_on_f1(ui.ctx(), p.under_pointer.as_deref());
                if ui.input(|i| i.key_pressed(Key::F1)) {
                    p.f1_left += 1;
                }

                let tool = ui.button("Clip");
                register(&tool, Tool(ToolKind::Clip));
                if !open_on_ctrl_click(&tool, Tool(ToolKind::Clip)) && tool.clicked() {
                    p.picked += 1;
                }

                let _ = ui.button("Plain");
                p.under_pointer = take_hovered(ui.ctx());
                p.opened.extend(crate::panels::take_open_urls(ui.ctx()));
            },
            Probe::default(),
        );
        harness.run();
        let clip_url = url(Tool(ToolKind::Clip)).unwrap();

        harness.get_by_label("Clip").hover();
        harness.run();
        harness.key_press(Key::F1);
        harness.run();
        assert_eq!(harness.state().opened, std::slice::from_ref(&clip_url));
        assert_eq!(harness.state().f1_left, 0, "the key is taken, so the command palette stays shut");

        harness.get_by_label("Plain").hover();
        harness.run();
        harness.key_press(Key::F1);
        harness.run();
        assert_eq!(harness.state().opened.len(), 1, "no page there");
        assert_eq!(harness.state().f1_left, 1, "F1 goes on to its binding");

        harness.get_by_label("Clip").click_modifiers(Modifiers::COMMAND);
        harness.run();
        assert_eq!(harness.state().opened, [clip_url.clone(), clip_url]);
        assert_eq!(harness.state().picked, 0, "a Ctrl+click opens the manual instead of picking the tool");
        harness.get_by_label("Clip").click();
        harness.run();
        assert_eq!(harness.state().picked, 1);
    }
}
