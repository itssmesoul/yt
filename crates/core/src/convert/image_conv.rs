//! Image conversions using the pure-Rust `image` crate.
use super::{ext_of, output_path};
use crate::error::{Error, Result};
use image::{codecs::jpeg::JpegEncoder, DynamicImage, ImageFormat, Rgba, RgbaImage};
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageKind {
    Jpg,
    Png,
    WebP,
    Gif,
    Bmp,
    Tiff,
}

impl ImageKind {
    pub fn from_ext(ext: &str) -> Result<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "jpg" | "jpeg" => Ok(Self::Jpg),
            "png" => Ok(Self::Png),
            "webp" => Ok(Self::WebP),
            "gif" => Ok(Self::Gif),
            "bmp" => Ok(Self::Bmp),
            "tif" | "tiff" => Ok(Self::Tiff),
            "heic" | "heif" => Err(Error::UnsupportedFormat("HEIC is not supported yet".into())),
            other => Err(Error::UnsupportedFormat(other.to_string())),
        }
    }

    pub fn ext(self) -> &'static str {
        match self {
            Self::Jpg => "jpg",
            Self::Png => "png",
            Self::WebP => "webp",
            Self::Gif => "gif",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
        }
    }

    fn format(self) -> ImageFormat {
        match self {
            Self::Jpg => ImageFormat::Jpeg,
            Self::Png => ImageFormat::Png,
            Self::WebP => ImageFormat::WebP,
            Self::Gif => ImageFormat::Gif,
            Self::Bmp => ImageFormat::Bmp,
            Self::Tiff => ImageFormat::Tiff,
        }
    }
}

/// Flatten transparency onto white (JPEG has no alpha channel).
fn flatten_on_white(img: &DynamicImage) -> DynamicImage {
    let rgba = img.to_rgba8();
    let mut bg = RgbaImage::from_pixel(rgba.width(), rgba.height(), Rgba([255, 255, 255, 255]));
    image::imageops::overlay(&mut bg, &rgba, 0, 0);
    DynamicImage::ImageRgba8(bg).to_rgb8().into()
}

/// Convert `input` to `target`. Returns the written file path.
pub fn convert_image(input: &Path, target: ImageKind, quality: u8, out_dir: Option<&Path>) -> Result<PathBuf> {
    let src_ext = ext_of(input).ok_or_else(|| Error::UnknownExtension(input.to_path_buf()))?;
    ImageKind::from_ext(&src_ext)?;
    let img = image::open(input)?;
    let out = output_path(input, target.ext(), out_dir)?;

    match target {
        ImageKind::Jpg => {
            let rgb = flatten_on_white(&img).to_rgb8();
            let writer = BufWriter::new(File::create(&out)?);
            JpegEncoder::new_with_quality(writer, quality.clamp(1, 100)).encode_image(&rgb)?;
        }
        other => img.save_with_format(&out, other.format())?,
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(path: &Path, fmt: ImageFormat) {
        let img = RgbaImage::from_fn(32, 24, |x, y| Rgba([(x * 8) as u8, (y * 10) as u8, 120, 255]));
        DynamicImage::ImageRgba8(img).save_with_format(path, fmt).unwrap();
    }

    #[test]
    fn webp_to_jpg() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("pic.webp");
        sample(&src, ImageFormat::WebP);
        let out = convert_image(&src, ImageKind::Jpg, 90, None).unwrap();
        assert_eq!(out.file_name().unwrap(), "pic.jpg");
        let back = image::open(&out).unwrap();
        assert_eq!((back.width(), back.height()), (32, 24));
    }

    #[test]
    fn png_with_alpha_to_jpg_flattens_on_white() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("t.png");
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(8, 8, Rgba([0, 0, 0, 0]))).save(&src).unwrap();
        let out = convert_image(&src, ImageKind::Jpg, 95, None).unwrap();
        let px = image::open(out).unwrap().to_rgb8().get_pixel(4, 4).0;
        assert!(px.iter().all(|c| *c > 240), "expected near-white, got {px:?}");
    }

    #[test]
    fn jpg_to_png_to_webp_and_no_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("a.jpg");
        sample(&src, ImageFormat::Jpeg);
        let png = convert_image(&src, ImageKind::Png, 90, None).unwrap();
        let png2 = convert_image(&src, ImageKind::Png, 90, None).unwrap();
        assert_ne!(png, png2);
        let webp = convert_image(&png, ImageKind::WebP, 90, None).unwrap();
        assert!(image::open(webp).is_ok());
    }

    #[test]
    fn rejects_heic_and_unknown() {
        assert!(ImageKind::from_ext("heic").is_err());
        assert!(ImageKind::from_ext("xyz").is_err());
    }
}
