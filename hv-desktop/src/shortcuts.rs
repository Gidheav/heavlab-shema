//! Keyboard shortcuts for the broadcast operator.
//! Zoom: ui_scale is a text-only multiplier (0.5–2.0). Geometry is unaffected.

use eframe::egui::{Context, Key};

use crate::app::HvBibleApp;

pub fn handle(ctx: &Context, app: &mut HvBibleApp) {
    let typing = ctx.wants_keyboard_input();
    ctx.input(|input| {
        let ctrl = input.modifiers.ctrl || input.modifiers.command;

        if !typing && input.key_pressed(Key::Space) {
            app.approve_current();
        }
        if input.key_pressed(Key::Escape) {
            app.reject_current();
        }
        if ctrl && input.key_pressed(Key::Z) {
            app.undo_verse();
        }
        if ctrl && input.key_pressed(Key::S) {
            app.toggle_listening();
        }
        if ctrl && input.key_pressed(Key::F) {
            app.focus_manual = true;
        }
        if ctrl && input.key_pressed(Key::Comma) {
            app.config.layout_state.settings_open = !app.config.layout_state.settings_open;
        }
        if input.key_pressed(Key::F5) {
            app.toggle_listening();
        }
        if input.key_pressed(Key::F11) {
            app.program_out = !app.program_out;
        }

        // ── Zoom: Ctrl+= zoom in · Ctrl+- zoom out · Ctrl+0 reset ────────────
        // ui_scale is a pure text multiplier; geometry panels stay fixed.
        // Range: 0.5× (tiny) → 2.0× (large). Steps of 0.1×.
        // At min/max, further key presses do nothing (no bounce).
        const ZOOM_MIN: f32 = 0.5;
        const ZOOM_MAX: f32 = 2.0;
        const ZOOM_STEP: f32 = 0.1;

        if ctrl && (input.key_pressed(Key::Equals) || input.key_pressed(Key::Plus)) {
            let current = app.current_theme.font_config.ui_scale;
            if current < ZOOM_MAX {
                let s = (current + ZOOM_STEP).min(ZOOM_MAX);
                app.current_theme.font_config.ui_scale = (s * 10.0).round() / 10.0;
                app.config.font_config = Some(app.current_theme.font_config.clone());
                let _ = app.config.save();
            }
        }
        if ctrl && input.key_pressed(Key::Minus) {
            let current = app.current_theme.font_config.ui_scale;
            if current > ZOOM_MIN {
                let s = (current - ZOOM_STEP).max(ZOOM_MIN);
                app.current_theme.font_config.ui_scale = (s * 10.0).round() / 10.0;
                app.config.font_config = Some(app.current_theme.font_config.clone());
                let _ = app.config.save();
            }
        }
        if ctrl && input.key_pressed(Key::Num0) {
            app.current_theme.font_config.ui_scale = 1.0;
            app.config.font_config = Some(app.current_theme.font_config.clone());
            let _ = app.config.save();
        }
        // ─────────────────────────────────────────────────────────────────────

        if ctrl {
            let slot = if input.key_pressed(Key::Num1) {
                Some(1)
            } else if input.key_pressed(Key::Num2) {
                Some(2)
            } else if input.key_pressed(Key::Num3) {
                Some(3)
            } else if input.key_pressed(Key::Num4) {
                Some(4)
            } else if input.key_pressed(Key::Num5) {
                Some(5)
            } else if input.key_pressed(Key::Num6) {
                Some(6)
            } else if input.key_pressed(Key::Num7) {
                Some(7)
            } else if input.key_pressed(Key::Num8) {
                Some(8)
            } else if input.key_pressed(Key::Num9) {
                Some(9)
            } else {
                None
            };
            if let Some(slot) = slot {
                app.select_log_hotkey(slot);
            }
        }
    });
}
