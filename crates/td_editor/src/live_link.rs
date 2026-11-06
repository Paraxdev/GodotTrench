//! Connection to a Godot editor running the TrenchDot addon: save notifications, builds, focus and live mode.
//! Protocol: one JSON object per line over TCP on 127.0.0.1, each answered by one JSON line echoing its `seq`.

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::live_sync::{Frame, LiveSession};

pub const DEFAULT_PORT: u16 = 7842;
/// Port of the TrenchDotHotReload autoload inside a running game.
pub const DEFAULT_GAME_PORT: u16 = 7843;

const HEARTBEAT: Duration = Duration::from_secs(1);
const RECONNECT: Duration = Duration::from_secs(3);
const CONNECT_TIMEOUT: Duration = Duration::from_millis(400);
const QUICK_REPLY: Duration = Duration::from_secs(15);
const BUILD_REPLY: Duration = Duration::from_secs(180);
const SCAN_REPLY: Duration = Duration::from_secs(3);

/// Called with Godot's reply to a capture or walkable request, on the live link thread.
pub type Done = Box<dyn FnOnce(Result<Value, String>) + Send>;

pub fn send(port: u16, message: &Value) -> Result<Value, String> {
    send_timeout(port, message, Duration::from_secs(30))
}

/// One message on its own connection.
pub fn send_timeout(port: u16, message: &Value, timeout: Duration) -> Result<Value, String> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT).map_err(|_| "Godot is not running with the TrenchDot addon".to_string())?;
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

fn rebuilt_status(reply: &Value, path: &str) -> String {
    match reply["rebuilt"].as_u64() {
        Some(0) => format!("Live link: {}", nothing_built(reply, path, true)),
        Some(n) => format!("Live link: Godot rebuilt {n} map node(s){}", build_report(reply).map(|r| r.note()).unwrap_or_default()),
        None => format!("Live link: {}", reply["error"].as_str().unwrap_or("unexpected reply")),
    }
}

fn strings(value: &Value) -> Vec<String> {
    value.as_array().into_iter().flatten().filter_map(Value::as_str).map(str::to_string).collect()
}

/// What a save or build reply says about the maps it rebuilt. None when it rebuilt none.
fn build_report(reply: &Value) -> Option<GodotReport> {
    (reply["rebuilt"].as_u64().unwrap_or(0) > 0).then(|| GodotReport {
        missing: strings(&reply["missing_classes"]).into_iter().collect(),
        settings: strings(&reply["settings"]),
        warnings: strings(&reply["warnings"]).into_iter().filter(|w| !names_missing_classes(w)).collect(),
    })
}

/// The addon's own warning about the classes without a definition, which the editor already names one by one.
fn names_missing_classes(warning: &str) -> bool {
    warning.split_once(' ').is_some_and(|(count, rest)| {
        count.parse::<u32>().is_ok() && ["entity class has no definition in ", "entity classes have no definition in "].iter().any(|s| rest.starts_with(s))
    })
}

/// "a", "a and b", "a, b and c" or "a, b, c and 2 more".
pub fn name_list<S: AsRef<str>>(names: &[S]) -> String {
    let names: Vec<&str> = names.iter().map(AsRef::as_ref).collect();
    match &names[..] {
        [] => String::new(),
        [one] => one.to_string(),
        [first @ .., last] if names.len() <= 3 => format!("{} and {last}", first.join(", ")),
        more => format!("{} and {} more", more[..3].join(", "), more.len() - 3),
    }
}

/// What Godot reported when it last built one map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GodotReport {
    /// Classnames Godot built without a definition, as plain nodes without script or collision.
    pub missing: BTreeSet<String>,
    /// Resource paths of the map settings the build used. Empty from addons that do not say.
    pub settings: Vec<String>,
    pub warnings: Vec<String>,
}

impl GodotReport {
    /// The map settings by name with the verb that follows, "res://a.tres has" or "res://a.tres and res://b.tres have".
    pub fn settings_have(&self) -> String {
        match self.settings.len() {
            0 => "the FGD Godot builds with has".to_string(),
            1 => format!("{} has", self.settings[0]),
            _ => format!("{} have", name_list(&self.settings)),
        }
    }

    /// Takes in what a live step built: the classes of `rebuilt` entities count again, Godot built the `unresolved`
    /// ones without a definition, with the map settings of `settings`. Returns the unresolved classes that were not
    /// missing before.
    fn live_step(&mut self, rebuilt: &BTreeSet<String>, unresolved: BTreeSet<String>, settings: Vec<String>) -> Vec<String> {
        let new = unresolved.iter().filter(|c| !self.missing.contains(*c)).cloned().collect();
        self.missing.retain(|c| !rebuilt.contains(c));
        self.missing.extend(unresolved);
        self.settings = settings;
        new
    }

    /// What a status line about the build adds.
    fn note(&self) -> String {
        let missing: Vec<&String> = self.missing.iter().collect();
        match (missing.len(), self.warnings.len()) {
            (0, 0) => String::new(),
            (0, 1) => " with a warning, see the Issues panel".to_string(),
            (0, n) => format!(" with {n} warnings, see the Issues panel"),
            (1, _) => format!(", but {} no definition for {}, so it builds as a plain node. See the Issues panel", self.settings_have(), missing[0]),
            _ => format!(", but {} no definition for {}, so they build as plain nodes. See the Issues panel", self.settings_have(), name_list(&missing)),
        }
    }
}

