//! Swiss Kit app: a small local web server + browser UI on top of swiss-kit-core.
//! Binds to 127.0.0.1 only and rejects requests from other sites (Host/Origin check).
use axum::{
    extract::{DefaultBodyLimit, Multipart, Request, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use swiss_kit_core::convert::image_conv::{convert_image, ImageKind};
use swiss_kit_core::convert::media::{codec_args, convert_media};
use swiss_kit_core::download::{download, DownloadRequest, Format, Platform, Progress};
use swiss_kit_core::info::{fetch_info, VideoInfo};
use swiss_kit_core::{tools, Error};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::unbounded_channel;

#[derive(Clone, Serialize)]
struct Job {
    id: u64,
    label: String,
    kind: &'static str,
    state: &'static str, // running | done | error
    percent: Option<f32>,
    file: Option<String>,
    error: Option<String>,
    error_code: Option<&'static str>,
    error_args: Option<Value>,
}

struct AppState {
    jobs: Mutex<Vec<Job>>,
    next: AtomicU64,
    out_dir: PathBuf,
    port: u16,
}

impl AppState {
    fn add(&self, label: String, kind: &'static str) -> u64 {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        self.jobs.lock().unwrap().push(Job { id, label, kind, state: "running", percent: None, file: None, error: None, error_code: None, error_args: None });
        id
    }
    fn update(&self, id: u64, f: impl FnOnce(&mut Job)) {
        if let Some(j) = self.jobs.lock().unwrap().iter_mut().find(|j| j.id == id) {
            f(j);
        }
    }
    fn finish(&self, id: u64, res: Result<PathBuf, Error>) {
        self.update(id, |j| match res {
            Ok(f) => {
                j.state = "done";
                j.percent = Some(100.0);
                j.file = Some(f.to_string_lossy().into_owned());
            }
            Err(e) => {
                let (code, args) = describe(&e);
                j.state = "error";
                j.error = Some(e.to_string());
                j.error_code = Some(code);
                j.error_args = Some(args);
            }
        });
    }
}

type Shared = Arc<AppState>;

/// A stable code plus arguments for each error, so the browser UI can show it in the user's language.
fn describe(e: &Error) -> (&'static str, Value) {
    match e {
        Error::InvalidUrl(m) => ("invalid_url", json!({ "message": m })),
        Error::WrongPlatform { expected } => ("wrong_platform", json!({ "site": expected })),
        Error::ToolMissing(t) => ("tool_missing", json!({ "tool": t })),
        Error::ToolFailed { tool, message } => ("tool_failed", json!({ "tool": tool, "message": message })),
        Error::UnsupportedFormat(f) => ("unsupported_format", json!({ "format": f })),
        Error::UnknownExtension(p) => ("unknown_extension", json!({ "file": p.to_string_lossy() })),
        Error::Image(e) => ("image_error", json!({ "message": e.to_string() })),
        Error::Io(e) => ("io_error", json!({ "message": e.to_string() })),
    }
}

struct ApiError {
    status: StatusCode,
    code: &'static str,
    args: Value,
    message: String,
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message, "code": self.code, "args": self.args }))).into_response()
    }
}
impl From<Error> for ApiError {
    fn from(e: Error) -> Self {
        let (code, args) = describe(&e);
        ApiError { status: StatusCode::BAD_REQUEST, code, args, message: e.to_string() }
    }
}
fn bad(code: &'static str, msg: impl Into<String>) -> ApiError {
    ApiError { status: StatusCode::BAD_REQUEST, code, args: json!({}), message: msg.into() }
}

fn platform(s: &str) -> Result<Platform, ApiError> {
    Platform::from_id(s).ok_or_else(|| bad("unknown_platform", "unknown site"))
}

// ---- security: only our own page may talk to this server -------------------

async fn guard(State(st): State<Shared>, req: Request, next: Next) -> Response {
    let allowed = [format!("127.0.0.1:{}", st.port), format!("localhost:{}", st.port)];
    let host_ok = req
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .is_some_and(|h| allowed.iter().any(|a| a == h));
    let origin_ok = match req.headers().get(header::ORIGIN) {
        None => true,
        Some(o) => o.to_str().is_ok_and(|o| allowed.iter().any(|a| o == format!("http://{a}"))),
    };
    if host_ok && origin_ok {
        next.run(req).await
    } else {
        StatusCode::FORBIDDEN.into_response()
    }
}

// ---- handlers ---------------------------------------------------------------

