use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};
use std::sync::Arc;

/// Counterpart of `juce::dsp::FFT`. Real-input FFT wrapper around `realfft`.
pub struct RealFft {
    forward: Arc<dyn RealToComplex<f32>>,
    inverse: Arc<dyn ComplexToReal<f32>>,
    size: usize,
}

impl RealFft {
    pub fn new(size: usize) -> Self {
        let mut planner = RealFftPlanner::<f32>::new();
        Self {
            forward: planner.plan_fft_forward(size),
            inverse: planner.plan_fft_inverse(size),
            size,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn forward(&self, input: &mut [f32], output: &mut [num_complex::Complex<f32>]) {
        self.forward
            .process(input, output)
            .expect("realfft forward");
    }

    pub fn inverse(&self, input: &mut [num_complex::Complex<f32>], output: &mut [f32]) {
        self.inverse
            .process(input, output)
            .expect("realfft inverse");
    }
}
