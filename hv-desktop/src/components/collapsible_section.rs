//! CollapsibleSection component - expandable/collapsible panel section.
#![allow(dead_code)]

use egui::Ui;

use crate::theme::{bg_surface_sunken, border_subtle, text_secondary};

pub struct CollapsibleSection {
    title: String,
    badge: Option<String>,
    expanded: bool,
}

impl CollapsibleSection {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            badge: None,
            expanded: true,
        }
    }

    pub fn with_badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> bool {
        let title = if let Some(badge) = &self.badge {
            format!("{}   [{}]", self.title, badge)
        } else {
            self.title.clone()
        };

        let response = egui::CollapsingHeader::new(
            egui::RichText::new(title)
                .strong()
                .size(11.0)
                .color(crate::theme::text_secondary()),
        )
        .id_salt(format!("section_{}", self.title))
        .default_open(self.expanded)
        .show(ui, |ui| {
            egui::Frame::none()
                .fill(crate::theme::bg_surface_sunken().linear_multiply(0.96))
                .stroke(egui::Stroke::new(1.0, crate::theme::border_subtle()))
                .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                .show(ui, |ui| {
                    content(ui);
                });
        });

        ui.add_space(6.0);
        response.fully_open()
    }
}