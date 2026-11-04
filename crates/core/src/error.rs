use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    #[error("that link is not a {expected} link")]
    WrongPlatform { expected: &'static str },
    #[error("required tool `{0}` was not found on PATH")]
    ToolMissing(&'static str),
    #[error("`{tool}` failed: {message}")]
    ToolFailed { tool: &'static str, message: String },
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("could not detect a format for {0}")]
    UnknownExtension(PathBuf),
    #[error("image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
