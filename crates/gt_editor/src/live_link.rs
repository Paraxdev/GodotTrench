//! Connection to a Godot editor running the GodotTrench addon: save notifications, builds, focus and live mode.
//! Protocol: one JSON object per line over TCP on 127.0.0.1, each answered by one JSON line echoing its `seq`.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::live_sync::{Frame, LiveSession};

pub const DEFAULT_PORT: u16 = 7842;
/// Port of the GodotTrenchHotReload autoload inside a running game.
pub const DEFAULT_GAME_PORT: u16 = 7843;

const HEARTBEAT: Duration = Duration::from_secs(1);
const RECONNECT: Duration = Duration::from_secs(3);
const CONNECT_TIMEOUT: Duration = Duration::from_millis(400);
const QUICK_REPLY: Duration = Duration::from_secs(15);
const BUILD_REPLY: Duration = Duration::from_secs(180);

/// Called with Godot's reply to a capture or walkable request, on the live link thread.
pub type Done = Box<dyn FnOnce(Result<Value, String>) + Send>;

pub fn send(port: u16, message: &Value) -> Result<Value, String> {
    send_timeout(port, message, Duration::from_secs(30))
}

/// One message on its own connection.
pub fn send_timeout(port: u16, message: &Value, timeout: Duration) -> Result<Value, String> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT).map_err(|_| "Godot is not running with the GodotTrench addon".to_string())?;
    stream.set_read_timeout(Some(timeout)).map_err(|e| e.to_string())?;
    writeln!(stream, "{message}").map_err(|e| e.to_string())?;
    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line).map_err(|e| e.to_string())?;
    serde_json::from_str(&line).map_err(|e| e.to_string())
}

/// Absolute path with forward slashes, as Godot writes paths.
pub fn godot_path(path: &Path) -> String {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf()).to_string_lossy().replace('\\', "/")
}

/// Key for comparing paths from both sides: forward slashes, no trailing slash, case folded on Windows.
pub fn path_key(path: &str) -> String {
    let mut key = path.replace('\\', "/");
    while key.len() > 1 && key.ends_with('/') {
        key.pop();
    }

    if cfg!(windows) { key.to_lowercase() } else { key }
}

fn saved_message(path: &str) -> Value {
    json!({ "event": "map_saved", "path": path })
}

fn rebuilt_status(reply: &Value) -> String {
    match reply["rebuilt"].as_u64() {
        Some(0) => "Live link: Godot has no open scene using this map".to_string(),
        Some(n) => format!("Live link: Godot rebuilt {n} map node(s)"),
        None => format!("Live link: {}", reply["error"].as_str().unwrap_or("unexpected reply")),
    }
}

fn game_status(game_port: Option<u16>, message: &Value) -> String {
    match game_port.map(|port| send(port, message)) {
        Some(Ok(reply)) if reply["rebuilt"].as_u64().is_some() => format!(", running game reloaded {} map(s)", reply["rebuilt"]),
        _ => String::new(),
    }
}

/// Sends `map_saved` on a background thread. The receiver yields a status message.
pub fn notify_saved(port: u16, path: &Path) -> Receiver<String> {
    notify_saved_all(port, None, path)
}

/// Like `notify_saved`, also telling a running game on `game_port` to hot reload the map.
pub fn notify_saved_all(port: u16, game_port: Option<u16>, path: &Path) -> Receiver<String> {
    let (tx, rx) = mpsc::channel();
    let message = saved_message(&godot_path(path));
    std::thread::spawn(move || {
        let status = match send(port, &message) {
            Ok(reply) => rebuilt_status(&reply),
            Err(e) => format!("Live link: {e}"),
        };
        let _ = tx.send(status + &game_status(game_port, &message));
    });
    rx
}

/// What the last heartbeat learned about the Godot editor.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LinkState {
    pub connected: bool,
    /// The addon answers but predates the status heartbeat, only save rebuilds work.
    pub outdated: bool,
    /// `path_key` of the open project folder.
    pub project: Option<String>,
    pub godot: String,
    pub pid: Option<u32>,
    /// `path_key` of each map used by a FuncGodotMap in the edited scene, with its build epoch.
    pub maps: BTreeMap<String, u64>,
    /// A long request (rebuild) is running.
    pub busy: bool,
    /// Goes up whenever live sessions were dropped, so the editor sends its map again.
    pub resync: u64,
}

