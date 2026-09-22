use eframe::egui::{self, Ui};

use crate::app::HvBibleApp;
use crate::panels::audio::bar;

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    let vad = &mut app.audio_mock.vad;

    // VAD status badge
    let (status_color, status_text) = match vad.status.as_str() {
        "Active" | "Speaking" => (crate::theme::STATUS_SUCCESS, "● SPEAKING"),
        "Armed" | "Listening" => (crate::theme::STATUS_WARNING, "◉ ARMED"),
        "Idle" | "Silence" => (crate::theme::text_secondary(), "○ IDLE"),
        _ => (crate::theme::STATUS_ERROR, "✕ OFF"),
    };
    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new(status_text)
                .size(13.0)
                .strong()
                .color(status_color),
        );
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(format!("{:.0}% confidence", vad.confidence * 100.0))
                .size(11.0)
                .color(crate::theme::text_secondary()),
        );
    });

    ui.add_space(6.0);

    // Bars
    bar(ui, "Confidence", vad.confidence, format!("{:.2}", vad.confidence));
    bar(
        ui,
        "Silence",
        vad.silence_seconds / vad.silence_threshold_seconds,
        format!("{:.1}s / {:.1}s", vad.silence_seconds, vad.silence_threshold_seconds),
    );
    bar(
        ui,
        "Speech",
        vad.speech_seconds / vad.min_speech_seconds,
        format!("{:.2}s / {:.2}s", vad.speech_seconds, vad.min_speech_seconds),
    );

    ui.add_space(6.0);

    // Sensitivity slider
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Sensitivity")
                .size(11.0)
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
        param_badge(ui, "Sleep", &format!("{:.1}s", vad.silence_threshold_seconds));
        param_badge(ui, "Min spk", &format!("{:.0}ms", vad.min_speech_seconds * 1000.0));
    });
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
                        .size(10.0)
                        .color(crate::theme::text_secondary()),
                );
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(value)
                        .size(10.0)
                        .strong()
                        .monospace()
                        .color(crate::theme::text_primary()),
                );
            });
        });
    ui.add_space(4.0);
}