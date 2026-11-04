//! Core library for Swiss Kit: downloaders + converters. UI-agnostic so a CLI,
//! a Tauri app, or anything else can sit on top.
pub mod convert;
pub mod download;
pub mod error;
pub mod tools;

pub use error::{Error, Result};
