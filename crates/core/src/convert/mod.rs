pub mod image_conv;
pub mod media;

use crate::error::{Error, Result};
use std::path::{Path, PathBuf};

pub(crate) fn ext_of(path: &Path) -> Option<String> {
    path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase())
}

/// `photo.webp` + "jpg" -> `photo.jpg` (next to the input unless `out_dir` is given).
/// Never returns a path that already exists: appends " (1)", " (2)", ...
pub fn output_path(input: &Path, new_ext: &str, out_dir: Option<&Path>) -> Result<PathBuf> {
    let stem = input.file_stem().ok_or_else(|| Error::UnknownExtension(input.to_path_buf()))?;
    let dir = out_dir.map(Path::to_path_buf).unwrap_or_else(|| input.parent().map(Path::to_path_buf).unwrap_or_default());
    let mut candidate = dir.join(format!("{}.{new_ext}", stem.to_string_lossy()));
    let mut n = 1;
    while candidate.exists() {
        candidate = dir.join(format!("{} ({n}).{new_ext}", stem.to_string_lossy()));
        n += 1;
    }
    Ok(candidate)
}
