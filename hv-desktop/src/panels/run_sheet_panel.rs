use eframe::egui::{self, ScrollArea, Ui};

use crate::app::HvBibleApp;
use crate::mock::session_state::{mock_detected_references, mock_run_sheet};
use crate::theme::{
    STATUS_SUCCESS, STATUS_WARNING,
};

pub fn show(ui: &mut Ui, _app: &mut HvBibleApp) {
    ui.horizontal(|ui| {
        ui.heading(
            egui::RichText::new("RUN OF SERVICE")
                .color(crate::theme::accent())
                .strong(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let _ = ui.button("Import");
            let _ = ui.button("Add Block");
        });
    });
    ui.add_space(6.0);
    ScrollArea::vertical().id_salt("run_sheet").show(ui, |ui| {
        for block in mock_run_sheet() {
            egui::Frame::none()
                .fill(crate::theme::bg_surface_sunken())
                .stroke(egui::Stroke::new(1.0, crate::theme::border_subtle()))
                .inner_margin(egui::Margin::symmetric(8.0, 7.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(block.time)
                                .small()
                                .monospace()
                                .color(crate::theme::text_secondary()),
                        );
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(block.title)
                                    .strong()
                                    .color(crate::theme::text_primary()),
                            );
                            ui.label(
                                egui::RichText::new(block.detail)
                                    .small()
                                    .color(crate::theme::text_secondary()),
                            );
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let color = if block.status == "Live" {
                                STATUS_SUCCESS
                            } else {
                                STATUS_WARNING
                            };
                            ui.colored_label(color, block.status);
                        });
                    });
                });
            ui.add_space(5.0);
        }
    });
}

pub fn show_detected(ui: &mut Ui, _app: &mut HvBibleApp) {
    ui.horizontal(|ui| {
        ui.heading(
            egui::RichText::new("DETECTED REFERENCES")
                .color(crate::theme::accent())
                .strong(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.colored_label(crate::theme::accent(), "Review queue");
        });
    });
    ui.add_space(6.0);
    for item in mock_detected_references() {
        egui::Frame::none()
            .fill(crate::theme::bg_surface_sunken())
            .stroke(egui::Stroke::new(1.0, crate::theme::border_subtle()))
            .inner_margin(egui::Margin::symmetric(8.0, 7.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(item.reference)
                                .strong()
                                .color(crate::theme::text_primary()),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "{} | {}% confidence",
                                item.source, item.confidence
                            ))
                            .small()
                            .color(crate::theme::text_secondary()),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let _ = ui.button("Push");
                        ui.label(
                            egui::RichText::new(item.state)
                                .small()
                                .color(crate::theme::text_secondary()),
                        );
                    });
                });
            });
        ui.add_space(5.0);
    }
}
