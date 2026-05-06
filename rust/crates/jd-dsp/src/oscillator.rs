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
