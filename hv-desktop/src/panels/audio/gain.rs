//! gain.rs — Gain section.
//!
//! The one gain an operator reaches for mid-service, plus the three bypass
//! switches that decide what the ASR actually hears. Their tuning lives in
//! Settings → Audio; a switch is an operating decision, a threshold is not.

use eframe::egui::{self, Sense, Ui};

use crate::app::HvBibleApp;
use crate::components::column_controls::{elide, InlineToggle};
use crate::components::column_density::{ColumnDensity, UNIT};

/// Capture gain range, from README §2.1.4.
const GAIN_MIN_DB: f32 = -60.0;
const GAIN_MAX_DB: f32 = 24.0;

pub fn show(ui: &mut Ui, app: &mut HvBibleApp, density: ColumnDensity) {
    // Snapshot first: the widgets below borrow the config fields, so the
    // comparison has to happen after those borrows end.
    let was_agc = app.config.agc_enabled;
    let was_gate = app.config.noise_gate_enabled;
    let was_aec = app.config.aec_enabled;

    // `app.gain` is a linear multiplier; this slider speaks decibels and only
    // forwards a command when the value actually moves.
    let mut gain_db = app.gain_db();
    let slider_height = density.row_height().max(16.0);
    if ui
        .add_sized(
            [ui.available_width(), slider_height],
            egui::Slider::new(&mut gain_db, GAIN_MIN_DB..=GAIN_MAX_DB)
                .show_value(false)
                .step_by(0.5),
        )
        .changed()
    {
        app.set_gain_db(gain_db);
    }

    // The number gets its own line: a slider and a readout do not both fit on
    // one, and the operator reads the figure, not the position.
    if density.shows_labels() {
        ui.add_space(UNIT);
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, UNIT * 4.0), Sense::hover());
        let value = format!("{:+.1} dB", gain_db);
        let color = if gain_db > 0.0 {
            crate::theme::STATUS_WARNING
        } else {
            crate::theme::text_primary()
        };
        ui.painter().galley(
            egui::pos2(rect.left(), rect.center().y),
            elide(ui, &value, rect.width(), color, true),
            color,
        );
    }

    ui.add_space(UNIT * 1.5);

    InlineToggle::new("AGC", &mut app.config.agc_enabled, density).show(ui);
    ui.add_space(density.row_gap());
    InlineToggle::new("Gate", &mut app.config.noise_gate_enabled, density).show(ui);
    ui.add_space(density.row_gap());
    InlineToggle::new("AEC", &mut app.config.aec_enabled, density).show(ui);

    // Now the borrows are over. A bypass is only useful if the chain actually
    // changes, so the command goes out here rather than on draw.
    if app.config.agc_enabled != was_agc {
        app.set_agc_enabled(app.config.agc_enabled);
    }
    if app.config.noise_gate_enabled != was_gate {
        app.set_noise_gate_enabled(app.config.noise_gate_enabled);
    }
    if app.config.aec_enabled != was_aec {
        app.set_aec_enabled(app.config.aec_enabled);
    }

    if density.shows_secondary() {
        ui.add_space(UNIT);
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, UNIT * 4.0), Sense::hover());
        ui.painter().galley(
            egui::pos2(rect.left(), rect.center().y),
            elide(
                ui,
                "AGC target and thresholds: Settings → Audio",
                rect.width(),
                crate::theme::text_tertiary(),
                false,
            ),
            crate::theme::text_tertiary(),
        );
    }
}

/// The header light for this section: how much of the gain range is in use.
pub fn status(app: &HvBibleApp) -> Option<(egui::Color32, &'static str)> {
    let enabled = [
        ("AGC", app.config.agc_enabled),
        ("GATE", app.config.noise_gate_enabled),
        ("AEC", app.config.aec_enabled),
    ];
    let on = enabled.iter().filter(|(_, active)| *active).count();
    match on {
        0 => None,
        1 => Some((crate::theme::text_secondary(), "1×")),
        2 => Some((crate::theme::accent(), "2×")),
        _ => Some((crate::theme::STATUS_SUCCESS, "3×")),
    }
}
