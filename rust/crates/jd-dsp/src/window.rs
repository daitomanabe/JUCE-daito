//! Window functions. Counterpart of `juce::dsp::WindowingFunction`.

use std::f32::consts::{PI, TAU};

#[derive(Debug, Clone, Copy)]
pub enum WindowKind {
    Rectangular,
    Hann,
    Hamming,
    Blackman,
    BlackmanHarris,
}

/// Fill `out` with a window of length `out.len()`.
pub fn fill(kind: WindowKind, out: &mut [f32]) {
    let n = out.len();
    if n == 0 {
        return;
    }
    let denom = (n - 1).max(1) as f32;
    for (i, slot) in out.iter_mut().enumerate() {
        let t = i as f32 / denom;
        *slot = match kind {
            WindowKind::Rectangular => 1.0,
            WindowKind::Hann => 0.5 - 0.5 * (TAU * t).cos(),
            WindowKind::Hamming => 0.54 - 0.46 * (TAU * t).cos(),
            WindowKind::Blackman => 0.42 - 0.5 * (TAU * t).cos() + 0.08 * (2.0 * TAU * t).cos(),
            WindowKind::BlackmanHarris => {
                0.35875 - 0.48829 * (TAU * t).cos() + 0.14128 * (2.0 * TAU * t).cos()
                    - 0.01168 * (3.0 * TAU * t).cos()
            }
        };
    }
}

pub fn make(kind: WindowKind, len: usize) -> Vec<f32> {
    let mut v = vec![0.0; len];
    fill(kind, &mut v);
    v
}

/// Sum of squares — useful when normalising overlap-add convolution gain.
pub fn power(window: &[f32]) -> f32 {
    window.iter().map(|w| w * w).sum()
}

/// Equivalent noise bandwidth in bins.
pub fn enbw(window: &[f32]) -> f32 {
    let sum: f32 = window.iter().sum();
    let sq: f32 = window.iter().map(|w| w * w).sum();
    if sum.abs() < f32::EPSILON {
        0.0
    } else {
        window.len() as f32 * sq / (sum * sum)
    }
}

/// Build a windowed sinc low-pass FIR with cutoff in normalised frequency
/// (0..0.5, where 0.5 == Nyquist).
pub fn windowed_sinc_lowpass(num_taps: usize, cutoff: f32, kind: WindowKind) -> Vec<f32> {
    assert!(num_taps > 0);
    let mut taps = vec![0.0; num_taps];
    let center = (num_taps as f32 - 1.0) / 2.0;
    let win = make(kind, num_taps);
    let two_fc = 2.0 * cutoff;
    for (i, tap) in taps.iter_mut().enumerate() {
        let n = i as f32 - center;
        let sinc = if n.abs() < 1e-9 {
            two_fc
        } else {
            (PI * two_fc * n).sin() / (PI * n)
        };
        *tap = sinc * win[i];
    }
    let sum: f32 = taps.iter().sum();
    if sum.abs() > f32::EPSILON {
        for tap in &mut taps {
            *tap /= sum;
        }
    }
    taps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hann_endpoints_are_zero() {
        let w = make(WindowKind::Hann, 64);
        assert!(w[0].abs() < 1e-6);
        assert!(w[w.len() - 1].abs() < 1e-6);
    }

    #[test]
    fn lowpass_passes_dc() {
        let taps = windowed_sinc_lowpass(33, 0.25, WindowKind::Hann);
        let dc_gain: f32 = taps.iter().sum();
        assert!((dc_gain - 1.0).abs() < 1e-4);
    }
}
