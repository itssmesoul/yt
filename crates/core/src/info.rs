//! Look up a video's title, thumbnail and available qualities (via yt-dlp).
use crate::download::Platform;
use crate::error::{Error, Result};
use crate::tools;
use serde::Serialize;
use std::time::Duration;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct VideoInfo {
    pub title: String,
    pub uploader: Option<String>,
    pub duration_secs: Option<u64>,
    pub thumbnail: Option<String>,
    /// Available video heights, highest first (e.g. [1080, 720, 480]).
    pub heights: Vec<u32>,
}

fn fail(message: String) -> Error {
    Error::ToolFailed { tool: "yt-dlp", message }
}

pub fn parse_info(json: &str) -> Result<VideoInfo> {
    let v: serde_json::Value =
        serde_json::from_str(json).map_err(|e| fail(format!("unreadable response: {e}")))?;
    let title = v["title"].as_str().ok_or_else(|| fail("response had no title".into()))?.to_string();
    let mut heights: Vec<u32> = v["formats"]
        .as_array()
        .map(|fs| {
            fs.iter()
                .filter(|f| f["vcodec"].as_str().map_or(true, |c| c != "none"))
                .filter_map(|f| f["height"].as_u64())
                .map(|h| h as u32)
                .collect()
        })
        .unwrap_or_default();
    heights.sort_unstable_by(|a, b| b.cmp(a));
    heights.dedup();
    Ok(VideoInfo {
        title,
        uploader: v["uploader"].as_str().or_else(|| v["channel"].as_str()).map(String::from),
        duration_secs: v["duration"].as_f64().map(|d| d.round() as u64),
        thumbnail: v["thumbnail"].as_str().map(String::from),
        heights,
    })
}

pub async fn fetch_info(platform: Platform, raw_url: &str) -> Result<VideoInfo> {
    let url = platform.validate(raw_url)?;
    let ytdlp = tools::find("yt-dlp")?;
    let run = Command::new(ytdlp)
        .args(["--dump-single-json", "--no-playlist", "--no-warnings"])
        .arg(url.as_str())
        .kill_on_drop(true)
        .output();
    let out = tokio::time::timeout(Duration::from_secs(60), run)
        .await
        .map_err(|_| fail("timed out while looking up the video".into()))??;
    if !out.status.success() {
        let msg = String::from_utf8_lossy(&out.stderr).lines().last().unwrap_or("unknown error").to_string();
        return Err(fail(msg));
    }
    parse_info(&String::from_utf8_lossy(&out.stdout))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_yt_dlp_json() {
        let j = r#"{"title":"Demo","uploader":"Chan","duration":125.4,"thumbnail":"https://x/t.jpg",
          "formats":[{"vcodec":"avc1","height":720},{"vcodec":"avc1","height":1080},{"vcodec":"none","height":null},
                     {"vcodec":"vp9","height":720},{"vcodec":"none","audio":"x"}]}"#;
        let i = parse_info(j).unwrap();
        assert_eq!(i.title, "Demo");
        assert_eq!(i.duration_secs, Some(125));
        assert_eq!(i.heights, vec![1080, 720]);
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_info("not json").is_err());
        assert!(parse_info("{}").is_err());
    }
}
