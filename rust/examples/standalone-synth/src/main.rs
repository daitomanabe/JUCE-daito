//! Standalone monophonic synth driven by jd-audio-io and jd-gui.
//!
//! Plays a sine wave triggered by the on-screen "hold to play" button or by
//! a connected MIDI input device (last-note priority). The GUI also shows a
//! live waveform, level meter and an XY pad bound to filter cutoff /
//! resonance.

use crossbeam_channel::Receiver;
use jd_audio_io::midi::MidiMessage;
use jd_audio_io::{AudioCallback, AudioDevice, AudioDeviceConfig, MidiInput};
use jd_core::atomic::AtomicF32;
use jd_dsp::svf::{Svf, SvfMode};
use jd_dsp::{envelope::Adsr, oscillator};
use jd_gui::meter::PeakHoldMeter;
use jd_gui::waveform::ScrollingBuffer;
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const VIS_BUFFER_LEN: usize = 4096;

struct SharedState {
    freq_hz: AtomicF32,
    gain: AtomicF32,
    cutoff_norm: AtomicF32,
    resonance_norm: AtomicF32,
    gui_gate: AtomicBool,
    out_peak: AtomicF32,
}

struct Engine {
    state: Arc<SharedState>,
    midi_rx: Option<Receiver<MidiMessage>>,
    osc: oscillator::Oscillator<Box<dyn FnMut(f32) -> f32 + Send>>,
    env: Adsr,
    svf: Svf,
    sample_rate: f32,
    last_freq: f32,
    midi_held_note: Option<u8>,
    last_gui_gate: bool,
    vis_tx: ringbuf::HeapProd<f32>,
}

impl Engine {
    fn handle_midi(&mut self) {
        let Some(rx) = self.midi_rx.as_ref() else {
            return;
        };
        while let Ok(msg) = rx.try_recv() {
            if msg.is_note_on() {
                if let (Some(note), Some(_vel)) = (msg.note_number(), msg.velocity()) {
                    let freq = midi_note_to_hz(note);
                    self.osc.set_frequency(freq, self.sample_rate);
                    self.state.freq_hz.store(freq);
                    self.last_freq = freq;
                    self.midi_held_note = Some(note);
                    self.env.note_on();
                }
            } else if msg.is_note_off()
                && Some(msg.note_number().unwrap_or(0)) == self.midi_held_note
            {
                self.env.note_off();
                self.midi_held_note = None;
            }
        }
    }

    fn handle_gui_gate(&mut self) {
        let gate = self.state.gui_gate.load(Ordering::Relaxed);
        if gate && !self.last_gui_gate {
            let freq = self.state.freq_hz.load();
            if (freq - self.last_freq).abs() > f32::EPSILON {
                self.osc.set_frequency(freq, self.sample_rate);
                self.last_freq = freq;
            }
            self.env.note_on();
        } else if !gate && self.last_gui_gate && self.midi_held_note.is_none() {
            self.env.note_off();
        }
        self.last_gui_gate = gate;
    }
}

impl AudioCallback for Engine {
    fn process(&mut self, output: &mut [f32], num_channels: usize, sample_rate: f32) {
        if (sample_rate - self.sample_rate).abs() > f32::EPSILON {
            self.sample_rate = sample_rate;
            self.env = Adsr::new(sample_rate);
            self.osc.set_frequency(self.last_freq, sample_rate);
        }

        self.handle_midi();
        self.handle_gui_gate();

        if self.last_gui_gate {
            let freq = self.state.freq_hz.load();
            if (freq - self.last_freq).abs() > f32::EPSILON {
                self.osc.set_frequency(freq, sample_rate);
                self.last_freq = freq;
            }
        }

        // Map normalised XY to cutoff (log) and Q.
        let cutoff_norm = self.state.cutoff_norm.load().clamp(0.0, 1.0);
        let res_norm = self.state.resonance_norm.load().clamp(0.0, 1.0);
        let cutoff = 40.0 * (sample_rate * 0.45 / 40.0).powf(cutoff_norm);
        let q = 0.5 + res_norm * 9.5;
        self.svf.set(cutoff, q, sample_rate);
        self.svf.set_mode(SvfMode::Lowpass);

        let gain = self.state.gain.load();
        let frames = output.len() / num_channels;
        let mut peak: f32 = 0.0;
        for frame in 0..frames {
            let raw = self.osc.process() * self.env.tick();
            let s = self.svf.process(raw) * gain;
            peak = peak.max(s.abs());
            self.vis_tx.try_push(s).ok();
            for ch in 0..num_channels {
                output[frame * num_channels + ch] = s;
            }
        }
        // Smoothed peak update — read by GUI thread.
        let prev = self.state.out_peak.load();
        let smoothed = (prev * 0.7).max(peak);
        self.state.out_peak.store(smoothed);
    }
}

