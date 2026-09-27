//! device.rs — Input Device section.
//!
//! Which mic is live, what it is negotiated as, and a way to re-scan. The
//! operator checks this before every service, so it is the one section that
//! opens expanded by default.

use eframe::egui::{self, Ui};

use crate::app::HvBibleApp;
use crate::components::column_controls::{
    column_width, elide, line, strip_size, InlineButton,
};
use crate::components::column_density::{ColumnDensity, UNIT};

pub fn show(ui: &mut Ui, app: &mut HvBibleApp, density: ColumnDensity) {
    let device = app.audio_mock.device.clone();
    let color = health_color(&device.health);

    // The device name is the first thing the operator looks for and the one
    // string most likely to be too long, so it gets the full width and an
    // ellipsis rather than a fixed column.
    if density.shows_labels() {
        line(ui, &device.name, crate::theme::text_primary(), false);
    } else {
        // Icon-only: one status light is the whole readout.
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), UNIT * 4.0),
            egui::Sense::hover(),
        );
        ui.painter().circle_filled(rect.center(), UNIT * 1.25, color);
    }

    // Driver badge and format, sharing the width evenly.
    let per_column = column_width(ui.available_width(), 2, UNIT);
    ui.horizontal(|ui| {
        let (badge_rect, _) =
            ui.allocate_exact_size(egui::vec2(per_column, UNIT * 4.0), egui::Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(badge_rect, 2.0, crate::theme::bg_surface_sunken());
        painter.rect_stroke(
            badge_rect,
            2.0,
            egui::Stroke::new(1.0, color.linear_multiply(0.45)),
        );
        painter.galley(
            egui::pos2(badge_rect.left() + UNIT, badge_rect.center().y),
            elide(
                ui,
                &device.driver,
                badge_rect.width() - UNIT * 2.0,
                color,
                true,
            ),
            color,
        );

        if density.shows_labels() {
            let format = format!(
                "{}k · {}-bit · {}",
                device.sample_rate / 1000,
                device.bit_depth,
                device.buffer_size
            );
            let (format_rect, _) =
                ui.allocate_exact_size(egui::vec2(per_column, UNIT * 4.0), egui::Sense::hover());
            ui.painter().galley(
                egui::pos2(format_rect.left(), format_rect.center().y),
                elide(
                    ui,
                    &format,
                    format_rect.width(),
                    crate::theme::text_secondary(),
                    true,
                ),
                crate::theme::text_secondary(),
            );
        }
    });

    // At expanded width there is room to say what the health is, and the
    // latency the buffer size implies.
    if density.shows_secondary() {
        ui.add_space(density.row_gap());
        let (health_rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), strip_size(ui) + UNIT),
            egui::Sense::hover(),
        );
        ui.painter().text(
            egui::pos2(health_rect.left(), health_rect.center().y),
            egui::Align2::LEFT_CENTER,
            format!("{} · {:.1} ms latency", device.health, device.buffer_latency_ms),
            egui::FontId::proportional(strip_size(ui)),
            color,
        );
    }

    if density.shows_labels() {
        ui.add_space(density.row_gap());
        if InlineButton::new("Rescan", density)
            .icon("↻")
            .show(ui)
            .on_hover_text("Re-read the list of capture devices from the host")
            .clicked()
        {
            app.refresh_devices();
        }
    }
}

/// The dot colour for a device health string. A device that will not open is
/// the one thing the operator must be able to see from across the room.
pub(super) fn health_color(health: &str) -> egui::Color32 {
    match health {
        "Active" => crate::theme::STATUS_SUCCESS,
        "Warning" => crate::theme::STATUS_WARNING,
        _ => crate::theme::STATUS_ERROR,
    }
}

