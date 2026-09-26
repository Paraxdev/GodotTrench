//! Ready made content for a Godot project: the nature pack and the demo. The Blockbench nature models are embedded in
//! the editor, the rest is downloaded from the GitHub release that matches the editor, or from the rolling beta, and
//! unpacked into the project without replacing any file.

use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

/// A release download and the project folders it adds to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Download {
    pub asset: &'static str,
    /// The archive's top folders, which is also where they land in the project. Anything else in it is refused.
    pub folders: &'static [&'static str],
    /// The size the wizard shows, the release gives the exact one once the download starts.
    pub approx_mb: u32,
}

pub const NATURE_PACK: Download = Download { asset: "godottrench-nature-pack.zip", folders: &["godottrench/nature"], approx_mb: 23 };
pub const DEMO_CONTENT: Download = Download { asset: "godottrench-demo-content.zip", folders: &["demo", "models"], approx_mb: 107 };

pub const RELEASES_API: &str = "https://api.github.com/repos/Paraxdev/GodotTrench";
/// Replaces [`RELEASES_API`], so tests can serve the downloads themselves.
pub const API_ENV: &str = "GODOTTRENCH_RELEASES_API";

/// Refuses archives that would unpack to more than this, whatever their headers claim per file.
const MAX_UNPACKED: u64 = 4 << 30;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Choice {
    #[default]
    Nothing,
    Nature,
    Demo,
}

impl Choice {
    pub const ALL: [Choice; 3] = [Choice::Nothing, Choice::Nature, Choice::Demo];

    pub fn downloads(self) -> &'static [Download] {
        match self {
            Choice::Nothing => &[],
            Choice::Nature => &[NATURE_PACK],
            Choice::Demo => &[NATURE_PACK, DEMO_CONTENT],
        }
    }

    pub fn approx_mb(self) -> u32 {
        self.downloads().iter().map(|d| d.approx_mb).sum()
    }

    pub fn name(self) -> &'static str {
        match self {
            Choice::Nothing => "nothing",
            Choice::Nature => "nature",
            Choice::Demo => "demo",
        }
    }
}

/// The project's nature folder, where the scatter presets look for their models.
pub fn nature_dir(root: &Path) -> PathBuf {
    root.join(gt_doc::scatter::NATURE_DIR.trim_start_matches("res://"))
}

/// A project with nature models, like the demo project, is not offered content when it first opens.
pub fn has_content(root: &Path) -> bool {
    nature_dir(root).is_dir()
}

pub fn has_nature_pack(root: &Path) -> bool {
    gt_formats::nature::pack_installed(&nature_dir(root))
}

pub fn has_demo(root: &Path) -> bool {
    root.join("demo/demo.tscn").is_file()
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// The server could not be reached, with what failed.
    Offline(String),
    NotPublished(&'static str),
    Status(u16),
    /// The download or the archive is not what the release promised.
    Damaged(String),
    DiskFull,
    /// The archive names a path outside the folders it may add to. Nothing is written then.
    Unsafe(String),
    Io(String),
    Cancelled,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Offline(why) => write!(f, "Could not reach GitHub ({why}). Check the internet connection and try again."),
            Error::NotPublished(asset) => {
                write!(f, "{asset} is not published yet, neither with the v{} release nor with the rolling beta.", crate::VERSION)
            }
            Error::Status(code @ (403 | 429)) => write!(f, "GitHub refused the request ({code}), it limits how often one address may ask. Try again later."),
            Error::Status(code) => write!(f, "GitHub answered with error {code}. Try again later."),
            Error::Damaged(why) => write!(f, "The download is damaged: {why}. Try again."),
            Error::DiskFull => write!(f, "The disk is full. Free some space and try again, the files added so far stay."),
            Error::Unsafe(path) => write!(f, "The archive holds {path}, outside the folders it may add to, so nothing was added."),
            Error::Io(why) => write!(f, "{why}"),
            Error::Cancelled => write!(f, "Cancelled. The files added so far stay, installing again adds the rest."),
        }
    }
}

fn io_error(e: std::io::Error, what: &Path) -> Error {
    match e.kind() {
        std::io::ErrorKind::StorageFull => Error::DiskFull,
        // The zip reader reports a CRC mismatch as invalid data.
        std::io::ErrorKind::InvalidData => Error::Damaged(format!("{}: {e}", what.display())),
        _ => Error::Io(format!("Could not write {}: {e}", what.display())),
    }
}

fn net_error(e: ureq::Error) -> Error {
    match e {
        ureq::Error::StatusCode(code) => Error::Status(code),
        ureq::Error::HostNotFound => Error::Offline("the server name was not found".into()),
        ureq::Error::ConnectionFailed => Error::Offline("the connection failed".into()),
        ureq::Error::Timeout(_) => Error::Offline("no answer in time".into()),
        ureq::Error::Io(e) => Error::Offline(e.to_string()),
        ureq::Error::Tls(why) => Error::Offline(format!("secure connection failed: {why}")),
        other => Error::Offline(other.to_string()),
    }
}

