use eframe::egui::Ui;

use crate::app::HvBibleApp;
use crate::panels::audio::data_row;

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    let d = &app.audio_mock.diagnostics;

    ui.columns(2, |cols| {
        data_row(&mut cols[0], "CPU Usage", format!("{}%", d.cpu_percent));
        data_row(&mut cols[0], "Thread Jitter", format!("{:.1} ms", d.jitter_ms));
        data_row(&mut cols[0], "Dropped Frames", d.dropped_frames.to_string());
        data_row(&mut cols[1], "Last Buffer", format!("{:.1} ms", d.last_buffer_ms));
        data_row(&mut cols[1], "ASR Queue", d.asr_queue.to_string());
        data_row(&mut cols[1], "Uptime", d.uptime.clone());
    });

    eframe::egui::Frame::none()
        .fill(crate::theme::bg_surface_sunken())
        .rounding(3.0)
        .inner_margin(eframe::egui::Margin::symmetric(6.0, 3.0))
        .show(ui, |ui| {
            ui.label(
                eframe::egui::RichText::new(format!("Watchdog: {}", d.watchdog))
                    .size(11.0)
                    .color(if d.watchdog == "Healthy" {
                        crate::theme::STATUS_SUCCESS
                    } else {
                        crate::theme::STATUS_ERROR
                    }),
            );
        });

    ui.add_space(4.0);

    ui.horizontal_wrapped(|ui| {
        for label in ["📊 Monitor", "📁 Export", "🔄 Restart"] {
            if ui
                .add(
                    eframe::egui::Button::new(
                        eframe::egui::RichText::new(label)
                            .size(11.0)
                            .color(crate::theme::text_primary()),
                    )
                    .fill(crate::theme::bg_surface_sunken()),
                )
                .clicked()
            {}
        }
    });
}
