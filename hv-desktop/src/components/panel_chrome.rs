//! PanelChrome component - title bar + icon + collapse/float/close for dockable panels
//! Reference: Section 0.9
#![allow(dead_code)]

use egui::{Response, Ui};

use crate::theme::{accent, bg_surface, border_subtle, text_primary, text_secondary};

pub struct PanelChrome {
    title: String,
    icon: Option<String>,
}

impl PanelChrome {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            icon: None,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> Response {
        let frame = egui::Frame::none()
            .fill(crate::theme::bg_surface())
            .stroke(egui::Stroke::new(1.0, crate::theme::border_subtle()))
            .inner_margin(egui::Margin::symmetric(14.0, 12.0));

        frame
            .show(ui, |ui| {
                // Title bar
                ui.horizontal(|ui| {
                    if let Some(icon) = &self.icon {
                        ui.label(egui::RichText::new(icon).size(16.0).color(crate::theme::accent()));
                        ui.add_space(8.0);
                    }
                    ui.label(
                        egui::RichText::new(&self.title)
                            .strong()
                            .size(14.0)
                            .color(crate::theme::text_primary()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Close button
                        if ui
                            .small_button(egui::RichText::new("×").color(crate::theme::text_secondary()))
                            .clicked()
                        {
                            // TODO: Handle close
                        }
                        ui.add_space(8.0);
                        // Float button
                        if ui
                            .small_button(egui::RichText::new("⛶").color(crate::theme::text_secondary()))
                            .clicked()
                        {
                            // TODO: Handle float
                        }
                        ui.add_space(8.0);
                        // Collapse button
                        if ui
                            .small_button(egui::RichText::new("⚙").color(crate::theme::text_secondary()))
                            .clicked()
                        {
                            // TODO: Handle collapse
                        }
                    });
                });

                ui.add_space(8.0);

                // Content
                content(ui);
            })
            .response
    }
}