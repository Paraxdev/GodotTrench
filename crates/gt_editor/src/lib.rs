pub mod addon_install;
pub mod addon_wizard;
pub mod app;
pub mod blend_tool;
pub mod brand;
pub mod camera;
pub mod code_refs;
pub mod commands;
pub mod content;
pub mod content_wizard;
pub mod dialogs;
pub mod entity_wizards;
pub mod export3d;
pub mod extra_tools;
pub mod face_cull;
pub mod gizmos;
pub mod godot;
pub mod help;
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
pub mod texture_convert;
pub mod texture_ops;
pub mod texture_tool;
pub mod theme;
pub mod tools;
pub mod toolset;
pub mod transform_gizmo;
pub mod uv_editor;
pub mod validate;
pub mod viewport;
pub mod volume_tool;
pub mod walkable;
pub mod welcome;
pub mod widgets;
pub mod zfight;

use std::path::PathBuf;

/// The editor's own version, compared against the Godot addon's `plugin.cfg` when a project loads.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

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
    /// Print [`USAGE`] and exit.
    pub help: bool,
    /// Unknown options and options missing their value. The editor refuses to start with any, so a typo never opens
    /// a window.
    pub errors: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Convert {
    /// Print any map file as readable JSON, for diffs and quick looks.
    Dump(PathBuf),
    ToJson(PathBuf, PathBuf),
    /// Any map file to the binary `.gtm`, keeping its content exactly.
    ToGtm(PathBuf, PathBuf),
    /// A map's geometry and materials as a glTF binary or OBJ model, for Blender and other 3D tools.
    Export(PathBuf, PathBuf, export3d::Format),
}

pub const DEFAULT_MCP_PORT: u16 = 7841;

pub const USAGE: &str = "\
Usage: godottrench [options] [map]

Opens the editor, with the map when one is given.

Options:
  --project <dir>        Load the Godot project in <dir> or a folder above it
  --mcp                  Serve MCP over stdin and stdout
  --mcp-http[=<port>]    Serve MCP over HTTP on 127.0.0.1, port 7841 by default
  --default-prefs        Start with default preferences and never save them
  --dump <map>           Print a map as JSON and exit
  --to-json <map> <out>  Write a map as JSON and exit
  --to-gtm <map> <out>   Write a map as a binary .gtm and exit
  --export-glb <map> <out.glb>
                         Write a map's geometry and materials as glTF binary and exit
  --export-obj <map> <out.obj>
                         Write a map's geometry and materials as OBJ and exit
  -h, --help             Print this help and exit
";

impl CliArgs {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Self {
        let mut out = CliArgs::default();
        let mut it = args.into_iter().peekable();
        while let Some(arg) = it.next() {
            let mut value = |what: &str| match it.next_if(|v| !v.starts_with("--")) {
                Some(v) => Some(PathBuf::from(v)),
                None => {
                    out.errors.push(format!("{arg} needs {what}"));
                    None
                }
            };
            match arg.as_str() {
                "-h" | "--help" | "/?" => out.help = true,
                "--mcp" => out.mcp_stdio = true,
                "--project" => out.project = value("a folder"),
                "--mcp-http" => out.mcp_http = Some(DEFAULT_MCP_PORT),
                "--default-prefs" => out.default_prefs = true,
                "--dump" => out.convert = value("a map file").map(Convert::Dump),
                "--to-json" | "--to-gtm" | "--export-glb" | "--export-obj" => {
                    let what = "a map file and an output file";
                    let Some(from) = value(what) else { continue };
                    let Some(to) = value(what) else { continue };
                    out.convert = Some(match arg.as_str() {
                        "--to-json" => Convert::ToJson(from, to),
                        "--to-gtm" => Convert::ToGtm(from, to),
                        "--export-glb" => Convert::Export(from, to, export3d::Format::Glb),
                        _ => Convert::Export(from, to, export3d::Format::Obj),
                    });
                }
                a if a.starts_with("--mcp-http=") => match a["--mcp-http=".len()..].parse() {
                    Ok(port) => out.mcp_http = Some(port),
                    Err(_) => out.errors.push(format!("{a}: the port must be a number")),
                },
                a if a.starts_with('-') => out.errors.push(format!("unknown option {a}")),
                a => out.map = Some(PathBuf::from(a)),
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
        Convert::Export(from, to, format) => {
            let done = export3d::export_file(from, to, *format)?;
            eprintln!("{}", done.summary(to));
            Ok(())
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
        let c = CliArgs::parse(["--export-glb", "a.gtm", "a.glb"].map(String::from)).convert;
        assert_eq!(c, Some(Convert::Export("a.gtm".into(), "a.glb".into(), export3d::Format::Glb)));
        let c = CliArgs::parse(["--export-obj", "a.gtm", "a.obj"].map(String::from)).convert;
        assert_eq!(c, Some(Convert::Export("a.gtm".into(), "a.obj".into(), export3d::Format::Obj)));
        assert!(a.errors.is_empty() && !a.help);
    }

    #[test]
    fn help_and_bad_options_never_reach_the_editor() {
        assert!(CliArgs::parse(["--help".to_string()]).help);
        assert!(CliArgs::parse(["-h".to_string()]).help);
        let errors = |args: &[&str]| CliArgs::parse(args.iter().map(|a| a.to_string())).errors;
        assert_eq!(errors(&["--halp"]), ["unknown option --halp"]);
        assert_eq!(errors(&["-v", "level.gtm"]), ["unknown option -v"]);
        assert_eq!(errors(&["--project"]), ["--project needs a folder"]);
        assert_eq!(errors(&["--dump", "--mcp"]), ["--dump needs a map file"]);
        assert_eq!(errors(&["--to-json", "a.gtm"]), ["--to-json needs a map file and an output file"]);
        assert_eq!(errors(&["--export-glb", "a.gtm"]), ["--export-glb needs a map file and an output file"]);
        assert_eq!(errors(&["--mcp-http=abc"]), ["--mcp-http=abc: the port must be a number"]);
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
