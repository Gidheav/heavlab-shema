//! PanelSection — the one collapsible pattern every left-column section uses.
//!
//! Four rules, and every section obeys all four:
//!   * the header is a full-width click target with a chevron on the right;
//!   * the body is suppressed entirely, not merely clipped, when collapsed;
//!   * open/closed is owned by the caller's persisted state, not by egui's
//!     memory, so it survives a restart;
//!   * nothing slides. The body is either drawn or it is not.

use eframe::egui::{self, Color32, Response, Sense, Stroke, Ui, Vec2};

use super::column_controls::{body_font, elide, strip_size};
use super::column_density::{ColumnDensity, UNIT};

/// Vertical space between two sections. Fixed, so the column keeps its rhythm
/// whether one section is open or all six are.
pub const SECTION_GAP: f32 = UNIT * 3.0;
/// Vertical space inside a section body.
pub const BODY_PAD_Y: f32 = UNIT * 2.0;
/// Header height.
pub const HEADER_HEIGHT: f32 = UNIT * 6.0;

/// A live state light shown in the header, so a collapsed section still reports
/// its condition. Without this, folding INPUT LEVEL would hide the clip light —
/// the one thing the operator must never lose sight of.
#[derive(Clone, Copy)]
pub struct SectionStatus {
    pub color: Color32,
    pub label: &'static str,
}

pub struct PanelSection<'a> {
    title: &'a str,
    icon: &'a str,
    open: &'a mut bool,
    density: ColumnDensity,
    status: Option<SectionStatus>,
}

impl<'a> PanelSection<'a> {
    pub fn new(
        title: &'a str,
        icon: &'a str,
        open: &'a mut bool,
        density: ColumnDensity,
    ) -> Self {
        Self {
            title,
            icon,
            open,
            density,
            status: None,
        }
    }

    pub fn with_status(mut self, status: SectionStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Attach a status light only if there is one. Keeps the call sites in
    /// `panels::audio::sections` to a single line each.
    pub fn with_status_opt(
        self,
        status: Option<(Color32, &'static str)>,
    ) -> Self {
        match status {
            Some((color, label)) => self.with_status(SectionStatus { color, label }),
            None => self,
        }
    }

    /// Renders the section and returns the header's response. `body` is only
    /// called when the section is open and the column has room for a body.
    pub fn show(mut self, ui: &mut Ui, body: impl FnOnce(&mut Ui)) -> Response {
        let width = ui.available_width();
        let response = self.header(ui, width);

        if self.density.shows_bodies() && *self.open {
            egui::Frame::none()
                .inner_margin(egui::Margin {
                    left: self.density.pad_x(),
                    right: self.density.pad_x(),
                    top: BODY_PAD_Y,
                    bottom: BODY_PAD_Y,
                })
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    body(ui);
                });
        }

        ui.add_space(SECTION_GAP);
        response
    }

    fn header(&mut self, ui: &mut Ui, width: f32) -> Response {
        let icon_only = !self.density.shows_labels();
        let (rect, mut response) =
            ui.allocate_exact_size(Vec2::new(width, HEADER_HEIGHT), Sense::click());

        // Icon-only headers are not interactive: their bodies are suppressed, so
        // a disclosure chevron there would promise something that never appears.
        if !icon_only {
            response = ui.interact(
                rect,
                ui.make_persistent_id(("panel_section", self.title)),
                Sense::click(),
            );
            if response.clicked() {
                *self.open = !*self.open;
            }
        }

        if !ui.is_rect_visible(rect) {
            return response;
        }

        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, crate::theme::bg_surface_raised());
        painter.line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            Stroke::new(1.0, crate::theme::border_subtle()),
        );

        let text_color = if *self.open {
            crate::theme::text_primary()
        } else {
            crate::theme::text_secondary()
        };

        if icon_only {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                self.icon,
                body_font(ui),
                text_color,
            );
            if let Some(status) = self.status {
                painter.circle_filled(
                    egui::pos2(rect.right() - UNIT * 1.5, rect.center().y),
                    UNIT * 0.75,
                    status.color,
                );
            }
            return response;
        }

        let text_size = strip_size(ui);
        let chevron = if *self.open { "▾" } else { "▸" };
        let chevron_width = text_size * 1.6;
        let mut cursor = rect.left() + UNIT * 2.0;

        painter.text(
            egui::pos2(cursor, rect.center().y),
            egui::Align2::LEFT_CENTER,
            self.icon,
            body_font(ui),
            crate::theme::text_tertiary(),
        );
        cursor += text_size * 1.4;

        if let Some(status) = self.status {
            let label_width = text_size * (status.label.len() as f32 * 0.66 + 1.0);
            let label_x = rect.right() - chevron_width - UNIT * 1.5 - label_width;
            if label_x > cursor + UNIT * 2.0 {
                painter.galley(
                    egui::pos2(label_x, rect.center().y),
                    elide(ui, status.label, label_width, status.color, true),
                    status.color,
                );
                painter.circle_filled(
                    egui::pos2(label_x - UNIT * 1.5, rect.center().y),
                    UNIT * 0.9,
                    status.color.linear_multiply(0.30),
                );
                painter.circle_stroke(
                    egui::pos2(label_x - UNIT * 1.5, rect.center().y),
                    UNIT * 0.9,
                    Stroke::new(1.5, status.color),
                );
            }
        }

        let title_width = (rect.right() - chevron_width - UNIT - cursor).max(0.0);
        painter.galley(
            egui::pos2(cursor, rect.center().y),
            elide(ui, &self.title.to_uppercase(), title_width, text_color, false),
            text_color,
        );

        painter.text(
            egui::pos2(rect.right() - chevron_width, rect.center().y),
            egui::Align2::LEFT_CENTER,
            chevron,
            body_font(ui),
            crate::theme::text_tertiary(),
        );

        response
    }
}