impl LinkState {
    pub fn has_project(&self, root: &Path) -> bool {
        self.connected && self.project.as_deref() == Some(path_key(&godot_path(root)).as_str())
    }

    pub fn has_map(&self, path: &Path) -> bool {
        self.connected && self.maps.contains_key(&path_key(&godot_path(path)))
    }
}

pub enum Request {
    /// `live` is the saved map while live mode syncs it, the session then carries on from the rebuilt scene.
    MapSaved {
        path: String,
        game_port: Option<u16>,
        live: Option<gt_doc::Map>,
    },
    /// Full build of `map` as shown in the editor. With `live` the session carries on from the built scene.
    Build {
        path: String,
        map: gt_doc::Map,
        live: bool,
    },
    Focus,
    /// A PNG of the scene Godot has open, through `camera` (`position`, `forward`, `fov` in map units). With `build` Godot
    /// builds that map first, otherwise a live session catches up first.
    Capture {
        path: String,
        camera: Value,
        size: [u32; 2],
        build: Option<gt_doc::Map>,
        live: bool,
        done: Done,
    },
    /// The navigation mesh of the scene Godot has open, for an `agent` of `radius`, `height`, `max_climb` and `max_slope`
    /// in meters and degrees. With `build` Godot builds that map first, otherwise a live session catches up first.
    Walkable {
        path: String,
        agent: Value,
        build: Option<gt_doc::Map>,
        done: Done,
    },
    LiveEnd {
        path: String,
        revert: bool,
    },
}

struct LiveJob {
    path: String,
    frame: Frame,
}

#[derive(Default)]
struct Shared {
    port: u16,
    enabled: bool,
    shutdown: bool,
    state: LinkState,
    requests: VecDeque<Request>,
    live: Option<LiveJob>,
}

type SharedLock = Arc<(Mutex<Shared>, Condvar)>;

/// Background worker owning one connection to the Godot editor.
pub struct LiveLink {
    shared: SharedLock,
    statuses: Receiver<String>,
}

impl LiveLink {
    /// `repaint` is asked to redraw whenever the link state changes or a status message arrives.
    pub fn start(repaint: Option<egui::Context>) -> Self {
        let shared: SharedLock = Arc::new((Mutex::new(Shared { port: DEFAULT_PORT, ..Default::default() }), Condvar::new()));
        let (tx, statuses) = mpsc::channel();
        let worker_shared = shared.clone();
        std::thread::Builder::new().name("godot-live-link".into()).spawn(move || Worker::new(worker_shared, tx, repaint).run()).expect("spawn live link");
        Self { shared, statuses }
    }

    fn with<R>(&self, f: impl FnOnce(&mut Shared) -> R) -> R {
        let (lock, cvar) = &*self.shared;
        let out = f(&mut lock.lock().unwrap_or_else(|e| e.into_inner()));
        cvar.notify_all();
        out
    }

    pub fn configure(&self, port: u16, enabled: bool) {
        let (lock, cvar) = &*self.shared;
        let mut s = lock.lock().unwrap_or_else(|e| e.into_inner());
        if s.port != port || s.enabled != enabled {
            s.port = port;
            s.enabled = enabled;
            cvar.notify_all();
        }
    }

    pub fn state(&self) -> LinkState {
        self.shared.0.lock().unwrap_or_else(|e| e.into_inner()).state.clone()
    }

    pub fn request(&self, request: Request) {
        self.with(|s| s.requests.push_back(request));
    }

    /// Replaces any map not picked up yet, so only the newest state is sent.
    pub fn sync(&self, path: &Path, frame: Frame) {
        self.with(|s| s.live = Some(LiveJob { path: godot_path(path), frame }));
    }

    pub fn poll_status(&self) -> Option<String> {
        self.statuses.try_recv().ok()
    }
}

impl Drop for LiveLink {
    fn drop(&mut self) {
        self.with(|s| s.shutdown = true);
    }
}

struct Connection {
    writer: TcpStream,
    reader: BufReader<TcpStream>,
}

enum Work {
    Request(Request),
    Live(LiveJob),
    Heartbeat,
}

