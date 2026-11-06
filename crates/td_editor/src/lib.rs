pub mod addon_install;
pub mod addon_wizard;
pub mod app;
pub mod bake;
pub mod blend_tool;
pub mod brand;
pub mod camera;
pub mod code_refs;
pub mod code_scan;
pub mod commands;
pub mod content;
pub mod content_wizard;
pub mod dialogs;
pub mod entity_pack;
pub mod entity_wizards;
pub mod export3d;
pub mod extra_tools;
pub mod face_cull;
pub mod gizmos;
pub mod godot;
pub mod help;
pub mod hotspot_editor;
pub mod i18n;
pub mod icons;
pub mod legacy;
pub mod live_link;
pub mod live_sync;
pub mod logic_graph;
pub mod logic_sim;
pub mod materials;
pub mod mcp;
pub mod mesh_tool;
pub mod model_thumbs;
pub mod models;
pub mod overlays;
pub mod panels;
pub mod picking;
pub mod prefab_library;
pub mod prefab_panel;
pub mod prefabs;
pub mod random_fill;
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
pub mod trim;
pub mod trust;
pub mod uv_editor;
pub mod validate;
pub mod viewport;
pub mod volume_tool;
pub mod walkable;
pub mod welcome;
pub mod widgets;
pub mod zfight;

use std::path::PathBuf;

/// The editor's own version, compared against the Godot addon's `plugin.cfg` when a project loads. CI stamps beta builds
/// like their addon, see `TRENCHDOT_BUILD` in ci.yml, and a local build has the plain [`RELEASE`].
pub const VERSION: &str = match option_env!("TRENCHDOT_BUILD") {
    Some(build) => build,
    None => RELEASE,
};
/// The release this build carries the number of, which names its downloads.
pub const RELEASE: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug, Default)]
pub struct CliArgs {
    pub map: Option<PathBuf>,
    pub project: Option<PathBuf>,
    /// Serve MCP over stdin/stdout.
    pub mcp_stdio: bool,
    /// Serve MCP over streamable HTTP on 127.0.0.1.
    pub mcp_http: Option<u16>,
    /// A token MCP HTTP clients must send, whatever the preferences say.
    pub mcp_token: Option<String>,
    /// Start with default preferences and never write them back (automated tests).
    pub default_prefs: bool,
    /// Open the window outside every screen, without focus or a taskbar button (automated tests).
    pub offscreen: bool,
    /// Live link and hot reload ports for this run, over the ones in the preferences.
    pub live_link_port: Option<u16>,
    pub hot_reload_port: Option<u16>,
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
    /// Any map file to the binary `.tdm`, keeping its content exactly.
    ToTdm(PathBuf, PathBuf),
    /// A map's geometry and materials as a glTF binary or OBJ model, for Blender and other 3D tools.
    Export(PathBuf, PathBuf, export3d::Format),
}

pub const DEFAULT_MCP_PORT: u16 = 7841;

pub(crate) const APP_ID: &str = "TrenchDot";

pub const USAGE: &str = "\
Usage: trenchdot [options] [map]

Opens the editor, with the map when one is given.

Options:
  --project <dir>        Load the Godot project in <dir> or a folder above it, and
                         trust it for the session
  --mcp                  Serve MCP over stdin and stdout
  --mcp-http[=<port>]    Serve MCP over HTTP on 127.0.0.1, port 7841 by default
  --mcp-token=<token>    Require <token> from MCP HTTP clients, TRENCHDOT_MCP_TOKEN
                         sets it too. Without either, clients need a token only when
                         Preferences asks for one
  --default-prefs        Start with default preferences and never save them
  --offscreen            Open the window outside every screen, for tests and agents
  --live-link-port=<port>
                         Talk to the Godot editor on <port> for this run instead of
                         the port in Preferences
  --hot-reload-port=<port>
                         Tell a running game on <port> to reload saved maps for this
                         run instead of the port in Preferences
  --dump <map>           Print a map as JSON and exit
  --to-json <map> <out>  Write a map as JSON and exit
  --to-tdm <map> <out>   Write a map as a binary .tdm and exit
  --export-glb <map> <out.glb>
                         Write a map's geometry and materials as glTF binary and exit
  --export-obj <map> <out.obj>
                         Write a map's geometry and materials as OBJ and exit
  -h, --help             Print this help and exit

