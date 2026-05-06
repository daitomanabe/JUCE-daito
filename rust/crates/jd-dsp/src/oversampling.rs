//! 2× oversampling using a windowed-sinc half-band low-pass. Counterpart of
//! `juce::dsp::Oversampling` in its simplest 2× form. For higher ratios chain
//! these stages.

use crate::convolution::FirFilter;
use crate::window::{windowed_sinc_lowpass, WindowKind};

/// 2× upsampler / downsampler pair sharing the same anti-imaging /
/// anti-aliasing FIR. Cutoff is fixed at 0.25 (half-band) of the upsampled
/// rate, giving a transition band centred at the original Nyquist.
pub struct Oversampler2x {
    upsample_filter: FirFilter,
    downsample_filter: FirFilter,
}

impl Oversampler2x {
    pub fn new(num_taps: usize) -> Self {
        let kernel = windowed_sinc_lowpass(num_taps | 1, 0.25, WindowKind::BlackmanHarris);
        // Multiply by 2 to compensate for zero-stuffing energy loss.
        let up_kernel: Vec<f32> = kernel.iter().map(|t| 2.0 * t).collect();
        Self {
            upsample_filter: FirFilter::new(up_kernel),
            downsample_filter: FirFilter::new(kernel),
        }
    }

    /// Upsample `input` by 2× into `output` (output.len() == 2 * input.len()).
    pub fn upsample(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(output.len(), input.len() * 2);
        for (i, &x) in input.iter().enumerate() {
            output[2 * i] = self.upsample_filter.process(x);
            output[2 * i + 1] = self.upsample_filter.process(0.0);
        }
    }

    /// Downsample `input` by 2× into `output` (input.len() == 2 * output.len()).
    pub fn downsample(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len() * 2);
        for (i, slot) in output.iter_mut().enumerate() {
            let _discard = self.downsample_filter.process(input[2 * i]);
            *slot = self.downsample_filter.process(input[2 * i + 1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::TAU;

    #[test]
    fn round_trip_preserves_low_frequency() {
        let sr = 48_000.0;
        let input: Vec<f32> = (0..2048)
            .map(|i| (TAU * 1_000.0 * i as f32 / sr).sin())
            .collect();

        let mut up = vec![0.0; input.len() * 2];
        let mut down = vec![0.0; input.len()];
        let mut os = Oversampler2x::new(63);
        os.upsample(&input, &mut up);
        os.downsample(&up, &mut down);

        // Skip filter ramp-up. The remaining signal must approximately match.
        let skip = 200;
        let in_rms: f32 =
            (input[skip..].iter().map(|x| x * x).sum::<f32>() / (input.len() - skip) as f32).sqrt();
        let out_rms: f32 =
            (down[skip..].iter().map(|x| x * x).sum::<f32>() / (down.len() - skip) as f32).sqrt();
        let ratio = out_rms / in_rms;
        assert!(ratio > 0.85 && ratio < 1.15, "round-trip ratio {ratio}");
    }
}