/// What the status bar and the live mode tooltip say while Godot shows `classes` without their definition.
pub fn unresolved_text<S: AsRef<str>>(classes: &[S], report: &GodotReport) -> String {
    let (shows, them) = if classes.len() == 1 { ("it shows a plain node", "the class") } else { ("they show as plain nodes", "the classes") };
    let fgd = if report.settings.is_empty() { "the map settings FGD".to_string() } else { format!("the FGD of {}", name_list(&report.settings)) };
    let what = if classes.iter().any(|c| crate::entity_pack::in_pack(c.as_ref())) {
        format!("Install the Gameplay entities pack or add {them} to {fgd}")
    } else {
        format!("Add {them} to {fgd}")
    };
    format!("Godot has no definition for {}, {shows}. {what}", name_list(classes))
}

/// What the status bar says when Godot shows scenes built from the maps at `paths` with other entity definitions than
/// their map settings hold now.
pub fn stale_text<S: AsRef<str>>(paths: &[S]) -> String {
    let files: Vec<&str> = paths.iter().map(|p| p.as_ref().rsplit(['/', '\\']).next().unwrap_or(p.as_ref())).collect();
    format!("Godot shows {} built with other entity definitions than its map settings hold now, use Godot > Build in Godot", name_list(&files))
}

/// Classnames of the entities whose definitions the `reply` to a live message tells about again. The start of a
/// session names every class without a definition, from addons that name the map settings in their live replies.
fn rebuilt_classes(message: &Value, map: &td_doc::Map, reply: &Value) -> BTreeSet<String> {
    match message["event"].as_str() {
        Some("live_resync") => map.entities().map(|(_, e)| e.classname.clone()).collect(),
        Some("live_begin") if reply["settings"].is_array() => map.entities().map(|(_, e)| e.classname.clone()).collect(),
        Some("live_delta") => message["ops"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|op| op["op"] == "set")
            .filter_map(|op| op["id"].as_u64().map(td_core::NodeId))
            .filter_map(|id| map.entity(id).or_else(|| map.owning_entity(id).and_then(|e| map.entity(e))))
            .map(|e| e.classname.clone())
            .collect(),
        _ => BTreeSet::new(),
    }
}

/// Why Godot built nothing for the map at `path`, and what to do about it. Godot only builds the scene tab it shows, and
/// with `saved` a background tab that uses the map builds the saved file once it is shown.
fn nothing_built(reply: &Value, path: &str, saved: bool) -> String {
    let file = |p: &str| p.rsplit('/').next().unwrap_or(p).to_string();
    let map = file(path);
    let waiting: Vec<String> = reply["waiting"].as_array().into_iter().flatten().filter_map(Value::as_str).map(file).collect();
    match &waiting[..] {
        [] => {}
        [scene] => {
            let then = if saved { "it rebuilds when you switch to it" } else { "switch to it and build again" };
            return format!("{scene} uses {map} but is not the current scene tab in Godot, {then}");
        }
        [first @ .., last] => {
            let then = if saved { "each rebuilds when you switch to it" } else { "switch to one and build again" };
            return format!("{} and {last} use {map} but are not the current scene tab in Godot, {then}", first.join(", "));
        }
    }

    match reply["scene"].as_str() {
        Some("") => {
            format!("Godot has no scene open. Open the scene with the FuncGodotMap for {map}, or close a second Godot editor of this project")
        }
        Some(scene) => {
            format!(
                "{} is the current scene tab in Godot but has no FuncGodotMap for {map} with Auto Rebuild On Save on, open the scene that builds it",
                file(scene)
            )
        }
        None => format!("Godot has no open scene using {map}, open the scene with its FuncGodotMap in Godot"),
    }
}

fn game_status(game_port: Option<u16>, message: &Value) -> String {
    match game_port.map(|port| send(port, message)) {
        Some(Ok(reply)) if reply["rebuilt"].as_u64().is_some() => format!(", running game reloaded {} map(s)", reply["rebuilt"]),
        _ => String::new(),
    }
}

/// Has the Godot editor on `port` look for new files, so its live sessions rebuild what their definitions changed.
/// Returns the number of the scan, which [`LinkState::scanned`] reaches once it is done, or None from an addon before
/// the scan event. The reply comes before the scan finishes.
pub fn scan(port: u16) -> Option<u64> {
    let reply = send_timeout(port, &json!({ "event": "scan" }), SCAN_REPLY).ok().filter(|reply| reply["ok"].as_bool() == Some(true))?;
    Some(reply["scan"].as_u64().unwrap_or(0))
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
            Ok(reply) => rebuilt_status(&reply, message["path"].as_str().unwrap_or_default()),
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
    /// By `path_key` of the map.
    pub reports: BTreeMap<String, GodotReport>,
    /// The number of the last scan Godot finished, see [`scan`].
    pub scanned: u64,
    /// `path_key` of each map whose scene Godot built with other entity definitions than its map settings hold now.
    pub stale: BTreeSet<String>,
}

