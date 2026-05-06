//! jd-formats: counterpart of `juce_audio_formats`.
//!
//! Reading: `symphonia` (WAV/FLAC/MP3/AAC/M4A).
//! Writing: `hound` (WAV — the only format JUCE itself bundles a writer for
//! by default in most projects).

pub mod reader;
pub mod writer;
