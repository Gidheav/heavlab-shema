//! MeterBar component - horizontal meter with peak-hold
//! Reference: Section 0.9
#![allow(dead_code)]

use egui::{Color32, Rect, Sense, Stroke, Ui, Vec2};

use crate::theme::{
    STATUS_ERROR, STATUS_SUCCESS, STATUS_WARNING,
};

pub struct MeterBar {
    value: f32,
    peak: f32,
    clipping: bool,
}

impl MeterBar {
    pub fn new(value: f32, peak: f32, clipping: bool) -> Self {
        Self {
            value,
            peak,
            clipping,
        }
    }

    pub fn show(self, ui: &mut Ui) {
        self.show_width(ui, ui.available_width());
    }

    pub fn show_width(self, ui: &mut Ui, width: f32) {
        let height = 22.0;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width.max(24.0), height), Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 3.0, crate::theme::bg_surface_sunken());
        painter.rect_stroke(rect, 3.0, Stroke::new(1.0, crate::theme::border_subtle()));

        let fill = db_to_fraction(self.value);
        let fill_rect = Rect::from_min_size(
            rect.min,
            Vec2::new(
                (rect.width() * fill).clamp(0.0, rect.width()),
                rect.height(),
            ),
        );
        painter.rect_filled(fill_rect, 3.0, meter_color(self.value));

        for db in [-48.0, -24.0, -12.0, -6.0, 0.0] {
            let x = rect.left() + rect.width() * db_to_fraction(db);
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                Stroke::new(1.0, crate::theme::border_subtle()),
            );
        }

        let peak_x = rect.left() + rect.width() * db_to_fraction(self.peak);
        painter.line_segment(
            [
                egui::pos2(peak_x, rect.top()),
                egui::pos2(peak_x, rect.bottom()),
            ],
            Stroke::new(
                2.0,
                if self.clipping {
                    STATUS_ERROR
                } else {
                    STATUS_WARNING
                },
            ),
        );
    }
}

fn db_to_fraction(db: f32) -> f32 {
    ((db + 48.0) / 48.0).clamp(0.0, 1.0)
}

fn meter_color(rms_db: f32) -> Color32 {
    if rms_db > -3.0 {
        STATUS_ERROR
    } else if rms_db > -12.0 {
        STATUS_WARNING
    } else {
        STATUS_SUCCESS
    }
}
