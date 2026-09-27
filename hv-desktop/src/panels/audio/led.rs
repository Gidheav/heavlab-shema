//! led.rs — LED Status section.
//!
//! The state light the operator glances at to know whether the system believes
//! it is on air, plus the two switches that decide where that light appears.
//! Brightness stays in Settings: it is set once per venue.

use eframe::egui::{self, Sense, Ui};

use crate::app::HvBibleApp;
use crate::components::column_controls::{elide, status_dot, InlineToggle};
use crate::components::column_density::{ColumnDensity, UNIT};

pub fn show(ui: &mut Ui, app: &mut HvBibleApp, density: ColumnDensity) {
    let state = app.status_label();
    let color = status_color(app);

    if density.shows_labels() {
        ui.horizontal(|ui| {
            status_dot(ui, color, UNIT * 1.5);
            ui.add_space(UNIT * 1.5);
            let width = ui.available_width();
            let (rect, _) = ui.allocate_exact_size(egui::vec2(width, UNIT * 4.0), Sense::hover());
            ui.painter().galley(
                egui::pos2(rect.left(), rect.center().y),
                elide(ui, state, rect.width(), color, false),
                color,
            );
        });
    } else {
        status_dot(ui, color, UNIT * 1.5);
        return;
    }

    ui.add_space(UNIT * 1.5);

    // The switches mirror the app's live LED state, so the screen LED the
    // operator is looking at can itself be switched off without confusion.
    let mut hardware = app.audio_mock.led.hardware_enabled;
    let mut screen = app.audio_mock.led.screen_enabled;

    InlineToggle::new("Hardware", &mut hardware, density).show(ui);
    ui.add_space(density.row_gap());
    InlineToggle::new("Screen", &mut screen, density).show(ui);

    app.audio_mock.led.hardware_enabled = hardware;
    app.audio_mock.led.screen_enabled = screen;

    if density.shows_secondary() {
        ui.add_space(density.row_gap());
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, UNIT * 4.0), Sense::hover());
        let (caption, value) = (
            "Brightness",
            format!("{:.0}%", app.audio_mock.led.brightness_percent),
        );
        ui.painter().galley(
            egui::pos2(rect.left(), rect.center().y),
            elide(
                ui,
                caption,
                rect.width() * 0.6,
                crate::theme::text_tertiary(),
                false,
            ),
            crate::theme::text_tertiary(),
        );
        ui.painter().galley(
            egui::pos2(rect.right() - rect.width() * 0.35, rect.center().y),
            elide(
                ui,
                &value,
                rect.width() * 0.35,
                crate::theme::text_primary(),
                true,
            ),
            crate::theme::text_primary(),
        );
    }
}

/// Mirrors the app's own LED colour rather than inventing a second opinion.
fn status_color(app: &HvBibleApp) -> egui::Color32 {
    if app.is_paused {
        return crate::theme::STATUS_WARNING;
    }
    match app.status_label() {
        "SPEAKING" => crate::theme::STATUS_SUCCESS,
        "LISTENING" => crate::theme::STATUS_INFO,
        "SILENCE" => crate::theme::STATUS_NEUTRAL,
        _ => crate::theme::STATUS_ERROR,
    }
}

/// The header light: the state word, so a folded section still says whether the
/// system thinks it is live.
pub fn status(app: &HvBibleApp) -> Option<(egui::Color32, &'static str)> {
    let label: &'static str = match app.status_label() {
        "SPEAKING" => "ON AIR",
        "LISTENING" => "ARMED",
        "PAUSED" => "HELD",
        "SILENCE" => "QUIET",
        _ => "OFF",
    };
    Some((status_color(app), label))
}
