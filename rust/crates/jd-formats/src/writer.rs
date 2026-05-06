use jd_core::{Error, Result};
use jd_dsp::AudioBuffer;
use std::path::Path;

/// Counterpart of `juce::WavAudioFormat::createWriterFor`. Writes a 32-bit
/// float WAV to disk.
pub fn write_wav_f32<P: AsRef<Path>>(
    path: P,
    buffer: &AudioBuffer,
    sample_rate: u32,
) -> Result<()> {
    let spec = hound::WavSpec {
        channels: buffer.num_channels() as u16,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer =
        hound::WavWriter::create(path, spec).map_err(|e| Error::Other(e.to_string()))?;

    let frames = buffer.num_samples();
    let channels = buffer.num_channels();
    for frame in 0..frames {
        for ch in 0..channels {
            writer
                .write_sample(buffer.channel(ch)[frame])
                .map_err(|e| Error::Other(e.to_string()))?;
        }
    }

    writer.finalize().map_err(|e| Error::Other(e.to_string()))?;
    Ok(())
}
