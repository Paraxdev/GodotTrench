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

    fn is_json(&self) -> bool {
        self.header("content-type").and_then(|v| v.split(';').next()).is_some_and(|t| t.trim().eq_ignore_ascii_case("application/json"))
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

fn serve_connection(stream: TcpStream, exec: &dyn ToolExecutor) -> std::io::Result<()> {
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);
    let Some(req) = read_request(&mut reader)? else { return Ok(()) };

    // Browsers send Origin with every POST and every cross-origin fetch while MCP clients never do, so refusing it keeps
    // out web pages, even ones served from another port on this machine.
    if req.header("origin").is_some() {
        return respond(&mut writer, "403 Forbidden", &[], b"browser requests are refused");
    }

    if req.path != "/mcp" && req.path != "/" {
        return respond(&mut writer, "404 Not Found", &[], b"not found");
    }

    match req.method.as_str() {
        "POST" => {
            // A page can post text/plain or form data without a CORS preflight, a JSON body always needs one.
            if !req.is_json() {
                return respond(&mut writer, "415 Unsupported Media Type", &[], b"the body must be sent as application/json");
            }

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
                        let session = format!("td-{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0));
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

    fn send(addr: SocketAddr, headers: &str, body: &str) -> String {
        let mut s = TcpStream::connect(addr).unwrap();
        write!(s, "POST /mcp HTTP/1.1\r\nHost: localhost\r\n{headers}Content-Length: {}\r\n\r\n{body}", body.len()).unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        out
    }

    fn post(addr: SocketAddr, body: &str) -> String {
        send(addr, "Content-Type: application/json\r\n", body)
    }

    const PING: &str = r#"{"jsonrpc":"2.0","id":3,"method":"ping"}"#;

    #[test]
    fn http_round_trip() {
        let addr = spawn_http(0, Arc::new(Echo)).unwrap();
        let init = post(addr, r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#);
        assert!(init.starts_with("HTTP/1.1 200"));
        assert!(init.contains("Mcp-Session-Id"));
        let call = post(addr, r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_state"}}"#);
        assert!(call.contains("\"called\":\"get_state\""));
        let note = post(addr, r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);
        assert!(note.starts_with("HTTP/1.1 202"));
    }

    #[test]
    fn requests_from_browsers_are_refused() {
        let addr = spawn_http(0, Arc::new(Echo)).unwrap();
        for origin in ["http://localhost:3000", "http://127.0.0.1:7841", "http://[::1]", "null", "https://evil.example"] {
            let r = send(addr, &format!("Content-Type: application/json\r\nOrigin: {origin}\r\n"), PING);
            assert!(r.starts_with("HTTP/1.1 403"), "{origin}: {r}");
        }
    }

    #[test]
    fn post_bodies_must_be_json() {
        let addr = spawn_http(0, Arc::new(Echo)).unwrap();
        for content_type in ["text/plain", "application/x-www-form-urlencoded", "multipart/form-data; boundary=x", "text/plain; x=application/json"] {
            let r = send(addr, &format!("Content-Type: {content_type}\r\n"), PING);
            assert!(r.starts_with("HTTP/1.1 415"), "{content_type}: {r}");
        }

        assert!(send(addr, "", PING).starts_with("HTTP/1.1 415"));
        for content_type in ["application/json; charset=utf-8", "Application/JSON"] {
            let r = send(addr, &format!("Content-Type: {content_type}\r\n"), PING);
            assert!(r.starts_with("HTTP/1.1 200"), "{content_type}: {r}");
        }
    }
}
