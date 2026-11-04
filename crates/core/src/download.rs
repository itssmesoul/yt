//! YouTube / TikTok downloads, powered by yt-dlp (+ ffmpeg for mp3 and merging).
use crate::error::{Error, Result};
use crate::tools;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::UnboundedSender;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    YouTube,
    TikTok,
}

impl Platform {
    pub fn name(self) -> &'static str {
        match self {
            Platform::YouTube => "YouTube",
            Platform::TikTok => "TikTok",
        }
    }

    fn hosts(self) -> &'static [&'static str] {
        match self {
            Platform::YouTube => &["youtube.com", "youtu.be", "music.youtube.com"],
            Platform::TikTok => &["tiktok.com", "vm.tiktok.com", "vt.tiktok.com"],
        }
    }

    /// Check that `raw` is an http(s) link on this platform's domains.
    pub fn validate(self, raw: &str) -> Result<Url> {
        let url = Url::parse(raw.trim()).map_err(|e| Error::InvalidUrl(e.to_string()))?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(Error::InvalidUrl("only http(s) links are supported".into()));
        }
        let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
        let ok = self
            .hosts()
            .iter()
            .any(|h| host == *h || host.ends_with(&format!(".{h}")));
        if ok {
            Ok(url)
        } else {
            Err(Error::WrongPlatform { expected: self.name() })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Mp4,
    Mp3,
}

#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub platform: Platform,
    pub url: String,
    pub format: Format,
    /// Max video height (e.g. 1080). Ignored for mp3.
    pub max_height: Option<u32>,
    pub out_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub enum Progress {
    Percent(f32),
    Finished(PathBuf),
}

/// Build the yt-dlp argument list. Pure function so it is easy to test.
pub fn build_args(req: &DownloadRequest, url: &Url) -> Vec<String> {
    let mut a: Vec<String> = vec![
        "--no-playlist".into(),
        "--newline".into(),
        "--progress-template".into(),
        "download:PROGRESS %(progress._percent_str)s".into(),
        "--print".into(),
        "after_move:FILE %(filepath)s".into(),
        "-o".into(),
        req.out_dir.join("%(title).150B [%(id)s].%(ext)s").to_string_lossy().into_owned(),
    ];
    match req.format {
        Format::Mp3 => {
            a.extend(["-x", "--audio-format", "mp3", "--audio-quality", "0"].map(String::from));
        }
        Format::Mp4 => {
            let h = req.max_height.unwrap_or(1080);
            a.push("-f".into());
            a.push(format!(
                "bv*[height<={h}][ext=mp4]+ba[ext=m4a]/b[height<={h}][ext=mp4]/b[height<={h}]"
            ));
            a.extend(["--merge-output-format", "mp4"].map(String::from));
        }
    }
    a.push(url.as_str().to_string());
    a
}

fn parse_percent(line: &str) -> Option<f32> {
    let rest = line.strip_prefix("PROGRESS")?.trim().trim_end_matches('%');
    rest.trim().parse::<f32>().ok()
}

/// Run a download. Progress updates are sent on `tx`; returns the final file path.
pub async fn download(req: &DownloadRequest, tx: Option<UnboundedSender<Progress>>) -> Result<PathBuf> {
    let url = req.platform.validate(&req.url)?;
    let ytdlp = tools::find("yt-dlp")?;
    if req.format == Format::Mp3 || req.format == Format::Mp4 {
        tools::find("ffmpeg")?;
    }
    std::fs::create_dir_all(&req.out_dir)?;

    let mut child = tools::command(ytdlp)
        .args(build_args(req, &url))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let stdout = child.stdout.take().expect("piped stdout");
    // Drain stderr concurrently so a chatty yt-dlp can never block on a full pipe.
    let mut stderr = child.stderr.take().expect("piped stderr");
    let err_task = tokio::spawn(async move {
        let mut buf = String::new();
        let _ = tokio::io::AsyncReadExt::read_to_string(&mut stderr, &mut buf).await;
        buf
    });
    let mut lines = BufReader::new(stdout).lines();
    let mut file: Option<PathBuf> = None;
    while let Some(line) = lines.next_line().await? {
        if let Some(p) = parse_percent(&line) {
            if let Some(tx) = &tx {
                let _ = tx.send(Progress::Percent(p));
            }
        } else if let Some(f) = line.strip_prefix("FILE ") {
            file = Some(PathBuf::from(f.trim()));
        }
    }

    let status = child.wait().await?;
    let err_text = err_task.await.unwrap_or_default();
    if !status.success() {
        let msg = err_text.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("unknown error").to_string();
        return Err(Error::ToolFailed { tool: "yt-dlp", message: msg });
    }
    let file = file.ok_or_else(|| Error::ToolFailed { tool: "yt-dlp", message: "no output file reported".into() })?;
    if let Some(tx) = &tx {
        let _ = tx.send(Progress::Finished(file.clone()));
    }
    Ok(file)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VideoInfo {
    pub title: String,
    pub channel: String,
    pub duration: Option<u64>,
    pub thumbnail: Option<String>,
    /// Available video heights, descending (e.g. [1080, 720, 480]).
    pub heights: Vec<u32>,
}

/// Ask yt-dlp for metadata without downloading.
pub async fn fetch_info(platform: Platform, raw_url: &str) -> Result<VideoInfo> {
    let url = platform.validate(raw_url)?;
    let ytdlp = tools::find("yt-dlp")?;
    let out = tools::command(ytdlp)
        .args(["-J", "--no-playlist", "--no-warnings", url.as_str()])
        .output()
        .await?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let msg = err.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("unknown error").to_string();
        return Err(Error::ToolFailed { tool: "yt-dlp", message: msg });
    }
    parse_info(&out.stdout)
}

