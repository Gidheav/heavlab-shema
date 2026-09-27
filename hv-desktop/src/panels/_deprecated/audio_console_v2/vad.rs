use eframe::egui::{self, Ui};

use crate::app::HvBibleApp;
use crate::panels::audio::bar;

/// Dock card: live voice state. Read-only — the thresholds live in Settings.
pub fn show_status(ui: &mut Ui, app: &mut HvBibleApp) {
    let (color, text) = status_badge(&app.audio_mock.vad.status);

    ui.horizontal_wrapped(|ui| {
        let (dot, _) = ui.allocate_exact_size(egui::vec2(9.0, 9.0), egui::Sense::hover());
        ui.painter().circle_filled(dot.center(), 4.5, color);
        ui.label(egui::RichText::new(text).small().strong().color(color));

        // Report the confidence the pipeline actually produced. Showing a
        // hard-coded number here would read as a live reading while standing
        // still, which is exactly the kind of lie an operator cannot afford.
        if app.asr_confidence > 0.0 {
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(format!("{:.0}%", app.asr_confidence * 100.0))
                    .small()
                    .monospace()
                    .color(crate::theme::text_secondary()),
            );
        }
    });
}

/// Settings page: the thresholds an operator tunes once per venue.
pub fn show_tuning(ui: &mut Ui, app: &mut HvBibleApp) {
    let vad = &mut app.audio_mock.vad;

    bar(
        ui,
        "Confidence",
        vad.confidence,
        format!("{:.2}", vad.confidence),
    );
    bar(
        ui,
        "Silence",
        vad.silence_seconds / vad.silence_threshold_seconds,
        format!(
            "{:.1}s / {:.1}s",
            vad.silence_seconds, vad.silence_threshold_seconds
        ),
    );
    bar(
        ui,
        "Speech",
        vad.speech_seconds / vad.min_speech_seconds,
        format!(
            "{:.2}s / {:.2}s",
            vad.speech_seconds, vad.min_speech_seconds
        ),
    );

    ui.add_space(6.0);

    // Sensitivity slider
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Sensitivity")
                .small()
                .color(crate::theme::text_secondary()),
        );
        let w = (ui.available_width() - 50.0).clamp(30.0, 400.0);
        ui.add_sized(
            egui::vec2(w, 18.0),
            egui::Slider::new(&mut vad.sensitivity, 0.1..=0.9).show_value(false),
        );
        ui.colored_label(crate::theme::accent(), format!("{:.2}", vad.sensitivity));
    });

    ui.add_space(4.0);

    // Params summary
    ui.horizontal_wrapped(|ui| {
        param_badge(ui, "Wake", &format!("{:.2}", vad.sensitivity));
        param_badge(
            ui,
            "Sleep",
            &format!("{:.1}s", vad.silence_threshold_seconds),
        );
        param_badge(
            ui,
            "Min spk",
            &format!("{:.0}ms", vad.min_speech_seconds * 1000.0),
        );
    });
}

/// Maps a VAD status string to its dot colour.
pub(super) fn status_color(status: &str) -> eframe::egui::Color32 {
    match status {
        "Active" | "Speaking" => crate::theme::STATUS_SUCCESS,
        "Armed" | "Listening" => crate::theme::STATUS_WARNING,
        "Idle" | "Silence" => crate::theme::text_secondary(),
        _ => crate::theme::STATUS_ERROR,
    }
}

fn status_badge(status: &str) -> (eframe::egui::Color32, &'static str) {
    let color = status_color(status);
    let text = match status {
        "Active" | "Speaking" => "SPEAKING",
        "Armed" | "Listening" => "ARMED",
        "Idle" | "Silence" => "IDLE",
        _ => "OFF",
    };
    (color, text)
}

fn param_badge(ui: &mut Ui, label: &str, value: &str) {
    egui::Frame::none()
        .fill(crate::theme::bg_surface_sunken())
        .rounding(3.0)
        .inner_margin(egui::Margin::symmetric(6.0, 3.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(label)
                        .small()
                        .color(crate::theme::text_secondary()),
                );
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(value)
                        .small()
                        .strong()
                        .monospace()
                        .color(crate::theme::text_primary()),
                );
            });
        });
    ui.add_space(4.0);
}
