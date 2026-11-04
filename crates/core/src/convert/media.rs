//! Audio / video conversions via ffmpeg.
use super::{ext_of, output_path};
use crate::error::{Error, Result};
use crate::tools;
use std::path::{Path, PathBuf};

/// Output formats we know good ffmpeg settings for.
pub fn supported_targets() -> &'static [&'static str] {
    &["mp3", "wav", "m4a", "ogg", "flac", "mp4", "webm", "gif", "jpg"]
}

/// ffmpeg arguments (excluding input/output paths) for a target extension.
pub fn codec_args(target: &str) -> Result<Vec<&'static str>> {
    Ok(match target {
        "mp3" => vec!["-vn", "-c:a", "libmp3lame", "-q:a", "2"],
        "wav" => vec!["-vn", "-c:a", "pcm_s16le"],
        "m4a" => vec!["-vn", "-c:a", "aac", "-b:a", "192k"],
        "ogg" => vec!["-vn", "-c:a", "libvorbis", "-q:a", "5"],
        "flac" => vec!["-vn", "-c:a", "flac"],
        "mp4" => vec!["-c:v", "libx264", "-preset", "medium", "-crf", "23", "-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "160k", "-movflags", "+faststart"],
        "webm" => vec!["-c:v", "libvpx-vp9", "-crf", "32", "-b:v", "0", "-c:a", "libopus"],
        "gif" => vec!["-an", "-vf", "fps=12,scale=480:-1:flags=lanczos"],
        // single still frame (used for HEIC and video frames)
        "jpg" => vec!["-frames:v", "1", "-q:v", "2"],
        other => return Err(Error::UnsupportedFormat(other.to_string())),
    })
}

pub fn build_args(input: &Path, output: &Path, target: &str) -> Result<Vec<String>> {
    let mut a: Vec<String> = ["-hide_banner", "-loglevel", "error", "-n", "-i"].map(String::from).to_vec();
    a.push(input.to_string_lossy().into_owned());
    a.extend(codec_args(target)?.into_iter().map(String::from));
    a.push(output.to_string_lossy().into_owned());
    Ok(a)
}

pub async fn convert_media(input: &Path, target: &str, out_dir: Option<&Path>) -> Result<PathBuf> {
    let target = target.to_ascii_lowercase();
    codec_args(&target)?;
    if ext_of(input).is_none() {
        return Err(Error::UnknownExtension(input.to_path_buf()));
    }
    let ffmpeg = tools::find("ffmpeg")?;
    let out = output_path(input, &target, out_dir)?;
    let result = tools::command(ffmpeg).args(build_args(input, &out, &target)?).output().await?;
    if !result.status.success() {
        let msg = String::from_utf8_lossy(&result.stderr).lines().last().unwrap_or("unknown error").to_string();
        return Err(Error::ToolFailed { tool: "ffmpeg", message: msg });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_for_mp3() {
        let a = build_args(Path::new("in.mp4"), Path::new("out.mp3"), "mp3").unwrap();
        assert!(a.contains(&"libmp3lame".to_string()));
        assert_eq!(a.last().unwrap(), "out.mp3");
    }

    #[test]
    fn rejects_unknown_target() {
        assert!(codec_args("exe").is_err());
    }
}
