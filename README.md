# Swiss Kit

Local media toolkit in Rust: download from YouTube, TikTok, Instagram, X, Facebook, Vimeo, Reddit, SoundCloud, Twitch, Dailymotion, Pinterest or almost any other site that yt-dlp supports, as MP4 or MP3, plus image, audio and video converters. The browser UI is available in English and Turkish.
Runs on your own computer with a browser UI.

- **Using it, or sending it to friends:** read [`dist/README.md`](dist/README.md) (English) or [`dist/README.tr.md`](dist/README.tr.md) (Türkçe). That file is the user guide and is shipped inside the zip.
- **Building a zip for friends (Windows):** double-click `build-release.bat`. It builds the app and creates `Swiss-Kit-Windows.zip` containing the app, the launcher, the tool installer and the guide.

## Layout
- `crates/core` - UI-agnostic library: downloads (yt-dlp, 11 named sites + any site), converters (image crate, ffmpeg), video info
- `crates/app` - local web server + browser UI (`swiss-kit-app`), binds to 127.0.0.1 only and checks Host/Origin. UI strings live in the `I18N` object in `crates/app/ui/index.html` (add a language there)
- `crates/cli` - command-line tool (`swiss-kit`)
- `dist/` - files shipped with the app (guide, launcher, tool installer)

## Develop
```
cargo test
cargo run -p swiss-kit-app
```
Needs `ffmpeg` and `yt-dlp` on PATH for everything except image conversion.
