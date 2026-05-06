# juce-daito (Rust)

Rust workspace re-imagining the audio / DSP / plug-in slice of JUCE.
Lives alongside the original C++ tree in `../`. Existing JUCE C++ code is
**not** removed — the two trees can co-exist while migration progresses.

## Layout

```
rust/
├── Cargo.toml              workspace root
├── crates/
│   ├── jd-core             primitive types, error, time, atomics      (~ juce_core)
│   ├── jd-dsp              oscillators, filters, ADSR, gain, FFT      (~ juce_dsp)
│   ├── jd-audio-io         cpal + midir wrappers                      (~ juce_audio_devices)
│   ├── jd-formats          symphonia (read) + hound (write)           (~ juce_audio_formats)
│   └── jd-gui              egui widgets and theme                     (~ juce_gui_basics, partial)
└── examples/
    ├── standalone-synth    eframe app driving cpal directly
    └── plugin-synth        nih-plug VST3 / CLAP plug-in
```

## Building

```bash
cd rust
cargo build --workspace
cargo run -p standalone-synth
```

For the plug-in, follow the `nih-plug` bundle workflow (see `ROADMAP.md`).

## Status

Skeleton. See `ROADMAP.md` for the migration plan and what is intentionally
out of scope.