/// A release asset: where to get it and what it must be.
#[derive(Clone, Debug, PartialEq)]
pub struct Asset {
    pub url: String,
    pub size: u64,
    /// Lower case hex, when the release lists one.
    pub sha256: Option<String>,
    pub tag: String,
}

fn agent() -> ureq::Agent {
    // The operating system's certificates, so a proxy or antivirus that re-signs TLS works as it does in a browser.
    let tls = ureq::tls::TlsConfig::builder().root_certs(ureq::tls::RootCerts::PlatformVerifier).build();
    ureq::Agent::config_builder()
        .tls_config(tls)
        .http_status_as_error(false)
        // Followed by `get`, which leaves a redirect's body unread. Some proxies garble the empty chunked body of
        // GitHub's redirects, and ureq fails reading it when it follows them itself.
        .max_redirects(0)
        .timeout_connect(Some(Duration::from_secs(20)))
        .timeout_recv_response(Some(Duration::from_secs(60)))
        .user_agent(format!("GodotTrench/{}", crate::VERSION))
        .build()
        .into()
}

/// GETs `url`, following up to five redirects, never from https to http.
fn get(agent: &ureq::Agent, url: &str, accept: &str) -> Result<ureq::http::Response<ureq::Body>, Error> {
    let mut url = url.to_string();
    for _ in 0..5 {
        let response = agent.get(&url).header("Accept", accept).call().map_err(net_error)?;
        if !response.status().is_redirection() {
            return Ok(response);
        }

        let next = response.headers().get("location").and_then(|l| l.to_str().ok()).unwrap_or_default().to_string();
        if !next.starts_with("https://") && !(next.starts_with("http://") && url.starts_with("http://")) {
            return Err(Error::Damaged(format!("the server sent it on to {next:?}")));
        }

        url = next;
    }

    Err(Error::Damaged("the server sent it on too many times".into()))
}

/// Finds `name` in the release of this editor version, else in the rolling beta.
pub fn find_asset(agent: &ureq::Agent, api: &str, name: &'static str) -> Result<Asset, Error> {
    for tag in [format!("v{}", crate::VERSION), "beta".to_string()] {
        let response = get(agent, &format!("{api}/releases/tags/{tag}"), "application/vnd.github+json")?;
        match response.status().as_u16() {
            200 => {}
            404 => continue,
            code => return Err(Error::Status(code)),
        }

        let release: serde_json::Value = serde_json::from_reader(response.into_body().into_with_config().limit(8 << 20).reader())
            .map_err(|e| Error::Damaged(format!("the release description: {e}")))?;
        let Some(asset) = release["assets"].as_array().and_then(|a| a.iter().find(|a| a["name"] == name)) else { continue };
        let (Some(url), Some(size)) = (asset["browser_download_url"].as_str(), asset["size"].as_u64()) else {
            return Err(Error::Damaged(format!("the release lists {name} without a link or size")));
        };
        let sha256 = asset["digest"].as_str().and_then(|d| d.strip_prefix("sha256:")).map(str::to_ascii_lowercase);
        return Ok(Asset { url: url.to_string(), size, sha256, tag });
    }

    Err(Error::NotPublished(name))
}

/// What a running install shows: a sentence and how far along it is.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Progress {
    pub stage: String,
    pub done: u64,
    pub total: u64,
    /// `done` and `total` count bytes, else files.
    pub bytes: bool,
}

impl Progress {
    pub fn fraction(&self) -> f32 {
        if self.total == 0 { 0.0 } else { (self.done as f64 / self.total as f64).clamp(0.0, 1.0) as f32 }
    }

    pub fn amount(&self) -> String {
        match (self.total, self.bytes) {
            (0, _) => String::new(),
            (_, true) => format!("{} of {}", megabytes(self.done), megabytes(self.total)),
            (_, false) => format!("{} of {} files", self.done, self.total),
        }
    }
}

pub fn megabytes(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / 1e6)
}

/// Shares [`Progress`] with the UI thread, waking it at most ten times a second.
#[derive(Clone, Default)]
pub struct Reporter {
    progress: Arc<Mutex<Progress>>,
    cancel: Arc<AtomicBool>,
    repaint: Option<egui::Context>,
}

