use eframe::egui::{self, Context};

use crate::app::HvBibleApp;
use crate::theme::{self, bg_base, text_primary, text_secondary};

pub fn show(ctx: &Context, app: &HvBibleApp) {
    if !app.program_out {
        return;
    }

    let verse = app.current_verse.clone();
    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("program_out"),
        egui::ViewportBuilder::default()
            .with_title("HV-Bible - Program Out")
            .with_fullscreen(true)
            .with_decorations(false),
        move |ctx, _class| {
            theme::apply_visuals(ctx, crate::theme::is_dark());
            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(crate::theme::bg_base()).inner_margin(64.0))
                .show(ctx, |ui| match &verse {
                    Some((reference, text)) => {
                        ui.vertical_centered(|ui| {
                            ui.add_space(120.0);
                            ui.label(
                                egui::RichText::new(reference)
                                    .size(54.0)
                                    .strong()
                                    .color(crate::theme::text_primary()),
                            );
                            ui.add_space(28.0);
                            ui.label(egui::RichText::new(text).size(34.0).color(crate::theme::text_primary()));
                        });
                    }
                    None => {
                        ui.centered_and_justified(|ui| {
                            ui.label(
                                egui::RichText::new("PROGRAM CLEAR")
                                    .size(24.0)
                                    .color(crate::theme::text_secondary()),
                            );
                        });
                    }
                });
        },
    );
}