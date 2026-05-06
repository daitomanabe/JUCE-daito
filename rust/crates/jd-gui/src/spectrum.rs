//! Log-frequency / dB spectrum display. Pair with `jd_dsp::fft::RealFft` to
//! produce the magnitude bins.

use egui::{Pos2, Response, Sense, Stroke, Ui, Vec2};

/// Render `magnitudes` (length = bins) as a log-x / dB-y curve.
///
/// `bin_hz` is the frequency resolution of the FFT (sample_rate / fft_len).
/// The display covers `min_hz..=max_hz` along x (log scale) and
/// `floor_db..=0.0` along y.
pub fn spectrum(
    ui: &mut Ui,
    magnitudes: &[f32],
    bin_hz: f32,
    desired: Vec2,
    min_hz: f32,
    max_hz: f32,
    floor_db: f32,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(desired, Sense::hover());
    let painter = ui.painter();

    painter.rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);
    if magnitudes.is_empty() || bin_hz <= 0.0 || max_hz <= min_hz {
        return response;
    }

    // Grid lines at decade boundaries.
    let grid_stroke = Stroke::new(1.0, ui.visuals().weak_text_color());
    let log_min = min_hz.log10();
    let log_max = max_hz.log10();
    let mut decade = 10f32.powf(log_min.ceil());
    while decade <= max_hz {
        let frac = (decade.log10() - log_min) / (log_max - log_min);
        let x = rect.left() + frac * rect.width();
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            grid_stroke,
        );
        decade *= 10.0;
    }

    let stroke = Stroke::new(1.5, ui.visuals().selection.bg_fill);
    let mut points: Vec<Pos2> = Vec::with_capacity(rect.width().ceil() as usize);

    let cols = rect.width().ceil() as usize;
    for col in 0..cols {
        let frac = col as f32 / cols.max(1) as f32;
        let log_f = log_min + frac * (log_max - log_min);
        let hz = 10f32.powf(log_f);
        let bin = (hz / bin_hz).round() as usize;
        if bin >= magnitudes.len() {
            break;
        }
        let mag = magnitudes[bin].max(1e-9);
        let db = 20.0 * mag.log10();
        let y_frac = ((db - floor_db) / (-floor_db)).clamp(0.0, 1.0);
        let x = rect.left() + frac * rect.width();
        let y = rect.bottom() - y_frac * rect.height();
        points.push(egui::pos2(x, y));
    }

    if points.len() >= 2 {
        painter.add(egui::Shape::line(points, stroke));
    }

    response
}