impl LinkState {
    /// What Godot reported building the map at `path`.
    pub fn report(&self, path: &Path) -> Option<&GodotReport> {
        self.reports.get(&path_key(&godot_path(path)))
    }

    /// Godot reported `classes` missing before their definitions appeared, so its next build tells again whether it has
    /// them.
    pub fn forget_missing(&mut self, classes: &BTreeSet<String>) {
        for report in self.reports.values_mut() {
            report.missing.retain(|c| !classes.contains(c));
        }
    }

    pub fn is_stale(&self, path: &Path) -> bool {
        self.stale.contains(&path_key(&godot_path(path)))
    }

    pub fn has_project(&self, root: &Path) -> bool {
        self.connected && self.project.as_deref() == Some(path_key(&godot_path(root)).as_str())
    }

    pub fn has_map(&self, path: &Path) -> bool {
        self.connected && self.maps.contains_key(&path_key(&godot_path(path)))
    }
}

pub enum Request {
    /// `live` is the saved map while live mode syncs it, the session then carries on from the rebuilt scene. `users` are
    /// the open maps that instance the saved file, by path and as shown in the editor, which Godot builds next when its
    /// addon did not build them for the save.
    MapSaved {
        path: String,
        game_port: Option<u16>,
        live: Option<td_doc::Map>,
        users: Vec<(String, td_doc::Map)>,
    },
    /// Full build of `map` as shown in the editor. With `live` the session carries on from the built scene.
    Build {
        path: String,
        map: td_doc::Map,
        live: bool,
    },
    Focus,
    /// A PNG of the scene Godot has open, through `camera` (`position`, `forward`, `fov` in map units). With `build` Godot
    /// builds that map first, otherwise a live session catches up first.
    Capture {
        path: String,
        camera: Value,
        size: [u32; 2],
        build: Option<td_doc::Map>,
        live: bool,
        done: Done,
    },
    /// The navigation mesh of the scene Godot has open, for an `agent` of `radius`, `height`, `max_climb` and `max_slope`
    /// in meters and degrees. With `build` Godot builds that map first, otherwise a live session catches up first.
    Walkable {
        path: String,
        agent: Value,
        build: Option<td_doc::Map>,
        done: Done,
    },
    LiveEnd {
        path: String,
        revert: bool,
    },
    /// Godot saves its open scenes and runs the main scene. `done` gets its reply, which is an unknown event from an
    /// older addon.
    Play {
        done: Done,
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

    /// Takes `classes` out of what Godot reported missing, see [`LinkState::forget_missing`].
    pub fn forget_missing(&self, classes: &BTreeSet<String>) {
        self.with(|s| s.state.forget_missing(classes));
    }
}

#[cfg(test)]
impl LiveLink {
    /// A link without a worker, so its requests stay queued for a test to look at.
    pub(crate) fn idle() -> Self {
        Self { shared: Arc::default(), statuses: mpsc::channel().1 }
    }

