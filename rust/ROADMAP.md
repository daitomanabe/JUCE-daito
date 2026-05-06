# Migration roadmap

This is a focused subset migration of JUCE — **audio, DSP, and plug-in
hosting/clients** — re-implemented in Rust. The full JUCE GUI / graphics /
video / JS / cryptography / analytics surface is intentionally **out of
scope**. Where JUCE provides functionality that already has a high-quality
Rust crate, we wrap rather than re-implement.

## Module mapping

| JUCE C++ module             | Rust replacement                          | Status        |
|-----------------------------|-------------------------------------------|---------------|
| `juce_core`                 | `std` + `jd-core`                         | skeleton      |
| `juce_dsp`                  | `jd-dsp` (+ `realfft`, `rustfft`)         | skeleton      |
| `juce_audio_basics`         | `jd-dsp::buffer`, `jd-audio-io::midi`     | skeleton      |
| `juce_audio_devices`        | `jd-audio-io` (`cpal`, `midir`)           | skeleton      |
| `juce_audio_formats`        | `jd-formats` (`symphonia`, `hound`)       | skeleton      |
| `juce_audio_processors`     | `nih-plug` host + `jd-dsp`                | adopt nih-plug |
| `juce_audio_plugin_client`  | `nih-plug` (VST3 / CLAP / AU)             | adopt nih-plug |
| `juce_audio_utils`          | _portions only, on demand_                | not started   |
| `juce_data_structures`      | `serde` + small helpers in `jd-core`      | not started   |
| `juce_events`               | `crossbeam-channel` / `tokio`             | adopted        |
| `juce_gui_basics`           | `jd-gui` widgets on `egui`                | partial       |
| `juce_graphics`             | `egui` painter (no JUCE-style retained tree) | partial    |
| `juce_opengl`               | `eframe`'s `glow` integration             | adopted       |
| `juce_osc`                  | `rosc` crate (planned)                    | not started   |
| `juce_cryptography`         | `ring` / `rustls` if needed (out of scope) | not started   |
| `juce_javascript`           | _out of scope_                            | dropped       |
| `juce_video`                | _out of scope_                            | dropped       |
| `juce_box2d`                | _out of scope_                            | dropped       |
| `juce_analytics`            | _out of scope_                            | dropped       |
| `juce_product_unlocking`    | _out of scope_                            | dropped       |
| `juce_animation`            | egui animation primitives                 | not started   |
| `juce_midi_ci`              | _out of scope (initial)_                  | dropped       |

## Phases

1. **Skeleton** (this commit) — workspace builds, examples run on host audio.
2. **DSP fill-out** — port the oscillator/filter/FFT/window helpers used by
   the JUCE DSP examples; add tests and benchmarks (`criterion`).
3. **Plug-in client parity** — replace `juce_audio_plugin_client` use cases
   with `nih-plug` instrument and effect templates.
4. **GUI parity (audio-focused)** — knob, slider, level meter, spectrum,
   waveform display in `jd-gui`. No JUCE Component tree clone.
5. **Format parity** — confirm WAV/AIFF/FLAC/MP3 read paths through
   `symphonia`; AIFF write via `hound` if needed.
6. **CI** — `cargo build --workspace`, `cargo test`, `cargo clippy`,
   plug-in bundle build via `nih-plug`'s `cargo xtask`.

## Plug-in build

`nih-plug` ships an `xtask` for bundling. Once integrated:

```bash
cargo xtask bundle plugin-synth --release
```

Outputs land in `target/bundled/`. Both VST3 (`.vst3`) and CLAP (`.clap`)
artifacts are produced from the same crate.

### macOS

Apple Silicon and Intel each have their own `target-cpu` baseline configured
in `.cargo/config.toml`. To produce a Universal 2 bundle covering both, run
`./scripts/mac-bundle-universal.sh plugin-synth` — this performs both
per-arch bundle builds and `lipo`s the resulting Mach-O binaries.

`./scripts/mac-install-plugin.sh` copies bundles into the user's
`~/Library/Audio/Plug-Ins/{VST3,CLAP}/`.

AU (Audio Unit) bundling is not yet supported by `nih-plug` upstream. When
that lands we can add it to the same xtask invocation; the plug-in trait
implementations remain unchanged.

## What we don't try to reproduce

- JUCE's `Component` / `LookAndFeel` retained-mode GUI tree. egui is
  immediate-mode and the porting cost of mimicking JUCE's API is not worth
  the API ergonomics loss vs. idiomatic egui.
- `Projucer`. Cargo replaces it.
- AAX / RTAS. Pro Tools support requires Avid SDK access — out of scope
  unless a contributor brings the SDK and bindings.
- iOS / Android targets. Possible later via `nih-plug`'s standalone mode +
  platform-specific glue, not a near-term goal.
