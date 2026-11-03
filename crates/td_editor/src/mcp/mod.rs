//! Model Context Protocol server, so agents and test drivers can inspect and drive a running editor.
//! Transports run on background threads, tool calls are executed on the UI thread (see `tools.rs`).

pub mod content_tools;
pub mod geometry_tools;
pub mod protocol;
pub mod review_tools;
pub mod script;
pub mod token;
pub mod tools;
pub mod transport;

use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::Value;

#[derive(Debug, Clone)]
pub enum ToolResult {
    Json(Value),
    Image { png: Vec<u8>, note: String },
    Error(String),
}

/// What the UI thread does with a tool call: answer it now, or keep the reply sender and answer in a later frame.
pub enum Reply {
    Now(ToolResult),
    Deferred,
}

pub trait ToolExecutor: Send + Sync {
    fn call(&self, name: &str, args: Value) -> ToolResult;
}

pub struct ToolCall {
    pub name: String,
    pub args: Value,
    pub reply: Sender<ToolResult>,
}

/// How long a transport waits for the UI thread.
pub fn reply_timeout(tool: &str) -> Option<Duration> {
    match tool {
        // Scripts replay whole maps, project_content downloads and bakes of big maps take minutes, none has a sensible limit.
        "run_script" | "project_content" | "bake_lighting" => None,
        _ => Some(Duration::from_secs(120)),
    }
}

fn wait_for_reply(rx: &Receiver<ToolResult>, timeout: Option<Duration>) -> ToolResult {
    let dropped = || ToolResult::Error("the editor dropped the request without answering it".into());
    match timeout {
        None => rx.recv().unwrap_or_else(|_| dropped()),
        Some(t) => match rx.recv_timeout(t) {
            Ok(r) => r,
            Err(RecvTimeoutError::Disconnected) => dropped(),
            Err(RecvTimeoutError::Timeout) => ToolResult::Error(format!("timed out after {} s waiting for the editor", t.as_secs())),
        },
    }
}

/// Thread-safe handle used by transports to reach the UI thread.
#[derive(Clone)]
pub struct Bridge {
    tx: Sender<ToolCall>,
    ctx: Arc<Mutex<Option<egui::Context>>>,
}

impl ToolExecutor for Bridge {
    fn call(&self, name: &str, args: Value) -> ToolResult {
        let (reply, rx) = mpsc::channel();
        if self.tx.send(ToolCall { name: name.to_string(), args, reply }).is_err() {
            return ToolResult::Error("editor is shutting down".into());
        }

        if let Some(ctx) = self.ctx.lock().ok().and_then(|c| c.clone()) {
            ctx.request_repaint();
        }

        wait_for_reply(&rx, reply_timeout(name))
    }
}

pub struct McpHost {
    pub rx: Receiver<ToolCall>,
    pub bridge: Bridge,
}

impl McpHost {
    pub fn new(ctx: egui::Context) -> Self {
        let (tx, rx) = mpsc::channel();
        Self { rx, bridge: Bridge { tx, ctx: Arc::new(Mutex::new(Some(ctx))) } }
    }
}

/// MCP settings kept in files of their own next to the preferences, each there while its setting is on and changed the
/// moment the setting is. An editor writes back all the preferences it loaded when it closes, which would undo a change
/// made in another editor since. On is the careful side of every setting kept here, so a file that cannot be looked at
/// counts as on.
pub struct Saved(Option<std::path::PathBuf>);

impl Saved {
    /// Without a folder the settings last for this run only.
    pub fn new(dir: Option<std::path::PathBuf>) -> Self {
        Self(dir)
    }

    pub fn get(&self, name: &str) -> bool {
        self.0.as_ref().is_some_and(|dir| !matches!(dir.join(name).try_exists(), Ok(false)))
    }

    pub fn set(&self, name: &str, on: bool) -> std::io::Result<()> {
        let Some(dir) = &self.0 else { return Ok(()) };
        if on {
            std::fs::create_dir_all(dir)?;
            return std::fs::write(dir.join(name), b"");
        }

        match std::fs::remove_file(dir.join(name)) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropped_requests_are_not_reported_as_timeouts() {
        let (tx, rx) = mpsc::channel::<ToolResult>();
        drop(tx);
        let ToolResult::Error(e) = wait_for_reply(&rx, Some(Duration::from_millis(10))) else { panic!() };
        assert!(e.contains("dropped"));

        let (_tx, rx) = mpsc::channel::<ToolResult>();
        let ToolResult::Error(e) = wait_for_reply(&rx, Some(Duration::from_millis(10))) else { panic!() };
        assert!(e.contains("timed out"));
        assert_eq!(reply_timeout("run_script"), None);
    }

    #[test]
    fn saved_settings_reach_every_editor_at_once() {
        let dir = std::env::temp_dir().join(format!("td_mcp_saved_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let (one, other) = (Saved::new(Some(dir.join("data"))), Saved::new(Some(dir.join("data"))));
        assert!(!one.get(token::REQUIRE_FILE));
        one.set(token::REQUIRE_FILE, true).unwrap();
        assert!(other.get(token::REQUIRE_FILE), "another editor sees it without a restart of the first");
        other.set(token::REQUIRE_FILE, false).unwrap();
        other.set(token::REQUIRE_FILE, false).unwrap();
        assert!(!one.get(token::REQUIRE_FILE));

        let run_only = Saved::new(None);
        run_only.set(token::REQUIRE_FILE, true).unwrap();
        assert!(!run_only.get(token::REQUIRE_FILE));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_preferences_carry_no_saved_setting() {
        let prefs = crate::state::Prefs { mcp_require_token: true, ..Default::default() };
        let written = serde_json::to_value(&prefs).unwrap();
        assert!(written.get("mcp_require_token").is_none(), "an editor closing later would write back its own");
        let read: crate::state::Prefs = serde_json::from_value(serde_json::json!({ "mcp_require_token": true })).unwrap();
        assert!(!read.mcp_require_token);
    }
}
