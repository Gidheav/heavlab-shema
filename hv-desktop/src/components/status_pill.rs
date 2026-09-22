//! StatusPill component - LED status indicator with optional label.
#![allow(dead_code)]

use egui::{Color32, Sense, Ui, Vec2};

use crate::theme::{
    STATUS_ERROR, STATUS_INFO, STATUS_NEUTRAL, STATUS_SUCCESS, STATUS_WARNING,
};

#[derive(Clone, Copy, Debug)]
pub enum Status {
    Success,
    Warning,
    Error,
    Info,
    Neutral,
}

impl Status {
    pub fn color(self) -> Color32 {
        match self {
            Status::Success => STATUS_SUCCESS,
            Status::Warning => STATUS_WARNING,
            Status::Error => STATUS_ERROR,
            Status::Info => STATUS_INFO,
            Status::Neutral => STATUS_NEUTRAL,
        }
    }
}

pub struct StatusPill {
    status: Status,
    label: String,
    blink: bool,
}

impl StatusPill {
    pub fn new(status: Status, label: impl Into<String>) -> Self {
        Self {
            status,
            label: label.into(),
            blink: false,
        }
    }

    pub fn blink(mut self, blink: bool) -> Self {
        self.blink = blink;
        self
    }

    pub fn show(self, ui: &mut Ui) {
        let size = if self.label.is_empty() {
            Vec2::splat(22.0)
        } else {
            Vec2::new(116.0, 22.0)
        };
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let core = if self.blink {
            self.status.color().linear_multiply(0.25)
        } else {
            self.status.color()
        };
        let painter = ui.painter();
        let center = egui::pos2(rect.left() + 11.0, rect.center().y);
        painter.circle_filled(center, 9.0, core.linear_multiply(0.35));
        painter.circle_stroke(
            center,
            8.0,
            egui::Stroke::new(
                3.0,
                Color32::from_rgba_unmultiplied(core.r(), core.g(), core.b(), 90),
            ),
        );
        painter.circle_filled(center, 4.5, core);

        if !self.label.is_empty() {
            painter.text(
                egui::pos2(rect.left() + 25.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                self.label,
                egui::FontId::proportional(11.0),
                crate::theme::text_primary(),
            );
        }
    }
}
