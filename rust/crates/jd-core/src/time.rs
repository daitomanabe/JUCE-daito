use std::time::Duration;

/// High-resolution monotonic timer counterpart of `juce::Time::getMillisecondCounterHiRes`.
pub fn now_millis() -> f64 {
    let elapsed = std::time::Instant::now().duration_since(start_instant());
    elapsed.as_secs_f64() * 1000.0
}

fn start_instant() -> std::time::Instant {
    use std::sync::OnceLock;
    static START: OnceLock<std::time::Instant> = OnceLock::new();
    *START.get_or_init(std::time::Instant::now)
}

pub fn sleep(duration: Duration) {
    std::thread::sleep(duration);
}
