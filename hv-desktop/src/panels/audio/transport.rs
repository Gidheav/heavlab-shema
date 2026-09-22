use eframe::egui::{self, Ui};

use crate::app::HvBibleApp;
use crate::theme::{
    STATUS_ERROR, STATUS_SUCCESS,
};

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    ui.horizontal_wrapped(|ui| {
        let label = if app.is_listening { "⏸" } else { "▶" };
        let fill = if app.is_listening {
            crate::theme::accent()
        } else {
            STATUS_SUCCESS
        };
        if ui.add(button(label, fill)).clicked() {
            app.toggle_listening();
        }
        if ui.add(button("⏹", STATUS_ERROR)).clicked() && app.is_listening {
            app.toggle_listening();
        }
        let muted_fill = if app.audio_mock.monitoring.muted {
            crate::theme::accent()
        } else {
            crate::theme::bg_surface_sunken()
        };
        if ui.add(button("Mute", muted_fill)).clicked() {
            app.audio_mock.monitoring.muted = !app.audio_mock.monitoring.muted;
        }
        ui.add(
            egui::Button::new(
                egui::RichText::new("Settings")
                    .size(11.0)
                    .color(crate::theme::text_primary()),
            )
            .fill(crate::theme::bg_surface_sunken())
            .min_size(egui::vec2(76.0, 28.0)),
        );
    });
}

fn button(label: &str, fill: egui::Color32) -> egui::Button<'_> {
    egui::Button::new(
        egui::RichText::new(label)
            .size(11.0)
            .strong()
            .color(crate::theme::text_inverse()),
    )
    .fill(fill)
    .min_size(egui::vec2(66.0, 28.0))
}
