use eframe::egui::{self, Ui};

use crate::app::HvBibleApp;
use crate::theme::{bg_surface_sunken, border_subtle, text_primary, text_secondary};

pub fn show(ui: &mut Ui, _app: &mut HvBibleApp) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("SERVICE QUEUE")
                .size(13.0)
                .strong()
                .color(crate::theme::text_primary()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let _ = ui.button("Import");
            let _ = ui.button("Paste List");
            let _ = ui.button("Add Verse");
        });
    });
    ui.separator();
    for (index, (reference, text)) in [
        (
            "Romans 8:28",
            "And we know that all things work together for good...",
        ),
        ("John 14:6", "I am the way, and the truth, and the life..."),
        ("Psalm 119:105", "Your word is a lamp to my feet..."),
        (
            "Isaiah 40:31",
            "They who wait for the Lord shall renew their strength...",
        ),
    ]
    .iter()
    .enumerate()
    {
        egui::Frame::none()
            .fill(crate::theme::bg_surface_sunken())
            .stroke(egui::Stroke::new(1.0, crate::theme::border_subtle()))
            .inner_margin(egui::Margin::symmetric(8.0, 7.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{}", index + 1))
                            .size(11.0)
                            .monospace()
                            .color(crate::theme::text_secondary()),
                    );
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(*reference)
                                .size(12.0)
                                .strong()
                                .color(crate::theme::text_primary()),
                        );
                        ui.label(egui::RichText::new(*text).size(11.0).color(crate::theme::text_secondary()));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let _ = ui.button("x");
                        let _ = ui.button("Push");
                    });
                });
            });
        ui.add_space(5.0);
    }
}