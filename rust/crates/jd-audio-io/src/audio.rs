use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use jd_core::Result;

/// Counterpart of `juce::AudioIODeviceCallback`.
///
/// Called from the realtime thread; allocations are not allowed.
pub trait AudioCallback: Send + 'static {
    fn process(&mut self, output: &mut [f32], num_channels: usize, sample_rate: f32);
}

#[derive(Debug, Clone, Copy)]
pub struct AudioDeviceConfig {
    pub sample_rate: u32,
    pub num_channels: u16,
    pub buffer_size: u32,
}

impl Default for AudioDeviceConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48_000,
            num_channels: 2,
            buffer_size: 512,
        }
    }
}

/// Counterpart of `juce::AudioDeviceManager` (output side only).
pub struct AudioDevice {
    _stream: Stream,
    config: AudioDeviceConfig,
}

impl AudioDevice {
    pub fn open_default_output<C: AudioCallback>(
        config: AudioDeviceConfig,
        mut callback: C,
    ) -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| jd_core::Error::Other("no default output device".into()))?;

        let supported = device
            .default_output_config()
            .map_err(|e| jd_core::Error::Other(e.to_string()))?;

        let stream_config = cpal::StreamConfig {
            channels: config.num_channels,
            sample_rate: cpal::SampleRate(config.sample_rate),
            buffer_size: cpal::BufferSize::Fixed(config.buffer_size),
        };

        let channels = config.num_channels as usize;
        let sample_rate = config.sample_rate as f32;

        let err_fn = |err| log::error!("cpal stream error: {err}");

        let stream = match supported.sample_format() {
            SampleFormat::F32 => device.build_output_stream(
                &stream_config,
                move |data: &mut [f32], _| callback.process(data, channels, sample_rate),
                err_fn,
                None,
            ),
            other => {
                return Err(jd_core::Error::Unsupported(format!(
                    "sample format {other:?}"
                )))
            }
        }
        .map_err(|e| jd_core::Error::Other(e.to_string()))?;

        stream
            .play()
            .map_err(|e| jd_core::Error::Other(e.to_string()))?;

        Ok(Self {
            _stream: stream,
            config,
        })
    }

    pub fn config(&self) -> AudioDeviceConfig {
        self.config
    }
}
