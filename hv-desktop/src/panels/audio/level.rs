//! level.rs — Input Level section.
//!
//! Two channel strips, a peak-hold tick on each, a latched clip light, and the
//! SNR figure. This is the section that must never be scrolled out of view, so
//! its header carries a live status light even when folded.

use eframe::egui::{self, Ui};

use crate::app::HvBibleApp;
use crate::components::channel_meter::ChannelMeter;
use crate::components::column_controls::{readout, InlineButton};
use crate::components::column_density::ColumnDensity;

pub fn show(ui: &mut Ui, app: &mut HvBibleApp, density: ColumnDensity) {
    for channel in 0..2 {
        ChannelMeter {
            channel: if channel == 0 { "L" } else { "R" },
            rms_db: app.meter_rms,
            peak_hold_db: app.channel_peak_hold[channel],
            clipped: app.channel_clip[channel],
            density,
        }
        .show(ui);
        if density.shows_labels() {
            ui.add_space(density.row_gap());
        }
    }

    if density.shows_labels() {
        readout(ui, "SNR", &format!("{} dB", app.snr_db));
    }

    if density.shows_secondary() {
        ui.add_space(density.row_gap());
        readout(ui, "LUFS", &format!("{:.1}", app.audio_mock.level.lufs));
        readout(ui, "Meter", &app.audio_mock.level.meter_mode.clone());
    }

    // The clip lights latch, so they need a way out. Offering it only when
    // something has actually latched keeps the button from being decoration.
    if app.channel_clip.iter().any(|clipped| *clipped) {
        ui.add_space(density.row_gap());
        if InlineButton::new("Clear clip", density)
            .icon("✕")
            .danger(true)
            .show(ui)
            .on_hover_text("Acknowledge the clip and drop the peak-hold markers")
            .clicked()
        {
            app.clear_clip();
        }
    }
}

/// Whether the header should show a warning light for this section.
pub fn status(app: &HvBibleApp) -> Option<(egui::Color32, &'static str)> {
    if app.channel_clip.iter().any(|clipped| *clipped) {
        return Some((crate::theme::STATUS_ERROR, "CLIP"));
    }
    if app.is_clipping {
        return Some((crate::theme::STATUS_ERROR, "CLIP"));
    }
    None
}
