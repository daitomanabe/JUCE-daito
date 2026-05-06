//! Standalone sine-wave synth driven by jd-audio-io and jd-gui.

use jd_audio_io::{AudioCallback, AudioDevice, AudioDeviceConfig};
use jd_core::atomic::AtomicF32;
use jd_dsp::{envelope::Adsr, oscillator};
use std::sync::Arc;

struct SharedState {
    freq_hz: AtomicF32,
    gain: AtomicF32,
    gate: std::sync::atomic::AtomicBool,
}

struct Engine {
    state: Arc<SharedState>,
    osc: oscillator::Oscillator<Box<dyn FnMut(f32) -> f32 + Send>>,
    env: Adsr,
    sample_rate: f32,
    last_freq: f32,
}

impl AudioCallback for Engine {
    fn process(&mut self, output: &mut [f32], num_channels: usize, sample_rate: f32) {
        if (sample_rate - self.sample_rate).abs() > f32::EPSILON {
            self.sample_rate = sample_rate;
            self.env = Adsr::new(sample_rate);
        }

        let freq = self.state.freq_hz.load();
        if (freq - self.last_freq).abs() > f32::EPSILON {
            self.osc.set_frequency(freq, sample_rate);
            self.last_freq = freq;
        }

        let gain = self.state.gain.load();
        let gate = self.state.gate.load(std::sync::atomic::Ordering::Relaxed);
        if gate && !self.env.is_active() {
            self.env.note_on();
        } else if !gate && self.env.is_active() {
            self.env.note_off();
        }

        let frames = output.len() / num_channels;
        for frame in 0..frames {
            let s = self.osc.process() * self.env.tick() * gain;
            for ch in 0..num_channels {
                output[frame * num_channels + ch] = s;
            }
        }
    }
}

struct App {
    state: Arc<SharedState>,
    _device: AudioDevice,
}

impl App {
    fn new() -> Self {
        let state = Arc::new(SharedState {
            freq_hz: AtomicF32::new(440.0),
            gain: AtomicF32::new(0.2),
            gate: std::sync::atomic::AtomicBool::new(false),
        });

        let mut osc = oscillator::Oscillator::new(
            Box::new(|p: f32| p.sin()) as Box<dyn FnMut(f32) -> f32 + Send>
        );
        osc.set_frequency(440.0, 48_000.0);

        let engine = Engine {
            state: Arc::clone(&state),
            osc,
            env: Adsr::new(48_000.0),
            sample_rate: 48_000.0,
            last_freq: 440.0,
        };

        let device = AudioDevice::open_default_output(AudioDeviceConfig::default(), engine)
            .expect("failed to open audio output");

        Self {
            state,
            _device: device,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(jd_gui::theme::dark_audio());

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("juce-daito · standalone synth");
            ui.add_space(8.0);

            let mut freq = self.state.freq_hz.load();
            if ui
                .add(
                    egui::Slider::new(&mut freq, 20.0..=4000.0)
                        .logarithmic(true)
                        .text("Freq Hz"),
                )
                .changed()
            {
                self.state.freq_hz.store(freq);
            }

            let mut gain = self.state.gain.load();
            if ui
                .add(egui::Slider::new(&mut gain, 0.0..=1.0).text("Gain"))
                .changed()
            {
                self.state.gain.store(gain);
            }

            ui.add_space(8.0);
            let pressed = ui.button("Hold to play").is_pointer_button_down_on();
            self.state
                .gate
                .store(pressed, std::sync::atomic::Ordering::Relaxed);
        });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    env_logger::init();
    eframe::run_native(
        "juce-daito standalone-synth",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(App::new()))),
    )
}
