//! Peak-hold level meter. Counterpart of the meters in
//! `juce::AudioVisualiserComponent` style displays.

use egui::{Color32, Response, Sense, Stroke, Ui, Vec2};

#[derive(Debug, Clone, Copy)]
pub struct PeakHoldMeter {
    pub current: f32,
    pub peak: f32,
    pub peak_age_ms: f32,
}

impl PeakHoldMeter {
    pub const HOLD_MS: f32 = 1500.0;
    pub const FALL_DB_PER_SEC: f32 = 24.0;

    pub fn new() -> Self {
        Self {
            current: 0.0,
            peak: 0.0,
            peak_age_ms: 0.0,
        }
    }

    /// Push a new amplitude reading (linear 0..=1).
    pub fn push(&mut self, value: f32, dt_ms: f32) {
        let v = value.abs();
        if v >= self.peak {
            self.peak = v;
            self.peak_age_ms = 0.0;
        } else {
            self.peak_age_ms += dt_ms;
            if self.peak_age_ms > Self::HOLD_MS {
                let fall_per_ms = Self::FALL_DB_PER_SEC / 1000.0;
                let new_db = 20.0 * self.peak.max(1e-6).log10() - fall_per_ms * dt_ms;
                self.peak = (10f32.powf(new_db / 20.0)).max(0.0);
            }
        }

        let smooth = 0.8;
        self.current = smooth * self.current + (1.0 - smooth) * v;
    }

    pub fn ui(&self, ui: &mut Ui) -> Response {
        let desired = Vec2::new(14.0, 96.0);
        let (rect, response) = ui.allocate_exact_size(desired, Sense::hover());
        let painter = ui.painter();

        painter.rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);

        let level_h = rect.height() * self.current.clamp(0.0, 1.0);
        let level_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left(), rect.bottom() - level_h),
            egui::vec2(rect.width(), level_h),
        );
        painter.rect_filled(level_rect, 2.0, level_color(self.current));

        if self.peak > 0.0 {
            let y = rect.bottom() - rect.height() * self.peak.clamp(0.0, 1.0);
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                Stroke::new(1.5, Color32::WHITE),
            );
        }

        response
    }
}

impl Default for PeakHoldMeter {
    fn default() -> Self {
        Self::new()
    }
}

fn level_color(level: f32) -> Color32 {
    if level > 0.95 {
        Color32::from_rgb(220, 60, 60)
    } else if level > 0.7 {
        Color32::from_rgb(220, 180, 60)
    } else {
        Color32::from_rgb(80, 200, 120)
    }
}
