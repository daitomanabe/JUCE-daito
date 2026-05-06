//! jd-gui: minimal counterpart of `juce_gui_basics`.
//!
//! Reusable widgets layered on top of `egui`. The full JUCE component tree is
//! intentionally NOT replicated — instead this crate hosts a small library of
//! audio-oriented widgets that fit the way egui apps and `nih-plug` plug-in
//! editors are typically structured.

pub mod meter;
pub mod spectrum;
pub mod theme;
pub mod waveform;
pub mod widgets;
pub mod xy_pad;
