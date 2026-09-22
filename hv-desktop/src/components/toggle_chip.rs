//! ToggleChip component - binary toggle setting
//! Reference: Section 0.9
#![allow(dead_code)]

use egui::Ui;

pub struct ToggleChip {
    label: String,
    value: bool,
}

impl ToggleChip {
    pub fn new(label: impl Into<String>, value: bool) -> Self {
        Self {
            label: label.into(),
            value,
        }
    }

    pub fn show(self, ui: &mut Ui) -> bool {
        let mut value = self.value;
        let response = ui.checkbox(&mut value, &self.label);
        if response.changed() {
            value
        } else {
            self.value
        }
    }
}
