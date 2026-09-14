pub mod app;
pub mod blend_tool;
pub mod camera;
pub mod code_refs;
pub mod commands;
pub mod dialogs;
pub mod entity_wizards;
pub mod extra_tools;
pub mod face_cull;
pub mod gizmos;
pub mod godot;
pub mod guide;
pub mod hotspot_editor;
pub mod icons;
pub mod live_link;
pub mod live_sync;
pub mod materials;
pub mod mcp;
pub mod mesh_tool;
pub mod models;
pub mod panels;
pub mod picking;
pub mod prefabs;
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
                a if a.starts_with("--mcp-http=") => out.mcp_http = a["--mcp-http=".len()..].parse().ok().or(Some(DEFAULT_MCP_PORT)),
                a if !a.starts_with("--") => out.map = Some(PathBuf::from(a)),
                _ => {}
            }
        }
        out
    }
}

pub fn run(args: CliArgs) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_title("GodotTrench").with_inner_size([1600.0, 960.0]).with_min_inner_size([800.0, 500.0]),
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
    }
}
