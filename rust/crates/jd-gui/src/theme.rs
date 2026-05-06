use egui::Visuals;

/// Counterpart of `juce::LookAndFeel` — a single function that swaps the
/// default colour palette into something more plug-in-like.
pub fn dark_audio() -> Visuals {
    let mut v = Visuals::dark();
    v.window_fill = egui::Color32::from_rgb(20, 22, 26);
    v.panel_fill = egui::Color32::from_rgb(28, 30, 36);
    v.selection.bg_fill = egui::Color32::from_rgb(0, 168, 232);
    v
}
