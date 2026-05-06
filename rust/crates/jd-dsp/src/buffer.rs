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