async fn index() -> Html<&'static str> {
    Html(include_str!("../ui/index.html"))
}

async fn config(State(st): State<Shared>) -> Json<Value> {
    Json(json!({
        "out_dir": st.out_dir.to_string_lossy(),
        "yt_dlp": tools::find("yt-dlp").is_ok(),
        "ffmpeg": tools::find("ffmpeg").is_ok(),
    }))
}

#[derive(Deserialize)]
struct FetchReq {
    platform: String,
    url: String,
}

async fn fetch(Json(r): Json<FetchReq>) -> Result<Json<VideoInfo>, ApiError> {
    Ok(Json(fetch_info(platform(&r.platform)?, &r.url).await?))
}

#[derive(Deserialize)]
struct DlReq {
    platform: String,
    url: String,
    format: String,
    max_height: Option<u32>,
    title: Option<String>,
}

async fn start_download(State(st): State<Shared>, Json(r): Json<DlReq>) -> Result<Json<Value>, ApiError> {
    let p = platform(&r.platform)?;
    p.validate(&r.url)?;
    let format = match r.format.as_str() {
        "mp3" => Format::Mp3,
        "mp4" => Format::Mp4,
        _ => return Err(bad("bad_format", "format must be mp3 or mp4")),
    };
    let label = format!("{} ({})", r.title.filter(|t| !t.is_empty()).unwrap_or_else(|| r.url.clone()), r.format.to_uppercase());
    let id = st.add(label, "download");
    let req = DownloadRequest { platform: p, url: r.url, format, max_height: r.max_height, out_dir: st.out_dir.clone() };
    tokio::spawn(async move {
        let (tx, mut rx) = unbounded_channel();
        let st_progress = st.clone();
        let updater = tokio::spawn(async move {
            while let Some(p) = rx.recv().await {
                if let Progress::Percent(p) = p {
                    st_progress.update(id, |j| j.percent = Some(p));
                }
            }
        });
        let res = download(&req, Some(tx)).await;
        let _ = updater.await;
        st.finish(id, res);
    });
    Ok(Json(json!({ "id": id })))
}

/// Multipart fields `kind` (image|media), `to`, `quality`, then `file` (must come last).
async fn start_convert(State(st): State<Shared>, mut mp: Multipart) -> Result<Json<Value>, ApiError> {
    let (mut kind, mut to, mut quality) = (String::new(), String::new(), 90u8);
    while let Some(mut field) = mp.next_field().await.map_err(|e| bad("bad_request", e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "kind" => kind = field.text().await.map_err(|e| bad("bad_request", e.to_string()))?,
            "to" => to = field.text().await.map_err(|e| bad("bad_request", e.to_string()))?.to_ascii_lowercase(),
            "quality" => quality = field.text().await.map_err(|e| bad("bad_request", e.to_string()))?.parse().unwrap_or(90),
            "file" => {
                let file_name = field
                    .file_name()
                    .and_then(|n| Path::new(n).file_name())
                    .map(|n| n.to_string_lossy().into_owned())
                    .filter(|n| !n.is_empty())
                    .ok_or_else(|| bad("no_filename", "the file has no name"))?;
                // Validate the request before accepting any bytes.
                let image_target = match kind.as_str() {
                    "image" => Some(ImageKind::from_ext(&to)?),
                    "media" => {
                        codec_args(&to)?;
                        None
                    }
                    _ => return Err(bad("unknown_kind", "unknown conversion kind")),
                };
                let id = st.add(format!("{file_name} → {}", to.to_uppercase()), "convert");
                let upload_dir = std::env::temp_dir().join("swiss-kit-uploads").join(id.to_string());
                let path = upload_dir.join(&file_name);
                let saved: std::io::Result<()> = async {
                    tokio::fs::create_dir_all(&upload_dir).await?;
                    let mut f = tokio::fs::File::create(&path).await?;
                    while let Some(chunk) = field.chunk().await.map_err(std::io::Error::other)? {
                        f.write_all(&chunk).await?;
                    }
                    f.flush().await
                }
                .await;
                if let Err(e) = saved {
                    st.finish(id, Err(Error::Io(e)));
                    let _ = tokio::fs::remove_dir_all(&upload_dir).await;
                    return Err(bad("receive_failed", "could not receive the file"));
                }
                let out_dir = st.out_dir.clone();
                tokio::spawn(async move {
                    let res = match image_target {
                        Some(target) => {
                            let (p, o) = (path.clone(), out_dir);
                            tokio::task::spawn_blocking(move || convert_image(&p, target, quality, Some(&o)))
                                .await
                                .unwrap_or_else(|e| Err(Error::Io(std::io::Error::other(e.to_string()))))
                        }
                        None => convert_media(&path, &to, Some(&out_dir)).await,
                    };
                    let _ = tokio::fs::remove_dir_all(&upload_dir).await;
                    st.finish(id, res);
                });
                return Ok(Json(json!({ "id": id })));
            }
            _ => {}
        }
    }
    Err(bad("no_file", "no file was received"))
}