impl Reporter {
    fn set(&self, f: impl FnOnce(&mut Progress)) {
        if let Ok(mut p) = self.progress.lock() {
            f(&mut p);
        }

        if let Some(ctx) = &self.repaint {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    fn check(&self) -> Result<(), Error> {
        if self.cancel.load(Ordering::Relaxed) { Err(Error::Cancelled) } else { Ok(()) }
    }
}

/// Downloads `asset` into `to`, checking its size and checksum.
pub fn download(agent: &ureq::Agent, asset: &Asset, to: &Path, report: &Reporter) -> Result<(), Error> {
    let response = get(agent, &asset.url, "application/octet-stream")?;
    match response.status().as_u16() {
        200 => {}
        404 => return Err(Error::Damaged("the release lists it, but the file is gone".into())),
        code => return Err(Error::Status(code)),
    }

    let length = response.headers().get("content-length").and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok());
    if length.is_some_and(|l| l != asset.size) {
        return Err(Error::Damaged(format!("the server sends {} instead of {}", megabytes(length.unwrap_or(0)), megabytes(asset.size))));
    }

    let name = asset.url.rsplit('/').next().unwrap_or_default().to_string();
    report.set(|p| *p = Progress { stage: format!("Downloading {name} from the {} release", asset.tag), done: 0, total: asset.size, bytes: true });
    let mut body = response.into_body().into_with_config().limit(asset.size.saturating_add(1)).reader();
    let mut file = std::fs::File::create(to).map_err(|e| io_error(e, to))?;
    let mut hash = ring::digest::Context::new(&ring::digest::SHA256);
    let mut buf = vec![0u8; 256 * 1024];
    let mut done = 0u64;
    loop {
        report.check()?;
        let n = match body.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(Error::Offline(format!("the download broke off after {}: {e}", megabytes(done)))),
        };
        hash.update(&buf[..n]);
        file.write_all(&buf[..n]).map_err(|e| io_error(e, to))?;
        done += n as u64;
        report.set(|p| p.done = done);
    }

    file.flush().map_err(|e| io_error(e, to))?;
    if done != asset.size {
        return Err(Error::Damaged(format!("it has {} instead of {}", megabytes(done), megabytes(asset.size))));
    }

    let sum: String = hash.finish().as_ref().iter().map(|b| format!("{b:02x}")).collect();
    if asset.sha256.as_ref().is_some_and(|expected| *expected != sum) {
        return Err(Error::Damaged("its checksum does not match the release".into()));
    }

    Ok(())
}

/// The project path an archive entry unpacks to, or None when the name is absolute, climbs out with `..`, uses
/// backslashes or drive letters, or lies outside `folders`.
pub fn entry_path(name: &str, folders: &[&str]) -> Option<PathBuf> {
    if name.is_empty() || name.contains(['\\', ':', '\0']) || name.starts_with('/') {
        return None;
    }

    let parts: Vec<&str> = name.trim_end_matches('/').split('/').collect();
    if parts.iter().any(|p| p.is_empty() || *p == "." || *p == "..") {
        return None;
    }

    let inside = folders.iter().any(|f| {
        let folder: Vec<&str> = f.split('/').collect();
        parts.len() > folder.len() && parts[..folder.len()] == folder[..]
    });
    let path: PathBuf = parts.iter().collect();
    (inside && path.components().all(|c| matches!(c, Component::Normal(_)))).then_some(path)
}

/// Creates `rel` below `root` one folder at a time, refusing a folder that is a link leading out of the project.
fn make_dirs(root: &Path, real_root: &Path, rel: &Path) -> Result<(), Error> {
    let mut dir = root.to_path_buf();
    for part in rel.components() {
        dir.push(part);
        match std::fs::symlink_metadata(&dir) {
            Ok(meta) if meta.file_type().is_symlink() => {
                let real = dir.canonicalize().map_err(|e| io_error(e, &dir))?;
                if !real.starts_with(real_root) || !real.is_dir() {
                    return Err(Error::Unsafe(format!("{}, a link that leads out of the project", dir.display())));
                }
            }
            Ok(meta) if meta.is_dir() => {}
            Ok(_) => return Err(Error::Io(format!("Could not add files to {}, it is a file and not a folder", dir.display()))),
            Err(_) => std::fs::create_dir(&dir).map_err(|e| io_error(e, &dir))?,
        }
    }

    Ok(())
}

/// Files an install added, and how many it left alone because the project already had them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Added {
    pub written: Vec<PathBuf>,
    pub kept: usize,
}

/// Unpacks the files of `archive` that lie in `folders` into `root`. Every name is checked before anything is written,
/// files the project has are kept, links in the archive are refused, and each file appears under its name only once
/// it is complete.
pub fn extract(archive: &Path, root: &Path, folders: &[&str], report: &Reporter) -> Result<Added, Error> {
    let file = std::fs::File::open(archive).map_err(|e| io_error(e, archive))?;
    let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file)).map_err(|e| Error::Damaged(e.to_string()))?;
    let mut files = Vec::new();
    let mut total = 0u64;
    for i in 0..zip.len() {
        let entry = zip.by_index_raw(i).map_err(|e| Error::Damaged(e.to_string()))?;
        if entry.is_dir() {
            continue;
        }

        let rel = entry_path(entry.name(), folders).filter(|_| !entry.is_symlink()).ok_or_else(|| Error::Unsafe(entry.name().to_string()))?;
        total = total.saturating_add(entry.size());
        files.push((i, rel));
    }

    if total > MAX_UNPACKED {
        return Err(Error::Damaged(format!("it would unpack to {}", megabytes(total))));
    }

    std::fs::create_dir_all(root).map_err(|e| io_error(e, root))?;
    let real_root = root.canonicalize().map_err(|e| io_error(e, root))?;
    let count = files.len() as u64;
    report.set(|p| *p = Progress { stage: format!("Adding files to the project from {}", archive_name(archive)), done: 0, total: count, bytes: false });
    let mut added = Added::default();
    for (n, (i, rel)) in files.into_iter().enumerate() {
        report.check()?;
        let target = root.join(&rel);
        if std::fs::symlink_metadata(&target).is_ok() {
            added.kept += 1;
        } else {
            make_dirs(root, &real_root, rel.parent().unwrap_or(Path::new("")))?;
            let mut entry = zip.by_index(i).map_err(|e| Error::Damaged(e.to_string()))?;
            gt_formats::write_atomic(&target, &mut entry).map_err(|e| io_error(e, &target))?;
            added.written.push(target);
        }

        report.set(|p| p.done = n as u64 + 1);
    }

    Ok(added)
}

