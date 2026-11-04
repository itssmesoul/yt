#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::path::PathBuf;
use swiss_kit_core::convert::image_conv::{convert_image, ImageKind};
use swiss_kit_core::convert::media::convert_media;
use swiss_kit_core::download::{self, DownloadRequest, Format, Platform, Progress, VideoInfo};
use swiss_kit_core::tools;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc::unbounded_channel;

const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "bmp", "tif", "tiff"];

#[derive(Serialize, Clone)]
struct JobEvent {
    id: u64,
    /// "progress" | "done" | "error"
    kind: &'static str,
    percent: Option<f32>,
    path: Option<String>,
    message: Option<String>,
}

fn emit(app: &AppHandle, id: u64, kind: &'static str, percent: Option<f32>, path: Option<String>, message: Option<String>) {
    let _ = app.emit("job", JobEvent { id, kind, percent, path, message });
}

fn platform(name: &str) -> Result<Platform, String> {
    match name {
        "youtube" => Ok(Platform::YouTube),
        "tiktok" => Ok(Platform::TikTok),
        other => Err(format!("unknown platform: {other}")),
    }
}

#[derive(Serialize)]
struct ToolStatus {
    ytdlp: bool,
    ffmpeg: bool,
}

#[tauri::command]
fn tools_status() -> ToolStatus {
    ToolStatus { ytdlp: tools::find("yt-dlp").is_ok(), ffmpeg: tools::find("ffmpeg").is_ok() }
}

#[tauri::command]
fn default_dir(app: AppHandle) -> String {
    app.path()
        .download_dir()
        .or_else(|_| app.path().home_dir())
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| ".".into())
}

#[tauri::command]
async fn fetch_info(platform_name: String, url: String) -> Result<VideoInfo, String> {
    download::fetch_info(platform(&platform_name)?, &url).await.map_err(|e| e.to_string())
}

#[tauri::command]
fn start_download(
    app: AppHandle,
    id: u64,
    platform_name: String,
    url: String,
    format: String,
    max_height: Option<u32>,
    out_dir: String,
) -> Result<(), String> {
    let req = DownloadRequest {
        platform: platform(&platform_name)?,
        url,
        format: if format == "mp3" { Format::Mp3 } else { Format::Mp4 },
        max_height,
        out_dir: PathBuf::from(out_dir),
    };
    tauri::async_runtime::spawn(async move {
        let (tx, mut rx) = unbounded_channel();
        let fwd_app = app.clone();
        let forwarder = tauri::async_runtime::spawn(async move {
            while let Some(p) = rx.recv().await {
                if let Progress::Percent(p) = p {
                    emit(&fwd_app, id, "progress", Some(p), None, None);
                }
            }
        });
        let result = download::download(&req, Some(tx)).await;
        let _ = forwarder.await;
        match result {
            Ok(path) => emit(&app, id, "done", Some(100.0), Some(path.to_string_lossy().into_owned()), None),
            Err(e) => emit(&app, id, "error", None, None, Some(e.to_string())),
        }
    });
    Ok(())
}

#[tauri::command]
fn start_convert(app: AppHandle, id: u64, input: String, to: String) -> Result<(), String> {
    let input = PathBuf::from(input);
    let to = to.to_ascii_lowercase();
    tauri::async_runtime::spawn(async move {
        let src = input.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
        let result: swiss_kit_core::Result<PathBuf> = if IMAGE_EXTS.contains(&src.as_str()) && IMAGE_EXTS.contains(&to.as_str()) {
            let (i, t) = (input.clone(), to.clone());
            match tauri::async_runtime::spawn_blocking(move || ImageKind::from_ext(&t).and_then(|k| convert_image(&i, k, 90, None))).await {
                Ok(r) => r,
                Err(e) => Err(swiss_kit_core::Error::Io(std::io::Error::other(e.to_string()))),
            }
        } else {
            // HEIC/HEIF and all audio/video go through ffmpeg
            convert_media(&input, &to, None).await
        };
        match result {
            Ok(path) => emit(&app, id, "done", Some(100.0), Some(path.to_string_lossy().into_owned()), None),
            Err(e) => emit(&app, id, "error", None, None, Some(e.to_string())),
        }
    });
    Ok(())
}

/// Show a file in the system file manager.
#[tauri::command]
fn reveal(path: String) {
    #[cfg(windows)]
    let _ = std::process::Command::new("explorer").arg(format!("/select,{path}")).spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").args(["-R", &path]).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let dir = std::path::Path::new(&path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
        let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![tools_status, default_dir, fetch_info, start_download, start_convert, reveal])
        .run(tauri::generate_context!())
        .expect("error while running Swiss Kit");
}
