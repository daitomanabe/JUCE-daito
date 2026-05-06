use std::f32::consts::TAU;

/// Counterpart of `juce::dsp::Oscillator`. Polymorphic over the wave function.
pub struct Oscillator<F: FnMut(f32) -> f32> {
    phase: f32,
    phase_inc: f32,
    wave: F,
}

impl<F: FnMut(f32) -> f32> Oscillator<F> {
    pub fn new(wave: F) -> Self {
        Self {
            phase: 0.0,
            phase_inc: 0.0,
            wave,
        }
    }

    pub fn set_frequency(&mut self, freq_hz: f32, sample_rate: f32) {
        self.phase_inc = TAU * freq_hz / sample_rate;
    }

    pub fn process(&mut self) -> f32 {
        let value = (self.wave)(self.phase);
        self.phase += self.phase_inc;
        if self.phase >= TAU {
            self.phase -= TAU;
        }
        value
    }
}

pub fn sine() -> Oscillator<impl FnMut(f32) -> f32> {
    Oscillator::new(|p| p.sin())
}

pub fn saw() -> Oscillator<impl FnMut(f32) -> f32> {
    Oscillator::new(|p| (p / TAU) * 2.0 - 1.0)
}

pub fn square() -> Oscillator<impl FnMut(f32) -> f32> {
    Oscillator::new(|p| if p < std::f32::consts::PI { 1.0 } else { -1.0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_oscillator_outputs_in_range() {
        let mut osc = sine();
        osc.set_frequency(440.0, 48_000.0);
        for _ in 0..1000 {
            let s = osc.process();
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn frequency_drives_period() {
        let sample_rate: f32 = 48_000.0;
        let freq: f32 = 1_000.0;
        let period_samples = (sample_rate / freq).round() as usize;

        let mut osc = sine();
        osc.set_frequency(freq, sample_rate);

        let first = osc.process();
        for _ in 1..period_samples {
            let _ = osc.process();
        }
        let after_period = osc.process();
        assert!((first - after_period).abs() < 0.05);
    }
}
