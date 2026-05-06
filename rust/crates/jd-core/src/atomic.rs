use std::sync::atomic::{AtomicU32, Ordering};

/// Lock-free f32 cell, useful for parameter passing between UI and audio threads.
/// Counterpart of `juce::Atomic<float>` patterns commonly used in JUCE plug-ins.
#[derive(Debug, Default)]
pub struct AtomicF32 {
    bits: AtomicU32,
}

impl AtomicF32 {
    pub const fn new(value: f32) -> Self {
        Self {
            bits: AtomicU32::new(value.to_bits()),
        }
    }

    pub fn load(&self) -> f32 {
        f32::from_bits(self.bits.load(Ordering::Relaxed))
    }

    pub fn store(&self, value: f32) {
        self.bits.store(value.to_bits(), Ordering::Relaxed);
    }
}
