use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, SupportedStreamConfigRange};
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

/// Resolved configuration after negotiating with the device.
#[derive(Debug, Clone, Copy)]
pub struct AudioDeviceInfo {
    pub sample_rate: u32,
    pub num_channels: u16,
    pub buffer_size: BufferSize,
    pub sample_format: SampleFormat,
    pub host_id: cpal::HostId,
}

#[derive(Debug, Clone, Copy)]
pub enum BufferSize {
    Fixed(u32),
    Default,
}

/// Counterpart of `juce::AudioDeviceManager` (output side only).
pub struct AudioDevice {
    _stream: Stream,
    info: AudioDeviceInfo,
}

impl AudioDevice {
    pub fn open_default_output<C: AudioCallback>(
        config: AudioDeviceConfig,
        callback: C,
    ) -> Result<Self> {
        Self::open_with_host(cpal::default_host(), config, callback)
    }

    /// Opens an output stream on the system's preferred CoreAudio device.
    /// Available on macOS only.
    #[cfg(target_os = "macos")]
    pub fn open_coreaudio_output<C: AudioCallback>(
        config: AudioDeviceConfig,
        callback: C,
    ) -> Result<Self> {
        let host = cpal::host_from_id(cpal::HostId::CoreAudio)
            .map_err(|e| jd_core::Error::Other(format!("CoreAudio host: {e}")))?;
        Self::open_with_host(host, config, callback)
    }

    fn open_with_host<C: AudioCallback>(
        host: cpal::Host,
        config: AudioDeviceConfig,
        mut callback: C,
    ) -> Result<Self> {
        let host_id = host.id();
        let device = host
            .default_output_device()
            .ok_or_else(|| jd_core::Error::Other("no default output device".into()))?;

        let supported_configs: Vec<SupportedStreamConfigRange> = device
            .supported_output_configs()
            .map_err(|e| jd_core::Error::Other(e.to_string()))?
            .collect();

        let chosen = pick_supported_config(&supported_configs, &config).ok_or_else(|| {
            jd_core::Error::Other(format!(
                "device cannot satisfy request {:?} (no F32 output config found)",
                config
            ))
        })?;

        let resolved_buffer = clamp_buffer_size(*chosen.buffer_size(), config.buffer_size);
        let resolved_sample_rate = clamp_sample_rate(
            chosen.min_sample_rate().0,
            chosen.max_sample_rate().0,
            config.sample_rate,
        );
        let resolved_channels = config.num_channels.min(chosen.channels());

        let stream_config = cpal::StreamConfig {
            channels: resolved_channels,
            sample_rate: cpal::SampleRate(resolved_sample_rate),
            buffer_size: match resolved_buffer {
                BufferSize::Fixed(n) => cpal::BufferSize::Fixed(n),
                BufferSize::Default => cpal::BufferSize::Default,
            },
        };

        let channels = resolved_channels as usize;
        let sample_rate_f = resolved_sample_rate as f32;
        let err_fn = |err| log::error!("cpal stream error: {err}");

        let stream = device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    callback.process(data, channels, sample_rate_f);
                },
                err_fn,
                None,
            )
            .map_err(|e| jd_core::Error::Other(format!("cpal build_output_stream: {e}")))?;

        stream
            .play()
            .map_err(|e| jd_core::Error::Other(e.to_string()))?;

        Ok(Self {
            _stream: stream,
            info: AudioDeviceInfo {
                sample_rate: resolved_sample_rate,
                num_channels: resolved_channels,
                buffer_size: resolved_buffer,
                sample_format: chosen.sample_format(),
                host_id,
            },
        })
    }

    pub fn info(&self) -> AudioDeviceInfo {
        self.info
    }
}

fn pick_supported_config(
    ranges: &[SupportedStreamConfigRange],
    requested: &AudioDeviceConfig,
) -> Option<SupportedStreamConfigRange> {
    let want_sr = cpal::SampleRate(requested.sample_rate);

    // Prefer F32 with a range that covers the requested sample rate and
    // enough channels.
    if let Some(&r) = ranges.iter().find(|r| {
        r.sample_format() == SampleFormat::F32
            && r.channels() >= requested.num_channels
            && r.min_sample_rate() <= want_sr
            && r.max_sample_rate() >= want_sr
    }) {
        return Some(r);
    }
    // Fall back to any F32 with enough channels.
    if let Some(&r) = ranges
        .iter()
        .find(|r| r.sample_format() == SampleFormat::F32 && r.channels() >= requested.num_channels)
    {
        return Some(r);
    }
    // Final fallback: anything F32.
    ranges
        .iter()
        .find(|r| r.sample_format() == SampleFormat::F32)
        .copied()
}

fn clamp_buffer_size(range: cpal::SupportedBufferSize, requested: u32) -> BufferSize {
    match range {
        cpal::SupportedBufferSize::Range { min, max } => {
            BufferSize::Fixed(requested.clamp(min, max))
        }
        cpal::SupportedBufferSize::Unknown => BufferSize::Default,
    }
}

fn clamp_sample_rate(min: u32, max: u32, requested: u32) -> u32 {
    requested.clamp(min, max)
}