struct Worker {
    shared: SharedLock,
    status: Sender<String>,
    repaint: Option<egui::Context>,
    conn: Option<Connection>,
    port: u16,
    sessions: HashMap<String, LiveSession>,
    last_attempt: Option<Instant>,
    last_heartbeat: Option<Instant>,
    seq: u64,
}

impl Worker {
    fn new(shared: SharedLock, status: Sender<String>, repaint: Option<egui::Context>) -> Self {
        Self { shared, status, repaint, conn: None, port: 0, sessions: HashMap::new(), last_attempt: None, last_heartbeat: None, seq: 0 }
    }

    fn update(&self, f: impl FnOnce(&mut LinkState)) {
        let changed = {
            let mut s = self.shared.0.lock().unwrap_or_else(|e| e.into_inner());
            let before = s.state.clone();
            f(&mut s.state);
            s.state != before
        };
        if changed && let Some(ctx) = &self.repaint {
            ctx.request_repaint();
        }
    }

    fn say(&self, message: String) {
        let _ = self.status.send(message);
        if let Some(ctx) = &self.repaint {
            ctx.request_repaint();
        }
    }

    fn heartbeat_due(&self) -> bool {
        self.last_heartbeat.is_none_or(|t| t.elapsed() >= HEARTBEAT)
    }

    fn next_work(&mut self) -> Option<Work> {
        let (lock, cvar) = &*self.shared;
        let mut s = lock.lock().unwrap_or_else(|e| e.into_inner());
        loop {
            if s.shutdown {
                return None;
            }

            if s.port != self.port {
                self.port = s.port;
                self.conn = None;
            }

            if !s.enabled {
                s.requests.clear();
                s.live = None;
                if self.conn.take().is_some() || s.state.connected {
                    self.sessions.clear();
                    s.state = LinkState { resync: s.state.resync + 1, ..Default::default() };
                }

                s = cvar.wait_timeout(s, Duration::from_secs(60)).unwrap_or_else(|e| e.into_inner()).0;
                continue;
            }

            if let Some(r) = s.requests.pop_front() {
                return Some(Work::Request(r));
            }

            if self.heartbeat_due() {
                return Some(Work::Heartbeat);
            }

            if let Some(job) = s.live.take() {
                return Some(Work::Live(job));
            }

            let wait = self.last_heartbeat.map_or(Duration::ZERO, |t| HEARTBEAT.saturating_sub(t.elapsed()));
            s = cvar.wait_timeout(s, wait.max(Duration::from_millis(5))).unwrap_or_else(|e| e.into_inner()).0;
        }
    }

    fn run(mut self) {
        while let Some(work) = self.next_work() {
            match work {
                Work::Heartbeat => self.heartbeat(),
                Work::Request(r) => self.handle(r),
                Work::Live(job) => self.live(job),
            }
        }
    }

