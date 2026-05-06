use jd_core::{Error, Result};
use jd_dsp::AudioBuffer;
use std::fs::File;
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Counterpart of `juce::AudioFormatReader::read`. Loads an entire file into
/// memory as f32. For streaming use, drop down to `symphonia` directly.
pub fn read_to_buffer<P: AsRef<Path>>(path: P) -> Result<(AudioBuffer, u32)> {
    let path = path.as_ref();
    let file = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| Error::Other(e.to_string()))?;

    let mut format = probed.format;
    let track = format
        .default_track()
        .ok_or_else(|| Error::Other("no default audio track".into()))?
        .clone();

    let sample_rate = track
        .codec_params
        .sample_rate
        .ok_or_else(|| Error::Other("missing sample rate".into()))?;
    let channels = track
        .codec_params
        .channels
        .ok_or_else(|| Error::Other("missing channel layout".into()))?
        .count();

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| Error::Other(e.to_string()))?;

    let mut buffer = AudioBuffer::new(channels, 0);
    let mut sample_buf: Option<SampleBuffer<f32>> = None;
    let mut total_samples: usize = 0;

    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track.id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(Error::Other(e.to_string())),
        };

        if sample_buf.is_none() {
            let spec = *decoded.spec();
            let duration = decoded.capacity() as u64;
            sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
        }
        let sb = sample_buf.as_mut().unwrap();
        sb.copy_interleaved_ref(decoded);
        let interleaved = sb.samples();
        let frames = interleaved.len() / channels;

        for ch in &mut (0..channels).map(|_| ()).collect::<Vec<_>>() {
            let _ = ch;
        }

        let new_len = total_samples + frames;
        buffer.resize(channels, new_len);
        for frame in 0..frames {
            for ch in 0..channels {
                buffer.channel_mut(ch)[total_samples + frame] = interleaved[frame * channels + ch];
            }
        }
        total_samples = new_len;
    }

    Ok((buffer, sample_rate))
}
