/// Counterpart of `juce::Range<T>`. A half-open interval `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range<T> {
    pub start: T,
    pub end: T,
}

impl<T: Copy + PartialOrd + std::ops::Sub<Output = T>> Range<T> {
    pub fn new(start: T, end: T) -> Self {
        Self { start, end }
    }

    pub fn length(self) -> T {
        self.end - self.start
    }

    pub fn contains(self, value: T) -> bool {
        value >= self.start && value < self.end
    }
}
