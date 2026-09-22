//! EmptyState component - placeholder for empty panels
//! Reference: Section 0.9
#![allow(dead_code)]

use egui::Ui;

use crate::theme::{text_secondary, text_tertiary};

pub struct EmptyState {
    message: String,
    icon: Option<String>,
}

impl EmptyState {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            icon: None,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn show(self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            if let Some(icon) = &self.icon {
                ui.label(egui::RichText::new(icon).size(48.0).color(crate::theme::text_tertiary()));
                ui.add_space(16.0);
            }
            ui.label(egui::RichText::new(&self.message).color(crate::theme::text_secondary()));
            ui.add_space(32.0);
        });
    }
}