//! Time-domain waveform display. Counterpart of
//! `juce::AudioVisualiserComponent` for live monitoring.

use egui::{Color32, Pos2, Response, Sense, Stroke, Ui, Vec2};

/// Draws `samples` as a polyline scaled to `[-1, 1]` over the allocated rect.
pub fn waveform(ui: &mut Ui, samples: &[f32], desired: Vec2) -> Response {
    let (rect, response) = ui.allocate_exact_size(desired, Sense::hover());
    let painter = ui.painter();

    painter.rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);

    if samples.is_empty() {
        return response;
    }

    let mid_y = rect.center().y;
    let half_h = rect.height() * 0.5 - 2.0;

    painter.line_segment(
        [
            egui::pos2(rect.left(), mid_y),
            egui::pos2(rect.right(), mid_y),
        ],
        Stroke::new(1.0, ui.visuals().weak_text_color()),
    );

    let stroke = Stroke::new(1.0, ui.visuals().selection.bg_fill);
    let n = samples.len();
    let points: Vec<Pos2> = samples
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            let x = rect.left() + rect.width() * (i as f32 / (n.saturating_sub(1).max(1)) as f32);
            let y = mid_y - s.clamp(-1.0, 1.0) * half_h;
            egui::pos2(x, y)
        })
        .collect();
    painter.add(egui::Shape::line(points, stroke));

    response
}

/// Min/max-condensed waveform — appropriate when `samples.len()` greatly
/// exceeds available pixels (typical for offline file rendering).
pub fn waveform_minmax(ui: &mut Ui, samples: &[f32], desired: Vec2) -> Response {
    let (rect, response) = ui.allocate_exact_size(desired, Sense::hover());
    let painter = ui.painter();

    painter.rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);
    if samples.is_empty() {
        return response;
    }

    let cols = rect.width().ceil() as usize;
    let mid_y = rect.center().y;
    let half_h = rect.height() * 0.5 - 2.0;
    let stroke = Stroke::new(1.0, ui.visuals().selection.bg_fill);

    for col in 0..cols {
        let s_start = col * samples.len() / cols.max(1);
        let s_end = ((col + 1) * samples.len() / cols.max(1)).min(samples.len());
        if s_end <= s_start {
            continue;
        }
        let slice = &samples[s_start..s_end];
        let mut lo = f32::INFINITY;
        let mut hi = f32::NEG_INFINITY;
        for &s in slice {
            lo = lo.min(s);
            hi = hi.max(s);
        }
        let x = rect.left() + col as f32;
        let y_lo = mid_y - hi.clamp(-1.0, 1.0) * half_h;
        let y_hi = mid_y - lo.clamp(-1.0, 1.0) * half_h;
        painter.line_segment([egui::pos2(x, y_lo), egui::pos2(x, y_hi)], stroke);
    }

    response
}

/// Simple ring buffer that the audio thread can write into and the GUI
/// thread can snapshot. NOT lock-free; for a quick visual.
pub struct ScrollingBuffer {
    data: Vec<f32>,
    write: usize,
}

impl ScrollingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![0.0; capacity],
            write: 0,
        }
    }

    pub fn push(&mut self, sample: f32) {
        if self.data.is_empty() {
            return;
        }
        self.data[self.write] = sample;
        self.write = (self.write + 1) % self.data.len();
    }

    pub fn extend(&mut self, samples: &[f32]) {
        for &s in samples {
            self.push(s);
        }
    }

    pub fn snapshot(&self) -> Vec<f32> {
        if self.data.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::with_capacity(self.data.len());
        out.extend_from_slice(&self.data[self.write..]);
        out.extend_from_slice(&self.data[..self.write]);
        out
    }
}

#[allow(dead_code)]
fn _silence_color_unused(_ui: &Ui) -> Color32 {
    Color32::TRANSPARENT
}