fn archive_name(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

/// The name a Godot project gives itself in `project.godot`, else its folder's name.
pub fn project_name(root: &Path) -> String {
    let text = std::fs::read_to_string(root.join("project.godot")).unwrap_or_default();
    let named = text.lines().find_map(|l| l.trim().strip_prefix("config/name=")).map(|v| v.trim().trim_matches('"').to_string());
    named.filter(|n| !n.is_empty()).unwrap_or_else(|| root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default())
}

/// What an install did, and why it stopped early if it did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outcome {
    pub choice: Choice,
    pub added: Added,
    /// The releases the downloads came from.
    pub tags: Vec<String>,
    pub error: Option<Error>,
}

impl Outcome {
    /// One or two sentences for the wizard and the status bar.
    pub fn summary(&self) -> String {
        let written = self.added.written.len();
        let from = match self.tags.first() {
            Some(tag) if tag == "beta" => " from the rolling beta",
            Some(_) => " from the release",
            None => "",
        };
        let mut text = match (written, self.added.kept) {
            (0, 0) => String::new(),
            (0, kept) => format!("The project already had all {kept} files, nothing was replaced."),
            (n, 0) => format!("Added {n} files to the project{from}."),
            (n, kept) => format!("Added {n} files to the project{from} and kept the {kept} it already had."),
        };
        if let Some(e) = &self.error {
            if !text.is_empty() {
                text.push(' ');
            }

            text.push_str(&e.to_string());
        }

        text
    }
}

/// Installs `choice` into the project at `root`, one download after the other, stopping at the first error and keeping
/// what was added. When a download fails, the embedded Blockbench models are added instead, they need no network.
pub fn install(root: &Path, choice: Choice, api: &str, report: &Reporter) -> Outcome {
    let mut outcome = Outcome { choice, ..Default::default() };
    let agent = agent();
    for download in choice.downloads() {
        report.set(|p| *p = Progress { stage: format!("Looking up {} in the releases", download.asset), ..Default::default() });
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let folder = std::env::temp_dir().join(format!("godottrench-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        let result = std::fs::create_dir_all(&folder)
            .map_err(|e| io_error(e, &folder))
            .and_then(|_| find_asset(&agent, api, download.asset))
            .and_then(|asset| download_and_extract(&agent, &asset, &folder.join(download.asset), root, download, report).map(|added| (asset.tag, added)));
        let _ = std::fs::remove_dir_all(&folder);
        match result {
            Ok((tag, added)) => {
                outcome.added.written.extend(added.written);
                outcome.added.kept += added.kept;
                outcome.tags.push(tag);
            }
            Err(e) => {
                outcome.error = Some(e);
                break;
            }
        }
    }

    if outcome.error.as_ref().is_some_and(|e| *e != Error::Cancelled) {
        let nature = nature_dir(root);
        if let Ok(written) = gt_formats::nature::install(&nature, &[], false) {
            outcome.added.written.extend(written);
        }
    }

    outcome
}

fn download_and_extract(agent: &ureq::Agent, asset: &Asset, part: &Path, root: &Path, what: &Download, report: &Reporter) -> Result<Added, Error> {
    download(agent, asset, part, report)?;
    extract(part, root, what.folders, report)
}

/// [`RELEASES_API`], or the server [`API_ENV`] names.
pub fn api_url() -> String {
    std::env::var(API_ENV).ok().filter(|v| !v.is_empty()).unwrap_or_else(|| RELEASES_API.to_string())
}

/// An install running on its own thread.
pub struct Job {
    pub choice: Choice,
    pub root: PathBuf,
    report: Reporter,
    rx: mpsc::Receiver<Outcome>,
}

impl Job {
    pub fn start(root: PathBuf, choice: Choice, api: String, repaint: Option<egui::Context>) -> Job {
        let report = Reporter { repaint, ..Default::default() };
        let (tx, rx) = mpsc::channel();
        let (thread_root, thread_report) = (root.clone(), report.clone());
        std::thread::spawn(move || {
            let outcome = install(&thread_root, choice, &api, &thread_report);
            let _ = tx.send(outcome);
            if let Some(ctx) = &thread_report.repaint {
                ctx.request_repaint();
            }
        });
        Job { choice, root, report, rx }
    }

    pub fn progress(&self) -> Progress {
        self.report.progress.lock().map(|p| p.clone()).unwrap_or_default()
    }

    /// Asks the thread to stop. A read that hangs on a dead connection only notices once it returns, so the caller
    /// should not wait for it.
    pub fn cancel(&self) {
        self.report.cancel.store(true, Ordering::Relaxed);
    }

    pub fn poll(&self) -> Option<Outcome> {
        match self.rx.try_recv() {
            Ok(outcome) => Some(outcome),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Outcome { choice: self.choice, error: Some(Error::Io("The install stopped unexpectedly.".into())), ..Default::default() })
            }
        }
    }
}