async fn list(State(st): State<Shared>) -> Json<Vec<Job>> {
    Json(st.jobs.lock().unwrap().clone())
}

async fn clear(State(st): State<Shared>) -> StatusCode {
    st.jobs.lock().unwrap().retain(|j| j.state == "running");
    StatusCode::NO_CONTENT
}

#[derive(Deserialize)]
struct OpenReq {
    id: u64,
}

async fn open_job(State(st): State<Shared>, Json(r): Json<OpenReq>) -> Result<StatusCode, ApiError> {
    let file = st
        .jobs
        .lock()
        .unwrap()
        .iter()
        .find(|j| j.id == r.id)
        .and_then(|j| j.file.clone())
        .ok_or_else(|| bad("no_job_file", "that job has no file"))?;
    open_path(Path::new(&file), true);
    Ok(StatusCode::NO_CONTENT)
}

async fn open_folder(State(st): State<Shared>) -> StatusCode {
    open_path(&st.out_dir, false);
    StatusCode::NO_CONTENT
}

// ---- opening things on the user's computer ----------------------------------

/// Open a folder, or show a file selected in its folder.
fn open_path(path: &Path, select: bool) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let mut c = Command::new("explorer");
        if select {
            c.raw_arg(format!("/select,\"{}\"", path.display()));
        } else {
            c.arg(path);
        }
        let _ = c.spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let mut c = Command::new("open");
        if select {
            c.arg("-R");
        }
        let _ = c.arg(path).spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let dir = if select { path.parent().unwrap_or(path) } else { path };
        let _ = Command::new("xdg-open").arg(dir).spawn();
    }
}

fn open_browser(url: &str) {
    #[cfg(windows)]
    let _ = Command::new("cmd").args(["/C", "start", "", url]).spawn();
    #[cfg(target_os = "macos")]
    let _ = Command::new("open").arg(url).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let _ = Command::new("xdg-open").arg(url).spawn();
}

fn default_out_dir() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(|h| PathBuf::from(h).join("Downloads").join("Swiss Kit"))
        .unwrap_or_else(|| PathBuf::from("Swiss Kit"))
}

#[tokio::main]
async fn main() {
    let mut out_dir = default_out_dir();
    let mut no_open = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out" => out_dir = args.next().map(PathBuf::from).unwrap_or(out_dir),
            "--no-open" => no_open = true,
            "--help" | "-h" => {
                println!("swiss-kit-app [--out <folder>] [--no-open]");
                return;
            }
            other => eprintln!("ignoring unknown option: {other}"),
        }
    }
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        eprintln!("could not create {}: {e}", out_dir.display());
        std::process::exit(1);
    }

    let listener = match tokio::net::TcpListener::bind("127.0.0.1:7878").await {
        Ok(l) => l,
        Err(_) => tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("could not open a local port"),
    };
    let port = listener.local_addr().unwrap().port();
    let state = Arc::new(AppState { jobs: Mutex::new(Vec::new()), next: AtomicU64::new(1), out_dir, port });

    let app = Router::new()
        .route("/", get(index))
        .route("/api/config", get(config))
        .route("/api/fetch", post(fetch))
        .route("/api/download", post(start_download))
        .route("/api/convert", post(start_convert).layer(DefaultBodyLimit::disable()))
        .route("/api/jobs", get(list))
        .route("/api/jobs/clear", post(clear))
        .route("/api/open", post(open_job))
        .route("/api/open-folder", post(open_folder))
        .layer(middleware::from_fn_with_state(state.clone(), guard))
        .with_state(state.clone());

    let url = format!("http://127.0.0.1:{port}");
    println!("Swiss Kit is running at {url}");
    println!("Saving files to {}", state.out_dir.display());
    println!("Close this window (or press Ctrl+C) to quit.");
    if !no_open {
        open_browser(&url);
    }
    axum::serve(listener, app).await.expect("server error");
}
