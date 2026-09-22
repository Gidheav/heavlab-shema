//! DropdownWithMeta component - dropdown with metadata display
//! Reference: Section 0.9
#![allow(dead_code)]

use egui::{ComboBox, Ui};


pub struct DropdownWithMeta<'a> {
    id: egui::Id,
    label: String,
    selected: String,
    options: Vec<(&'a str, &'a str)>, // (display, meta)
}

impl<'a> DropdownWithMeta<'a> {
    pub fn new(id: impl Into<egui::Id>, label: impl Into<String>, selected: String) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            selected,
            options: Vec::new(),
        }
    }

    pub fn with_options(mut self, options: Vec<(&'a str, &'a str)>) -> Self {
        self.options = options;
        self
    }

    pub fn show(self, ui: &mut Ui, selected: &mut String) -> bool {
        ui.label(
            egui::RichText::new(&self.label)
                .small()
                .color(crate::theme::text_secondary()),
        );
        let mut changed = false;
        ComboBox::from_id_salt(self.id)
            .selected_text(selected.clone())
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for (display, _meta) in &self.options {
                    if ui
                        .selectable_value(selected, display.to_string(), *display)
                        .changed()
                    {
                        changed = true;
                    }
                }
            });
        changed
    }
}