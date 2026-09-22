use eframe::egui::{self, Context};

use crate::app::HvBibleApp;
use crate::theme::{
    STATUS_ERROR, STATUS_SUCCESS,
};

pub fn show(ctx: &Context, app: &mut HvBibleApp) {
    egui::TopBottomPanel::top("command_bar")
        .exact_height(44.0)
        .frame(
            egui::Frame::none()
                .fill(crate::theme::bg_surface_raised())
                .stroke(egui::Stroke::new(1.0, crate::theme::border_subtle())),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(8.0);
                let label = if app.is_listening { "⏸" } else { "▶" };
                let fill = if app.is_listening {
                    crate::theme::accent()
                } else {
                    STATUS_SUCCESS
                };
                if command_button(ui, label, fill).clicked() {
                    app.toggle_listening();
                }
                if command_button(ui, "⏹", STATUS_ERROR).clicked() && app.is_listening {
                    app.toggle_listening();
                }
                divider(ui);
                if secondary_button(ui, "🔍").clicked() {}
                if secondary_button(ui, "Manual").clicked() {
                    app.focus_manual = true;
                }
                if secondary_button(ui, "Quick Verse").clicked() {
                    app.focus_manual = true;
                }
                divider(ui);
                if secondary_button(ui, "Program Out").clicked() {
                    app.program_out = !app.program_out;
                }
                secondary_button(ui, "Lower Third");
                secondary_button(ui, "Clean Feed");
                divider(ui);
                ui.label(
                    egui::RichText::new("Workspace")
                        .size(12.0)
                        .color(crate::theme::text_secondary()),
                );
                egui::ComboBox::from_id_salt("workspace_preset")
                    .selected_text(app.workspace_preset.name)
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        let _ = ui.selectable_label(true, "Operator Broadcast");
                        let _ = ui.selectable_label(false, "Broadcast Focus");
                        let _ = ui.selectable_label(false, "Diagnostics");
                        let _ = ui.selectable_label(false, "Minimal");
                    });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    secondary_button(ui, "Settings");
                    secondary_button(ui, "Health");
                });
            });
        });
}

fn command_button(ui: &mut egui::Ui, label: &str, fill: egui::Color32) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(label).strong().color(crate::theme::text_inverse()))
            .fill(fill)
            .min_size(egui::vec2(82.0, 28.0)),
    )
}

fn secondary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(label).color(crate::theme::text_primary()))
            .fill(crate::theme::bg_surface_raised())
            .stroke(egui::Stroke::new(1.0, crate::theme::border_subtle()))
            .min_size(egui::vec2(84.0, 28.0)),
    )
}

fn divider(ui: &mut egui::Ui) {
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);
}
