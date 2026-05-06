//! Topology-Preserving Transform state-variable filter (Vadim Zavalishin /
//! Andrew Simper). Cleaner-sounding alternative to a biquad — useful for
//! self-oscillating resonance and modulated cutoff sweeps.

use std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub enum SvfMode {
    Lowpass,
    Highpass,
    Bandpass,
    Notch,
}

#[derive(Debug, Clone, Copy)]
pub struct Svf {
    g: f32,
    k: f32,
    a1: f32,
    a2: f32,
    a3: f32,
    ic1eq: f32,
    ic2eq: f32,
    mode: SvfMode,
}

impl Svf {
    pub fn new() -> Self {
        let mut s = Self {
            g: 0.0,
            k: 0.0,
            a1: 0.0,
            a2: 0.0,
            a3: 0.0,
            ic1eq: 0.0,
            ic2eq: 0.0,
            mode: SvfMode::Lowpass,
        };
        s.set(1_000.0, 0.707, 48_000.0);
        s
    }

    pub fn set_mode(&mut self, mode: SvfMode) {
        self.mode = mode;
    }

    pub fn set(&mut self, cutoff_hz: f32, q: f32, sample_rate: f32) {
        let g = (PI * cutoff_hz / sample_rate).tan();
        let k = 1.0 / q.max(1e-3);
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        self.g = g;
        self.k = k;
        self.a1 = a1;
        self.a2 = a2;
        self.a3 = a3;
    }

    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    pub fn process(&mut self, x: f32) -> f32 {
        let v3 = x - self.ic2eq;
        let v1 = self.a1 * self.ic1eq + self.a2 * v3;
        let v2 = self.ic2eq + self.a2 * self.ic1eq + self.a3 * v3;
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;
        match self.mode {
            SvfMode::Lowpass => v2,
            SvfMode::Highpass => x - self.k * v1 - v2,
            SvfMode::Bandpass => v1,
            SvfMode::Notch => x - self.k * v1,
        }
    }
}

impl Default for Svf {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::TAU;

    fn rms(s: &[f32]) -> f32 {
        let sum: f32 = s.iter().map(|x| x * x).sum();
        (sum / s.len() as f32).sqrt()
    }

    fn sine(freq: f32, sr: f32, n: usize) -> Vec<f32> {
        (0..n).map(|i| (TAU * freq * i as f32 / sr).sin()).collect()
    }

    #[test]
    fn lowpass_attenuates_above_cutoff() {
        let sr = 48_000.0;
        let mut svf = Svf::default();
        svf.set(500.0, 0.707, sr);
        svf.set_mode(SvfMode::Lowpass);

        let high = sine(8_000.0, sr, 8192);
        let out: Vec<f32> = high.iter().map(|&x| svf.process(x)).collect();
        assert!(rms(&out) / rms(&high) < 0.2);
    }

    #[test]
    fn highpass_passes_above_cutoff() {
        let sr = 48_000.0;
        let mut svf = Svf::default();
        svf.set(500.0, 0.707, sr);
        svf.set_mode(SvfMode::Highpass);

        let high = sine(8_000.0, sr, 8192);
        let out: Vec<f32> = high.iter().map(|&x| svf.process(x)).collect();
        assert!(rms(&out) / rms(&high) > 0.8);
    }
}