/// A local stand-in for the GitHub release API and its downloads, so tests never touch the network.
#[cfg(test)]
pub(crate) mod test_server {
    use std::io::{BufRead, BufReader, Write};
    use std::sync::{Arc, Mutex};

    /// Path, status and body. A 302 redirects to the url in its body.
    type Routes = Arc<Mutex<Vec<(String, u16, Vec<u8>)>>>;

    pub struct Server {
        pub url: String,
        routes: Routes,
        pub requests: Arc<Mutex<Vec<String>>>,
    }

    impl Server {
        pub fn start() -> Server {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let url = format!("http://{}", listener.local_addr().unwrap());
            let routes: Routes = Default::default();
            let requests: Arc<Mutex<Vec<String>>> = Default::default();
            let (served, log) = (routes.clone(), requests.clone());
            std::thread::spawn(move || {
                for stream in listener.incoming() {
                    let Ok(mut stream) = stream else { continue };
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut line = String::new();
                    let _ = reader.read_line(&mut line);
                    loop {
                        let mut header = String::new();
                        if reader.read_line(&mut header).unwrap_or(0) == 0 || header == "\r\n" {
                            break;
                        }
                    }

                    let path = line.split(' ').nth(1).unwrap_or_default().to_string();
                    log.lock().unwrap().push(path.clone());
                    let found = served.lock().unwrap().iter().find(|(p, _, _)| *p == path).map(|(_, s, b)| (*s, b.clone()));
                    let (status, body) = found.unwrap_or((404, b"{\"message\": \"Not Found\"}".to_vec()));
                    if status == 302 {
                        // Like GitHub: a redirect with an empty chunked body.
                        let to = String::from_utf8_lossy(&body);
                        let _ = write!(stream, "HTTP/1.1 302 Found\r\nLocation: {to}\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n0\r\n\r\n");
                        continue;
                    }

                    let _ = write!(stream, "HTTP/1.1 {status} X\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
                    let _ = stream.write_all(&body);
                }
            });
            Server { url, routes, requests }
        }

        pub fn route(&self, path: &str, status: u16, body: impl Into<Vec<u8>>) {
            self.routes.lock().unwrap().retain(|(p, _, _)| p != path);
            self.routes.lock().unwrap().push((path.to_string(), status, body.into()));
        }

        /// A release `tag` with `assets` as (name, bytes), each downloadable from this server, with its digest when
        /// `digest` is set.
        pub fn release(&self, tag: &str, assets: &[(&str, &[u8])], digest: bool) {
            let listed: Vec<serde_json::Value> = assets
                .iter()
                .map(|(name, bytes)| {
                    let path = format!("/download/{tag}/{name}");
                    let stored = format!("/assets/{tag}/{name}");
                    self.route(&path, 302, format!("{}{stored}", self.url));
                    self.route(&stored, 200, bytes.to_vec());
                    let sum: String = ring::digest::digest(&ring::digest::SHA256, bytes).as_ref().iter().map(|b| format!("{b:02x}")).collect();
                    let digest = digest.then(|| format!("sha256:{sum}"));
                    serde_json::json!({ "name": name, "size": bytes.len(), "digest": digest, "browser_download_url": format!("{}{path}", self.url) })
                })
                .collect();
            self.route(&format!("/releases/tags/{tag}"), 200, serde_json::json!({ "tag_name": tag, "assets": listed }).to_string());
        }
    }

    /// A zip archive of (name, contents), stored or deflated.
    pub fn zip(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (i, (name, bytes)) in files.iter().enumerate() {
            let method = if i % 2 == 0 { zip::CompressionMethod::Deflated } else { zip::CompressionMethod::Stored };
            out.start_file(*name, zip::write::SimpleFileOptions::default().compression_method(method)).unwrap();
            out.write_all(bytes).unwrap();
        }

        out.finish().unwrap().into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::test_server::{Server, zip};
    use super::*;

    fn project(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("gt_content_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("project.godot"), "config_version=5\n").unwrap();
        dir
    }

    /// Every file under `dir`, relative and with forward slashes.
    fn files(dir: &Path) -> Vec<String> {
        fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
            for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(root, &path, out);
                } else {
                    out.push(path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
                }
            }
        }

        let mut out = Vec::new();
        walk(dir, dir, &mut out);
        out.sort();
        out
    }

