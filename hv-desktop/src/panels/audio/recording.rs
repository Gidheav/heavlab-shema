use eframe::egui::{self, ComboBox, Ui};

use crate::app::HvBibleApp;
use crate::panels::audio::data_row;

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    let rec = &mut app.audio_mock.recording;

    // Status row
    ui.horizontal(|ui| {
        if rec.active {
            ui.colored_label(crate::theme::STATUS_ERROR, "⏺ RECORDING");
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(&rec.elapsed)
                    .size(14.0)
                    .strong()
                    .monospace()
                    .color(crate::theme::STATUS_ERROR),
            );
        } else {
            ui.label(
                egui::RichText::new("⏹ STOPPED")
                    .size(12.0)
                    .color(crate::theme::text_secondary()),
            );
        }
    });

    ui.add_space(6.0);

    // Controls
    ui.horizontal_wrapped(|ui| {
        let (btn_label, btn_fill, btn_text_color) = if rec.active {
            (
                "⏹ Stop Recording",
                crate::theme::STATUS_ERROR,
                crate::theme::text_inverse(),
            )
        } else {
            (
                "⏺ Start Recording",
                crate::theme::bg_surface_sunken(),
                crate::theme::text_primary(),
            )
        };

        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new(btn_label)
                        .size(11.0)
                        .color(btn_text_color),
                )
                .fill(btn_fill),
            )
            .clicked()
        {
            rec.active = !rec.active;
        }

        ui.add_space(8.0);
        ui.checkbox(&mut rec.auto_record, "Auto-record on session start");
    });

    ui.add_space(6.0);

    // Format and disk space
    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new("Format")
                .size(11.0)
                .color(crate::theme::text_secondary()),
        );
        let format_width = (ui.available_width() - 12.0).clamp(90.0, 110.0);
        ComboBox::from_id_salt("rec_format_v2")
            .selected_text(rec.format.clone())
            .width(format_width)
            .show_ui(ui, |ui| {
                for item in ["WAV 16-bit", "WAV 24-bit", "WAV 32-bit Float", "FLAC", "MP3 320k"] {
                    ui.selectable_value(&mut rec.format, item.to_string(), item);
                }
            });

        ui.add_space(12.0);

        ui.label(
            egui::RichText::new(format!("💽 {} GB free", rec.disk_free_gb))
                .size(11.0)
                .color(crate::theme::text_secondary()),
        );
    });

    ui.add_space(4.0);

    // File path
    data_row(ui, "Path", rec.path.clone());

    ui.add_space(4.0);

    // Actions
    ui.horizontal_wrapped(|ui| {
        for label in ["📁 Folder", "📋 Copy", "🗑 Delete"] {
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(label)
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