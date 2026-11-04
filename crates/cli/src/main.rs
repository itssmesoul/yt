use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use swiss_kit_core::convert::image_conv::{convert_image, ImageKind};
use swiss_kit_core::convert::media::{convert_media, supported_targets};
use swiss_kit_core::download::{download, DownloadRequest, Format, Platform, Progress};
use tokio::sync::mpsc::unbounded_channel;

#[derive(Parser)]
#[command(name = "swiss-kit", version, about = "Download and convert media")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Clone, Copy, ValueEnum)]
enum Fmt {
    Mp4,
    Mp3,
}

#[derive(Subcommand)]
enum Cmd {
    /// Download a YouTube video as mp4 or mp3
    Youtube(DlArgs),
    /// Download a TikTok video as mp4 or mp3
    Tiktok(DlArgs),
    /// Convert an image (webp/png/jpg/gif/bmp/tiff)
    Image {
        input: PathBuf,
        /// Target format, e.g. jpg, png, webp
        to: String,
        #[arg(short, long, default_value_t = 90)]
        quality: u8,
        #[arg(short, long)]
        out_dir: Option<PathBuf>,
    },
    /// Convert audio/video with ffmpeg (mp3, wav, m4a, ogg, flac, mp4, webm, gif)
    Media {
        input: PathBuf,
        to: String,
        #[arg(short, long)]
        out_dir: Option<PathBuf>,
    },
}

#[derive(clap::Args)]
struct DlArgs {
    url: String,
    #[arg(short, long, value_enum, default_value = "mp4")]
    format: Fmt,
    /// Max video height, e.g. 1080
    #[arg(short = 'q', long)]
    max_height: Option<u32>,
    #[arg(short, long, default_value = ".")]
    out_dir: PathBuf,
}

async fn run_download(platform: Platform, a: DlArgs) -> swiss_kit_core::Result<()> {
    let req = DownloadRequest {
        platform,
        url: a.url,
        format: match a.format { Fmt::Mp4 => Format::Mp4, Fmt::Mp3 => Format::Mp3 },
        max_height: a.max_height,
        out_dir: a.out_dir,
    };
    let (tx, mut rx) = unbounded_channel();
    let printer = tokio::spawn(async move {
        while let Some(p) = rx.recv().await {
            if let Progress::Percent(p) = p {
                eprint!("\r{p:5.1}%");
            }
        }
        eprintln!();
    });
    let file = download(&req, Some(tx)).await;
    let _ = printer.await;
    println!("saved: {}", file?.display());
    Ok(())
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let result = match cli.cmd {
        Cmd::Youtube(a) => run_download(Platform::YouTube, a).await,
        Cmd::Tiktok(a) => run_download(Platform::TikTok, a).await,
        Cmd::Image { input, to, quality, out_dir } => ImageKind::from_ext(&to)
            .and_then(|k| convert_image(&input, k, quality, out_dir.as_deref()))
            .map(|p| println!("saved: {}", p.display())),
        Cmd::Media { input, to, out_dir } => convert_media(&input, &to, out_dir.as_deref())
            .await
            .map(|p| println!("saved: {}", p.display())),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        if matches!(e, swiss_kit_core::Error::UnsupportedFormat(_)) {
            eprintln!("media targets: {}", supported_targets().join(", "));
        }
        std::process::exit(1);
    }
}