    pub(crate) fn take_requests(&self) -> Vec<Request> {
        self.with(|s| s.requests.drain(..).collect())
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
        let Some(conn) = self.conn.as_mut() else { return Err("Godot is not running with the TrenchDot addon".into()) };
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
        let stale: BTreeSet<String> =
            reply["maps"].as_array().into_iter().flatten().filter(|m| m["stale"] == true).filter_map(|m| m["path"].as_str()).map(path_key).collect();
        let known = self.shared.0.lock().unwrap_or_else(|e| e.into_inner()).state.stale.clone();
        let files: Vec<&str> = reply["maps"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|m| m["path"].as_str())
            .filter(|p| stale.contains(&path_key(p)) && !known.contains(&path_key(p)))
            .collect();
        if !files.is_empty() {
            self.say(stale_text(&files));
        }

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
            s.scanned = reply["scanned"].as_u64().unwrap_or(0);
            s.stale = stale;
            if dropped {
                s.resync += 1;
            }
        });
    }

    /// A full build of `map` as shown in the editor.
    fn build(&mut self, path: &str, map: &td_doc::Map) -> Result<Value, String> {
        let key = path_key(path);
        self.sessions.remove(&key);
        let text = td_doc::format::to_string_with_lightmap(map);
        let reply = self.long_call(json!({ "event": "build", "path": path, "text": text, "force": true }))?;
        if reply["ok"].as_bool() == Some(true) {
            self.keep_report(&key, &reply);
        }

        Ok(reply)
    }

    fn keep_report(&self, key: &str, reply: &Value) {
        if let Some(report) = build_report(reply) {
            self.update(|s| {
                s.reports.insert(key.to_string(), report);
            });
        }
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
                    self.say(format!("Live link: Godot is not running with the TrenchDot addon{}", game_status(game_port, &message)));
                }
                Request::Capture { done, .. } | Request::Walkable { done, .. } | Request::Play { done } => {
                    done(Err(format!("Godot is not running with the TrenchDot addon on port {}, open the project in the Godot editor", self.port)))
                }
                Request::LiveEnd { .. } => {}
                _ => self.say("Godot is not running with the TrenchDot addon".into()),
            }

            return;
        }

        match request {
            Request::MapSaved { path, game_port, live, users } => {
                let key = path_key(&path);
                self.sessions.remove(&key);
                let message = saved_message(&path);
                let reply = self.long_call(message.clone());
                // An addon that builds the maps using a saved prefab itself counts them in rebuilt.
                let users = if matches!(&reply, Ok(r) if r["rebuilt"].as_u64().is_some_and(|n| n > 0)) { Vec::new() } else { users };
                let game = game_status(game_port, &message);
                let status = match reply {
                    // A prefab no scene builds itself reaches Godot through the maps that use it.
                    Ok(reply) if reply["rebuilt"].as_u64() == Some(0) && !users.is_empty() && game.is_empty() => None,
                    Ok(reply) => {
                        self.keep_report(&key, &reply);
                        Some(rebuilt_status(&reply, &path))
                    }
                    Err(e) => Some(format!("Live link: {e}")),
                };
                if let Some(status) = status {
                    self.say(status + &game);
                }

                if let Some(map) = live {
                    self.restart(&key, &path, map);
                }

                let file = |p: &str| p.rsplit('/').next().unwrap_or(p).to_string();
                // A live session in Godot already builds again when a prefab of its map is saved.
                let users: Vec<_> = users.into_iter().filter(|(user, _)| !self.sessions.contains_key(&path_key(user))).collect();
                for (user, map) in users {
                    let status = match self.build(&user, &map) {
                        Ok(reply) if reply["ok"].as_bool() != Some(true) => {
                            format!("Live link: Godot could not build {}: {}", file(&user), reply["error"].as_str().unwrap_or("unexpected reply"))
                        }
                        Ok(reply) => match build_report(&reply) {
                            Some(report) => format!("Live link: Godot rebuilt {}, which uses {}{}", file(&user), file(&path), report.note()),
                            None => format!("Live link: {}", nothing_built(&reply, &user, false)),
                        },
                        Err(e) => format!("Live link: {e}"),
                    };
                    self.say(status);
                }
            }
            Request::Build { path, map, live } => {
                let key = path_key(&path);
                let status = match self.build(&path, &map) {
                    Ok(reply) if reply["ok"].as_bool() == Some(true) => match (reply["rebuilt"].as_u64(), build_report(&reply)) {
                        (Some(n), Some(report)) => format!("Godot built {n} map node(s) from the map as shown here{}", report.note()),
                        _ => nothing_built(&reply, &path, false),
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
                    message["text"] = json!(td_doc::format::to_string_with_lightmap(map));
                } else if let Some(job) = pending {
                    // Godot has to hold the latest edits before it draws them.
                    self.live(job);
                }

                done(match self.long_call(message) {
                    Ok(reply) if reply["ok"].as_bool() == Some(true) => Ok(reply),
                    Ok(reply) if reply["error"] == "unknown event" => Err("the TrenchDot addon in Godot is too old to capture, update it".into()),
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
                    message["text"] = json!(td_doc::format::to_string(map));
                } else if let Some(job) = pending {
                    self.live(job);
                }

                done(match self.long_call(message) {
                    Ok(reply) if reply["ok"].as_bool() == Some(true) => Ok(reply),
                    Ok(reply) if reply["error"] == "unknown event" => {
                        Err("the TrenchDot addon in Godot is too old to bake the walkable area, update it".into())
                    }
                    Ok(reply) => Err(reply["error"].as_str().unwrap_or("unexpected reply").to_string()),
                    Err(e) => Err(format!("Godot did not answer the walkable request: {e}")),
                });
            }
            Request::LiveEnd { path, revert } => {
                self.sessions.remove(&path_key(&path));
                let _ = self.long_call(json!({ "event": "live_end", "path": path, "revert": revert }));
            }
            Request::Play { done } => {
                done(self.call(json!({ "event": "play" }), BUILD_REPLY));
                if let Some(ctx) = &self.repaint {
                    ctx.request_repaint();
                }
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
        match self.call(message.clone(), BUILD_REPLY) {
            Ok(reply) if reply["ok"].as_bool() == Some(true) => {
                if let (Some(session), Some(epoch)) = (self.sessions.get_mut(&key), reply["epoch"].as_u64()) {
                    session.epoch = epoch;
                }

                self.keep_unresolved(&key, &rebuilt_classes(&message, &map, &reply), &reply);
            }
            Ok(_) => {
                // Godot lost track of the map (a build of its own, a changed scene), start over from the whole map.
                let message = self.begin(&key, &job.path, map.clone());
                if let Ok(reply) = self.call(message.clone(), BUILD_REPLY) {
                    if let (Some(session), Some(epoch)) = (self.sessions.get_mut(&key), reply["epoch"].as_u64()) {
                        session.epoch = epoch;
                    }

                    self.keep_unresolved(&key, &rebuilt_classes(&message, &map, &reply), &reply);
                }
            }
            Err(_) => {}
        }
    }

    /// Takes in the classes a live step built without a definition, and says so when there are new ones.
    fn keep_unresolved(&self, key: &str, rebuilt: &BTreeSet<String>, reply: &Value) {
        let unresolved: BTreeSet<String> = strings(&reply["unresolved"]).into_iter().collect();
        let before = self.shared.0.lock().unwrap_or_else(|e| e.into_inner()).state.reports.get(key).cloned();
        let mut report = before.clone().unwrap_or_default();
        let new = report.live_step(rebuilt, unresolved, strings(&reply["settings"]));
        let unchanged = match &before {
            Some(b) => *b == report,
            None => report.missing.is_empty(),
        };
        if unchanged {
            return;
        }

        if !new.is_empty() {
            self.say(unresolved_text(&new, &report));
        }

        self.update(|s| {
            s.reports.insert(key.to_string(), report);
        });
    }

    /// Starts a session from the map Godot has just fully built. The text matches that build, so Godot does not build again,
    /// and the next edit only sends what changed.
    fn restart(&mut self, key: &str, path: &str, map: td_doc::Map) {
        let message = self.begin(key, path, map.clone());
        match self.call(message.clone(), BUILD_REPLY) {
            Ok(reply) if reply["ok"].as_bool() == Some(true) => {
                if let (Some(session), Some(epoch)) = (self.sessions.get_mut(key), reply["epoch"].as_u64()) {
                    session.epoch = epoch;
                }

                self.keep_unresolved(key, &rebuilt_classes(&message, &map, &reply), &reply);
            }
            _ => {
                self.sessions.remove(key);
            }
        }
    }

    fn begin(&mut self, key: &str, path: &str, map: td_doc::Map) -> Value {
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
        let rx = notify_saved(port, Path::new("maps/level.tdm"));
        let status = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(status.contains("rebuilt 2"), "{status}");
    }

    #[test]
    fn explains_why_godot_built_nothing() {
        let path = "C:/game/maps/shop.tdm";
        let status = |reply: Value| rebuilt_status(&reply, path);
        assert_eq!(status(json!({ "ok": true, "rebuilt": 2 })), "Live link: Godot rebuilt 2 map node(s)");
        let waiting = status(json!({ "ok": true, "rebuilt": 0, "scene": "res://menu.tscn", "waiting": ["res://levels/shop.tscn"] }));
        assert!(waiting.contains("shop.tscn uses shop.tdm") && waiting.contains("rebuilds when you switch to it"), "{waiting}");
        let other = status(json!({ "ok": true, "rebuilt": 0, "scene": "res://menu.tscn" }));
        assert!(other.contains("menu.tscn is the current scene tab") && other.contains("FuncGodotMap for shop.tdm"), "{other}");
        let none = status(json!({ "ok": true, "rebuilt": 0, "scene": "" }));
        assert!(none.contains("no scene open") && none.contains("second Godot editor"), "{none}");
        let old_addon = status(json!({ "ok": true, "rebuilt": 0 }));
        assert!(old_addon.contains("no open scene using shop.tdm"), "{old_addon}");
        let several = status(json!({ "ok": true, "rebuilt": 0, "scene": "res://menu.tscn", "waiting": ["res://a.tscn", "res://b.tscn", "res://c.tscn"] }));
        assert!(several.contains("a.tscn, b.tscn and c.tscn use shop.tdm but are not"), "{several}");
        let build = nothing_built(&json!({ "ok": true, "rebuilt": 0, "waiting": ["res://shop.tscn"] }), path, false);
        assert!(build.contains("build again"), "a build from unsaved edits is not kept for later: {build}");
    }

    #[test]
    fn a_build_names_the_classes_godot_has_no_definition_for() {
        assert_eq!(name_list(&["a"]), "a");
        assert_eq!(name_list(&["a", "b", "c"]), "a, b and c");
        assert_eq!(name_list(&["a", "b", "c", "d", "e"]), "a, b, c and 2 more");
        let path = "C:/game/maps/shop.tdm";
        let status = |extra: Value| {
            let mut reply = json!({ "ok": true, "rebuilt": 1 });
            reply.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
            rebuilt_status(&reply, path)
        };
        assert_eq!(status(json!({})), "Live link: Godot rebuilt 1 map node(s)", "an older addon says nothing more");
        assert_eq!(
            status(json!({ "missing_classes": ["light", "func_door"], "settings": ["res://my_settings.tres"], "warnings": ["a"] })),
            "Live link: Godot rebuilt 1 map node(s), but res://my_settings.tres has no definition for func_door and light, so they build as plain nodes. See the Issues panel"
        );
        assert_eq!(
            status(json!({ "missing_classes": ["func_door"] })),
            "Live link: Godot rebuilt 1 map node(s), but the FGD Godot builds with has no definition for func_door, so it builds as a plain node. See the Issues panel"
        );
        assert_eq!(status(json!({ "warnings": ["a", "b"] })), "Live link: Godot rebuilt 1 map node(s) with 2 warnings, see the Issues panel");
        assert_eq!(build_report(&json!({ "ok": true, "rebuilt": 0, "missing_classes": ["func_door"] })), None, "nothing built, nothing learned");
        let summary = "2 entity classes have no definition in res://my_settings.tres (Entity Fgd res://my.tres): func_door, my_lamp. They build as plain nodes";
        let report = build_report(
            &json!({ "ok": true, "rebuilt": 1, "missing_classes": ["func_door", "my_lamp"], "warnings": [summary, "model res://a.glb not found"] }),
        );
        assert_eq!(report.unwrap().warnings, ["model res://a.glb not found"], "the classes are named one by one already");
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
            assert_eq!(path_key("C:/Game/Maps/A.tdm"), path_key("c:/game/maps/a.tdm"));
        }
    }

    fn fake_godot(map: String, delay: Duration) -> (u16, Arc<Mutex<Vec<Value>>>) {
        fake_godot_with(map, delay, json!({}))
    }

    /// Answers like the addon: status lists `map`, live messages are counted and acknowledged after `delay`. The replies
    /// to each event in `answers` carry its fields too.
    fn fake_godot_with(map: String, delay: Duration, answers: Value) -> (u16, Arc<Mutex<Vec<Value>>>) {
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

                    if let Some(extra) = answers[event.as_str()].as_object() {
                        reply.as_object_mut().unwrap().extend(extra.clone());
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
        let map_path = std::env::temp_dir().join("td_live_link_test").join("level.tdm");
        let (port, log) = fake_godot(godot_path(&map_path), Duration::from_millis(40));
        let link = LiveLink::start(None);
        link.configure(port, true);
        wait_for("heartbeat", || link.state().connected);
        let state = link.state();
        assert!(state.has_map(&map_path));
        assert!(state.has_project(Path::new("C:/game")) || !cfg!(windows));
        assert_eq!(state.pid, Some(42));

        let mut map = td_doc::Map::new();
        let layer = map.default_layer();
        link.sync(&map_path, Frame { map: map.clone(), dragging: false, idle: true });
        wait_for("live_begin", || events(&log, "live_begin") == 1);
        for i in 0..50 {
            let mut e = td_doc::Entity::new("light");
            e.origin.x = i as f64;
            map.insert(layer, td_doc::NodeKind::Entity(e));
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
        map.insert(layer, td_doc::NodeKind::Entity(td_doc::Entity::new("light")));
        link.sync(&map_path, Frame { map: map.clone(), dragging: false, idle: false });
        wait_for("delta after the build", || events(&log, "live_delta") == deltas + 1);
        assert_eq!(events(&log, "live_begin"), 2, "an edit after a build only sends what changed");

        link.request(Request::Build { path: godot_path(&map_path), map: map.clone(), live: false });
        map.insert(layer, td_doc::NodeKind::Entity(td_doc::Entity::new("light")));
        link.sync(&map_path, Frame { map: map.clone(), dragging: false, idle: false });
        wait_for("new session after a build outside live mode", || events(&log, "live_begin") == 3);
    }

    #[test]
    fn what_godot_reports_about_a_build_is_kept_per_map() {
        let map_path = std::env::temp_dir().join("td_live_link_test").join("report.tdm");
        let answers = json!({ "build": { "missing_classes": ["func_door"], "settings": ["res://my_settings.tres"], "warnings": ["Node Class AnimatedBody3D not found"] } });
        let (port, _) = fake_godot_with(godot_path(&map_path), Duration::ZERO, answers);
        let link = LiveLink::start(None);
        link.configure(port, true);
        wait_for("heartbeat", || link.state().connected);
        link.request(Request::Build { path: godot_path(&map_path), map: td_doc::Map::new(), live: false });
        let named = "Godot built 1 map node(s) from the map as shown here, but res://my_settings.tres has no definition for func_door";
        wait_for("build status", || link.poll_status().is_some_and(|s| s.starts_with(named)));
        let state = link.state();
        let report = state.report(&map_path).expect("a report for the map");
        assert_eq!(report.missing.iter().collect::<Vec<_>>(), ["func_door"]);
        assert_eq!((report.settings_have(), report.warnings.len()), ("res://my_settings.tres has".to_string(), 1));
        assert!(state.report(&map_path.with_file_name("other.tdm")).is_none());
    }

    #[test]
    fn live_steps_keep_track_of_the_classes_godot_lacks() {
        let classes = |list: &[&str]| list.iter().map(|c| c.to_string()).collect::<BTreeSet<String>>();
        let settings = || vec!["res://my_settings.tres".to_string()];
        let mut report = GodotReport::default();
        assert_eq!(report.live_step(&classes(&[]), classes(&["func_door", "my_lamp"]), settings()), ["func_door", "my_lamp"]);
        assert!(report.live_step(&classes(&["light"]), classes(&[]), settings()).is_empty());
        assert_eq!(report.missing, classes(&["func_door", "my_lamp"]), "a step that built other classes leaves them");
        assert!(report.live_step(&classes(&["func_door"]), classes(&[]), settings()).is_empty());
        assert_eq!(report.missing, classes(&["my_lamp"]), "the door was built again and found its definition");
        assert!(report.live_step(&classes(&["my_lamp"]), classes(&["my_lamp"]), settings()).is_empty(), "nothing new to say");
        assert_eq!(report.settings, settings(), "the reply names the map settings");

        assert_eq!(
            unresolved_text(&["func_door"], &GodotReport::default()),
            "Godot has no definition for func_door, it shows a plain node. Install the Gameplay entities pack or add the class to the map settings FGD"
        );
        assert_eq!(
            unresolved_text(&["my_lamp", "my_crate"], &report),
            "Godot has no definition for my_lamp and my_crate, they show as plain nodes. Add the classes to the FGD of res://my_settings.tres"
        );

        let mut map = td_doc::Map::new();
        let layer = map.default_layer();
        let door = map.insert(layer, td_doc::NodeKind::Entity(td_doc::Entity::new("func_door")));
        let leaf =
            map.insert(door, td_doc::NodeKind::Brush(td_geom::Brush::from_aabb(&td_core::Aabb::new(td_core::DVec3::ZERO, td_core::DVec3::ONE), "a").unwrap()));
        map.insert(layer, td_doc::NodeKind::Entity(td_doc::Entity::new("light")));
        let delta = json!({ "event": "live_delta", "ops": [{ "op": "set", "id": leaf.0 }, { "op": "remove", "id": 99 }] });
        let ok = json!({ "ok": true });
        let named = json!({ "ok": true, "settings": [] });
        assert_eq!(rebuilt_classes(&delta, &map, &ok), classes(&["func_door"]), "a brush rebuilds its entity");
        assert_eq!(rebuilt_classes(&json!({ "event": "live_resync" }), &map, &ok), classes(&["func_door", "light"]));
        assert!(rebuilt_classes(&json!({ "event": "live_begin" }), &map, &ok).is_empty(), "an older addon only names what a begin built");
        assert_eq!(rebuilt_classes(&json!({ "event": "live_begin" }), &map, &named), classes(&["func_door", "light"]), "a newer one names every class");
    }

    #[test]
    fn a_live_reply_with_unresolved_classes_is_told() {
        let map_path = std::env::temp_dir().join("td_live_link_test").join("unresolved.tdm");
        let answers = json!({ "live_begin": { "unresolved": ["func_door"] } });
        let (port, _) = fake_godot_with(godot_path(&map_path), Duration::ZERO, answers);
        let link = LiveLink::start(None);
        link.configure(port, true);
        wait_for("heartbeat", || link.state().connected);
        link.sync(&map_path, Frame { map: td_doc::Map::new(), dragging: false, idle: true });
        wait_for("the status", || link.poll_status().is_some_and(|s| s.starts_with("Godot has no definition for func_door, it shows a plain node")));
        assert_eq!(link.state().report(&map_path).map(|r| r.missing.iter().cloned().collect::<Vec<_>>()), Some(vec!["func_door".to_string()]));
        assert!(link.state().report(&map_path).is_some_and(|r| r.settings.is_empty()), "a reply that names no map settings names no file");
    }

    #[test]
    fn a_new_session_drops_the_classes_godot_no_longer_lacks() {
        let map_path = std::env::temp_dir().join("td_live_link_test").join("resolved.tdm");
        let answers = json!({ "live_begin": { "settings": ["res://crypt_settings.tres"] } });
        let (port, _) = fake_godot_with(godot_path(&map_path), Duration::ZERO, answers);
        let link = LiveLink::start(None);
        link.configure(port, true);
        wait_for("heartbeat", || link.state().connected);
        let stale = GodotReport { missing: ["func_door".to_string()].into(), settings: vec!["res://other_scene_settings.tres".into()], ..Default::default() };
        link.with(|s| s.state.reports.insert(path_key(&godot_path(&map_path)), stale));
        let mut map = td_doc::Map::new();
        map.insert(map.default_layer(), td_doc::NodeKind::Entity(td_doc::Entity::new("func_door")));
        link.sync(&map_path, Frame { map, dragging: false, idle: true });
        wait_for("the report", || link.state().report(&map_path).is_some_and(|r| r.missing.is_empty()));
        assert_eq!(link.state().report(&map_path).unwrap().settings, ["res://crypt_settings.tres"]);
    }

    #[test]
    fn a_saved_prefab_is_built_through_the_maps_that_use_it() {
        let dir = std::env::temp_dir().join("td_live_link_test");
        let (prefab, level) = (dir.join("crate.tdm"), dir.join("level.tdm"));
        let answers = json!({ "map_saved": { "rebuilt": 0 }, "build": { "missing_classes": ["func_door"] } });
        let (port, log) = fake_godot_with(godot_path(&level), Duration::ZERO, answers);
        let link = LiveLink::start(None);
        link.configure(port, true);
        wait_for("heartbeat", || link.state().connected);
        let users = vec![(godot_path(&level), td_doc::Map::new())];
        link.request(Request::MapSaved { path: godot_path(&prefab), game_port: None, live: None, users });
        let mut said = Vec::new();
        wait_for("the build status", || {
            said.extend(std::iter::from_fn(|| link.poll_status()));
            said.iter().any(|s| s.starts_with("Live link: Godot rebuilt level.tdm, which uses crate.tdm"))
        });
        assert!(said.iter().all(|s| !s.contains("no open scene")), "the prefab itself builds in no scene, which is fine: {said:?}");
        let builds: Vec<Value> = log.lock().unwrap().iter().filter(|m| m["event"] == "build").cloned().collect();
        assert_eq!((builds.len(), builds[0]["path"].as_str()), (1, Some(godot_path(&level).as_str())));
        assert!(builds[0]["text"].as_str().unwrap().contains("trenchdot-map"));
        assert!(link.state().report(&level).is_some_and(|r| r.missing.contains("func_door")), "the build of the map is reported for the map");

        link.sync(&level, Frame { map: td_doc::Map::new(), dragging: false, idle: true });
        wait_for("the live session", || events(&log, "live_begin") == 1);
        let users = vec![(godot_path(&level), td_doc::Map::new())];
        link.request(Request::MapSaved { path: godot_path(&prefab), game_port: None, live: None, users });
        link.request(Request::Focus);
        wait_for("the second save", || events(&log, "focus") == 1);
        assert_eq!(events(&log, "build"), 1, "Godot's live session builds the map again by itself");
    }

    #[test]
    fn an_addon_that_builds_the_users_of_a_saved_prefab_is_left_to_it() {
        let dir = std::env::temp_dir().join("td_live_link_test");
        let (prefab, level) = (dir.join("barrel.tdm"), dir.join("yard.tdm"));
        let (port, log) = fake_godot_with(godot_path(&level), Duration::ZERO, json!({ "map_saved": { "rebuilt": 1 } }));
        let link = LiveLink::start(None);
        link.configure(port, true);
        wait_for("heartbeat", || link.state().connected);
        let users = vec![(godot_path(&level), td_doc::Map::new())];
        link.request(Request::MapSaved { path: godot_path(&prefab), game_port: None, live: None, users });
        link.request(Request::Focus);
        wait_for("the save", || events(&log, "focus") == 1);
        assert_eq!(events(&log, "build"), 0);
        assert!(link.poll_status().is_some_and(|s| s.starts_with("Live link: Godot rebuilt 1 map node(s)")));
    }

    fn capture(link: &LiveLink, path: &Path, build: Option<td_doc::Map>) -> Result<Value, String> {
        let (tx, rx) = mpsc::channel();
        let camera = json!({ "position": [0, 64, 0], "forward": [0, 0, -1], "fov": 90 });
        link.request(Request::Capture { path: godot_path(path), camera, size: [64, 32], build, live: false, done: Box::new(move |r| tx.send(r).unwrap()) });
        rx.recv_timeout(Duration::from_secs(10)).expect("capture answered")
    }

    #[test]
    fn capture_builds_the_shown_map_first_and_reports_a_missing_godot() {
        let map_path = std::env::temp_dir().join("td_live_link_test").join("capture.tdm");
        let (port, log) = fake_godot(godot_path(&map_path), Duration::ZERO);
        let link = LiveLink::start(None);
        link.configure(port, true);
        assert!(capture(&link, &map_path, Some(td_doc::Map::new())).is_ok());
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
        let map_path = std::env::temp_dir().join("td_live_link_test").join("walkable.tdm");
        let (port, log) = fake_godot(godot_path(&map_path), Duration::ZERO);
        let link = LiveLink::start(None);
        link.configure(port, true);
        let walkable = |link: &LiveLink, build: Option<td_doc::Map>| {
            let (tx, rx) = mpsc::channel();
            let agent = json!({ "radius": 0.5, "height": 1.2, "max_climb": 0.2, "max_slope": 30 });
            link.request(Request::Walkable { path: godot_path(&map_path), agent, build, done: Box::new(move |r| tx.send(r).unwrap()) });
            rx.recv_timeout(Duration::from_secs(10)).expect("walkable answered")
        };
        assert!(walkable(&link, Some(td_doc::Map::new())).is_ok());
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
    fn scenes_built_with_other_definitions_are_told_once() {
        let map_path = std::env::temp_dir().join("td_live_link_test").join("stale.tdm");
        let map = godot_path(&map_path);
        let answers = json!({ "status": { "maps": [{ "path": map, "epoch": 1, "stale": true }] } });
        let (port, _) = fake_godot_with(map.clone(), Duration::ZERO, answers);
        let link = LiveLink::start(None);
        link.configure(port, true);
        wait_for("the stale map", || link.state().is_stale(&map_path));
        wait_for("the status", || link.poll_status().is_some_and(|s| s.starts_with("Godot shows stale.tdm built with other entity definitions")));
        std::thread::sleep(HEARTBEAT * 2);
        assert!(link.poll_status().is_none(), "told once");
    }

    #[test]
    fn only_an_addon_with_the_scan_event_scans() {
        let (port, log) = fake_godot_with("C:/x.tdm".into(), Duration::ZERO, json!({ "scan": { "scan": 4 }, "status": { "scanned": 3 } }));
        assert_eq!(scan(port), Some(4));
        assert_eq!(events(&log, "scan"), 1);
        let link = LiveLink::start(None);
        link.configure(port, true);
        wait_for("the status names the last scan done", || link.state().scanned == 3);
        let (old, _) = fake_godot_with("C:/x.tdm".into(), Duration::ZERO, json!({ "scan": { "ok": false, "error": "unknown event" } }));
        assert_eq!(scan(old), None);
        let dead = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        assert_eq!(scan(dead), None, "no Godot");
    }

    #[test]
    fn disabled_link_disconnects() {
        let (port, log) = fake_godot("C:/x.tdm".into(), Duration::ZERO);
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
