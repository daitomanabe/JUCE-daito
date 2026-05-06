//! Minimal monophonic sine synth plug-in built on `nih_plug`. Renders to
//! VST3 / CLAP via `nih_plug`'s build system (`cargo xtask bundle plugin-synth`).

use jd_dsp::{envelope::Adsr, oscillator};
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};
use std::sync::Arc;

struct PluginSynth {
    params: Arc<SynthParams>,
    osc: oscillator::Oscillator<Box<dyn FnMut(f32) -> f32 + Send>>,
    env: Adsr,
    sample_rate: f32,
    current_note: Option<u8>,
}

#[derive(Params)]
struct SynthParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "gain"]
    gain: FloatParam,

    #[id = "attack"]
    attack: FloatParam,

    #[id = "release"]
    release: FloatParam,
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(360, 220),
            gain: FloatParam::new("Gain", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }),
            attack: FloatParam::new(
                "Attack",
                0.01,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 2.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" s"),
            release: FloatParam::new(
                "Release",
                0.2,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 4.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" s"),
        }
    }
}

impl Default for PluginSynth {
    fn default() -> Self {
        let mut osc = oscillator::Oscillator::new(
            Box::new(|p: f32| p.sin()) as Box<dyn FnMut(f32) -> f32 + Send>,
        );
        osc.set_frequency(440.0, 48_000.0);
        Self {
            params: Arc::new(SynthParams::default()),
            osc,
            env: Adsr::new(48_000.0),
            sample_rate: 48_000.0,
            current_note: None,
        }
    }
}

impl Plugin for PluginSynth {
    const NAME: &'static str = "juce-daito plugin-synth";
    const VENDOR: &'static str = "juce-daito";
    const URL: &'static str = "https://github.com/daitomanabe/juce-daito";
    const EMAIL: &'static str = "noreply@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |ctx, setter, _| {
                ctx.set_visuals(jd_gui::theme::dark_audio());
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("plugin-synth");
                    ui.add_space(8.0);
                    param_slider(ui, setter, &params.gain);
                    param_slider(ui, setter, &params.attack);
                    param_slider(ui, setter, &params.release);
                });
            },
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        self.env = Adsr::new(self.sample_rate);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let attack = self.params.attack.smoothed.next();
        let release = self.params.release.smoothed.next();
        self.env.set_parameters(attack, 0.05, 0.85, release);

        for (sample_idx, channel_samples) in buffer.iter_samples().enumerate() {
            while let Some(event) = context.next_event() {
                if event.timing() != sample_idx as u32 {
                    continue;
                }
                match event {
                    NoteEvent::NoteOn { note, .. } => {
                        let freq = util::midi_note_to_freq(note);
                        self.osc.set_frequency(freq, self.sample_rate);
                        self.current_note = Some(note);
                        self.env.note_on();
                    }
                    NoteEvent::NoteOff { note, .. } if Some(note) == self.current_note => {
                        self.env.note_off();
                        self.current_note = None;
                    }
                    _ => {}
                }
            }

            let gain = self.params.gain.smoothed.next();
            let s = self.osc.process() * self.env.next() * gain;
            for sample in channel_samples {
                *sample = s;
            }
        }

        ProcessStatus::Normal
    }
}

fn param_slider(ui: &mut egui::Ui, setter: &ParamSetter, param: &FloatParam) {
    ui.horizontal(|ui| {
        ui.label(param.name());
        let mut value = param.value();
        if ui
            .add(
                egui::Slider::new(&mut value, param.range().min()..=param.range().max())
                    .show_value(true),
            )
            .changed()
        {
            setter.begin_set_parameter(param);
            setter.set_parameter(param, value);
            setter.end_set_parameter(param);
        }
    });
}

trait FloatRangeExt {
    fn min(&self) -> f32;
    fn max(&self) -> f32;
}

impl FloatRangeExt for FloatRange {
    fn min(&self) -> f32 {
        match self {
            FloatRange::Linear { min, .. } => *min,
            FloatRange::Skewed { min, .. } => *min,
            FloatRange::SymmetricalSkewed { min, .. } => *min,
            FloatRange::Reversed(inner) => inner.max(),
        }
    }
    fn max(&self) -> f32 {
        match self {
            FloatRange::Linear { max, .. } => *max,
            FloatRange::Skewed { max, .. } => *max,
            FloatRange::SymmetricalSkewed { max, .. } => *max,
            FloatRange::Reversed(inner) => inner.min(),
        }
    }
}

impl ClapPlugin for PluginSynth {
    const CLAP_ID: &'static str = "com.juce-daito.plugin-synth";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("juce-daito demo synth");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for PluginSynth {
    const VST3_CLASS_ID: [u8; 16] = *b"jdaito-synth-001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}

nih_export_clap!(PluginSynth);
nih_export_vst3!(PluginSynth);
