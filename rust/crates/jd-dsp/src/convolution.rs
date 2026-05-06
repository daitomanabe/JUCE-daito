//! Convolution. A direct-form FIR for short kernels and overlap-add FFT
//! convolution for long impulse responses. Counterpart of
//! `juce::dsp::FIR::Filter` and `juce::dsp::Convolution`.

use num_complex::Complex;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};
use std::sync::Arc;

/// Direct-form FIR filter with an internal delay line.
#[derive(Debug, Clone)]
pub struct FirFilter {
    taps: Vec<f32>,
    state: Vec<f32>,
    write: usize,
}

impl FirFilter {
    pub fn new(taps: Vec<f32>) -> Self {
        let len = taps.len();
        Self {
            taps,
            state: vec![0.0; len.max(1)],
            write: 0,
        }
    }

    pub fn reset(&mut self) {
        self.state.fill(0.0);
    }

    pub fn process(&mut self, x: f32) -> f32 {
        let n = self.taps.len();
        if n == 0 {
            return x;
        }
        self.state[self.write] = x;
        let mut acc = 0.0;
        let mut idx = self.write;
        for tap in &self.taps {
            acc += *tap * self.state[idx];
            idx = if idx == 0 { n - 1 } else { idx - 1 };
        }
        self.write = (self.write + 1) % n;
        acc
    }
}

/// Overlap-add FFT convolver. Block size B; kernel length up to `B`. Total
/// FFT length is `2*B`. For longer kernels, partition externally.
pub struct OverlapAdd {
    block: usize,
    fft_len: usize,
    forward: Arc<dyn RealToComplex<f32>>,
    inverse: Arc<dyn ComplexToReal<f32>>,
    kernel_spectrum: Vec<Complex<f32>>,
    overlap: Vec<f32>,
    fft_in: Vec<f32>,
    fft_out: Vec<Complex<f32>>,
    work: Vec<f32>,
}

impl OverlapAdd {
    pub fn new(block: usize, kernel: &[f32]) -> Self {
        assert!(block > 0 && kernel.len() <= block);
        let fft_len = (2 * block).next_power_of_two();
        let mut planner = RealFftPlanner::<f32>::new();
        let forward = planner.plan_fft_forward(fft_len);
        let inverse = planner.plan_fft_inverse(fft_len);

        let mut padded = vec![0.0; fft_len];
        padded[..kernel.len()].copy_from_slice(kernel);
        let mut spectrum = forward.make_output_vec();
        forward.process(&mut padded, &mut spectrum).unwrap();

        Self {
            block,
            fft_len,
            forward,
            inverse,
            kernel_spectrum: spectrum,
            overlap: vec![0.0; fft_len - block],
            fft_in: vec![0.0; fft_len],
            fft_out: vec![Complex::new(0.0, 0.0); fft_len / 2 + 1],
            work: vec![0.0; fft_len],
        }
    }

    pub fn block_size(&self) -> usize {
        self.block
    }

    /// Process exactly `block` input samples and write `block` output samples.
    /// `input.len()` and `output.len()` must equal the configured block size.
    pub fn process_block(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), self.block);
        assert_eq!(output.len(), self.block);

        self.fft_in[..self.block].copy_from_slice(input);
        for s in &mut self.fft_in[self.block..] {
            *s = 0.0;
        }

        self.forward
            .process(&mut self.fft_in, &mut self.fft_out)
            .expect("forward fft");

        for (bin, kbin) in self.fft_out.iter_mut().zip(self.kernel_spectrum.iter()) {
            *bin *= *kbin;
        }

        self.inverse
            .process(&mut self.fft_out, &mut self.work)
            .expect("inverse fft");
        let scale = 1.0 / self.fft_len as f32;
        for s in &mut self.work {
            *s *= scale;
        }

        for (i, slot) in output.iter_mut().enumerate() {
            *slot = self.work[i] + self.overlap.get(i).copied().unwrap_or(0.0);
        }

        let tail_start = self.block;
        let tail_len = self.fft_len - self.block;
        for i in 0..tail_len {
            let prev = self.overlap.get(self.block + i).copied().unwrap_or(0.0);
            self.overlap[i] = prev + self.work[tail_start + i];
        }
        for s in &mut self.overlap[tail_len..] {
            *s = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn naive_convolve(x: &[f32], h: &[f32]) -> Vec<f32> {
        let mut y = vec![0.0; x.len()];
        for n in 0..x.len() {
            let mut acc = 0.0;
            for (k, &hk) in h.iter().enumerate() {
                if n >= k {
                    acc += x[n - k] * hk;
                }
            }
            y[n] = acc;
        }
        y
    }

    #[test]
    fn fir_identity_kernel_passes_signal() {
        let mut fir = FirFilter::new(vec![1.0]);
        let out: Vec<f32> = (0..16).map(|i| fir.process(i as f32)).collect();
        for (i, v) in out.iter().enumerate() {
            assert_eq!(*v, i as f32);
        }
    }

    #[test]
    fn fir_matches_naive_convolution() {
        let h = vec![0.25, 0.5, 0.25];
        let x: Vec<f32> = (0..32).map(|i| (i as f32).sin()).collect();
        let mut fir = FirFilter::new(h.clone());
        let actual: Vec<f32> = x.iter().map(|&v| fir.process(v)).collect();
        let expected = naive_convolve(&x, &h);
        for (a, e) in actual.iter().zip(expected.iter()) {
            assert!((a - e).abs() < 1e-5);
        }
    }

    #[test]
    fn overlap_add_matches_direct_convolution() {
        let h: Vec<f32> = (0..32).map(|i| 1.0 / (1.0 + i as f32)).collect();
        let x: Vec<f32> = (0..256).map(|i| (i as f32 * 0.1).sin()).collect();
        let block = 64;

        let mut oa = OverlapAdd::new(block, &h);
        let mut actual = vec![0.0; x.len()];
        for chunk in 0..(x.len() / block) {
            let i0 = chunk * block;
            oa.process_block(&x[i0..i0 + block], &mut actual[i0..i0 + block]);
        }

        let expected = naive_convolve(&x, &h);
        for (a, e) in actual.iter().zip(expected.iter()) {
            assert!((a - e).abs() < 1e-3, "got {a} expected {e}");
        }
    }
}
