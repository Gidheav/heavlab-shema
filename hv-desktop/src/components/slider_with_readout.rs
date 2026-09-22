//! SliderWithReadout component - slider with numeric readout
//! Reference: Section 0.9
#![allow(dead_code)]

use egui::Ui;


pub struct SliderWithReadout {
    value: f32,
    min: f32,
    max: f32,
    label: String,
}

impl SliderWithReadout {
    pub fn new(label: impl Into<String>, value: f32, min: f32, max: f32) -> Self {
        Self {
            label: label.into(),
            value,
            min,
            max,
        }
    }

    pub fn show(self, ui: &mut Ui) -> f32 {
        ui.label(
            egui::RichText::new(&self.label)
                .small()
                .color(crate::theme::text_secondary()),
        );
        let mut value = self.value;
        ui.add(
            egui::Slider::new(&mut value, self.min..=self.max)
                .show_value(true)
                .custom_formatter(|v, _| format!("{v:.2}")),
        );
        value
    }
}