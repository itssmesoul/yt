//! Locating the external programs we shell out to.
use crate::error::{Error, Result};
use std::path::PathBuf;

pub fn find(name: &'static str) -> Result<PathBuf> {
    // 1) next to our own executable (portable setup), 2) PATH
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())) {
        dirs.push(dir);
    }
    if let Some(path) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&path));
    }
    for dir in dirs {
        for candidate in [name.to_string(), format!("{name}.exe")] {
            let full = dir.join(&candidate);
            if full.is_file() {
                return Ok(full);
            }
        }
    }
    Err(Error::ToolMissing(name))
}

/// Build a tokio Command that does not flash a console window on Windows.
pub fn command(program: impl AsRef<std::ffi::OsStr>) -> tokio::process::Command {
    #[allow(unused_mut)]
    let mut c = tokio::process::Command::new(program);
    #[cfg(windows)]
    c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    c
}
