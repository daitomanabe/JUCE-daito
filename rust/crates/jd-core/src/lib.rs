//! jd-core: counterpart of `juce_core`.
//!
//! Provides primitive types, error handling, and shared utilities used by the
//! rest of the workspace. Most of `juce_core` (strings, files, threads, time)
//! maps directly onto Rust's `std`, so this crate stays intentionally small —
//! only things that are not idiomatic in `std` live here.

pub mod atomic;
pub mod error;
pub mod range;
pub mod time;

pub use error::{Error, Result};
