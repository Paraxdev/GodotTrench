//! Model Context Protocol server, so agents and test drivers can inspect and drive a running editor.
//! Transports run on background threads, tool calls are executed on the UI thread (see `tools.rs`).

pub mod content_tools;
pub mod geometry_tools;
pub mod protocol;
pub mod script;
pub mod tools;
pub mod transport;

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::Value;

#[derive(Debug, Clone)]
pub enum ToolResult {
    Json(Value),
    Image { png: Vec<u8>, note: String },
    Error(String),
}

pub trait ToolExecutor: Send + Sync {
    fn call(&self, name: &str, args: Value) -> ToolResult;
}

pub struct ToolCall {
    pub name: String,
    pub args: Value,
    pub reply: Sender<ToolResult>,
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
        rx.recv_timeout(Duration::from_secs(120)).unwrap_or_else(|_| ToolResult::Error("timed out waiting for the editor".into()))
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