    fn connect(&mut self, eager: bool) -> bool {
        if self.conn.is_some() {
            return true;
        }

        if !eager && self.last_attempt.is_some_and(|t| t.elapsed() < RECONNECT) {
            return false;
        }

        self.last_attempt = Some(Instant::now());
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let Ok(stream) = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT) else { return false };
        let _ = stream.set_nodelay(true);
        let Ok(reader) = stream.try_clone() else { return false };
        self.conn = Some(Connection { writer: stream, reader: BufReader::new(reader) });
        true
    }

    fn disconnect(&mut self) {
        self.conn = None;
        let had_sessions = !self.sessions.is_empty();
        self.sessions.clear();
        self.update(|s| *s = LinkState { resync: s.resync + u64::from(had_sessions), ..Default::default() });
    }

    fn call(&mut self, mut message: Value, timeout: Duration) -> Result<Value, String> {
        let Some(conn) = self.conn.as_mut() else { return Err("Godot is not running with the GodotTrench addon".into()) };
        self.seq += 1;
        let seq = self.seq;
        message["seq"] = json!(seq);
        let result = (|| {
            writeln!(conn.writer, "{message}").map_err(|e| e.to_string())?;
            conn.reader.get_ref().set_read_timeout(Some(timeout)).map_err(|e| e.to_string())?;
            loop {
                let mut line = String::new();
                if conn.reader.read_line(&mut line).map_err(|e| e.to_string())? == 0 {
                    return Err("Godot closed the connection".to_string());
                }

                let Ok(reply) = serde_json::from_str::<Value>(&line) else { continue };
                // Addons from before the heartbeat answer without a seq.
                if reply["seq"].as_u64().is_none_or(|s| s == seq) {
                    return Ok(reply);
                }
            }
        })();
        if result.is_err() {
            self.disconnect();
        }

        result
    }

    fn heartbeat(&mut self) {
        self.last_heartbeat = Some(Instant::now());
        if !self.connect(false) {
            if self.last_attempt.is_some() {
                self.update(|s| s.connected = false);
            }

            return;
        }

        let Ok(reply) = self.call(json!({ "event": "status" }), QUICK_REPLY) else { return };
        if reply["ok"].as_bool() != Some(true) {
            self.update(|s| {
                *s = LinkState { connected: true, outdated: true, resync: s.resync, ..Default::default() };
            });
            return;
        }

        let maps: BTreeMap<String, u64> =
            reply["maps"].as_array().into_iter().flatten().filter_map(|m| Some((path_key(m["path"].as_str()?), m["epoch"].as_u64().unwrap_or(0)))).collect();
        let before = self.sessions.len();
        self.sessions.retain(|key, session| maps.get(key) == Some(&session.epoch));
        let dropped = self.sessions.len() != before;
        self.update(|s| {
            s.connected = true;
            s.outdated = false;
            s.project = reply["project"].as_str().map(path_key);
            s.godot = reply["godot"].as_str().unwrap_or_default().to_string();
            s.pid = reply["pid"].as_u64().map(|p| p as u32);
            s.maps = maps;
            if dropped {
                s.resync += 1;
            }
        });
    }

    fn long_call(&mut self, message: Value) -> Result<Value, String> {
        self.update(|s| s.busy = true);
        let reply = self.call(message, BUILD_REPLY);
        self.update(|s| s.busy = false);
        reply
    }

    fn handle(&mut self, request: Request) {
        if !self.connect(true) {
            self.update(|s| s.connected = false);
            match request {
                Request::MapSaved { path, game_port, .. } => {
                    let message = saved_message(&path);
                    self.say(format!("Live link: Godot is not running with the GodotTrench addon{}", game_status(game_port, &message)));
                }
                Request::Capture { done, .. } | Request::Walkable { done, .. } => {
                    done(Err(format!("Godot is not running with the GodotTrench addon on port {}, open the project in the Godot editor", self.port)))
                }
                Request::LiveEnd { .. } => {}
                _ => self.say("Godot is not running with the GodotTrench addon".into()),
            }

            return;
        }

        match request {
            Request::MapSaved { path, game_port, live } => {
                let key = path_key(&path);
                self.sessions.remove(&key);
                let message = saved_message(&path);
                let status = match self.long_call(message.clone()) {
                    Ok(reply) => rebuilt_status(&reply),
                    Err(e) => format!("Live link: {e}"),
                };
                self.say(status + &game_status(game_port, &message));
                if let Some(map) = live {
                    self.restart(&key, &path, map);
                }
            }
            Request::Build { path, map, live } => {
                let key = path_key(&path);
                self.sessions.remove(&key);
                let text = gt_doc::format::to_string(&map);
                let status = match self.long_call(json!({ "event": "build", "path": path, "text": text, "force": true })) {
                    Ok(reply) if reply["ok"].as_bool() == Some(true) => match reply["rebuilt"].as_u64() {
                        Some(0) | None => "Godot has no open scene using this map".to_string(),
                        Some(n) => format!("Godot built {n} map node(s) from the map as shown here"),
                    },
                    Ok(reply) => format!("Build in Godot failed: {}", reply["error"].as_str().unwrap_or("unexpected reply")),
                    Err(e) => format!("Build in Godot failed: {e}"),
                };
                self.say(status);
                if live {
                    self.restart(&key, &path, map);
                }
            }
            Request::Focus => {
                if let Some(pid) = self.shared.0.lock().unwrap_or_else(|e| e.into_inner()).state.pid {
                    allow_foreground(pid);
                }

                if let Err(e) = self.call(json!({ "event": "focus" }), QUICK_REPLY) {
                    self.say(format!("Could not reach Godot: {e}"));
                }
            }
            Request::Capture { path, camera, size, build, live, done } => {
                let key = path_key(&path);
                let mut message = json!({ "event": "capture", "path": path, "camera": camera, "width": size[0], "height": size[1] });
                let pending = self.shared.0.lock().unwrap_or_else(|e| e.into_inner()).live.take();
                if let Some(map) = &build {
                    self.sessions.remove(&key);
                    message["text"] = json!(gt_doc::format::to_string(map));
                } else if let Some(job) = pending {
                    // Godot has to hold the latest edits before it draws them.
                    self.live(job);
                }

                done(match self.long_call(message) {
                    Ok(reply) if reply["ok"].as_bool() == Some(true) => Ok(reply),
                    Ok(reply) if reply["error"] == "unknown event" => Err("the GodotTrench addon in Godot is too old to capture, update it".into()),
                    Ok(reply) => Err(reply["error"].as_str().unwrap_or("unexpected reply").to_string()),
                    Err(e) => Err(format!("Godot did not answer the capture: {e}")),
                });
                if live && let Some(map) = build {
                    self.restart(&key, &path, map);
                }
            }
            Request::Walkable { path, agent, build, done } => {
                let mut message = json!({ "event": "walkable", "path": path, "agent": agent });
                let pending = self.shared.0.lock().unwrap_or_else(|e| e.into_inner()).live.take();
                if let Some(map) = &build {
                    self.sessions.remove(&path_key(&path));
                    message["text"] = json!(gt_doc::format::to_string(map));
                } else if let Some(job) = pending {
                    self.live(job);
                }

                done(match self.long_call(message) {
                    Ok(reply) if reply["ok"].as_bool() == Some(true) => Ok(reply),
                    Ok(reply) if reply["error"] == "unknown event" => {
                        Err("the GodotTrench addon in Godot is too old to bake the walkable area, update it".into())
                    }
                    Ok(reply) => Err(reply["error"].as_str().unwrap_or("unexpected reply").to_string()),
                    Err(e) => Err(format!("Godot did not answer the walkable request: {e}")),
                });
            }
            Request::LiveEnd { path, revert } => {
                self.sessions.remove(&path_key(&path));
                let _ = self.long_call(json!({ "event": "live_end", "path": path, "revert": revert }));
            }
        }
    }

    fn live(&mut self, job: LiveJob) {
        let key = path_key(&job.path);
        let known = self.shared.0.lock().unwrap_or_else(|e| e.into_inner()).state.maps.contains_key(&key);
        if !known || !self.connect(false) {
            return;
        }

        let map = job.frame.map.clone();
        let message = match self.sessions.get_mut(&key) {
            Some(session) => match session.step(job.frame) {
                Some(m) => m,
                None => return,
            },
            None => self.begin(&key, &job.path, map.clone()),
        };
        match self.call(message, BUILD_REPLY) {
            Ok(reply) if reply["ok"].as_bool() == Some(true) => {
                if let (Some(session), Some(epoch)) = (self.sessions.get_mut(&key), reply["epoch"].as_u64()) {
                    session.epoch = epoch;
                }
            }
            Ok(_) => {
                // Godot lost track of the map (a build of its own, a changed scene), start over from the whole map.
                let message = self.begin(&key, &job.path, map);
                if let Ok(reply) = self.call(message, BUILD_REPLY)
                    && let (Some(session), Some(epoch)) = (self.sessions.get_mut(&key), reply["epoch"].as_u64())
                {
                    session.epoch = epoch;
                }
            }
            Err(_) => {}
        }
    }

    /// Starts a session from the map Godot has just fully built. The text matches that build, so Godot does not build again,
    /// and the next edit only sends what changed.
    fn restart(&mut self, key: &str, path: &str, map: gt_doc::Map) {
        let message = self.begin(key, path, map);
        match self.call(message, BUILD_REPLY) {
            Ok(reply) if reply["ok"].as_bool() == Some(true) => {
                if let (Some(session), Some(epoch)) = (self.sessions.get_mut(key), reply["epoch"].as_u64()) {
                    session.epoch = epoch;
                }
            }
            _ => {
                self.sessions.remove(key);
            }
        }
    }

    fn begin(&mut self, key: &str, path: &str, map: gt_doc::Map) -> Value {
        let (session, message) = LiveSession::begin(path.to_string(), map);
        self.sessions.insert(key.to_string(), session);
        message
    }
}

