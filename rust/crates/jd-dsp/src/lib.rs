//! jd-dsp: counterpart of `juce_dsp`.
//!
//! Building blocks for audio processing: oscillators, filters, FFT, gain,
//! envelope followers, etc. Each module is intentionally minimal and
//! `no_std`-friendly where possible (allocation only for buffers).

pub mod buffer;
pub mod envelope;
pub mod fft;
pub mod filter;
pub mod gain;
pub mod oscillator;

pub use buffer::AudioBuffer;
