//! audio/mod.rs — LEFT COLUMN SHELL (STRIPPED — PHASE 1 CHECKPOINT)
//!
//! The previous console lives, archived and uncompiled, in
//! `panels/_deprecated/audio_console_v2/`. This module renders an empty
//! container only: a title bar and a blank body.

use eframe::egui::{self, RichText, Sense, Vec2, Ui};

use crate::app::HvBibleApp;

const TITLE_BAR_HEIGHT: f32 = 28.0;

/// Renders the left column slot. Intentionally empty.
pub fn show(ui: &mut Ui, _app: &mut HvBibleApp) {
    let width = ui.available_width();
    let height = ui.available_height();
    if width <= 0.0 || height <= 0.0 {
        return;
    }

    ui.painter()
        .rect_filled(ui.max_rect(), 0.0, crate::theme::bg_surface());

    // ── Title bar ───────────────────────────────────────────────────────────
    let (title_rect, _) =
        ui.allocate_exact_size(Vec2::new(width, TITLE_BAR_HEIGHT), Sense::hover());
    ui.painter()
        .rect_filled(title_rect, 0.0, crate::theme::bg_surface());
    ui.painter().line_segment(
        [
            title_rect.left_top(),
            title_rect.right_top(),
        ],
        egui::Stroke::new(1.0, crate::theme::border_subtle()),
    );
    ui.painter().text(
        egui::pos2(title_rect.left() + 12.0, title_rect.center().y),
        egui::Align2::LEFT_CENTER,
        "AUDIO",
        egui::FontId::proportional(11.0),
        crate::theme::text_tertiary(),
    );

    // ── Blank body ──────────────────────────────────────────────────────────
    let body = egui::Rect::from_min_size(
        title_rect.left_bottom(),
        Vec2::new(width, (height - TITLE_BAR_HEIGHT).max(0.0)),
    );
    ui.painter()
        .rect_filled(body, 0.0, crate::theme::bg_surface());
    ui.allocate_space(body.size());

    let _ = RichText::new("");
}