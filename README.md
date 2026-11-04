# Swiss Kit

Media Swiss knife in Rust: YouTube/TikTok to MP4/MP3, plus common image and audio/video converters.

## Layout
- `crates/core` - UI-agnostic library (downloads, converters). A Tauri UI will sit on this.
- `crates/cli` - `swiss-kit` command line tool.
- `crates/app` - Tauri 2 desktop app (UI in `crates/app/ui/index.html`).

## Requirements
- Rust (recent stable)
- `ffmpeg` on PATH (mp3/mp4 conversion, merging)
- `yt-dlp` on PATH (YouTube / TikTok downloads)

## Usage
```
cargo run -p swiss-kit -- youtube <url> -f mp3 -o ~/Downloads
cargo run -p swiss-kit -- youtube <url> -f mp4 -q 720
cargo run -p swiss-kit -- tiktok <url> -f mp4
cargo run -p swiss-kit -- image photo.webp jpg -q 90
cargo run -p swiss-kit -- media clip.mov mp4
cargo run -p swiss-kit -- media video.mp4 mp3
```
Converted files are written next to the input and never overwrite existing files.

## Status
- Image conversion (webp/png/jpg/gif/bmp/tiff): tested
- ffmpeg conversion (mp3, wav, m4a, ogg, flac, mp4, webm, gif): mp3 tested end to end
- YouTube/TikTok download: argument building, URL validation and progress parsing tested; not yet run against live links
- HEIC: not supported yet

## Next
Tauri desktop UI matching the design preview; batch conversion; HEIC support.

Only download content you own or have permission to save; respect each site's terms.

## Desktop app
Windows: run `build-windows.bat`. It builds `dist\Swiss Kit.exe` and downloads `yt-dlp.exe` + `ffmpeg.exe` next to it
(the app also finds them on PATH). Needs Rust and WebView2 (built into Windows 11).
Dev run: `cargo run -p swiss-kit-app`.
HEIC to JPG goes through ffmpeg, so it depends on your ffmpeg build having a HEIC decoder.
