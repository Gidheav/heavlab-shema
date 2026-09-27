use eframe::egui::Ui;

use crate::app::HvBibleApp;
use crate::panels::audio::labeled_slider;

/// Capture range from README §2.1.4.
const GAIN_MIN_DB: f32 = -60.0;
const GAIN_MAX_DB: f32 = 24.0;
const TRIM_MIN_DB: f32 = -12.0;
const TRIM_MAX_DB: f32 = 12.0;

/// Dock card: the one gain an operator reaches for live.
pub fn show_gain(ui: &mut Ui, app: &mut HvBibleApp) {
    // `app.gain` is a linear multiplier; this slider speaks decibels and only
    // forwards a command to the pipeline when the value actually moves.
    let mut gain_db = app.gain_db();
    if db_slider(ui, "Input Gain", &mut gain_db, GAIN_MIN_DB, GAIN_MAX_DB) {
        app.set_gain_db(gain_db);
    }
}

/// Settings page: the fine trim that rides after the capture gain. It is
/// deliberately *not* in the dock — two stacked dB sliders read as one
/// ambiguous control at dock width.
pub fn show_trim(ui: &mut Ui, app: &mut HvBibleApp) {
    labeled_slider(
        ui,
        "Digital Trim",
        &mut app.audio_mock.gain.digital_trim_db,
        TRIM_MIN_DB,
        TRIM_MAX_DB,
        "dB",
    );
}

/// Labelled decibels slider; returns true when the operator moved it.
fn db_slider(ui: &mut Ui, label: &str, value: &mut f32, min: f32, max: f32) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        let label_width = if ui.available_width() < 250.0 {
            60.0
        } else {
            80.0
        };
        ui.add_sized(
            egui::vec2(label_width, 18.0),
            egui::Label::new(
                egui::RichText::new(label)
                    .small()
                    .color(crate::theme::text_secondary()),
            ),
        );
        let w = (ui.available_width() - 50.0).clamp(30.0, 400.0);
        changed = ui
            .add_sized(
                egui::vec2(w, 18.0),
                egui::Slider::new(value, min..=max).show_value(false),
            )
            .changed();
        ui.label(
            egui::RichText::new(format!("{:+.1} dB", *value))
                .small()
                .monospace()
                .color(crate::theme::text_primary()),
        );
    });
    changed
}
