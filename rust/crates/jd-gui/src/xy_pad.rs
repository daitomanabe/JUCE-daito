//! Two-axis touch pad. Common in synth UIs for binding two parameters
//! (e.g. cutoff vs. resonance) to a single drag gesture.

use egui::{Color32, Response, Sense, Stroke, Ui, Vec2};

/// `x` and `y` are normalised 0..=1 and updated in place when the user drags.
pub fn xy_pad(ui: &mut Ui, x: &mut f32, y: &mut f32, desired: Vec2) -> Response {
    let (rect, response) = ui.allocate_exact_size(desired, Sense::click_and_drag());

    if let Some(pos) = response
        .hover_pos()
        .filter(|_| response.is_pointer_button_down_on())
    {
        let nx = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        let ny = 1.0 - ((pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
        *x = nx;
        *y = ny;
    }

    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, ui.visuals().extreme_bg_color);
    painter.rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
        egui::StrokeKind::Inside,
    );

    let cross_stroke = Stroke::new(1.0, ui.visuals().weak_text_color());
    let cx = rect.left() + rect.width() * 0.5;
    let cy = rect.top() + rect.height() * 0.5;
    painter.line_segment(
        [egui::pos2(cx, rect.top()), egui::pos2(cx, rect.bottom())],
        cross_stroke,
    );
    painter.line_segment(
        [egui::pos2(rect.left(), cy), egui::pos2(rect.right(), cy)],
        cross_stroke,
    );

    let dot_x = rect.left() + x.clamp(0.0, 1.0) * rect.width();
    let dot_y = rect.top() + (1.0 - y.clamp(0.0, 1.0)) * rect.height();
    painter.circle_filled(
        egui::pos2(dot_x, dot_y),
        6.0,
        ui.visuals().selection.bg_fill,
    );
    painter.circle_stroke(
        egui::pos2(dot_x, dot_y),
        6.0,
        Stroke::new(1.5, Color32::WHITE),
    );

    response
}
