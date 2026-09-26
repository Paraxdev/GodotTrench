//! Ready made content for a Godot project: the nature pack and the demo. The Blockbench nature models are embedded in
//! the editor, the rest is downloaded from the GitHub release that matches the editor, or from the rolling beta, and
//! unpacked into the project without replacing any file.

use std::io::{Read, Seek, Write};
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

/// Refuses archives that would unpack to more than this. No entry may give more than its header claims.
const MAX_UNPACKED: u64 = 4 << 30;

/// A download that brings nothing new for this long has stalled.
const STALL: Duration = if cfg!(test) { Duration::from_millis(600) } else { Duration::from_secs(60) };

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
    /// A release has no such download.
    NotPublished(&'static str),
    /// Neither release was found, which happens while CI replaces the rolling beta.
    NoRelease,
    Status(u16),
    /// The download or the archive is not what the release promised.
    Damaged(String),
    DiskFull,
    /// The archive names a path outside the folders it may add to, or one Windows cannot hold. Nothing from it is
    /// written then.
    Unsafe(String),
    /// A folder the archive adds to is a link leading out of the project. Nothing from the archive is written then.
    Link(PathBuf),
    Io(String),
    Cancelled,
}

impl Error {
    /// The download went wrong, rather than being cancelled, refused or failing to write into the project.
    pub fn is_download(&self) -> bool {
        matches!(self, Error::Offline(_) | Error::NotPublished(_) | Error::NoRelease | Error::Status(_) | Error::Damaged(_))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Offline(why) => write!(f, "Could not reach GitHub ({why}). Check the internet connection and try again."),
            Error::NotPublished(asset) => {
                write!(f, "{asset} is not published yet, neither with the v{} release nor with the rolling beta.", crate::VERSION)
            }
            Error::NoRelease => write!(
                f,
                "Neither the v{} release nor the rolling beta was found. The beta is replaced after each change, try again in a few minutes.",
                crate::VERSION
            ),
            Error::Status(code @ (403 | 429)) => write!(f, "GitHub refused the request ({code}), it limits how often one address may ask. Try again later."),
            Error::Status(code) => write!(f, "GitHub answered with error {code}. Try again later."),
            Error::Damaged(why) => write!(f, "The download is damaged: {why}. Try again."),
            Error::DiskFull => write!(f, "The disk is full. Free some space and try again, the files added so far stay."),
            Error::Unsafe(path) => write!(f, "The archive holds {path}, which it may not add to the project, so nothing from it was added."),
            Error::Link(path) => write!(f, "{} is a link that leads out of the project, so nothing from the archive was added.", path.display()),
            Error::Io(why) => write!(f, "{why}"),
            Error::Cancelled => write!(f, "Cancelled. The files added so far stay, installing again adds the rest."),
        }
    }
}