fn midi_note_to_hz(note: u8) -> f32 {
    440.0 * 2f32.powf((note as f32 - 69.0) / 12.0)
}

struct App {
    state: Arc<SharedState>,
    _device: AudioDevice,
    _midi: Option<MidiInput>,
    midi_status: String,
    vis_rx: ringbuf::HeapCons<f32>,
    vis_buf: ScrollingBuffer,
    meter: PeakHoldMeter,
    last_frame_ms: f64,
}

impl App {
    fn new() -> Self {
        let state = Arc::new(SharedState {
            freq_hz: AtomicF32::new(440.0),
            gain: AtomicF32::new(0.2),
            cutoff_norm: AtomicF32::new(1.0),
            resonance_norm: AtomicF32::new(0.1),
            gui_gate: AtomicBool::new(false),
            out_peak: AtomicF32::new(0.0),
        });

        let mut osc = oscillator::Oscillator::new(
            Box::new(|p: f32| p.sin()) as Box<dyn FnMut(f32) -> f32 + Send>
        );
        osc.set_frequency(440.0, 48_000.0);

        let (midi_input, midi_rx, midi_status) =
            match MidiInput::open_first_available("juce-daito standalone-synth") {
                Ok(m) => {
                    let rx = m.receiver();
                    (
                        Some(m),
                        Some(rx),
                        "MIDI: connected to first port".to_string(),
                    )
                }
                Err(e) => (None, None, format!("MIDI: {e}")),
            };

        let rb = HeapRb::<f32>::new(VIS_BUFFER_LEN * 2);
        let (vis_tx, vis_rx) = rb.split();

        let engine = Engine {
            state: Arc::clone(&state),
            midi_rx,
            osc,
            env: Adsr::new(48_000.0),
            svf: Svf::default(),
            sample_rate: 48_000.0,
            last_freq: 440.0,
            midi_held_note: None,
            last_gui_gate: false,
            vis_tx,
        };

        let device = AudioDevice::open_default_output(AudioDeviceConfig::default(), engine)
            .expect("failed to open audio output");

        Self {
            state,
            _device: device,
            _midi: midi_input,
            midi_status,
            vis_rx,
            vis_buf: ScrollingBuffer::new(VIS_BUFFER_LEN),
            meter: PeakHoldMeter::new(),
            last_frame_ms: jd_core::time::now_millis(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(jd_gui::theme::dark_audio());

        // Drain audio samples for the waveform display.
        while let Some(s) = self.vis_rx.try_pop() {
            self.vis_buf.push(s);
        }

        let now = jd_core::time::now_millis();
        let dt_ms = (now - self.last_frame_ms) as f32;
        self.last_frame_ms = now;
        self.meter.push(self.state.out_peak.load(), dt_ms.max(1.0));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("juce-daito · standalone synth");
            ui.add_space(4.0);
            ui.label(&self.midi_status);
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
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

                    ui.add_space(4.0);
                    ui.label("Filter (cutoff x · resonance y)");
                    let mut x = self.state.cutoff_norm.load();
                    let mut y = self.state.resonance_norm.load();
                    let resp = jd_gui::xy_pad::xy_pad(ui, &mut x, &mut y, egui::vec2(140.0, 140.0));
                    if resp.dragged() || resp.clicked() {
                        self.state.cutoff_norm.store(x);
                        self.state.resonance_norm.store(y);
                    }

                    ui.add_space(4.0);
                    let pressed = ui.button("Hold to play").is_pointer_button_down_on();
                    self.state.gui_gate.store(pressed, Ordering::Relaxed);
                });

                ui.add_space(12.0);

                ui.vertical(|ui| {
                    ui.label("Waveform");
                    jd_gui::waveform::waveform(
                        ui,
                        &self.vis_buf.snapshot(),
                        egui::vec2(360.0, 100.0),
                    );

                    ui.add_space(8.0);
                    ui.label("Output level");
                    self.meter.ui(ui);
                });
            });
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
