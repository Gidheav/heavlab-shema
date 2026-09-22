use eframe::egui::{self, ComboBox, Ui};

use crate::app::HvBibleApp;
use crate::components::MeterBar;
use crate::panels::audio::data_row;

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    let left_db = app.audio_mock.level.left_db;
    let right_db = app.audio_mock.level.right_db;
    let peak_db = app.audio_mock.level.peak_db;
    let clip = app.audio_mock.level.clip;
    let lufs = app.audio_mock.level.lufs;
    let snr_db = app.audio_mock.level.snr_db;

    // Meter rows
    meter_row(ui, "L", left_db, peak_db, clip);
    meter_row(ui, "R", right_db, peak_db, clip);

    // Scale ticks
    ui.horizontal(|ui| {
        ui.add_space(20.0);
        ui.label(
            egui::RichText::new("  -48")
                .size(9.0)
                .monospace()
                .color(crate::theme::text_tertiary()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            for tick in ["0", "-6", "-12", "-24"] {
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new(tick)
                        .size(9.0)
                        .monospace()
                        .color(crate::theme::text_tertiary()),
                );
            }
        });
    });

    ui.add_space(6.0);

    // Stats grid
    ui.columns(2, |cols| {
        data_row(&mut cols[0], "Peak", format!("{:.1} dB", peak_db));
        data_row(&mut cols[0], "LUFS", format!("{:.1}", lufs));
        data_row(&mut cols[1], "SNR", format!("{} dB", snr_db));
        data_row(
            &mut cols[1],
            "Clip",
            if clip { "YES ⚠" } else { "No" }.to_string(),
        );
    });

    ui.add_space(4.0);

    // Controls row
    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new("Hold")
                .size(11.0)
                .color(crate::theme::text_secondary()),
        );
        let hold_width = (ui.available_width() - 8.0).clamp(50.0, 56.0);
        ComboBox::from_id_salt("meter_hold_v2")
            .selected_text(format!("{}s", app.audio_mock.level.hold_seconds))
            .width(hold_width)
            .show_ui(ui, |ui| {
                for s in [1u8, 3, 5, 30] {
                    ui.selectable_value(
                        &mut app.audio_mock.level.hold_seconds,
                        s,
                        format!("{}s", s),
                    );
                }
            });

        ui.label(
            egui::RichText::new("Mode")
                .size(11.0)
                .color(crate::theme::text_secondary()),
        );
        let mode_width = (ui.available_width() - 8.0).clamp(60.0, 70.0);
        ComboBox::from_id_salt("meter_mode_v2")
            .selected_text(app.audio_mock.level.meter_mode.clone())
            .width(mode_width)
            .show_ui(ui, |ui| {
                for m in ["RMS", "Peak", "VU", "LUFS-M", "LUFS-S"] {
                    ui.selectable_value(
                        &mut app.audio_mock.level.meter_mode,
                        m.to_string(),
                        m,
                    );
                }
            });

        ui.add_space(4.0);
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("Reset Peak")
                        .size(11.0)
                        .color(crate::theme::text_primary()),
                )
                .fill(crate::theme::bg_surface_sunken()),
            )
            .clicked()
        {}
        if clip {
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("Clear Clip")
                            .size(11.0)
                            .color(crate::theme::text_inverse()),
                    )
                    .fill(crate::theme::STATUS_ERROR),
                )
                .clicked()
            {}
        }
    });
}

fn meter_row(ui: &mut Ui, ch: &str, value: f32, peak: f32, clip: bool) {
    ui.horizontal(|ui| {
        ui.set_min_width(ui.available_width());
        ui.label(
            egui::RichText::new(ch)
                .size(11.0)
                .strong()
                .color(crate::theme::text_secondary()),
        );
        let meter_width = (ui.available_width() - 66.0).clamp(40.0, 400.0);
        MeterBar::new(value, peak, clip).show_width(ui, meter_width);
        ui.label(
            egui::RichText::new(format!("{:>6.1} dB", value))
                .size(11.0)
                .monospace()
                .color(crate::theme::text_primary()),
        );
    });
}