#[cfg(windows)]
fn allow_foreground(pid: u32) {
    // Windows only lets the foreground process hand focus to another one.
    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow(pid);
    }
}

#[cfg(not(windows))]
fn allow_foreground(_pid: u32) {}

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

    #[test]
    fn path_keys_ignore_slashes_and_case_on_windows() {
        assert_eq!(path_key("C:\\Game\\maps\\"), path_key("C:/Game/maps"));
        if cfg!(windows) {
            assert_eq!(path_key("C:/Game/Maps/A.gtm"), path_key("c:/game/maps/a.gtm"));
        }
    }

    /// Answers like the addon: status lists `map`, live messages are counted and acknowledged after `delay`.
    fn fake_godot(map: String, delay: Duration) -> (u16, Arc<Mutex<Vec<Value>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = seen.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { return };
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut epoch = 1;
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        break;
                    }

                    let msg: Value = serde_json::from_str(&line).unwrap();
                    let event = msg["event"].as_str().unwrap_or_default().to_string();
                    let mut reply = json!({ "ok": true, "seq": msg["seq"], "epoch": epoch });
                    match event.as_str() {
                        "status" => {
                            reply["project"] = json!("C:/game/");
                            reply["maps"] = json!([{ "path": map, "epoch": epoch }]);
                            reply["pid"] = json!(42);
                        }
                        "build" => {
                            epoch += 1;
                            reply["rebuilt"] = json!(1);
                        }
                        _ => std::thread::sleep(delay),
                    }

                    log.lock().unwrap().push(msg);
                    if writeln!(&stream, "{reply}").is_err() {
                        break;
                    }
                }
            }
        });
        (port, seen)
    }

    fn wait_for(what: &str, mut f: impl FnMut() -> bool) {
        let start = Instant::now();
        while !f() {
            assert!(start.elapsed() < Duration::from_secs(10), "timed out waiting for {what}");
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn events(log: &Arc<Mutex<Vec<Value>>>, name: &str) -> usize {
        log.lock().unwrap().iter().filter(|m| m["event"] == name).count()
    }

    #[test]
    fn heartbeat_live_backpressure_and_resync() {
        let map_path = std::env::temp_dir().join("gt_live_link_test").join("level.gtm");
        let (port, log) = fake_godot(godot_path(&map_path), Duration::from_millis(40));
        let link = LiveLink::start(None);
        link.configure(port, true);
        wait_for("heartbeat", || link.state().connected);
        let state = link.state();
        assert!(state.has_map(&map_path));
        assert!(state.has_project(Path::new("C:/game")) || !cfg!(windows));
        assert_eq!(state.pid, Some(42));

        let mut map = gt_doc::Map::new();
        let layer = map.default_layer();
        link.sync(&map_path, Frame { map: map.clone(), dragging: false, idle: true });
        wait_for("live_begin", || events(&log, "live_begin") == 1);
        for i in 0..50 {
            let mut e = gt_doc::Entity::new("light");
            e.origin.x = i as f64;
            map.insert(layer, gt_doc::NodeKind::Entity(e));
            link.sync(&map_path, Frame { map: map.clone(), dragging: false, idle: false });
            std::thread::sleep(Duration::from_millis(2));
        }

        wait_for("last delta", || {
            log.lock().unwrap().iter().filter(|m| m["event"] == "live_delta").flat_map(|m| m["ops"].as_array().cloned().unwrap_or_default()).count() == 50
        });
        let deltas = events(&log, "live_delta");
        assert!(deltas < 25, "{deltas} deltas for 50 quick edits, they should coalesce");

        link.request(Request::Build { path: godot_path(&map_path), map: map.clone(), live: true });
        wait_for("build status", || link.poll_status().is_some_and(|s| s.contains("built 1")));
        wait_for("session restarted from the built map", || events(&log, "live_begin") == 2);
        map.insert(layer, gt_doc::NodeKind::Entity(gt_doc::Entity::new("light")));
        link.sync(&map_path, Frame { map: map.clone(), dragging: false, idle: false });
        wait_for("delta after the build", || events(&log, "live_delta") == deltas + 1);
        assert_eq!(events(&log, "live_begin"), 2, "an edit after a build only sends what changed");

        link.request(Request::Build { path: godot_path(&map_path), map: map.clone(), live: false });
        map.insert(layer, gt_doc::NodeKind::Entity(gt_doc::Entity::new("light")));
        link.sync(&map_path, Frame { map: map.clone(), dragging: false, idle: false });
        wait_for("new session after a build outside live mode", || events(&log, "live_begin") == 3);
    }

    fn capture(link: &LiveLink, path: &Path, build: Option<gt_doc::Map>) -> Result<Value, String> {
        let (tx, rx) = mpsc::channel();
        let camera = json!({ "position": [0, 64, 0], "forward": [0, 0, -1], "fov": 90 });
        link.request(Request::Capture { path: godot_path(path), camera, size: [64, 32], build, live: false, done: Box::new(move |r| tx.send(r).unwrap()) });
        rx.recv_timeout(Duration::from_secs(10)).expect("capture answered")
    }

    #[test]
    fn capture_builds_the_shown_map_first_and_reports_a_missing_godot() {
        let map_path = std::env::temp_dir().join("gt_live_link_test").join("capture.gtm");
        let (port, log) = fake_godot(godot_path(&map_path), Duration::ZERO);
        let link = LiveLink::start(None);
        link.configure(port, true);
        assert!(capture(&link, &map_path, Some(gt_doc::Map::new())).is_ok());
        assert!(capture(&link, &map_path, None).is_ok());
        let sent: Vec<Value> = log.lock().unwrap().iter().filter(|m| m["event"] == "capture").cloned().collect();
        assert_eq!(sent.len(), 2);
        assert!(sent[0]["text"].is_string() && sent[1]["text"].is_null(), "only the first capture carries the map to build");
        assert_eq!((sent[0]["width"].as_u64(), sent[0]["height"].as_u64()), (Some(64), Some(32)));
        assert_eq!(sent[0]["camera"]["forward"], json!([0, 0, -1]));

        let dead = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        link.configure(dead, true);
        let e = capture(&link, &map_path, None).unwrap_err();
        assert!(e.contains("not running") && e.contains(&dead.to_string()), "{e}");
    }

    #[test]
    fn walkable_sends_the_agent_and_the_map_to_build() {
        let map_path = std::env::temp_dir().join("gt_live_link_test").join("walkable.gtm");
        let (port, log) = fake_godot(godot_path(&map_path), Duration::ZERO);
        let link = LiveLink::start(None);
        link.configure(port, true);
        let walkable = |link: &LiveLink, build: Option<gt_doc::Map>| {
            let (tx, rx) = mpsc::channel();
            let agent = json!({ "radius": 0.5, "height": 1.2, "max_climb": 0.2, "max_slope": 30 });
            link.request(Request::Walkable { path: godot_path(&map_path), agent, build, done: Box::new(move |r| tx.send(r).unwrap()) });
            rx.recv_timeout(Duration::from_secs(10)).expect("walkable answered")
        };
        assert!(walkable(&link, Some(gt_doc::Map::new())).is_ok());
        assert!(walkable(&link, None).is_ok());
        let sent: Vec<Value> = log.lock().unwrap().iter().filter(|m| m["event"] == "walkable").cloned().collect();
        assert_eq!(sent.len(), 2);
        assert!(sent[0]["text"].is_string() && sent[1]["text"].is_null(), "only the first request carries the map to build");
        assert_eq!(sent[0]["agent"]["radius"], 0.5);

        let dead = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        link.configure(dead, true);
        let e = walkable(&link, None).unwrap_err();
        assert!(e.contains("not running") && e.contains(&dead.to_string()), "{e}");
    }

    #[test]
    fn disabled_link_disconnects() {
        let (port, log) = fake_godot("C:/x.gtm".into(), Duration::ZERO);
        let link = LiveLink::start(None);
        link.configure(port, true);
        wait_for("connect", || link.state().connected);
        link.configure(port, false);
        wait_for("disconnect", || !link.state().connected);
        let heartbeats = events(&log, "status");
        std::thread::sleep(Duration::from_millis(1500));
        assert_eq!(events(&log, "status"), heartbeats);
    }
}