pub(crate) fn parse_info(json: &[u8]) -> Result<VideoInfo> {
    let v: serde_json::Value = serde_json::from_slice(json)
        .map_err(|e| Error::ToolFailed { tool: "yt-dlp", message: format!("bad metadata: {e}") })?;
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).map(str::to_string);
    let mut heights: Vec<u32> = v
        .get("formats")
        .and_then(|f| f.as_array())
        .map(|a| {
            a.iter()
                .filter(|f| f.get("vcodec").and_then(|c| c.as_str()).map_or(false, |c| c != "none"))
                .filter_map(|f| f.get("height").and_then(|h| h.as_u64()))
                .map(|h| h as u32)
                .collect()
        })
        .unwrap_or_default();
    heights.sort_unstable_by(|a, b| b.cmp(a));
    heights.dedup();
    Ok(VideoInfo {
        title: s("title").unwrap_or_else(|| "Untitled".into()),
        channel: s("channel").or_else(|| s("uploader")).unwrap_or_default(),
        duration: v.get("duration").and_then(|d| d.as_f64()).map(|d| d as u64),
        thumbnail: s("thumbnail"),
        heights,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_platform_hosts() {
        assert!(Platform::YouTube.validate("https://www.youtube.com/watch?v=abc").is_ok());
        assert!(Platform::YouTube.validate("https://youtu.be/abc").is_ok());
        assert!(Platform::TikTok.validate("https://www.tiktok.com/@u/video/1").is_ok());
        assert!(Platform::TikTok.validate("https://vm.tiktok.com/xyz/").is_ok());
        assert!(Platform::YouTube.validate("https://www.tiktok.com/@u/video/1").is_err());
        assert!(Platform::YouTube.validate("https://evilyoutube.com/x").is_err());
        assert!(Platform::YouTube.validate("file:///etc/passwd").is_err());
        assert!(Platform::YouTube.validate("not a url").is_err());
    }

    #[test]
    fn builds_mp3_and_mp4_args() {
        let mut req = DownloadRequest {
            platform: Platform::YouTube,
            url: "https://youtu.be/abc".into(),
            format: Format::Mp3,
            max_height: None,
            out_dir: "/tmp/out".into(),
        };
        let url = req.platform.validate(&req.url).unwrap();
        let a = build_args(&req, &url);
        assert!(a.contains(&"-x".to_string()) && a.contains(&"mp3".to_string()));
        assert_eq!(a.last().unwrap(), "https://youtu.be/abc");

        req.format = Format::Mp4;
        req.max_height = Some(720);
        let a = build_args(&req, &url);
        assert!(a.iter().any(|s| s.contains("height<=720")));
        assert!(a.contains(&"--merge-output-format".to_string()));
    }

    #[test]
    fn parses_progress() {
        assert_eq!(parse_percent("PROGRESS  42.5%"), Some(42.5));
        assert_eq!(parse_percent("something else"), None);
    }

    #[test]
    fn parses_info() {
        let j = br#"{"title":"T","channel":"C","duration":754.2,"thumbnail":"http://x/y.jpg",
          "formats":[{"vcodec":"avc1","height":720},{"vcodec":"none","height":null},{"vcodec":"vp9","height":1080},{"vcodec":"avc1","height":720}]}"#;
        let i = parse_info(j).unwrap();
        assert_eq!((i.title.as_str(), i.channel.as_str(), i.duration), ("T", "C", Some(754)));
        assert_eq!(i.heights, vec![1080, 720]);
    }
}
