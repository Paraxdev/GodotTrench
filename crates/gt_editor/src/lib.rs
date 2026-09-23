pub mod app;
pub mod blend_tool;
pub mod brand;
pub mod camera;
pub mod code_refs;
pub mod commands;
pub mod dialogs;
pub mod entity_wizards;
pub mod extra_tools;
pub mod face_cull;
pub mod gizmos;
pub mod godot;
pub mod hotspot_editor;
pub mod icons;
pub mod live_link;
pub mod live_sync;
pub mod logic_sim;
pub mod materials;
pub mod mcp;
pub mod mesh_tool;
pub mod model_thumbs;
pub mod models;
pub mod overlays;
pub mod panels;
pub mod picking;
pub mod prefabs;
pub mod scatter_panel;
pub mod scatter_tool;
pub mod scene;
pub mod state;
pub mod texture_ops;
pub mod texture_tool;
pub mod theme;
pub mod tools;
pub mod toolset;
pub mod transform_gizmo;
pub mod uv_editor;
pub mod viewport;
pub mod volume_tool;
pub mod widgets;

use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct CliArgs {
    pub map: Option<PathBuf>,
    pub project: Option<PathBuf>,
    /// Serve MCP over stdin/stdout.
    pub mcp_stdio: bool,
    /// Serve MCP over streamable HTTP on 127.0.0.1.
    pub mcp_http: Option<u16>,
    /// Start with default preferences and never write them back (automated tests).
    pub default_prefs: bool,
    /// Convert a map file and exit instead of opening the editor.
    pub convert: Option<Convert>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Convert {
    /// Print any map file as readable JSON, for diffs and quick looks.
    Dump(PathBuf),
    ToJson(PathBuf, PathBuf),
    /// Any map file to the binary `.gtm`, keeping its content exactly.
    ToGtm(PathBuf, PathBuf),
}

pub const DEFAULT_MCP_PORT: u16 = 7841;

impl CliArgs {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Self {
        let mut out = CliArgs::default();
        let mut it = args.into_iter();
        while let Some(arg) = it.next() {
            match arg.as_str() {
                "--mcp" => out.mcp_stdio = true,
                "--project" => out.project = it.next().map(PathBuf::from),
                "--mcp-http" => out.mcp_http = Some(DEFAULT_MCP_PORT),
                "--default-prefs" => out.default_prefs = true,
                "--dump" => out.convert = it.next().map(|p| Convert::Dump(p.into())),
                "--to-json" | "--to-gtm" => {
                    let (Some(from), Some(to)) = (it.next(), it.next()) else { continue };
                    let (from, to) = (PathBuf::from(from), PathBuf::from(to));
                    out.convert = Some(if arg == "--to-json" { Convert::ToJson(from, to) } else { Convert::ToGtm(from, to) });
                }
                a if a.starts_with("--mcp-http=") => out.mcp_http = a["--mcp-http=".len()..].parse().ok().or(Some(DEFAULT_MCP_PORT)),
                a if !a.starts_with("--") => out.map = Some(PathBuf::from(a)),
                _ => {}
            }
        }

        out
    }
}

/// Runs a conversion, reporting what a damaged input lost on stderr.
pub fn convert(c: &Convert) -> Result<(), String> {
    let read = |path: &std::path::Path| std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()));
    let report = |path: &std::path::Path, problems: &[String]| problems.iter().for_each(|p| eprintln!("{}: {p}", path.display()));
    match c {
        Convert::Dump(from) | Convert::ToJson(from, _) => {
            let (text, problems) = gt_doc::format::file_to_json(&read(from)?).map_err(|e| format!("{}: {e}", from.display()))?;
            report(from, &problems);
            match c {
                Convert::ToJson(_, to) => std::fs::write(to, text).map_err(|e| format!("{}: {e}", to.display())),
                _ => {
                    use std::io::Write;
                    std::io::stdout().write_all(text.as_bytes()).map_err(|e| e.to_string())
                }
            }
        }
        Convert::ToGtm(from, to) => {
            let (bytes, problems) = gt_doc::format::file_to_binary(&read(from)?).map_err(|e| format!("{}: {e}", from.display()))?;
            report(from, &problems);
            std::fs::write(to, bytes).map_err(|e| format!("{}: {e}", to.display()))
        }
    }
}

pub fn run(args: CliArgs) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("GodotTrench")
            .with_icon(brand::icon())
            .with_inner_size([1600.0, 960.0])
            .with_min_inner_size([800.0, 500.0]),
        // eframe reads and writes the window geometry on its own, so automated runs get a separate, never written store
        // instead of loading or overwriting the user's window.
        persist_window: !args.default_prefs,
        persistence_path: args.default_prefs.then(|| std::env::temp_dir().join("godottrench-default-prefs")),
        ..Default::default()
    };
    eframe::run_native("GodotTrench", options, Box::new(move |cc| Ok(Box::new(app::App::new(cc, args)))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cli() {
        let a = CliArgs::parse(["level.gtm", "--mcp", "--mcp-http=9000", "--project", "C:/game"].map(String::from));
        assert_eq!(a.map, Some(PathBuf::from("level.gtm")));
        assert!(a.mcp_stdio);
        assert_eq!(a.mcp_http, Some(9000));
        assert_eq!(a.project, Some(PathBuf::from("C:/game")));
        assert_eq!(CliArgs::parse(["--mcp-http".to_string()]).mcp_http, Some(DEFAULT_MCP_PORT));
        assert_eq!(CliArgs::parse(["--dump", "a.gtm"].map(String::from)).convert, Some(Convert::Dump("a.gtm".into())));
        let c = CliArgs::parse(["--to-json", "a.gtm", "a.json"].map(String::from));
        assert_eq!(c.convert, Some(Convert::ToJson("a.gtm".into(), "a.json".into())));
        assert_eq!(c.map, None);
        let c = CliArgs::parse(["--to-gtm", "a.json", "b.gtm"].map(String::from)).convert;
        assert_eq!(c, Some(Convert::ToGtm("a.json".into(), "b.gtm".into())));
    }

    #[test]
    fn converts_between_binary_and_json() {
        let dir = std::env::temp_dir().join(format!("gt_convert_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut map = gt_doc::Map::new();
        map.properties.insert("message".into(), "hello".into());
        gt_doc::format::save(&map, &dir.join("a.gtm")).unwrap();
        assert!(gt_doc::binary::is_binary(&std::fs::read(dir.join("a.gtm")).unwrap()));

        convert(&Convert::ToJson(dir.join("a.gtm"), dir.join("a.json"))).unwrap();
        assert_eq!(std::fs::read_to_string(dir.join("a.json")).unwrap(), gt_doc::format::to_string(&map));
        convert(&Convert::ToGtm(dir.join("a.json"), dir.join("b.gtm"))).unwrap();
        assert_eq!(std::fs::read(dir.join("b.gtm")).unwrap(), std::fs::read(dir.join("a.gtm")).unwrap());
        assert!(convert(&Convert::Dump(dir.join("missing.gtm"))).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