TRENCHDOT_DISABLE_MCP=1 turns MCP off, so --mcp and --mcp-http serve nothing.
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
                "--offscreen" => out.offscreen = true,
                "--dump" => out.convert = value("a map file").map(Convert::Dump),
                "--to-json" | "--to-tdm" | "--export-glb" | "--export-obj" => {
                    let what = "a map file and an output file";
                    let Some(from) = value(what) else { continue };
                    let Some(to) = value(what) else { continue };
                    out.convert = Some(match arg.as_str() {
                        "--to-json" => Convert::ToJson(from, to),
                        "--to-tdm" => Convert::ToTdm(from, to),
                        "--export-glb" => Convert::Export(from, to, export3d::Format::Glb),
                        _ => Convert::Export(from, to, export3d::Format::Obj),
                    });
                }
                a if a.starts_with("--mcp-http=") => match a["--mcp-http=".len()..].parse() {
                    Ok(port) => out.mcp_http = Some(port),
                    Err(_) => out.errors.push(format!("{a}: the port must be a number")),
                },
                "--mcp-token" => out.errors.push("--mcp-token needs a token, as in --mcp-token=<token>".into()),
                "--live-link-port" | "--hot-reload-port" => out.errors.push(format!("{arg} needs a port, as in {arg}=<port>")),
                a if a.starts_with("--live-link-port=") => out.live_link_port = godot_port(a, &mut out.errors),
                a if a.starts_with("--hot-reload-port=") => out.hot_reload_port = godot_port(a, &mut out.errors),
                // The error leaves the value out, it is meant to be a secret.
                a if a.starts_with("--mcp-token=") => match &a["--mcp-token=".len()..] {
                    t if mcp::token::valid(t) => out.mcp_token = Some(t.to_string()),
                    _ => out.errors.push("--mcp-token: the token must be printable ASCII without spaces".into()),
                },
                a if a.starts_with('-') => out.errors.push(format!("unknown option {a}")),
                a => out.map = Some(PathBuf::from(a)),
            }
        }

        out
    }
}

/// The port after the `=` of `arg`, in the range Preferences takes.
fn godot_port(arg: &str, errors: &mut Vec<String>) -> Option<u16> {
    let port = arg.split_once('=').and_then(|(_, p)| p.parse().ok()).filter(|p| *p >= 1024);
    if port.is_none() {
        errors.push(format!("{arg}: the port must be a number from 1024 to 65535"));
    }

    port
}