    #[test]
    fn entry_names_outside_the_folders_are_refused() {
        let folders = ["godottrench/nature"];
        assert_eq!(entry_path("godottrench/nature/trees/oak.glb", &folders), Some(PathBuf::from("godottrench/nature/trees/oak.glb")));
        for bad in [
            "project.godot",
            "addons/func_godot/plugin.cfg",
            "godottrench/nature",
            "godottrench/naturex/a.glb",
            "godottrench/nature/../../project.godot",
            "godottrench/nature/./a.glb",
            "/etc/passwd",
            "godottrench\\nature\\a.glb",
            "C:/godottrench/nature/a.glb",
            "godottrench//nature/a.glb",
            "",
        ] {
            assert_eq!(entry_path(bad, &folders), None, "{bad}");
        }

        assert!(entry_path("models/polyhaven/barrel.gltf", DEMO_CONTENT.folders).is_some());
        assert!(entry_path("tests/run_tests.gd", DEMO_CONTENT.folders).is_none());
    }

    #[test]
    fn installs_from_the_beta_when_the_version_release_lacks_the_asset() {
        let server = Server::start();
        let pack = zip(&[
            ("godottrench/nature/pack.json", b"{}"),
            ("godottrench/nature/trees/pack.json", b"{}"),
            ("godottrench/nature/trees/oak.glb", b"glTF oak"),
            ("godottrench/nature/rock.bbmodel", b"from the archive"),
        ]);
        server.release(&format!("v{}", crate::VERSION), &[("godottrench-linux-x86_64.tar.gz", b"editor")], true);
        server.release("beta", &[(NATURE_PACK.asset, &pack)], true);
        let root = project("beta");
        std::fs::create_dir_all(root.join("godottrench/nature/trees")).unwrap();
        std::fs::write(root.join("godottrench/nature/trees/pack.json"), "mine").unwrap();

        let outcome = install(&root, Choice::Nature, &server.url, &Reporter::default());
        assert_eq!(outcome.error, None);
        assert_eq!(outcome.tags, ["beta"]);
        let installed = files(&root);
        assert!(installed.contains(&"godottrench/nature/trees/oak.glb".to_string()), "{installed:?}");
        assert!(installed.iter().all(|f| !f.ends_with(".gtpart")), "{installed:?}");
        assert_eq!(installed.iter().filter(|f| f.ends_with(".bbmodel")).count(), 1, "the embedded models are only a fallback");
        assert_eq!(std::fs::read_to_string(root.join("godottrench/nature/trees/pack.json")).unwrap(), "mine", "a file already there is kept");
        assert_eq!(std::fs::read_to_string(root.join("godottrench/nature/rock.bbmodel")).unwrap(), "from the archive");
        assert_eq!((outcome.added.written.len(), outcome.added.kept), (3, 1));
        assert!(outcome.summary().starts_with("Added 3 files to the project from the rolling beta and kept the 1"), "{}", outcome.summary());

        let again = install(&root, Choice::Nature, &server.url, &Reporter::default());
        assert_eq!((again.added.written.len(), again.added.kept, again.error.clone()), (0, 4, None));
        assert!(again.summary().starts_with("The project already had all 4 files"), "{}", again.summary());
        assert!(server.requests.lock().unwrap().iter().all(|r| !r.contains("..")));
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn damaged_and_unsafe_archives_add_nothing() {
        let server = Server::start();
        let root = project("unsafe");
        let good = zip(&[("demo/maps/a.gtm", b"map")]);
        let check = |archive: &[u8], digest: bool| {
            server.release("beta", &[(DEMO_CONTENT.asset, archive)], digest);
            let agent = agent();
            let asset = find_asset(&agent, &server.url, DEMO_CONTENT.asset).unwrap();
            let part = root.join("download.part");
            let result = download(&agent, &asset, &part, &Reporter::default()).and_then(|_| extract(&part, &root, DEMO_CONTENT.folders, &Reporter::default()));
            let _ = std::fs::remove_file(&part);
            result
        };

        for (bad, why) in [
            (zip(&[("demo/ok.txt", b"ok"), ("../evil.txt", b"x")]), "climbs out"),
            (zip(&[("demo/ok.txt", b"ok"), ("project.godot", b"x")]), "the project file"),
            (zip(&[("demo/ok.txt", b"ok"), ("addons/func_godot/plugin.cfg", b"x")]), "the addon"),
        ] {
            assert!(matches!(check(&bad, true), Err(Error::Unsafe(_))), "{why}");
            assert_eq!(files(&root), ["project.godot"], "{why}: nothing is written");
        }

        let mut link = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        link.add_symlink("demo/link", "/etc", zip::write::SimpleFileOptions::default()).unwrap();
        assert!(matches!(check(&link.finish().unwrap().into_inner(), true), Err(Error::Unsafe(_))), "links are refused");

        // The release promises one file and the server sends another.
        let asset = format!("/assets/beta/{}", DEMO_CONTENT.asset);
        server.release("beta", &[(DEMO_CONTENT.asset, &good)], true);
        let mut other = good.clone();
        *other.last_mut().unwrap() ^= 1;
        server.route(&asset, 200, other);
        let agent = agent();
        let found = find_asset(&agent, &server.url, DEMO_CONTENT.asset).unwrap();
        let part = root.join("download.part");
        assert!(matches!(download(&agent, &found, &part, &Reporter::default()), Err(Error::Damaged(why)) if why.contains("checksum")));
        server.route(&asset, 200, &good[..good.len() - 1]);
        assert!(matches!(download(&agent, &found, &part, &Reporter::default()), Err(Error::Damaged(_))), "cut short");
        let _ = std::fs::remove_file(&part);

        // A zip with a corrupt entry fails its CRC check, the files before it stay and no partial file is left behind.
        let mut corrupt = zip(&[("demo/maps/a.gtm", b"deflated"), ("demo/maps/b.gtm", &[7u8; 64])]);
        let at = corrupt.windows(64).position(|w| w == [7u8; 64]).expect("the stored bytes");
        corrupt[at] = 8;
        std::fs::write(&part, &corrupt).unwrap();
        assert!(matches!(extract(&part, &root, DEMO_CONTENT.folders, &Reporter::default()), Err(Error::Damaged(_))));
        std::fs::remove_file(&part).unwrap();
        assert_eq!(files(&root), ["demo/maps/a.gtm", "project.godot"]);
        std::fs::remove_dir_all(root.join("demo")).unwrap();

        assert_eq!(check(&good, false).unwrap().written, [root.join("demo/maps/a.gtm")], "without a digest the size still has to match");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn missing_releases_and_no_network_say_what_is_wrong() {
        let server = Server::start();
        let agent = agent();
        assert_eq!(find_asset(&agent, &server.url, NATURE_PACK.asset), Err(Error::NotPublished(NATURE_PACK.asset)));
        let requests = server.requests.lock().unwrap().clone();
        assert_eq!(requests, [format!("/releases/tags/v{}", crate::VERSION), "/releases/tags/beta".to_string()]);
        server.route("/releases/tags/beta", 403, "rate limited");
        assert_eq!(find_asset(&agent, &server.url, NATURE_PACK.asset), Err(Error::Status(403)));
        assert!(Error::Status(403).to_string().contains("Try again later"));

        let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
        let offline = find_asset(&agent, &format!("http://{closed}"), NATURE_PACK.asset);
        assert!(matches!(&offline, Err(Error::Offline(_))), "{offline:?}");
        assert!(offline.unwrap_err().to_string().starts_with("Could not reach GitHub"));

        let root = project("offline");
        let outcome = install(&root, Choice::Nature, &format!("http://{closed}"), &Reporter::default());
        assert_eq!(outcome.added.written.len(), 18, "the embedded models need no network");
        assert!(outcome.summary().starts_with("Added 18 files to the project. Could not reach GitHub"), "{}", outcome.summary());
        assert!(!has_nature_pack(&root));
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_job_reports_progress_and_can_be_cancelled() {
        let server = Server::start();
        let big = vec![3u8; 3 << 20];
        let pack = zip(&[("godottrench/nature/trees/pack.json", b"{}"), ("godottrench/nature/textures/big.png", &big)]);
        server.release("beta", &[(NATURE_PACK.asset, &pack)], true);
        let root = project("job");
        let job = Job::start(root.clone(), Choice::Nature, server.url.clone(), None);
        let outcome = loop {
            if let Some(o) = job.poll() {
                break o;
            }

            std::thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(outcome.error, None);
        assert_eq!(job.progress().done, 2, "the last stage counted both files");
        assert!(root.join("godottrench/nature/textures/big.png").is_file());

        let cancelled = Reporter::default();
        cancelled.cancel.store(true, Ordering::Relaxed);
        let other = project("cancel");
        let outcome = install(&other, Choice::Demo, &server.url, &cancelled);
        assert_eq!(outcome.error, Some(Error::Cancelled));
        assert!(!other.join("godottrench").exists(), "cancelled means nothing more, not even the fallback");
        assert_eq!(project_name(&other), format!("gt_content_cancel_{}", std::process::id()));
        std::fs::write(other.join("project.godot"), "[application]\n\nconfig/name=\"My Game\"\n").unwrap();
        assert_eq!(project_name(&other), "My Game");
        std::fs::remove_dir_all(&root).unwrap();
        std::fs::remove_dir_all(&other).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn a_link_leading_out_of_the_project_is_not_followed() {
        let root = project("link");
        let outside = project("outside");
        std::os::unix::fs::symlink(&outside, root.join("demo")).unwrap();
        let archive = root.join("demo.zip");
        std::fs::write(&archive, zip(&[("demo/maps/a.gtm", b"map")])).unwrap();
        assert!(matches!(extract(&archive, &root, DEMO_CONTENT.folders, &Reporter::default()), Err(Error::Unsafe(_))));
        assert_eq!(files(&outside), ["project.godot"]);

        std::fs::remove_file(root.join("demo")).unwrap();
        std::fs::create_dir_all(root.join("mine")).unwrap();
        std::os::unix::fs::symlink(root.join("mine"), root.join("demo")).unwrap();
        assert_eq!(extract(&archive, &root, DEMO_CONTENT.folders, &Reporter::default()).unwrap().written.len(), 1, "a link inside the project is fine");
        assert!(root.join("mine/maps/a.gtm").is_file());
        std::fs::remove_dir_all(&root).unwrap();
        std::fs::remove_dir_all(&outside).unwrap();
    }

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// Every file the publish job puts into `download`, by the zip command in ci.yml: the folders it names, from inside
    /// `godot`, leaving out what the command excludes.
    fn archived(download: &Download) -> Vec<String> {
        let ci = std::fs::read_to_string(repo().join(".github/workflows/ci.yml")).unwrap();
        let line = ci
            .lines()
            .find(|l| l.contains(&format!("{}\"", download.asset)) && l.contains("zip -qr"))
            .unwrap_or_else(|| panic!("ci.yml zips {}", download.asset));
        assert!(line.contains("cd godot &&"), "{line}");
        let args = line.split(&format!("{}\"", download.asset)).nth(1).unwrap();
        let (folders, excluded) = args.split_once(" -x ").unwrap_or((args, ""));
        let folders: Vec<&str> = folders.trim().trim_end_matches(')').split_whitespace().collect();
        assert_eq!(folders, download.folders, "ci.yml zips the folders the installer accepts");
        assert!(excluded.contains("*.import") == (download == &NATURE_PACK), "{line}");
        let mut out = Vec::new();
        for folder in download.folders {
            out.extend(files(&repo().join("godot").join(folder)).into_iter().map(|f| format!("{folder}/{f}")));
        }

        // A fresh checkout, which the publish job works from, has none of the ignored files Godot writes on import.
        out.retain(|f| !(download == &NATURE_PACK && f.ends_with(".import")));
        out
    }

    #[test]
    fn the_release_archives_hold_what_the_installer_expects() {
        let pack = archived(&NATURE_PACK);
        for name in &pack {
            assert!(entry_path(name, NATURE_PACK.folders).is_some(), "{name}");
        }

        let nature = |rel: &str| format!("godottrench/nature/{rel}");
        for preset in gt_doc::scatter::PRESETS {
            for item in gt_doc::scatter::preset(preset).unwrap().1 {
                let rel = item.source.trim_start_matches("res://").to_string();
                assert!(pack.contains(&rel), "{preset}: {rel} is not in the nature pack");
            }
        }

        for folder in gt_formats::nature::PACK_FOLDERS.iter().filter(|f| **f != "textures") {
            assert!(pack.contains(&nature(&format!("{folder}/pack.json"))), "pack_installed looks for {folder}/pack.json");
        }

        // The demo only refers to itself, the nature pack and the addon, so it works in any project.
        let demo = archived(&DEMO_CONTENT);
        let shipped = |res: &str| demo.iter().chain(&pack).any(|f| f == res) || res.starts_with("addons/func_godot/") || res.starts_with(".godot/");
        let mut refs = 0;
        for name in demo.iter().filter(|f| !f.ends_with(".md")) {
            let mut bytes = std::fs::read(repo().join("godot").join(name)).unwrap();
            if name.ends_with(".gd") {
                let text = String::from_utf8(bytes).unwrap();
                bytes = text.lines().filter(|l| !l.trim_start().starts_with('#')).collect::<Vec<_>>().join("\n").into_bytes();
            }

            for at in bytes.windows(6).enumerate().filter(|(_, w)| *w == b"res://").map(|(i, _)| i + 6) {
                let end = bytes[at..].iter().position(|b| !(b.is_ascii_alphanumeric() || b"_-./".contains(b))).map_or(bytes.len(), |n| at + n);
                let res = String::from_utf8_lossy(&bytes[at..end]).trim_end_matches('.').to_string();
                if res.contains('.') {
                    refs += 1;
                    assert!(shipped(&res), "{name} refers to res://{res}, which neither download ships");
                }
            }
        }

        assert!(refs > 300, "{refs}");
    }

    /// The wizard shows these sizes before it asks the release. The glTF models and the text files compress, the jpg
    /// and png textures hardly do, so the zips come to 70 to 90 percent of the files.
    #[test]
    fn the_sizes_shown_match_the_folders() {
        for download in [NATURE_PACK, DEMO_CONTENT] {
            let bytes: u64 = archived(&download).iter().map(|f| std::fs::metadata(repo().join("godot").join(f)).unwrap().len()).sum();
            let mb = bytes as f64 / 1e6;
            assert!((mb * 0.6..mb * 0.95).contains(&(download.approx_mb as f64)), "{}: {mb:.0} MB of files", download.asset);
        }
    }
}
