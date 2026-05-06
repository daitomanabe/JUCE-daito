/// Counterpart of `juce::ADSR`. Linear ADSR envelope.
#[derive(Debug, Clone, Copy)]
pub struct Adsr {
    sample_rate: f32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    state: State,
    value: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum State { Idle, Attack, Decay, Sustain, Release }

impl Adsr {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.2,
            state: State::Idle,
            value: 0.0,
        }
    }

    pub fn set_parameters(&mut self, a: f32, d: f32, s: f32, r: f32) {
        self.attack = a.max(1e-4);
        self.decay = d.max(1e-4);
        self.sustain = s.clamp(0.0, 1.0);
        self.release = r.max(1e-4);
    }

    pub fn note_on(&mut self) {
        self.state = State::Attack;
    }

    pub fn note_off(&mut self) {
        if self.state != State::Idle {
            self.state = State::Release;
        }
    }

    pub fn next(&mut self) -> f32 {
        let dt = 1.0 / self.sample_rate;
        match self.state {
            State::Idle => self.value = 0.0,
            State::Attack => {
                self.value += dt / self.attack;
                if self.value >= 1.0 {
                    self.value = 1.0;
                    self.state = State::Decay;
                }
            }
            State::Decay => {
                self.value -= dt * (1.0 - self.sustain) / self.decay;
                if self.value <= self.sustain {
                    self.value = self.sustain;
                    self.state = State::Sustain;
                }
            }
            State::Sustain => self.value = self.sustain,
            State::Release => {
                self.value -= dt * self.sustain / self.release;
                if self.value <= 0.0 {
                    self.value = 0.0;
                    self.state = State::Idle;
                }
            }
        }
        self.value
    }

    pub fn is_active(&self) -> bool {
        self.state != State::Idle
    }
}
