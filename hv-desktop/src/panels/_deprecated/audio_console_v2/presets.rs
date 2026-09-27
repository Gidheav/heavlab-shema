use eframe::egui::{self, ComboBox, Ui};

use crate::app::HvBibleApp;

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    ComboBox::from_id_salt("audio_preset_v2")
        .selected_text(app.audio_mock.preset.clone())
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            for item in [
                "Church Service",
                "Podcast Studio",
                "Conference Hall",
                "Quiet Room",
                "Outdoor Event",
                "Custom",
            ] {
                ui.selectable_value(&mut app.audio_mock.preset, item.to_string(), item);
            }
        });

    ui.add_space(4.0);

    ui.horizontal_wrapped(|ui| {
        for (icon, label) in [
            ("💾", "Save"),
            ("📋", "Save As"),
            ("📂", "Manage"),
            ("↺", "Reset"),
        ] {
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(format!("{} {}", icon, label))
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
