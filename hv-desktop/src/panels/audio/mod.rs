//! audio/mod.rs — the LEFT COLUMN: the operator's audio control surface.
//!
//! Six sections and a pinned transport bar. The rule that governs everything
//! below: *if the operator does not need it while the pastor is preaching, it
//! belongs in Settings.* Routing, the signal chain, monitoring, recording,
//! diagnostics, and format all live in the Settings window now; what is left
//! here is what gets touched in the room.
//!
//! Two structural rules, enforced by construction rather than by review:
//!   * the column never hardcodes a width — `ColumnDensity` reads the available
//!     width once and every section answers the same breakpoints, so there is
//!     nothing to go out of sync and nothing that can overflow;
//!   * the transport bar sits outside the scroll area, so it is reachable no
//!     matter how far the operator has scrolled.
//!
//! The previous console is archived, uncompiled, in
//! `panels/_deprecated/audio_console_v2/`.

mod device;
mod gain;
mod led;
mod level;
mod preset;
mod transport;
mod voice;

use eframe::egui::{self, ScrollArea, Sense, Stroke, Ui, Vec2};

use crate::app::HvBibleApp;
use crate::components::column_controls::elide;
use crate::components::column_density::ColumnDensity;
use crate::components::panel_section::PanelSection;

/// Height of the column's title bar.
const TITLE_BAR_HEIGHT: f32 = 28.0;

/// Renders the left column slot.
pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    let width = ui.available_width();
    let height = ui.available_height();
    if width <= 0.0 || height <= 0.0 {
        return;
    }

    // The one place the width is measured. Every section below is handed this
    // value and never asks again.
    let density = ColumnDensity::from_width(width);

    title_bar(ui, app, width, density);

    // Laid out bottom-up so the scroll area gets exactly the height that is
    // left over — the column can never grow past its parent.
    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        transport::show(ui, app, density);

        let body_top = ui.cursor().top();
        let body_height = (body_top - TITLE_BAR_HEIGHT).max(0.0);
        if body_height <= 0.0 {
            return;
        }
        let (body_rect, _) =
            ui.allocate_exact_size(Vec2::new(width, body_height), Sense::hover());

        // Vertical only, and pinned to the body rect. A horizontal scrollbar
        // here would mean a section had exceeded its width, which the density
        // model is built to make impossible.
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(body_rect)
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
        );
        ScrollArea::vertical()
            .id_salt("audio_column_body")
            .auto_shrink([false, false])
            .show(&mut child, |ui| {
                ui.set_min_width(ui.available_width());
                sections(ui, app, density);
            });
    });
}

fn title_bar(ui: &mut Ui, app: &HvBibleApp, width: f32, density: ColumnDensity) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, TITLE_BAR_HEIGHT), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, crate::theme::bg_surface());
    painter.line_segment(
        [rect.left_top(), rect.right_top()],
        Stroke::new(1.0, crate::theme::border_subtle()),
    );

    // The bar names the panel and states the pipeline's condition, so the
    // operator still gets a read with every section folded shut.
    let mut cursor = rect.left() + 12.0;
    painter.text(
        egui::pos2(cursor, rect.center().y),
        egui::Align2::LEFT_CENTER,
        "AUDIO",
        egui::FontId::proportional(11.0),
        crate::theme::text_tertiary(),
    );
    if density.shows_labels() {
        cursor += 46.0;
        let state = app.status_label();
        let color = pipeline_color(state);
        let available = (rect.right() - cursor - 12.0).max(0.0);
        painter.galley(
            egui::pos2(cursor, rect.center().y),
            elide(ui, state, available, color, true),
            color,
        );
    }
}

fn pipeline_color(state: &str) -> egui::Color32 {
    match state {
        "SPEAKING" => crate::theme::STATUS_SUCCESS,
        "LISTENING" => crate::theme::STATUS_INFO,
        "PAUSED" => crate::theme::STATUS_WARNING,
        "SILENCE" => crate::theme::STATUS_NEUTRAL,
        _ => crate::theme::text_tertiary(),
    }
}

/// The six sections, in the order an operator works through them.
fn sections(ui: &mut Ui, app: &mut HvBibleApp, density: ColumnDensity) {
    // A local mirror, because each `PanelSection` needs its own `&mut bool`
    // and six mutable borrows of one struct is not expressible. Writing the
    // mirror back at the end is what makes a fold survive the next frame, and
    // because it lives in the config, the next run.
    let mut open = app.config.layout_state.audio_sections;

    PanelSection::new("Input Device", "◉", &mut open.input_device, density)
        .with_status_opt(Some(device_status(app)))
        .show(ui, |ui| device::show(ui, app, density));

    PanelSection::new("Input Level", "▤", &mut open.input_level, density)
        .with_status_opt(level::status(app))
        .show(ui, |ui| level::show(ui, app, density));

    PanelSection::new("Gain", "⊞", &mut open.gain, density)
        .with_status_opt(gain::status(app))
        .show(ui, |ui| gain::show(ui, app, density));

    PanelSection::new("Voice Detection", "◐", &mut open.voice, density)
        .with_status_opt(voice::status(app))
        .show(ui, |ui| voice::show(ui, app, density));

    PanelSection::new("Preset", "⌘", &mut open.preset, density)
        .with_status_opt(preset::status(app))
        .show(ui, |ui| preset::show(ui, app, density));

    PanelSection::new("LED Status", "●", &mut open.led, density)
        .with_status_opt(led::status(app))
        .show(ui, |ui| led::show(ui, app, density));

    app.config.layout_state.audio_sections = open;
}

fn device_status(app: &HvBibleApp) -> (egui::Color32, &'static str) {
    let color = device::health_color(&app.audio_mock.device.health);
    let label: &'static str = match app.audio_mock.device.health.as_str() {
        "Active" => "OPEN",
        "Warning" => "WARN",
        _ => "FAIL",
    };
    (color, label)
}