pub(crate) fn io_error(e: std::io::Error, what: &Path) -> Error {
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
        // Only the system's certificates are used, a missing CA bundle or a proxy it does not trust ends up here.
        ureq::Error::Tls(why) => Error::Offline(format!("the secure connection failed, check that the system trusts GitHub's certificate: {why}")),
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

pub(crate) fn agent() -> ureq::Agent {
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
        // Frees the connection of a download given up on as stalled, see STALL.
        .timeout_recv_body(Some(Duration::from_secs(6 * 3600)))
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

/// Whether CI built this editor for the rolling beta, see `GODOTTRENCH_CHANNEL` in ci.yml.
pub fn beta_build() -> bool {
    option_env!("GODOTTRENCH_CHANNEL") == Some("beta")
}

/// The releases to look in, in order. A beta carries the version of the last release, so it looks in the rolling beta
/// first, or it would get that release's older downloads.
pub fn release_tags() -> [String; 2] {
    let version = format!("v{}", crate::VERSION);
    if beta_build() { ["beta".into(), version] } else { [version, "beta".into()] }
}

/// Finds `name` in the release of this editor version and the rolling beta, see [`release_tags`]. The GitHub API has to
/// link it over https with its checksum, a server [`API_ENV`] names only over https when it is https itself.
pub fn find_asset(agent: &ureq::Agent, api: &str, name: &'static str) -> Result<Asset, Error> {
    let mut found_release = false;
    for tag in release_tags() {
        let response = get(agent, &format!("{api}/releases/tags/{tag}"), "application/vnd.github+json")?;
        match response.status().as_u16() {
            200 => found_release = true,
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
        trusted(api, name, url, sha256.is_some())?;
        return Ok(Asset { url: url.to_string(), size, sha256, tag });
    }

    Err(if found_release { Error::NotPublished(name) } else { Error::NoRelease })
}

/// Refuses a download the release links without https while the API itself is https, or lists without its checksum on
/// GitHub. A server [`API_ENV`] names may leave the checksum out.
fn trusted(api: &str, name: &str, url: &str, checksum: bool) -> Result<(), Error> {
    if api.starts_with("https://") && !url.starts_with("https://") {
        return Err(Error::Damaged(format!("the release links {name} without https")));
    }

    if api == RELEASES_API && !checksum {
        return Err(Error::Damaged(format!("the release lists no checksum for {name}")));
    }

    Ok(())
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
    pub fn new(repaint: Option<egui::Context>) -> Reporter {
        Reporter { repaint, ..Default::default() }
    }

    pub fn progress(&self) -> Progress {
        self.progress.lock().map(|p| p.clone()).unwrap_or_default()
    }

    /// Asks the work to stop. It notices within a fraction of a second, or after the file it is writing.
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    pub fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    pub(crate) fn set(&self, f: impl FnOnce(&mut Progress)) {
        if let Ok(mut p) = self.progress.lock() {
            f(&mut p);
        }

        if let Some(ctx) = &self.repaint {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    pub(crate) fn check(&self) -> Result<(), Error> {
        if self.cancelled() { Err(Error::Cancelled) } else { Ok(()) }
    }
}

/// Downloads `asset` into a file without a name in the temporary folder, checking its size and checksum, and returns it
/// rewound for [`extract`]. Nothing else can open or replace the file, and it goes away once dropped, even after a crash.
pub fn download(agent: &ureq::Agent, asset: &Asset, report: &Reporter) -> Result<std::fs::File, Error> {
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
    let temp = std::env::temp_dir();
    let mut file = tempfile::tempfile().map_err(|e| io_error(e, &temp))?;
    // Read on a thread of its own, so a connection that stalls or a cancel does not have to wait for the next read.
    let mut body = response.into_body().into_with_config().limit(asset.size.saturating_add(1)).reader();
    let (tx, rx) = mpsc::sync_channel::<std::io::Result<Vec<u8>>>(4);
    std::thread::spawn(move || {
        loop {
            let mut buf = vec![0u8; 256 * 1024];
            let chunk = match body.read(&mut buf) {
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                read => read.map(|n| {
                    buf.truncate(n);
                    buf
                }),
            };
            let last = !chunk.as_ref().is_ok_and(|c| !c.is_empty());
            if tx.send(chunk).is_err() || last {
                break;
            }
        }
    });

    let mut hash = ring::digest::Context::new(&ring::digest::SHA256);
    let mut done = 0u64;
    let mut quiet = Duration::ZERO;
    loop {
        report.check()?;
        let tick = Duration::from_millis(200);
        let chunk = match rx.recv_timeout(tick) {
            Ok(Ok(chunk)) => chunk,
            Ok(Err(e)) => return Err(Error::Offline(format!("the download broke off after {}: {e}", megabytes(done)))),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                quiet += tick;
                if quiet >= STALL {
                    return Err(Error::Offline(format!("the download stalled after {}, nothing came for {} seconds", megabytes(done), STALL.as_secs())));
                }

                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err(Error::Offline(format!("the download broke off after {}", megabytes(done)))),
        };
        if chunk.is_empty() {
            break;
        }

        quiet = Duration::ZERO;
        hash.update(&chunk);
        file.write_all(&chunk).map_err(|e| io_error(e, &temp))?;
        done += chunk.len() as u64;
        report.set(|p| p.done = done);
    }

    if done != asset.size {
        return Err(Error::Damaged(format!("it has {} instead of {}", megabytes(done), megabytes(asset.size))));
    }

    let sum: String = hash.finish().as_ref().iter().map(|b| format!("{b:02x}")).collect();
    if asset.sha256.as_ref().is_some_and(|expected| *expected != sum) {
        return Err(Error::Damaged("its checksum does not match the release".into()));
    }

    file.seek(std::io::SeekFrom::Start(0)).map_err(|e| io_error(e, &temp))?;
    Ok(file)
}

/// The project path an archive entry unpacks to, or None when the name is absolute, climbs out with `..`, uses
/// backslashes or drive letters, lies outside `folders`, or has a part Windows cannot hold: a device name like `CON` or
/// `com1.txt`, a character it forbids, or a trailing dot or space.
pub fn entry_path(name: &str, folders: &[&str]) -> Option<PathBuf> {
    if name.is_empty() || name.contains(['\\', ':', '\0']) || name.starts_with('/') {
        return None;
    }

    let parts: Vec<&str> = name.trim_end_matches('/').split('/').collect();
    let windows_name = |p: &str| {
        let stem = p.split('.').next().unwrap_or_default().trim_end().to_ascii_uppercase();
        let device = ["CON", "PRN", "AUX", "NUL"].contains(&stem.as_str())
            || (stem.len() == 4 && (stem.starts_with("COM") || stem.starts_with("LPT")) && stem.as_bytes()[3].is_ascii_digit());
        !device && !p.ends_with(['.', ' ']) && !p.contains(['<', '>', '"', '|', '?', '*']) && !p.chars().any(char::is_control)
    };
    if parts.iter().any(|p| p.is_empty() || *p == "." || *p == ".." || !windows_name(p)) {
        return None;
    }

    let inside = folders.iter().any(|f| {
        let folder: Vec<&str> = f.split('/').collect();
        parts.len() > folder.len() && parts[..folder.len()] == folder[..]
    });
    let path: PathBuf = parts.iter().collect();
    (inside && path.components().all(|c| matches!(c, Component::Normal(_)))).then_some(path)
}

/// Checks the folders on the way from `root` to `rel` that exist already: each has to be a folder, and a link has to
/// lead to one inside the project. With `create`, the missing ones are made.
fn check_dirs(root: &Path, real_root: &Path, rel: &Path, create: bool) -> Result<(), Error> {
    let mut dir = root.to_path_buf();
    for part in rel.components() {
        dir.push(part);
        match std::fs::symlink_metadata(&dir) {
            Ok(meta) if meta.file_type().is_symlink() => {
                let inside = dir.canonicalize().is_ok_and(|real| real.starts_with(real_root) && real.is_dir());
                if !inside {
                    return Err(Error::Link(dir));
                }
            }
            Ok(meta) if meta.is_dir() => {}
            Ok(_) => return Err(Error::Io(format!("Could not add files to {}, it is a file and not a folder", dir.display()))),
            Err(_) if create => std::fs::create_dir(&dir).map_err(|e| io_error(e, &dir))?,
            // Nothing below a missing folder exists either.
            Err(_) => return Ok(()),
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

/// Passes on at most `left` bytes and fails on the next one, so an entry cannot unpack to more than its header claims.
struct Capped<R> {
    inner: R,
    left: u64,
}

impl<R: Read> Read for Capped<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.left = self.left.checked_sub(n as u64).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "more than its header says"))?;
        Ok(n)
    }
}

/// Unpacks the files of `archive`, named `name` in the progress, that lie in `folders` into `root`, adding each to
/// `added`, which keeps what was written when it fails part way. Every name and every folder already on the way is
/// checked before anything is written, files the project has are kept, links in the archive are refused, and each file
/// appears under its name only once it is complete.
pub fn extract(archive: std::fs::File, name: &str, root: &Path, folders: &[&str], report: &Reporter, added: &mut Added) -> Result<(), Error> {
    let mut zip = zip::ZipArchive::new(std::io::BufReader::new(archive)).map_err(|e| Error::Damaged(e.to_string()))?;
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
    let parents: std::collections::BTreeSet<&Path> = files.iter().filter_map(|(_, rel)| rel.parent()).collect();
    for parent in parents {
        check_dirs(root, &real_root, parent, false)?;
    }

    let count = files.len() as u64;
    report.set(|p| *p = Progress { stage: format!("Adding files to the project from {name}"), done: 0, total: count, bytes: false });
    for (n, (i, rel)) in files.into_iter().enumerate() {
        report.check()?;
        let target = root.join(&rel);
        let mut part = target.clone().into_os_string();
        part.push(gt_formats::PART_SUFFIX);
        let _ = std::fs::remove_file(part);
        if std::fs::symlink_metadata(&target).is_ok() {
            added.kept += 1;
        } else {
            check_dirs(root, &real_root, rel.parent().unwrap_or(Path::new("")), true)?;
            let mut entry = zip.by_index(i).map_err(|e| Error::Damaged(e.to_string()))?;
            let size = entry.size();
            match gt_formats::write_new(&target, &mut Capped { inner: &mut entry, left: size }) {
                Ok(_) => added.written.push(target),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => added.kept += 1,
                Err(e) => return Err(io_error(e, &target)),
            }
        }

        report.set(|p| p.done = n as u64 + 1);
    }

    Ok(())
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
/// what was added. The demo leaves out a nature pack the project has. When a download fails, the embedded Blockbench
/// models are added instead, they need no network.
pub fn install(root: &Path, choice: Choice, api: &str, report: &Reporter) -> Outcome {
    let mut outcome = Outcome { choice, ..Default::default() };
    let agent = agent();
    let skip_pack = choice == Choice::Demo && has_nature_pack(root);
    for download in choice.downloads().iter().filter(|d| !(skip_pack && **d == NATURE_PACK)) {
        report.set(|p| *p = Progress { stage: format!("Looking up {} in the releases", download.asset), ..Default::default() });
        let result = find_asset(&agent, api, download.asset).and_then(|asset| {
            let file = self::download(&agent, &asset, report)?;
            extract(file, download.asset, root, download.folders, report, &mut outcome.added)?;
            Ok(asset.tag)
        });
        match result {
            Ok(tag) => outcome.tags.push(tag),
            Err(e) => {
                outcome.error = Some(e);
                break;
            }
        }
    }

    if outcome.error.as_ref().is_some_and(Error::is_download) {
        let nature = nature_dir(root);
        if let Ok(written) = gt_formats::nature::install(&nature, &[], false) {
            outcome.added.written.extend(written);
        }
    }

    outcome
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
        let report = Reporter::new(repaint);
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
        self.report.progress()
    }

    /// Asks the thread to stop. [`Job::poll`] then delivers the outcome, with what was added until then.
    pub fn cancel(&self) {
        self.report.cancel();
    }

    pub fn cancelling(&self) -> bool {
        self.report.cancelled()
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

    /// `bytes` in a file without a name, the way [`download`] hands an archive to [`extract`].
    fn file(bytes: &[u8]) -> std::fs::File {
        let mut file = tempfile::tempfile().unwrap();
        file.write_all(bytes).unwrap();
        file.seek(std::io::SeekFrom::Start(0)).unwrap();
        file
    }

    fn unpack(bytes: &[u8], root: &Path, folders: &[&str]) -> Result<Added, (Added, Error)> {
        let mut added = Added::default();
        match extract(file(bytes), "test.zip", root, folders, &Reporter::default(), &mut added) {
            Ok(()) => Ok(added),
            Err(e) => Err((added, e)),
        }
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
            "godottrench/nature/CON",
            "godottrench/nature/com1.txt",
            "godottrench/nature/trees/aux.glb",
            "godottrench/nature/LPT9",
            "godottrench/nature/oak.",
            "godottrench/nature/oak ",
            "godottrench/nature/a?b.glb",
            "godottrench/nature/a|b.glb",
            "godottrench/nature/a\u{1}b.glb",
        ] {
            assert_eq!(entry_path(bad, &folders), None, "{bad:?}");
        }

        for fine in ["godottrench/nature/console.glb", "godottrench/nature/com10.txt", "godottrench/nature/lpt.png", "godottrench/nature/a b.glb"] {
            assert!(entry_path(fine, &folders).is_some(), "{fine}");
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
        std::fs::write(root.join("godottrench/nature/trees/oak.glb.gtpart"), "cut short by a crash").unwrap();
        std::fs::write(root.join("godottrench/nature/trees/pack.json.gtpart"), "cut short by a crash").unwrap();

        let outcome = install(&root, Choice::Nature, &server.url, &Reporter::default());
        assert_eq!(outcome.error, None);
        assert_eq!(outcome.tags, ["beta"]);
        let installed = files(&root);
        assert!(installed.contains(&"godottrench/nature/trees/oak.glb".to_string()), "{installed:?}");
        assert!(installed.iter().all(|f| !f.ends_with(".gtpart")), "left over temporary files go too: {installed:?}");
        assert_eq!(installed.iter().filter(|f| f.ends_with(".bbmodel")).count(), 1, "the embedded models are only a fallback");
        assert_eq!(std::fs::read_to_string(root.join("godottrench/nature/trees/pack.json")).unwrap(), "mine", "a file already there is kept");
        assert_eq!(std::fs::read_to_string(root.join("godottrench/nature/rock.bbmodel")).unwrap(), "from the archive");
        assert_eq!((outcome.added.written.len(), outcome.added.kept), (3, 1));
        assert!(outcome.summary().starts_with("Added 3 files to the project from the rolling beta and kept the 1"), "{}", outcome.summary());

        let again = install(&root, Choice::Nature, &server.url, &Reporter::default());
        assert_eq!((again.added.written.len(), again.added.kept, again.error.clone()), (0, 4, None));
        assert!(again.summary().starts_with("The project already had all 4 files"), "{}", again.summary());

        // The demo does not fetch a nature pack the project has again.
        for folder in gt_formats::nature::PACK_FOLDERS {
            std::fs::create_dir_all(root.join("godottrench/nature").join(folder)).unwrap();
            std::fs::write(root.join("godottrench/nature").join(folder).join("pack.json"), "{}").unwrap();
        }

        server.release("beta", &[(NATURE_PACK.asset, &pack), (DEMO_CONTENT.asset, &zip(&[("demo/demo.tscn", b"scene")]))], true);
        server.requests.lock().unwrap().clear();
        let demo = install(&root, Choice::Demo, &server.url, &Reporter::default());
        assert_eq!((demo.error, demo.added.written.len()), (None, 1));
        assert!(server.requests.lock().unwrap().iter().all(|r| !r.contains(NATURE_PACK.asset)));
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
            let mut added = Added::default();
            let file = download(&agent, &asset, &Reporter::default())?;
            extract(file, DEMO_CONTENT.asset, &root, DEMO_CONTENT.folders, &Reporter::default(), &mut added).map(|_| added)
        };

        for (bad, why) in [
            (zip(&[("demo/ok.txt", b"ok"), ("../evil.txt", b"x")]), "climbs out"),
            (zip(&[("demo/ok.txt", b"ok"), ("project.godot", b"x")]), "the project file"),
            (zip(&[("demo/ok.txt", b"ok"), ("addons/func_godot/plugin.cfg", b"x")]), "the addon"),
            (zip(&[("demo/ok.txt", b"ok"), ("demo/NUL.txt", b"x")]), "a device name on Windows"),
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
        assert!(matches!(download(&agent, &found, &Reporter::default()), Err(Error::Damaged(why)) if why.contains("checksum")));
        server.route(&asset, 200, &good[..good.len() - 1]);
        assert!(matches!(download(&agent, &found, &Reporter::default()), Err(Error::Damaged(_))), "cut short");

        // A zip with a corrupt entry fails its CRC check, the files before it stay and no partial file is left behind.
        let mut corrupt = zip(&[("demo/maps/a.gtm", b"deflated"), ("demo/maps/b.gtm", &[7u8; 64])]);
        let at = corrupt.windows(64).position(|w| w == [7u8; 64]).expect("the stored bytes");
        corrupt[at] = 8;
        let (added, error) = unpack(&corrupt, &root, DEMO_CONTENT.folders).unwrap_err();
        assert!(matches!(error, Error::Damaged(_)));
        assert_eq!(added.written, [root.join("demo/maps/a.gtm")], "what was written before the error is reported");
        assert_eq!(files(&root), ["demo/maps/a.gtm", "project.godot"]);
        std::fs::remove_dir_all(root.join("demo")).unwrap();

        // An entry that inflates to more than its header says stops at that, and nothing of it stays.
        let mut bomb = zip(&[("demo/big.bin", &[0u8; 4096])]);
        for (signature, offset) in [(&[0x50, 0x4b, 0x03, 0x04], 22), (&[0x50, 0x4b, 0x01, 0x02], 24)] {
            let at = bomb.windows(4).position(|w| w == signature).unwrap() + offset;
            bomb[at..at + 4].copy_from_slice(&16u32.to_le_bytes());
        }

        assert!(matches!(unpack(&bomb, &root, DEMO_CONTENT.folders), Err((_, Error::Damaged(_)))));
        assert_eq!(files(&root), ["project.godot"]);

        assert_eq!(check(&good, false).unwrap().written, [root.join("demo/maps/a.gtm")], "without a digest the size still has to match");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn github_downloads_need_https_and_a_checksum() {
        assert_eq!(trusted(RELEASES_API, "a.zip", "https://github.com/x/a.zip", true), Ok(()));
        assert!(matches!(trusted(RELEASES_API, "a.zip", "http://github.com/x/a.zip", true), Err(Error::Damaged(why)) if why.contains("https")));
        assert!(matches!(trusted(RELEASES_API, "a.zip", "https://github.com/x/a.zip", false), Err(Error::Damaged(why)) if why.contains("checksum")));
        assert_eq!(trusted("https://mirror.example", "a.zip", "https://mirror.example/a.zip", false), Ok(()), "another server may leave it out");
        assert!(trusted("https://mirror.example", "a.zip", "http://mirror.example/a.zip", false).is_err());
        assert_eq!(trusted("http://127.0.0.1:1", "a.zip", "http://127.0.0.1:1/a.zip", false), Ok(()));
    }

    #[test]
    fn a_stalled_download_gives_up_and_a_cancel_does_not_wait_for_it() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/stalls.zip", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                std::thread::spawn(move || {
                    let _ = std::io::BufRead::read_line(&mut std::io::BufReader::new(stream.try_clone().unwrap()), &mut String::new());
                    let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1000\r\nConnection: close\r\n\r\n0123456789");
                    std::thread::sleep(Duration::from_secs(5));
                });
            }
        });

        let asset = Asset { url, size: 1000, sha256: None, tag: "beta".into() };
        let started = std::time::Instant::now();
        let stalled = download(&agent(), &asset, &Reporter::default());
        assert!(matches!(&stalled, Err(Error::Offline(why)) if why.contains("stalled")), "{stalled:?}");
        assert!(started.elapsed() < Duration::from_secs(4));

        let report = Reporter::default();
        let cancel = report.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            cancel.cancel();
        });
        let started = std::time::Instant::now();
        assert!(matches!(download(&agent(), &asset, &report), Err(Error::Cancelled)));
        assert!(started.elapsed() < STALL, "cancelled before the stall shows");
    }

    #[test]
    fn missing_releases_and_no_network_say_what_is_wrong() {
        let server = Server::start();
        let agent = agent();
        assert_eq!(find_asset(&agent, &server.url, NATURE_PACK.asset), Err(Error::NoRelease));
        assert!(Error::NoRelease.to_string().contains("try again in a few minutes"));
        let requests = server.requests.lock().unwrap().clone();
        assert_eq!(requests, release_tags().map(|tag| format!("/releases/tags/{tag}")));
        assert_eq!(release_tags()[0] == "beta", beta_build(), "a beta build looks in the rolling beta first");
        server.release("beta", &[], true);
        assert_eq!(find_asset(&agent, &server.url, NATURE_PACK.asset), Err(Error::NotPublished(NATURE_PACK.asset)));
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
        cancelled.cancel();
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
        let archive = zip(&[("demo/maps/a.gtm", b"map")]);
        assert!(matches!(unpack(&archive, &root, DEMO_CONTENT.folders), Err((_, Error::Link(_)))));
        assert_eq!(files(&outside), ["project.godot"]);

        // Every folder is checked before the first file is written, whichever comes first in the archive.
        std::fs::remove_file(root.join("demo")).unwrap();
        std::os::unix::fs::symlink(&outside, root.join("models")).unwrap();
        let both = zip(&[("demo/maps/a.gtm", b"map"), ("demo/maps/b.gtm", b"map"), ("models/tree.glb", b"glTF")]);
        let (added, error) = unpack(&both, &root, DEMO_CONTENT.folders).unwrap_err();
        assert!(matches!(error, Error::Link(_)) && added == Added::default(), "{error:?}");
        assert!(!root.join("demo").exists() && files(&outside) == ["project.godot"]);
        std::fs::remove_file(root.join("models")).unwrap();

        std::fs::create_dir_all(root.join("mine")).unwrap();
        std::os::unix::fs::symlink(root.join("mine"), root.join("demo")).unwrap();
        assert_eq!(unpack(&archive, &root, DEMO_CONTENT.folders).unwrap().written.len(), 1, "a link inside the project is fine");
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
