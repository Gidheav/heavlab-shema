use egui::{FontId, Ui};

use crate::app::HvBibleApp;

pub fn show(ui: &mut Ui, app: &HvBibleApp) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("TRANSCRIPT")
                .small()
                .strong()
                .color(crate::theme::text_secondary()),
        );
        ui.add_space(12.0);
        let display = if app.transcript.is_empty() {
            "…"
        } else {
            app.transcript.as_str()
        };
        ui.scope(|ui| {
            ui.style_mut().interaction.selectable_labels = true;
            ui.add(
                egui::Label::new(
                    egui::RichText::new(display)
                        .font(FontId::monospace(14.0))
                        .color(crate::theme::text_secondary()),
                )
                .wrap(),
            );
        });
    });
}
