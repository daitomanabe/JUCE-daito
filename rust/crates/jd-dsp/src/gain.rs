/// Counterpart of `juce::dsp::Gain`. Linearly smooths a target gain to avoid
/// zipper noise when a parameter changes.
#[derive(Debug, Clone, Copy)]
pub struct SmoothedGain {
    current: f32,
    target: f32,
    step: f32,
    steps_remaining: u32,
}

impl SmoothedGain {
    pub fn new(initial: f32) -> Self {
        Self {
            current: initial,
            target: initial,
            step: 0.0,
            steps_remaining: 0,
        }
    }

    pub fn set_target(&mut self, target: f32, ramp_samples: u32) {
        self.target = target;
        self.steps_remaining = ramp_samples.max(1);
        self.step = (target - self.current) / self.steps_remaining as f32;
    }

    pub fn next(&mut self) -> f32 {
        if self.steps_remaining > 0 {
            self.current += self.step;
            self.steps_remaining -= 1;
            if self.steps_remaining == 0 {
                self.current = self.target;
            }
        }
        self.current
    }

    pub fn apply(&mut self, samples: &mut [f32]) {
        for s in samples {
            *s *= self.next();
        }
    }
}

pub fn db_to_linear(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

pub fn linear_to_db(linear: f32) -> f32 {
    20.0 * linear.max(1e-9).log10()
}
