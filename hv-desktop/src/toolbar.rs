//! Primary toolbar implementation per PRODUCT_BLUEPRINT.md Section 1.3

use egui::Ui;

use crate::app::HvBibleApp;
use crate::theme::{accent, STATUS_SUCCESS, STATUS_ERROR, text_inverse, text_secondary};

pub fn show_primary_toolbar(ui: &mut Ui, app: &mut HvBibleApp) {
    ui.horizontal(|ui| {
        // Session Control group
        ui.add_space(8.0);

        let start_stop_color = if app.is_listening {
            crate::theme::accent()
        } else {
            STATUS_SUCCESS
        };
        let start_stop_text = if app.is_listening {
            "⏸ Pause"
        } else {
            "▶ Start"
        };

        if ui
            .add(
                egui::Button::new(egui::RichText::new(start_stop_text).strong().color(crate::theme::text_inverse()))
                    .fill(start_stop_color)
                    .min_size(egui::vec2(80.0, 28.0)),
            )
            .clicked()
        {
            app.toggle_listening();
        }

        if app.is_listening {
            if ui
                .add(
                    egui::Button::new(egui::RichText::new("⏹ Stop").strong().color(crate::theme::text_inverse()))
                        .fill(STATUS_ERROR)
                        .min_size(egui::vec2(80.0, 28.0)),
                )
                .clicked()
            {
                app.toggle_listening();
            }
        }

        ui.add_space(16.0);

        // Navigation group
        if ui
            .button(egui::RichText::new("🔍 Search").color(crate::theme::text_secondary()))
            .clicked()
        {
            // TODO: Open search dialog
        }

        if ui
            .button(egui::RichText::new("📖 Manual").color(crate::theme::text_secondary()))
            .clicked()
        {
            app.focus_manual = true;
        }

        ui.add_space(16.0);

        // Output group
        if ui
            .button(egui::RichText::new("📺 Program Out").color(crate::theme::text_secondary()))
            .clicked()
        {
            app.program_out = !app.program_out;
        }

        ui.add_space(16.0);

        // Settings
        if ui
            .button(egui::RichText::new("⚙").color(crate::theme::text_secondary()))
            .clicked()
        {
            // TODO: Open settings dialog
        }

        ui.add_space(8.0);
    });
}