use egui::{Response, Sense, Stroke, Ui, Vec2};

/// Counterpart of `juce::Slider` in rotary-knob mode.
pub fn knob(ui: &mut Ui, value: &mut f32, range: std::ops::RangeInclusive<f32>) -> Response {
    let desired = Vec2::splat(48.0);
    let (rect, response) = ui.allocate_exact_size(desired, Sense::drag());

    if response.dragged() {
        let delta = response.drag_delta().y;
        let span = range.end() - range.start();
        let step = span / 200.0;
        *value = (*value - delta * step).clamp(*range.start(), *range.end());
    }

    let painter = ui.painter();
    let center = rect.center();
    let radius = rect.width() * 0.45;
    let stroke = Stroke::new(2.0, ui.visuals().widgets.active.fg_stroke.color);
    painter.circle_stroke(center, radius, stroke);

    let normalized = (*value - range.start()) / (range.end() - range.start());
    let angle = std::f32::consts::PI * 0.75 + normalized * std::f32::consts::PI * 1.5;
    let pointer = center + Vec2::angled(angle) * radius;
    painter.line_segment([center, pointer], stroke);

    response
}

/// Simple peak-style level meter (linear amplitude 0..=1).
pub fn level_meter(ui: &mut Ui, level: f32) -> Response {
    let desired = Vec2::new(12.0, 80.0);
    let (rect, response) = ui.allocate_exact_size(desired, Sense::hover());
    let painter = ui.painter();

    painter.rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);
    let fill_h = rect.height() * level.clamp(0.0, 1.0);
    let fill_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left(), rect.bottom() - fill_h),
        egui::vec2(rect.width(), fill_h),
    );
    painter.rect_filled(fill_rect, 2.0, ui.visuals().selection.bg_fill);
    response
}
