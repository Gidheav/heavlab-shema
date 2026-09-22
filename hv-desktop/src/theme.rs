//! HV-Bible Desktop design system.
//! Supports Purple Graphite (Dark) and Light variants.
#![allow(dead_code)]

use egui::Color32;
use std::sync::atomic::{AtomicBool, Ordering};

pub const THEME_NAME: &str = "Purple Graphite";

pub static IS_DARK_MODE: AtomicBool = AtomicBool::new(true);

pub fn set_dark_mode(dark: bool) {
    IS_DARK_MODE.store(dark, Ordering::Relaxed);
}

pub fn is_dark() -> bool {
    IS_DARK_MODE.load(Ordering::Relaxed)
}

// Status colors stay semantically distinct.
pub const STATUS_SUCCESS: Color32 = Color32::from_rgb(0x78, 0xB1, 0x78);
pub const STATUS_WARNING: Color32 = Color32::from_rgb(0xC8, 0x9B, 0x5D);
pub const STATUS_ERROR: Color32 = Color32::from_rgb(0xBE, 0x63, 0x72);
pub const STATUS_INFO: Color32 = Color32::from_rgb(0x7C, 0x92, 0xD6);
pub const STATUS_NEUTRAL: Color32 = Color32::from_rgb(0x84, 0x7D, 0x92);

pub fn apply_visuals(ctx: &egui::Context, dark_mode: bool) {
    set_dark_mode(dark_mode);

    let mut visuals = if dark_mode {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    visuals.override_text_color = Some(text_primary());
    visuals.panel_fill = bg_base();
    visuals.window_fill = bg_surface();
    visuals.extreme_bg_color = bg_surface_sunken();
    visuals.faint_bg_color = bg_surface();
    visuals.widgets.inactive.bg_fill = bg_surface_raised();
    visuals.widgets.hovered.bg_fill = if dark_mode {
        bg_surface_raised().linear_multiply(1.08)
    } else {
        bg_surface_raised().linear_multiply(0.92)
    };
    visuals.widgets.active.bg_fill = accent_muted();
    visuals.widgets.open.bg_fill = bg_surface_raised();
    visuals.widgets.noninteractive.bg_fill = bg_surface();
    visuals.selection.bg_fill = accent_muted();
    visuals.hyperlink_color = accent();
    
    let tp = text_primary();
    visuals.widgets.inactive.fg_stroke.color = tp;
    visuals.widgets.hovered.fg_stroke.color = tp;
    visuals.widgets.active.fg_stroke.color = tp;
    
    ctx.set_visuals(visuals);
}

pub fn bg_base() -> Color32 {
    if is_dark() { Color32::from_rgb(0x15, 0x13, 0x1D) } else { Color32::from_rgb(0xF5, 0xF5, 0xF7) }
}
pub fn bg_surface() -> Color32 {
    if is_dark() { Color32::from_rgb(0x20, 0x1D, 0x2A) } else { Color32::from_rgb(0xFF, 0xFF, 0xFF) }
}
pub fn bg_surface_raised() -> Color32 {
    if is_dark() { Color32::from_rgb(0x2A, 0x25, 0x36) } else { Color32::from_rgb(0xEA, 0xEA, 0xED) }
}
pub fn bg_surface_sunken() -> Color32 {
    if is_dark() { Color32::from_rgb(0x11, 0x10, 0x18) } else { Color32::from_rgb(0xE1, 0xE1, 0xE5) }
}
pub fn border_subtle() -> Color32 {
    if is_dark() { Color32::from_rgb(0x36, 0x30, 0x46) } else { Color32::from_rgb(0xD2, 0xD2, 0xD7) }
}
pub fn border_strong() -> Color32 {
    if is_dark() { Color32::from_rgb(0x4B, 0x42, 0x61) } else { Color32::from_rgb(0xAD, 0xAD, 0xB5) }
}
pub fn text_primary() -> Color32 {
    if is_dark() { Color32::from_rgb(0xEC, 0xE9, 0xF3) } else { Color32::from_rgb(0x1D, 0x1D, 0x1F) }
}
pub fn text_secondary() -> Color32 {
    if is_dark() { Color32::from_rgb(0xB4, 0xAD, 0xC4) } else { Color32::from_rgb(0x38, 0x38, 0x3D) }
}
pub fn text_tertiary() -> Color32 {
    if is_dark() { Color32::from_rgb(0x77, 0x6F, 0x88) } else { Color32::from_rgb(0x58, 0x58, 0x60) }
}
pub fn text_inverse() -> Color32 {
    if is_dark() { Color32::from_rgb(0x12, 0x10, 0x18) } else { Color32::from_rgb(0xFF, 0xFF, 0xFF) }
}
pub fn accent() -> Color32 {
    if is_dark() { Color32::from_rgb(0xA7, 0x78, 0xF2) } else { Color32::from_rgb(0x8C, 0x52, 0xFF) }
}
pub fn accent_hover() -> Color32 {
    if is_dark() { Color32::from_rgb(0xB8, 0x8F, 0xFF) } else { Color32::from_rgb(0x72, 0x37, 0xE6) }
}
pub fn accent_muted() -> Color32 {
    if is_dark() { Color32::from_rgb(0x4F, 0x39, 0x73) } else { Color32::from_rgb(0xDF, 0xD1, 0xF7) }
}

pub fn meter_color(rms_db: f32) -> Color32 {
    if rms_db > -3.0 {
        STATUS_ERROR
    } else if rms_db > -12.0 {
        STATUS_WARNING
    } else {
        STATUS_SUCCESS
    }
}
