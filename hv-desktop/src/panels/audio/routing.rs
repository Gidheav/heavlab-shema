use eframe::egui::{self, ComboBox, Ui};

use crate::app::HvBibleApp;
use crate::panels::audio::data_row;

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    let input = app.audio_mock.routing.input.clone();
    let processing = app.audio_mock.routing.processing.clone();
    let phase = app.audio_mock.routing.phase.clone();

    ui.columns(2, |cols| {
        cols[0].label(
            egui::RichText::new("Capture")
                .size(11.0)
                .color(crate::theme::text_secondary()),
        );
        let input_width = cols[0].available_width().clamp(80.0, 150.0);
        ComboBox::from_id_salt("routing_input_v2")
            .selected_text(input)
            .width(input_width)
            .show_ui(&mut cols[0], |ui| {
                for item in ["Mono", "Stereo", "Channel 1", "Channel 2", "Channel 1+2"] {
                    ui.selectable_value(
                        &mut app.audio_mock.routing.input,
                        item.to_string(),
                        item,
                    );
                }
            });

        cols[1].label(
            egui::RichText::new("Processing Bus")
                .size(11.0)
                .color(crate::theme::text_secondary()),
        );
        let proc_width = cols[1].available_width().clamp(80.0, 150.0);
        ComboBox::from_id_salt("routing_proc_v2")
            .selected_text(processing)
            .width(proc_width)
            .show_ui(&mut cols[1], |ui| {
                for item in ["Mono (L+R)", "Mono (L)", "Mono (R)", "Stereo"] {
                    ui.selectable_value(
                        &mut app.audio_mock.routing.processing,
                        item.to_string(),
                        item,
                    );
                }
            });
    });

    ui.add_space(6.0);

    // Channel table
    for ch in &app.audio_mock.routing.channels {
        data_row(
            ui,
            &format!("Ch {}", ch.id),
            format!(
                "{}  {:.1} dB",
                if ch.active { "● Active" } else { "○ Silent" },
                ch.peak_db
            ),
        );
    }

    ui.add_space(4.0);

    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new("Phase:")
                .size(11.0)
                .color(crate::theme::text_secondary()),
        );
        ui.label(
            egui::RichText::new(&phase)
                .size(11.0)
                .color(crate::theme::text_primary()),
        );

        ui.add_space(12.0);

        // Phase invert toggle
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("⊕ Invert Phase")
                        .size(11.0)
                        .color(crate::theme::text_primary()),
                )
                .fill(crate::theme::bg_surface_sunken()),
            )
            .clicked()
        {}
    });
}