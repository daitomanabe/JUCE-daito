use std::f32::consts::TAU;

/// Single-pole one-zero biquad. Counterpart of `juce::dsp::IIR::Filter` for
/// the common low/high-pass cases. Uses RBJ cookbook coefficients.
#[derive(Debug, Clone, Copy, Default)]
pub struct Biquad {
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    z1: f32, z2: f32,
}

impl Biquad {
    pub fn lowpass(&mut self, cutoff_hz: f32, q: f32, sample_rate: f32) {
        let w0 = TAU * cutoff_hz / sample_rate;
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / (2.0 * q);

        let b0 =  (1.0 - cos_w0) / 2.0;
        let b1 =   1.0 - cos_w0;
        let b2 =  (1.0 - cos_w0) / 2.0;
        let a0 =   1.0 + alpha;
        let a1 =  -2.0 * cos_w0;
        let a2 =   1.0 - alpha;

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    pub fn highpass(&mut self, cutoff_hz: f32, q: f32, sample_rate: f32) {
        let w0 = TAU * cutoff_hz / sample_rate;
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / (2.0 * q);

        let b0 =  (1.0 + cos_w0) / 2.0;
        let b1 = -(1.0 + cos_w0);
        let b2 =  (1.0 + cos_w0) / 2.0;
        let a0 =   1.0 + alpha;
        let a1 =  -2.0 * cos_w0;
        let a2 =   1.0 - alpha;

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }

    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }
}
