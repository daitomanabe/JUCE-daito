/// Counterpart of `juce::AudioBuffer<float>`.
///
/// Stored as channel-major (`channels[ch][sample]`), matching JUCE's layout
/// and the buffer shape that `cpal` and `nih-plug` typically hand us.
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    channels: Vec<Vec<f32>>,
}

impl AudioBuffer {
    pub fn new(num_channels: usize, num_samples: usize) -> Self {
        Self {
            channels: vec![vec![0.0; num_samples]; num_channels],
        }
    }

    pub fn num_channels(&self) -> usize {
        self.channels.len()
    }

    pub fn num_samples(&self) -> usize {
        self.channels.first().map_or(0, |c| c.len())
    }

    pub fn channel(&self, ch: usize) -> &[f32] {
        &self.channels[ch]
    }

    pub fn channel_mut(&mut self, ch: usize) -> &mut [f32] {
        &mut self.channels[ch]
    }

    pub fn clear(&mut self) {
        for ch in &mut self.channels {
            ch.fill(0.0);
        }
    }

    pub fn resize(&mut self, num_channels: usize, num_samples: usize) {
        self.channels.resize_with(num_channels, Vec::new);
        for ch in &mut self.channels {
            ch.resize(num_samples, 0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer_is_zeroed() {
        let buf = AudioBuffer::new(2, 64);
        assert_eq!(buf.num_channels(), 2);
        assert_eq!(buf.num_samples(), 64);
        for ch in 0..2 {
            assert!(buf.channel(ch).iter().all(|&s| s == 0.0));
        }
    }

    #[test]
    fn resize_preserves_existing_data() {
        let mut buf = AudioBuffer::new(1, 4);
        buf.channel_mut(0).copy_from_slice(&[1.0, 2.0, 3.0, 4.0]);
        buf.resize(1, 8);
        assert_eq!(&buf.channel(0)[..4], &[1.0, 2.0, 3.0, 4.0]);
    }
}
