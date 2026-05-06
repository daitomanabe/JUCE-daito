//! jd-audio-io: counterpart of `juce_audio_devices`.
//!
//! Wraps `cpal` (CoreAudio / WASAPI / ALSA / JACK) and `midir` (CoreMIDI /
//! WinMM / ALSA-seq) under a small JUCE-flavoured API.

pub mod audio;
pub mod midi;

pub use audio::{AudioCallback, AudioDevice, AudioDeviceConfig};
pub use midi::{MidiInput, MidiMessage};
