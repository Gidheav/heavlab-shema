use eframe::egui::{self, Ui};

use crate::app::HvBibleApp;
use crate::theme::{bg_surface_sunken, border_subtle, STATUS_SUCCESS, text_secondary};

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    ui.columns(4, |columns| {
        metric(&mut columns[0], "End-to-end", "142 ms");
        metric(&mut columns[1], "ASR inference", "98 ms");
        metric(&mut columns[2], "Parser", "2 us");
        metric(&mut columns[3], "Render", "16 ms");
    });
    ui.add_space(8.0);
    ui.columns(4, |columns| {
        metric(
            &mut columns[0],
            "CPU",
            &format!("{}%", app.audio_mock.diagnostics.cpu_percent),
        );
        metric(
            &mut columns[1],
            "Jitter",
            &format!("{:.1} ms", app.audio_mock.diagnostics.jitter_ms),
        );
        metric(
            &mut columns[2],
            "Dropped",
            &app.audio_mock.diagnostics.dropped_frames.to_string(),
        );
        metric(
            &mut columns[3],
            "Watchdog",
            &app.audio_mock.diagnostics.watchdog,
        );
    });
}

pub fn show_events(ui: &mut Ui, _app: &mut HvBibleApp) {
    for item in [
        "09:32:15  Verse John 3:16 approved to Program Out",
        "09:31:58  ASR confidence crossed display threshold",
        "09:31:40  Audio calibration completed",
        "09:30:12  Program Out opened on second display",
    ] {
        ui.label(
            egui::RichText::new(item)
                .size(12.0)
                .monospace()
                .color(crate::theme::text_secondary()),
        );
    }
}

fn metric(ui: &mut Ui, label: &str, value: &str) {
    egui::Frame::none()
        .fill(crate::theme::bg_surface_sunken())
        .stroke(egui::Stroke::new(1.0, crate::theme::border_subtle()))
        .inner_margin(egui::Margin::symmetric(8.0, 7.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(label).size(10.0).color(crate::theme::text_secondary()));
            ui.colored_label(
                STATUS_SUCCESS,
                egui::RichText::new(value).size(16.0).strong(),
            );
        });
}