//! Tells a running Godot editor (GodotTrench FuncGodot addon) that a map was saved so it rebuilds it.
//! Protocol: one JSON object per line over TCP on 127.0.0.1, answered by one JSON line.

use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use serde_json::{Value, json};

pub const DEFAULT_PORT: u16 = 7842;
/// Port of the GodotTrenchHotReload autoload inside a running game.
pub const DEFAULT_GAME_PORT: u16 = 7843;

pub fn send(port: u16, message: &Value) -> Result<Value, String> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream =
        TcpStream::connect_timeout(&addr, Duration::from_millis(400)).map_err(|_| "Godot is not running with the GodotTrench addon".to_string())?;
    stream.set_read_timeout(Some(Duration::from_secs(30))).map_err(|e| e.to_string())?;
    writeln!(stream, "{message}").map_err(|e| e.to_string())?;
    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line).map_err(|e| e.to_string())?;
    serde_json::from_str(&line).map_err(|e| e.to_string())
}

/// Sends `map_saved` on a background thread. The receiver yields a status message.
pub fn notify_saved(port: u16, path: &Path) -> Receiver<String> {
    notify_saved_all(port, None, path)
}

/// Like `notify_saved`, also telling a running game on `game_port` to hot reload the map.
pub fn notify_saved_all(port: u16, game_port: Option<u16>, path: &Path) -> Receiver<String> {
    let (tx, rx) = mpsc::channel();
    let path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let message = json!({ "event": "map_saved", "path": path.to_string_lossy().replace('\\', "/") });
    std::thread::spawn(move || {
        let mut status = match send(port, &message) {
            Ok(reply) => match reply["rebuilt"].as_u64() {
                Some(0) => "Live link: Godot has no open scene using this map".to_string(),
                Some(n) => format!("Live link: Godot rebuilt {n} map node(s)"),
                None => format!("Live link: {}", reply["error"].as_str().unwrap_or("unexpected reply")),
            },
            Err(e) => format!("Live link: {e}"),
        };
        if let Some(game) = game_port
            && let Ok(reply) = send(game, &message)
            && let Some(n) = reply["rebuilt"].as_u64()
        {
            status.push_str(&format!(", running game reloaded {n} map(s)"));
        }
        let _ = tx.send(status);
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn round_trip_with_fake_godot() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let msg: Value = serde_json::from_str(&line).unwrap();
            assert_eq!(msg["event"], "map_saved");
            writeln!(&stream, "{}", json!({ "ok": true, "rebuilt": 2 })).unwrap();
        });
        let rx = notify_saved(port, Path::new("maps/level.gtm"));
        let status = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(status.contains("rebuilt 2"), "{status}");
    }

    #[test]
    fn reports_missing_godot() {
        let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        assert!(send(port, &json!({ "event": "hello" })).is_err());
    }
}
