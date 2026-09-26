//! Model Context Protocol server, so agents and test drivers can inspect and drive a running editor.
//! Transports run on background threads, tool calls are executed on the UI thread (see `tools.rs`).

pub mod content_tools;
pub mod geometry_tools;
pub mod protocol;
pub mod review_tools;
pub mod script;
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
        // Scripts replay whole maps and project_content downloads, neither has a sensible limit.
        "run_script" | "project_content" => None,
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
}
