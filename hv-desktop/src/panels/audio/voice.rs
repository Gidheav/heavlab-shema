//! voice.rs — Voice Activity Detection section.
//!
//! Is the detector hearing speech, how sure is it, and how eager is it. The
//! sensitivity slider is here because it is the one VAD parameter an operator
//! legitimately nudges mid-service when a room turns noisy; the silence and
//! speech thresholds are per-venue and live in Settings → Processing.

use eframe::egui::{self, Sense, Ui};

use crate::app::HvBibleApp;
use crate::components::column_controls::{elide, status_dot, value_bar};
use crate::components::column_density::{ColumnDensity, UNIT};

pub fn show(ui: &mut Ui, app: &mut HvBibleApp, density: ColumnDensity) {
    let label = app.vad_label();
    let color = status_color(app);

    if density.shows_labels() {
        ui.horizontal(|ui| {
            status_dot(ui, color, UNIT * 1.5);
            ui.add_space(UNIT * 1.5);
            let width = ui.available_width();
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(width, UNIT * 4.0), Sense::hover());
            ui.painter().galley(
                egui::pos2(rect.left(), rect.center().y),
                elide(ui, label, rect.width(), color, false),
                color,
            );
        });
        ui.add_space(density.row_gap());
    } else {
        status_dot(ui, color, UNIT * 1.5);
        return;
    }

    // Confidence. The number the pipeline actually produced, never a placeholder
    // — a hard-coded figure on a live readout is a lie an operator cannot afford.
    let confidence = app.asr_confidence;
    let (label_text, value_text) = ("Confidence", format!("{:.2}", confidence));
    let width = ui.available_width();
    let height = UNIT * 4.0;
    let (bar_rect, _) = ui.allocate_exact_size(egui::vec2(width, height), Sense::hover());
    let painter = ui.painter();
    painter.galley(
        egui::pos2(bar_rect.left(), bar_rect.center().y),
        elide(
            ui,
            label_text,
            bar_rect.width() * 0.5,
            crate::theme::text_secondary(),
            false,
        ),
        crate::theme::text_secondary(),
    );
    let value_x = bar_rect.right() - bar_rect.width() * 0.3;
    painter.galley(
        egui::pos2(value_x, bar_rect.center().y),
        elide(
            ui,
            &value_text,
            bar_rect.width() * 0.3,
            confidence_color(confidence),
            true,
        ),
        confidence_color(confidence),
    );

    ui.add_space(density.row_gap());
    value_bar(ui, confidence, confidence_color(confidence), UNIT * 1.5);

    ui.add_space(UNIT * 1.5);

    // Sensitivity, retunes the live detector — no restart.
    let mut sensitivity = app.config.vad_sensitivity;
    if ui
        .add_sized(
            [ui.available_width(), density.row_height().max(16.0)],
            egui::Slider::new(&mut sensitivity, 0.0..=1.0)
                .show_value(false)
                .step_by(0.05),
        )
        .changed()
    {
        app.set_vad_sensitivity(sensitivity);
    }

    if density.shows_labels() {
        ui.add_space(density.row_gap());
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, UNIT * 4.0), Sense::hover());
        let (caption, value) = if sensitivity > 0.7 {
            ("Strict", format!("{:.2}", sensitivity))
        } else if sensitivity < 0.3 {
            ("Permissive", format!("{:.2}", sensitivity))
        } else {
            ("Balanced", format!("{:.2}", sensitivity))
        };
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

/// Maps a VAD state to its colour. A paused pipeline is deliberately neutral
/// rather than alarming: nothing is wrong, the operator asked for it.
pub(super) fn status_color(app: &HvBibleApp) -> egui::Color32 {
    if app.is_paused {
        return crate::theme::text_tertiary();
    }
    match app.vad_label() {
        "Speaking" => crate::theme::STATUS_SUCCESS,
        "Armed" => crate::theme::STATUS_WARNING,
        "Silence" => crate::theme::text_secondary(),
        _ => crate::theme::text_tertiary(),
    }
}

/// Confidence maps to the same green/amber/red scale as the level meter, so the
/// column has one colour language rather than two.
fn confidence_color(confidence: f32) -> egui::Color32 {
    if confidence >= 0.6 {
        crate::theme::STATUS_SUCCESS
    } else if confidence > 0.0 {
        crate::theme::STATUS_WARNING
    } else {
        crate::theme::text_tertiary()
    }
}

/// The header light for this section.
pub fn status(app: &HvBibleApp) -> Option<(egui::Color32, &'static str)> {
    if app.is_paused {
        return Some((crate::theme::text_tertiary(), "HELD"));
    }
    match app.vad_label() {
        "Speaking" => Some((crate::theme::STATUS_SUCCESS, "VOICE")),
        "Armed" => Some((crate::theme::STATUS_WARNING, "ARMED")),
        "Silence" => Some((crate::theme::text_secondary(), "QUIET")),
        _ => None,
    }
}
