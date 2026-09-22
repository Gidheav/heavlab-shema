//! Keyboard shortcuts for the broadcast operator.

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
        if input.key_pressed(Key::F5) {
            app.toggle_listening();
        }
        if input.key_pressed(Key::F11) {
            app.program_out = !app.program_out;
        }

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
