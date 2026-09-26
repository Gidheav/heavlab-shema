use eframe::egui::{self, Context};

use crate::app::HvBibleApp;
use crate::theme::{self};

pub fn show(ctx: &Context, app: &HvBibleApp) {
    if !app.program_out {
        return;
    }

    let verse = app.current_verse.clone();
    let base_px = app.current_theme.font_config.size.to_pixels();
    
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
                                    .size(base_px * 3.85)
                                    .strong()
                                    .color(crate::theme::text_primary()),
                            );
                            ui.add_space(base_px * 2.0);
                            ui.label(egui::RichText::new(text).size(base_px * 2.4).color(crate::theme::text_primary()));
                        });
                    }
                    None => {
                        ui.centered_and_justified(|ui| {
                            ui.label(
                                egui::RichText::new("PROGRAM CLEAR")
                                    .size(base_px * 1.7)
                                    .color(crate::theme::text_secondary()),
                            );
                        });
                    }
                });
        },
    );
}