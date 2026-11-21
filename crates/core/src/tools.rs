//! Locating the external programs we shell out to.
use crate::error::{Error, Result};
use std::path::PathBuf;

pub fn find(name: &'static str) -> Result<PathBuf> {
    let path = std::env::var_os("PATH").ok_or(Error::ToolMissing(name))?;
    for dir in std::env::split_paths(&path) {
        for candidate in [name.to_string(), format!("{name}.exe")] {
            let full = dir.join(&candidate);
            if full.is_file() {
                return Ok(full);
            }
        }
    }
    Err(Error::ToolMissing(name))
}
