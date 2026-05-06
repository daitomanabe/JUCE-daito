//! Dynamics processors. Counterpart of `juce::dsp::Compressor` /
//! `juce::dsp::NoiseGate` (basic feed-forward designs).

use crate::gain::{db_to_linear, linear_to_db};

/// Peak / RMS envelope follower with separate attack and release.
#[derive(Debug, Clone, Copy)]
pub struct EnvelopeFollower {
    attack_coeff: f32,
    release_coeff: f32,
    state: f32,
}

impl EnvelopeFollower {
    pub fn new(sample_rate: f32, attack_ms: f32, release_ms: f32) -> Self {
        let mut s = Self {
            attack_coeff: 0.0,
            release_coeff: 0.0,
            state: 0.0,
        };
        s.set_times(sample_rate, attack_ms, release_ms);
        s
    }

    pub fn set_times(&mut self, sample_rate: f32, attack_ms: f32, release_ms: f32) {
        self.attack_coeff = time_constant(sample_rate, attack_ms);
        self.release_coeff = time_constant(sample_rate, release_ms);
    }

    pub fn reset(&mut self) {
        self.state = 0.0;
    }

    pub fn tick(&mut self, x: f32) -> f32 {
        let level = x.abs();
        let coeff = if level > self.state {
            self.attack_coeff
        } else {
            self.release_coeff
        };
        self.state = coeff * self.state + (1.0 - coeff) * level;
        self.state
    }
}

fn time_constant(sample_rate: f32, time_ms: f32) -> f32 {
    if time_ms <= 0.0 {
        0.0
    } else {
        (-1.0 / (sample_rate * time_ms * 1e-3)).exp()
    }
}

/// Soft-knee feed-forward compressor in the log domain.
#[derive(Debug, Clone, Copy)]
pub struct Compressor {
    sample_rate: f32,
    threshold_db: f32,
    ratio: f32,
    knee_db: f32,
    makeup_db: f32,
    follower: EnvelopeFollower,
}

impl Compressor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            threshold_db: -12.0,
            ratio: 4.0,
            knee_db: 6.0,
            makeup_db: 0.0,
            follower: EnvelopeFollower::new(sample_rate, 5.0, 80.0),
        }
    }

    pub fn set_threshold_db(&mut self, db: f32) {
        self.threshold_db = db;
    }
    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.max(1.0);
    }
    pub fn set_knee_db(&mut self, db: f32) {
        self.knee_db = db.max(0.0);
    }
    pub fn set_makeup_db(&mut self, db: f32) {
        self.makeup_db = db;
    }
    pub fn set_times(&mut self, attack_ms: f32, release_ms: f32) {
        self.follower
            .set_times(self.sample_rate, attack_ms, release_ms);
    }

    pub fn process(&mut self, x: f32) -> f32 {
        let level = self.follower.tick(x);
        let level_db = linear_to_db(level);
        let gain_db = self.compute_gain_reduction_db(level_db) + self.makeup_db;
        x * db_to_linear(gain_db)
    }

    fn compute_gain_reduction_db(&self, level_db: f32) -> f32 {
        let above = level_db - self.threshold_db;
        let half_knee = self.knee_db * 0.5;

        if 2.0 * above < -self.knee_db {
            0.0
        } else if 2.0 * above.abs() <= self.knee_db {
            let x = above + half_knee;
            -(1.0 / self.ratio - 1.0) * x * x / (2.0 * self.knee_db)
        } else {
            (1.0 / self.ratio - 1.0) * above
        }
    }
}

/// Simple downward expander / gate.
#[derive(Debug, Clone, Copy)]
pub struct NoiseGate {
    threshold_db: f32,
    ratio: f32,
    follower: EnvelopeFollower,
}

impl NoiseGate {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold_db: -40.0,
            ratio: 10.0,
            follower: EnvelopeFollower::new(sample_rate, 1.0, 100.0),
        }
    }

    pub fn set_threshold_db(&mut self, db: f32) {
        self.threshold_db = db;
    }
    pub fn set_ratio(&mut self, r: f32) {
        self.ratio = r.max(1.0);
    }
    pub fn set_times(&mut self, sample_rate: f32, attack_ms: f32, release_ms: f32) {
        self.follower.set_times(sample_rate, attack_ms, release_ms);
    }

    pub fn process(&mut self, x: f32) -> f32 {
        let level_db = linear_to_db(self.follower.tick(x));
        if level_db >= self.threshold_db {
            x
        } else {
            let below = self.threshold_db - level_db;
            x * db_to_linear(-below * (self.ratio - 1.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::TAU;

    fn sine(freq: f32, sr: f32, n: usize, amp: f32) -> Vec<f32> {
        (0..n)
            .map(|i| amp * (TAU * freq * i as f32 / sr).sin())
            .collect()
    }

    fn peak(s: &[f32]) -> f32 {
        s.iter().fold(0.0_f32, |a, &b| a.max(b.abs()))
    }

    #[test]
    fn compressor_reduces_loud_signal() {
        let sr = 48_000.0;
        let mut comp = Compressor::new(sr);
        comp.set_threshold_db(-20.0);
        comp.set_ratio(10.0);
        comp.set_knee_db(0.0);
        comp.set_times(0.1, 5.0);

        let input = sine(440.0, sr, 4800, 1.0);
        let out: Vec<f32> = input.iter().map(|&x| comp.process(x)).collect();
        let in_peak = peak(&input);
        let out_peak = peak(&out[2400..]);
        assert!(out_peak < in_peak * 0.5, "in {in_peak} out {out_peak}");
    }

    #[test]
    fn gate_passes_loud_signal() {
        let sr = 48_000.0;
        let mut gate = NoiseGate::new(sr);
        gate.set_threshold_db(-40.0);
        gate.set_ratio(10.0);

        let input = sine(440.0, sr, 4800, 0.5);
        let out: Vec<f32> = input.iter().map(|&x| gate.process(x)).collect();
        assert!(peak(&out[2400..]) > 0.4);
    }

    #[test]
    fn gate_attenuates_quiet_signal() {
        let sr = 48_000.0;
        let mut gate = NoiseGate::new(sr);
        gate.set_threshold_db(-40.0);
        gate.set_ratio(10.0);

        let input = sine(440.0, sr, 4800, 0.001);
        let out: Vec<f32> = input.iter().map(|&x| gate.process(x)).collect();
        assert!(peak(&out[2400..]) < 0.001 * 0.5);
    }
}
