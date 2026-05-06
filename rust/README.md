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

Bundle the plug-in (produces both `.clap` and `.vst3` under
`target/bundled/`):

```bash
cargo xtask bundle plugin-synth --release
```

## macOS

The workspace targets CoreAudio (via `cpal`) and CoreMIDI (via `midir`) on
macOS — no system libraries need installing. Apple Silicon (`aarch64-apple-darwin`)
and Intel (`x86_64-apple-darwin`) are both supported and configured with
sensible default `-C target-cpu` flags in `.cargo/config.toml`.

### Apple Silicon native build

```bash
rustup target add aarch64-apple-darwin
cargo build --workspace --release --target aarch64-apple-darwin
cargo xtask bundle plugin-synth --release --target aarch64-apple-darwin
```

### Universal 2 bundle (M-series + Intel in one .vst3 / .clap)

```bash
./scripts/mac-bundle-universal.sh plugin-synth
```

Outputs go to `target/universal-bundled/`. The script `lipo`s the two
per-architecture builds together.

### Install plug-ins to the user library

```bash
./scripts/mac-install-plugin.sh target/bundled            # single arch
./scripts/mac-install-plugin.sh target/universal-bundled  # universal
```

This copies `*.vst3` and `*.clap` into `~/Library/Audio/Plug-Ins/{VST3,CLAP}/`.

### CoreAudio host explicitly

`AudioDevice::open_default_output` already picks CoreAudio on macOS via
`cpal::default_host()`. If you want to be explicit (e.g. when both AVFoundation
and CoreAudio are options on a future cpal version), use:

```rust
#[cfg(target_os = "macos")]
let device = AudioDevice::open_coreaudio_output(config, callback)?;
```

## Status

Skeleton. See `ROADMAP.md` for the migration plan and what is intentionally
out of scope.
