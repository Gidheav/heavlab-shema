use eframe::egui::{self, Ui};

use crate::app::HvBibleApp;
use crate::panels::audio::data_row;

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    let led = &mut app.audio_mock.led;

    // Status indicator
    ui.horizontal(|ui| {
        let (color, label) = match led.status.as_str() {
            "Speaking" | "Active" => (crate::theme::STATUS_SUCCESS, "● ACTIVE"),
            "Armed" | "Listening" => (crate::theme::STATUS_WARNING, "◉ ARMED"),
            _ => (crate::theme::STATUS_ERROR, "○ OFF"),
        };
        let (r, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
        ui.painter().circle_filled(r.center(), 7.0, color);
        ui.label(egui::RichText::new(label).size(12.0).strong().color(color));
    });

    ui.add_space(6.0);

    // Hardware / Screen toggles
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut led.hardware_enabled, "Hardware LED");
        ui.add_space(12.0);
        ui.checkbox(&mut led.screen_enabled, "Screen LED");
    });

    ui.add_space(4.0);

    // Brightness
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Brightness")
                .size(11.0)
                .color(crate::theme::text_secondary()),
        );
        let w = (ui.available_width() - 60.0).clamp(40.0, 180.0);
        ui.add_sized(
            egui::vec2(w, 18.0),
            egui::Slider::new(&mut led.brightness_percent, 0.0..=100.0).show_value(false),
        );
        ui.label(
            egui::RichText::new(format!("{:.0}%", led.brightness_percent))
                .size(11.0)
                .monospace()
                .color(crate::theme::text_primary()),
        );
    });

    ui.add_space(4.0);

    // Actions
    ui.horizontal_wrapped(|ui| {
        for label in ["🔆 Test", "🎨 Patterns"] {
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