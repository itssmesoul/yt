//! YouTube / TikTok downloads, powered by yt-dlp (+ ffmpeg for mp3 and merging).
use crate::error::{Error, Result};
use crate::tools;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::UnboundedSender;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    YouTube,
    TikTok,
    Instagram,
    X,
    Facebook,
    Vimeo,
    Reddit,
    SoundCloud,
    Twitch,
    Dailymotion,
    Pinterest,
    /// Any other site that yt-dlp supports. Only the link shape is checked.
    Other,
}

impl Platform {
    pub const ALL: [Platform; 12] = [
        Platform::YouTube,
        Platform::TikTok,
        Platform::Instagram,
        Platform::X,
        Platform::Facebook,
        Platform::Vimeo,
        Platform::Reddit,
        Platform::SoundCloud,
        Platform::Twitch,
        Platform::Dailymotion,
        Platform::Pinterest,
        Platform::Other,
    ];

    /// Short stable id used by the CLI and the app's API.
    pub fn id(self) -> &'static str {
        match self {
            Platform::YouTube => "youtube",
            Platform::TikTok => "tiktok",
            Platform::Instagram => "instagram",
            Platform::X => "x",
            Platform::Facebook => "facebook",
            Platform::Vimeo => "vimeo",
            Platform::Reddit => "reddit",
            Platform::SoundCloud => "soundcloud",
            Platform::Twitch => "twitch",
            Platform::Dailymotion => "dailymotion",
            Platform::Pinterest => "pinterest",
            Platform::Other => "other",
        }
    }

    pub fn from_id(id: &str) -> Option<Platform> {
        Platform::ALL.into_iter().find(|p| p.id() == id)
    }

    pub fn name(self) -> &'static str {
        match self {
            Platform::YouTube => "YouTube",
            Platform::TikTok => "TikTok",
            Platform::Instagram => "Instagram",
            Platform::X => "X (Twitter)",
            Platform::Facebook => "Facebook",
            Platform::Vimeo => "Vimeo",
            Platform::Reddit => "Reddit",
            Platform::SoundCloud => "SoundCloud",
            Platform::Twitch => "Twitch",
            Platform::Dailymotion => "Dailymotion",
            Platform::Pinterest => "Pinterest",
            Platform::Other => "this site",
        }
    }

    /// Domains that belong to this site (subdomains are matched automatically).
    fn hosts(self) -> &'static [&'static str] {
        match self {
            Platform::YouTube => &["youtube.com", "youtu.be"],
            Platform::TikTok => &["tiktok.com"],
            Platform::Instagram => &["instagram.com", "instagr.am"],
            Platform::X => &["x.com", "twitter.com"],
            Platform::Facebook => &["facebook.com", "fb.watch"],
            Platform::Vimeo => &["vimeo.com"],
            Platform::Reddit => &["reddit.com", "redd.it"],
            Platform::SoundCloud => &["soundcloud.com"],
            Platform::Twitch => &["twitch.tv"],
            Platform::Dailymotion => &["dailymotion.com", "dai.ly"],
            Platform::Pinterest => &["pinterest.com", "pin.it"],
            Platform::Other => &[],
        }
    }

    fn host_matches(self, host: &str) -> bool {
        self.hosts().iter().any(|h| host == *h || host.ends_with(&format!(".{h}")))
    }

    /// Work out which site a link belongs to (`Other` when it is none of the known ones).
    pub fn detect(raw: &str) -> Platform {
        let host = Url::parse(raw.trim())
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_ascii_lowercase()))
            .unwrap_or_default();
        Platform::ALL.into_iter().find(|p| p.host_matches(&host)).unwrap_or(Platform::Other)
    }

    /// Check that `raw` is an http(s) link that fits this site.
    pub fn validate(self, raw: &str) -> Result<Url> {
        let url = Url::parse(raw.trim()).map_err(|e| Error::InvalidUrl(e.to_string()))?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(Error::InvalidUrl("only http(s) links are supported".into()));
        }
        let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
        if host.is_empty() {
            return Err(Error::InvalidUrl("the link has no website in it".into()));
        }
        if self == Platform::Other || self.host_matches(&host) {
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
            // `<=?` keeps formats whose height is unknown; the trailing `/b` is a last resort
            // for sites that only offer one combined (or audio-only) stream.
            a.push(format!(
                "bv*[height<=?{h}][ext=mp4]+ba[ext=m4a]/b[height<=?{h}][ext=mp4]/bv*[height<=?{h}]+ba/b[height<=?{h}]/b"
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

    let mut child = Command::new(ytdlp)
        .args(build_args(req, &url))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let stdout = child.stdout.take().expect("piped stdout");
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

    let out = child.wait_with_output().await?;
    if !out.status.success() {
        let msg = String::from_utf8_lossy(&out.stderr).lines().last().unwrap_or("unknown error").to_string();
        return Err(Error::ToolFailed { tool: "yt-dlp", message: msg });
    }
    let file = file.ok_or_else(|| Error::ToolFailed { tool: "yt-dlp", message: "no output file reported".into() })?;
    if let Some(tx) = &tx {
        let _ = tx.send(Progress::Finished(file.clone()));
    }
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_platform_hosts() {
        assert!(Platform::YouTube.validate("https://www.youtube.com/watch?v=abc").is_ok());
        assert!(Platform::YouTube.validate("https://music.youtube.com/watch?v=abc").is_ok());
        assert!(Platform::YouTube.validate("https://youtu.be/abc").is_ok());
        assert!(Platform::TikTok.validate("https://www.tiktok.com/@u/video/1").is_ok());
        assert!(Platform::TikTok.validate("https://vm.tiktok.com/xyz/").is_ok());
        assert!(Platform::YouTube.validate("https://www.tiktok.com/@u/video/1").is_err());
        assert!(Platform::YouTube.validate("https://evilyoutube.com/x").is_err());
        assert!(Platform::YouTube.validate("file:///etc/passwd").is_err());
        assert!(Platform::YouTube.validate("not a url").is_err());
    }

    #[test]
    fn validates_the_new_sites() {
        let ok = [
            (Platform::Instagram, "https://www.instagram.com/reel/abc/"),
            (Platform::X, "https://x.com/name/status/1"),
            (Platform::X, "https://twitter.com/name/status/1"),
            (Platform::Facebook, "https://www.facebook.com/watch/?v=1"),
            (Platform::Facebook, "https://fb.watch/abc/"),
            (Platform::Vimeo, "https://vimeo.com/12345"),
            (Platform::Reddit, "https://www.reddit.com/r/x/comments/1/t/"),
            (Platform::Reddit, "https://v.redd.it/abc"),
            (Platform::SoundCloud, "https://soundcloud.com/a/b"),
            (Platform::Twitch, "https://clips.twitch.tv/Abc"),
            (Platform::Dailymotion, "https://www.dailymotion.com/video/x1"),
            (Platform::Pinterest, "https://www.pinterest.com/pin/1/"),
        ];
        for (p, u) in ok {
            assert!(p.validate(u).is_ok(), "{} should accept {u}", p.id());
        }
        assert!(Platform::Instagram.validate("https://notinstagram.com/x").is_err());
        assert!(Platform::X.validate("https://vimeo.com/1").is_err());
    }

    #[test]
    fn other_accepts_any_web_link_but_nothing_else() {
        assert!(Platform::Other.validate("https://example.org/video/1").is_ok());
        assert!(Platform::Other.validate("ftp://example.org/v").is_err());
        assert!(Platform::Other.validate("file:///etc/passwd").is_err());
        assert!(Platform::Other.validate("javascript:alert(1)").is_err());
        assert!(Platform::Other.validate("nonsense").is_err());
    }

    #[test]
    fn detects_sites_and_round_trips_ids() {
        assert_eq!(Platform::detect("https://youtu.be/abc"), Platform::YouTube);
        assert_eq!(Platform::detect("https://twitter.com/a/status/1"), Platform::X);
        assert_eq!(Platform::detect("https://unknown.example/v"), Platform::Other);
        assert_eq!(Platform::detect("garbage"), Platform::Other);
        for p in Platform::ALL {
            assert_eq!(Platform::from_id(p.id()), Some(p));
        }
        assert_eq!(Platform::from_id("nope"), None);
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
        assert!(a.iter().any(|s| s.contains("height<=?720")));
        assert!(a.contains(&"--merge-output-format".to_string()));
    }

    #[test]
    fn parses_progress() {
        assert_eq!(parse_percent("PROGRESS  42.5%"), Some(42.5));
        assert_eq!(parse_percent("something else"), None);
    }
}
