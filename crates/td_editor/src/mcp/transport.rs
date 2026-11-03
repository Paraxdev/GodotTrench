use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

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

type Token = Mutex<Option<Arc<str>>>;

/// A running MCP HTTP server.
pub struct HttpServer {
    addr: SocketAddr,
    token: Arc<Token>,
}

impl HttpServer {
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// The bearer token every request has to send, `None` when requests need none.
    pub fn token(&self) -> Option<Arc<str>> {
        current(&self.token)
    }

    /// Takes effect from the next request on.
    pub fn set_token(&self, token: Option<String>) {
        *self.token.lock().unwrap_or_else(|e| e.into_inner()) = token.map(Into::into);
    }
}

fn current(token: &Token) -> Option<Arc<str>> {
    token.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// Streamable HTTP transport (single JSON responses) on 127.0.0.1. With a token, clients have to send it as a bearer
/// token.
pub fn spawn_http(port: u16, token: Option<String>, exec: Arc<dyn ToolExecutor>) -> std::io::Result<HttpServer> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    let server = HttpServer { addr: listener.local_addr()?, token: Arc::new(Mutex::new(token.map(Into::into))) };
    let token = server.token.clone();
    std::thread::Builder::new().name("mcp-http".into()).spawn(move || {
        for stream in listener.incoming().flatten() {
            let exec = exec.clone();
            let token = token.clone();
            std::thread::spawn(move || {
                let _ = serve_connection(stream, &token, exec.as_ref());
            });
        }
    })?;
    Ok(server)
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

    fn has_token(&self, token: &str) -> bool {
        let Some((scheme, sent)) = self.header("authorization").and_then(|v| v.split_once(' ')) else { return false };
        scheme.eq_ignore_ascii_case("bearer") && same_secret(sent.trim().as_bytes(), token.as_bytes())
    }
}

/// Looks at every byte whatever the first difference, so response times do not give the token away byte by byte.
fn same_secret(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0, |diff, (x, y)| diff | (x ^ y)) == 0
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

fn serve_connection(stream: TcpStream, token: &Token, exec: &dyn ToolExecutor) -> std::io::Result<()> {
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);
    let Some(req) = read_request(&mut reader)? else { return Ok(()) };

    // Browsers send Origin with every POST and every cross-origin fetch while MCP clients never do, so refusing it keeps
    // out web pages, even ones served from another port on this machine.
    if req.header("origin").is_some() {
        return respond(&mut writer, "403 Forbidden", &[], b"browser requests are refused");
    }

    if let Some(token) = current(token)
        && !req.has_token(&token)
    {
        let body = b"send the token from the editor's File > Preferences as Authorization: Bearer <token>";
        return respond(&mut writer, "401 Unauthorized", &[("WWW-Authenticate", "Bearer".into())], body);
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

    const TOKEN: &str = "test-token";

    fn raw(addr: SocketAddr, request: &str) -> String {
        let mut s = TcpStream::connect(addr).unwrap();
        s.write_all(request.as_bytes()).unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        out
    }

    fn send(addr: SocketAddr, headers: &str, body: &str) -> String {
        raw(addr, &format!("POST /mcp HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {TOKEN}\r\n{headers}Content-Length: {}\r\n\r\n{body}", body.len()))
    }

    fn post(addr: SocketAddr, body: &str) -> String {
        send(addr, "Content-Type: application/json\r\n", body)
    }

    fn serve(token: Option<&str>) -> HttpServer {
        spawn_http(0, token.map(String::from), Arc::new(Echo)).unwrap()
    }

    const PING: &str = r#"{"jsonrpc":"2.0","id":3,"method":"ping"}"#;

    /// The status line a ping gets with this Authorization header, or with none.
    fn ping(addr: SocketAddr, auth: Option<&str>) -> String {
        let auth = auth.map(|a| format!("Authorization: {a}\r\n")).unwrap_or_default();
        let r = raw(addr, &format!("POST /mcp HTTP/1.1\r\nContent-Type: application/json\r\n{auth}Content-Length: {}\r\n\r\n{PING}", PING.len()));
        r.lines().next().unwrap_or_default().to_string()
    }

    #[test]
    fn http_round_trip() {
        let server = serve(Some(TOKEN));
        let addr = server.addr();
        let init = post(addr, r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#);
        assert!(init.starts_with("HTTP/1.1 200"));
        assert!(init.contains("Mcp-Session-Id"));
        let call = post(addr, r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_state"}}"#);
        assert!(call.contains("\"called\":\"get_state\""));
        let note = post(addr, r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);
        assert!(note.starts_with("HTTP/1.1 202"));
    }

    #[test]
    fn with_a_token_every_request_needs_it() {
        let server = serve(Some(TOKEN));
        let addr = server.addr();
        assert_eq!(ping(addr, None), "HTTP/1.1 401 Unauthorized");
        for auth in ["Bearer", "Bearer wrong", "Bearer test-toke", "Bearer test-tokenx", "Basic dGVzdC10b2tlbg==", "test-token"] {
            assert_eq!(ping(addr, Some(auth)), "HTTP/1.1 401 Unauthorized", "{auth}");
        }

        assert_eq!(ping(addr, Some("bearer test-token")), "HTTP/1.1 200 OK");
        let refused = raw(addr, "DELETE /mcp HTTP/1.1\r\n\r\n");
        assert!(refused.starts_with("HTTP/1.1 401") && refused.contains("WWW-Authenticate: Bearer"), "{refused}");
        assert!(raw(addr, "GET /elsewhere HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 401"));
        assert!(raw(addr, &format!("GET /elsewhere HTTP/1.1\r\nAuthorization: Bearer {TOKEN}\r\n\r\n")).starts_with("HTTP/1.1 404"));
    }

    #[test]
    fn the_token_can_change_while_the_server_runs() {
        let server = serve(None);
        let addr = server.addr();
        assert_eq!(ping(addr, None), "HTTP/1.1 200 OK");
        server.set_token(Some(TOKEN.into()));
        assert_eq!(server.token().as_deref(), Some(TOKEN));
        assert_eq!(ping(addr, None), "HTTP/1.1 401 Unauthorized");
        assert_eq!(ping(addr, Some("Bearer test-token")), "HTTP/1.1 200 OK");
        server.set_token(None);
        assert_eq!(ping(addr, None), "HTTP/1.1 200 OK");
    }

    #[test]
    fn secrets_compare_whole() {
        assert!(same_secret(b"abc", b"abc"));
        assert!(!same_secret(b"abc", b"abd"));
        assert!(!same_secret(b"abc", b"ab"));
        assert!(!same_secret(b"", b"a"));
    }

    #[test]
    fn requests_from_browsers_are_refused() {
        for server in [serve(Some(TOKEN)), serve(None)] {
            for origin in ["http://localhost:3000", "http://127.0.0.1:7841", "http://[::1]", "null", "https://evil.example"] {
                let r = send(server.addr(), &format!("Content-Type: application/json\r\nOrigin: {origin}\r\n"), PING);
                assert!(r.starts_with("HTTP/1.1 403"), "{origin}: {r}");
            }
        }
    }

    #[test]
    fn post_bodies_must_be_json() {
        for server in [serve(Some(TOKEN)), serve(None)] {
            let addr = server.addr();
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
}
