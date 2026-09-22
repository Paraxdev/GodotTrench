use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;

use serde_json::Value;

use super::{ToolExecutor, protocol};

/// Newline delimited JSON-RPC on stdin/stdout. Stdout must carry nothing else.
pub fn spawn_stdio(exec: Arc<dyn ToolExecutor>) {
    std::thread::Builder::new()
        .name("mcp-stdio".into())
        .spawn(move || {
            let stdin = std::io::stdin();
            for line in stdin.lock().lines() {
                let Ok(line) = line else { break };
                if line.trim().is_empty() {
                    continue;
                }

                let response = match serde_json::from_str::<Value>(&line) {
                    Ok(msg) => protocol::handle(msg, exec.as_ref()),
                    Err(e) => Some(serde_json::json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": e.to_string() } })),
                };
                if let Some(r) = response {
                    let mut out = std::io::stdout().lock();
                    let _ = writeln!(out, "{}", serde_json::to_string(&r).unwrap_or_default());
                    let _ = out.flush();
                }
            }
        })
        .expect("spawn mcp stdio thread");
}

/// Streamable HTTP transport (single JSON responses) on 127.0.0.1.
pub fn spawn_http(port: u16, exec: Arc<dyn ToolExecutor>) -> std::io::Result<SocketAddr> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    let addr = listener.local_addr()?;
    std::thread::Builder::new().name("mcp-http".into()).spawn(move || {
        for stream in listener.incoming().flatten() {
            let exec = exec.clone();
            std::thread::spawn(move || {
                let _ = serve_connection(stream, exec.as_ref());
            });
        }
    })?;
    Ok(addr)
}

struct Request {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Request {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }
}

fn read_request(reader: &mut BufReader<TcpStream>) -> std::io::Result<Option<Request>> {
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
        return Ok(None);
    }

    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let path = parts.next().unwrap_or_default().to_string();
    let mut headers = Vec::new();
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h)? == 0 {
            break;
        }

        let h = h.trim_end();
        if h.is_empty() {
            break;
        }

        if let Some((k, v)) = h.split_once(':') {
            headers.push((k.trim().to_string(), v.trim().to_string()));
        }
    }

    let len: usize = headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("content-length")).and_then(|(_, v)| v.parse().ok()).unwrap_or(0);
    let mut body = vec![0u8; len.min(64 * 1024 * 1024)];
    reader.read_exact(&mut body)?;
    Ok(Some(Request { method, path, headers, body }))
}

fn respond(stream: &mut TcpStream, status: &str, extra_headers: &[(&str, String)], body: &[u8]) -> std::io::Result<()> {
    let mut head = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n", body.len());
    for (k, v) in extra_headers {
        head.push_str(&format!("{k}: {v}\r\n"));
    }

    head.push_str("\r\n");
    stream.write_all(head.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

fn is_local_origin(origin: &str) -> bool {
    let authority = origin.split("://").nth(1).unwrap_or(origin);
    let authority = authority.split('/').next().unwrap_or_default();
    // IPv6 hosts are bracketed and contain colons themselves, so the port can only be split off after the bracket.
    let host = match authority.strip_prefix('[') {
        Some(rest) => rest.split(']').next().unwrap_or_default(),
        None => authority.split(':').next().unwrap_or_default(),
    };
    matches!(host.to_ascii_lowercase().as_str(), "localhost" | "127.0.0.1" | "::1")
}

fn serve_connection(stream: TcpStream, exec: &dyn ToolExecutor) -> std::io::Result<()> {
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);
    let Some(req) = read_request(&mut reader)? else { return Ok(()) };

    // Browsers send Origin, rejecting foreign ones blocks DNS rebinding attacks against the local server.
    if let Some(origin) = req.header("origin")
        && !is_local_origin(origin)
    {
        return respond(&mut writer, "403 Forbidden", &[], b"forbidden origin");
    }

    if req.path != "/mcp" && req.path != "/" {
        return respond(&mut writer, "404 Not Found", &[], b"not found");
    }

    match req.method.as_str() {
        "POST" => {
            let msg: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(e) => {
                    let body = serde_json::json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": e.to_string() } });
                    return respond(&mut writer, "400 Bad Request", &[("Content-Type", "application/json".into())], body.to_string().as_bytes());
                }
            };
            let is_initialize = msg.get("method").and_then(|m| m.as_str()) == Some("initialize");
            match protocol::handle(msg, exec) {
                Some(response) => {
                    let mut headers = vec![("Content-Type", "application/json".to_string())];
                    if is_initialize {
                        let session = format!("gt-{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0));
                        headers.push(("Mcp-Session-Id", session));
                    }

                    respond(&mut writer, "200 OK", &headers, response.to_string().as_bytes())
                }
                None => respond(&mut writer, "202 Accepted", &[], b""),
            }
        }
        "DELETE" => respond(&mut writer, "200 OK", &[], b""),
        _ => respond(&mut writer, "405 Method Not Allowed", &[("Allow", "POST, DELETE".into())], b""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::ToolResult;

    struct Echo;
    impl ToolExecutor for Echo {
        fn call(&self, name: &str, _args: Value) -> ToolResult {
            ToolResult::Json(serde_json::json!({ "called": name }))
        }
    }

    fn post(addr: SocketAddr, body: &str, origin: Option<&str>) -> String {
        let mut s = TcpStream::connect(addr).unwrap();
        let origin = origin.map(|o| format!("Origin: {o}\r\n")).unwrap_or_default();
        write!(s, "POST /mcp HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\n{origin}Content-Length: {}\r\n\r\n{body}", body.len()).unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        out
    }

    #[test]
    fn http_round_trip() {
        let addr = spawn_http(0, Arc::new(Echo)).unwrap();
        let init = post(addr, r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#, None);
        assert!(init.starts_with("HTTP/1.1 200"));
        assert!(init.contains("Mcp-Session-Id"));
        let call = post(addr, r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_state"}}"#, Some("http://localhost:3000"));
        assert!(call.contains("\"called\":\"get_state\""));
        let note = post(addr, r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#, None);
        assert!(note.starts_with("HTTP/1.1 202"));
        let evil = post(addr, r#"{"jsonrpc":"2.0","id":3,"method":"ping"}"#, Some("https://evil.example"));
        assert!(evil.starts_with("HTTP/1.1 403"));
    }

    #[test]
    fn local_origins_include_ipv6() {
        assert!(is_local_origin("http://[::1]:7841"));
        assert!(is_local_origin("http://[::1]"));
        assert!(is_local_origin("http://localhost:3000/app"));
        assert!(is_local_origin("http://127.0.0.1"));
        assert!(!is_local_origin("http://[::2]:7841"));
        assert!(!is_local_origin("http://localhost.evil.example"));
        assert!(!is_local_origin("https://evil.example"));
    }
}