/// Runs a conversion, reporting what a damaged input lost on stderr.
pub fn convert(c: &Convert) -> Result<(), String> {
    let read = |path: &std::path::Path| std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()));
    let report = |path: &std::path::Path, problems: &[String]| problems.iter().for_each(|p| eprintln!("{}: {p}", path.display()));
    match c {
        Convert::Dump(from) | Convert::ToJson(from, _) => {
            let (value, problems) = td_doc::format::file_json_value(&read(from)?).map_err(|e| format!("{}: {e}", from.display()))?;
            report(from, &problems);
            let write = |out: &mut dyn std::io::Write| {
                let mut out = std::io::BufWriter::new(out);
                td_doc::format::write_json(&mut out, &value).and_then(|()| std::io::Write::flush(&mut out))
            };
            match c {
                Convert::ToJson(_, to) => std::fs::File::create(to).and_then(|mut f| write(&mut f)).map_err(|e| format!("{}: {e}", to.display())),
                _ => write(&mut std::io::stdout().lock()).map_err(|e| e.to_string()),
            }
        }
        Convert::ToTdm(from, to) => {
            let (bytes, problems) = td_doc::format::file_to_binary(&read(from)?).map_err(|e| format!("{}: {e}", from.display()))?;
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
    legacy::migrate_settings(args.default_prefs);
    let mut viewport =
        egui::ViewportBuilder::default().with_title("TrenchDot").with_icon(brand::icon()).with_inner_size([1600.0, 960.0]).with_min_inner_size([800.0, 500.0]);
    if args.offscreen {
        // Moved away rather than hidden or minimized, so it keeps painting and MCP screenshots still work. Not -32000,
        // which Windows uses for minimized windows.
        viewport = viewport.with_position([-20000.0, -20000.0]).with_active(false).with_taskbar(false);
    }

    let options = eframe::NativeOptions {
        viewport,
        // eframe reads and writes the window geometry on its own, so automated runs get a separate, never written store
        // instead of loading or overwriting the user's window.
        persist_window: !args.default_prefs,
        persistence_path: args.default_prefs.then(default_prefs_store),
        ..Default::default()
    };
    eframe::run_native(APP_ID, options, Box::new(move |cc| Ok(Box::new(app::App::new(cc, args)))))
}

/// What a `--default-prefs` run keeps instead of the user's own settings.
fn default_prefs_store() -> PathBuf {
    std::env::temp_dir().join("trenchdot-default-prefs")
}

/// The file the MCP HTTP token is kept in, next to the settings the run uses.
pub fn mcp_token_file(default_prefs: bool) -> Option<PathBuf> {
    if default_prefs { Some(default_prefs_store().with_extension("mcp-token")) } else { eframe::storage_dir(APP_ID).map(|dir| dir.join("mcp-token")) }
}

/// The folder of the [`mcp::Saved`] settings. A `--default-prefs` run keeps them for the run only.
pub fn mcp_saved_dir(default_prefs: bool) -> Option<PathBuf> {
    if default_prefs { None } else { eframe::storage_dir(APP_ID) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cli() {
        let a = CliArgs::parse(["level.tdm", "--mcp", "--mcp-http=9000", "--mcp-token=s3cret", "--project", "C:/game"].map(String::from));
        assert_eq!(a.map, Some(PathBuf::from("level.tdm")));
        assert!(a.mcp_stdio);
        assert_eq!(a.mcp_http, Some(9000));
        assert_eq!(a.mcp_token.as_deref(), Some("s3cret"));
        assert_eq!(a.project, Some(PathBuf::from("C:/game")));
        assert_eq!((a.live_link_port, a.hot_reload_port), (None, None));
        let ports = CliArgs::parse(["--live-link-port=45001", "--hot-reload-port=45002"].map(String::from));
        assert_eq!((ports.live_link_port, ports.hot_reload_port), (Some(45001), Some(45002)));
        assert_eq!(CliArgs::parse(["--mcp-http".to_string()]).mcp_http, Some(DEFAULT_MCP_PORT));
        assert_eq!(CliArgs::parse(["--dump", "a.tdm"].map(String::from)).convert, Some(Convert::Dump("a.tdm".into())));
        let c = CliArgs::parse(["--to-json", "a.tdm", "a.json"].map(String::from));
        assert_eq!(c.convert, Some(Convert::ToJson("a.tdm".into(), "a.json".into())));
        assert_eq!(c.map, None);
        let c = CliArgs::parse(["--to-tdm", "a.json", "b.tdm"].map(String::from)).convert;
        assert_eq!(c, Some(Convert::ToTdm("a.json".into(), "b.tdm".into())));
        let c = CliArgs::parse(["--export-glb", "a.tdm", "a.glb"].map(String::from)).convert;
        assert_eq!(c, Some(Convert::Export("a.tdm".into(), "a.glb".into(), export3d::Format::Glb)));
        let c = CliArgs::parse(["--export-obj", "a.tdm", "a.obj"].map(String::from)).convert;
        assert_eq!(c, Some(Convert::Export("a.tdm".into(), "a.obj".into(), export3d::Format::Obj)));
        assert!(a.errors.is_empty() && !a.help);
    }

    #[test]
    fn help_and_bad_options_never_reach_the_editor() {
        assert!(CliArgs::parse(["--help".to_string()]).help);
        assert!(CliArgs::parse(["--offscreen".to_string()]).offscreen);
        assert!(!CliArgs::parse(["--default-prefs".to_string()]).offscreen);
        assert!(CliArgs::parse(["-h".to_string()]).help);
        let errors = |args: &[&str]| CliArgs::parse(args.iter().map(|a| a.to_string())).errors;
        assert_eq!(errors(&["--halp"]), ["unknown option --halp"]);
        assert_eq!(errors(&["-v", "level.tdm"]), ["unknown option -v"]);
        assert_eq!(errors(&["--project"]), ["--project needs a folder"]);
        assert_eq!(errors(&["--dump", "--mcp"]), ["--dump needs a map file"]);
        assert_eq!(errors(&["--to-json", "a.tdm"]), ["--to-json needs a map file and an output file"]);
        assert_eq!(errors(&["--export-glb", "a.tdm"]), ["--export-glb needs a map file and an output file"]);
        assert_eq!(errors(&["--mcp-http=abc"]), ["--mcp-http=abc: the port must be a number"]);
        assert_eq!(errors(&["--live-link-port"]), ["--live-link-port needs a port, as in --live-link-port=<port>"]);
        assert_eq!(errors(&["--hot-reload-port=80"]), ["--hot-reload-port=80: the port must be a number from 1024 to 65535"]);
        assert_eq!(errors(&["--live-link-port=x"]), ["--live-link-port=x: the port must be a number from 1024 to 65535"]);
        assert_eq!(errors(&["--mcp-token"]), ["--mcp-token needs a token, as in --mcp-token=<token>"]);
        assert_eq!(errors(&["--mcp-token="]), ["--mcp-token: the token must be printable ASCII without spaces"]);
        assert_eq!(errors(&["--mcp-token=two words"]), ["--mcp-token: the token must be printable ASCII without spaces"]);
    }

    #[test]
    fn converts_between_binary_and_json() {
        let dir = std::env::temp_dir().join(format!("td_convert_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut map = td_doc::Map::new();
        map.properties.insert("message".into(), "hello".into());
        td_doc::format::save(&map, &dir.join("a.tdm")).unwrap();
        assert!(td_doc::binary::is_binary(&std::fs::read(dir.join("a.tdm")).unwrap()));

        convert(&Convert::ToJson(dir.join("a.tdm"), dir.join("a.json"))).unwrap();
        assert_eq!(std::fs::read_to_string(dir.join("a.json")).unwrap(), td_doc::format::to_string(&map));
        convert(&Convert::ToTdm(dir.join("a.json"), dir.join("b.tdm"))).unwrap();
        assert_eq!(std::fs::read(dir.join("b.tdm")).unwrap(), std::fs::read(dir.join("a.tdm")).unwrap());
        assert!(convert(&Convert::Dump(dir.join("missing.tdm"))).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
