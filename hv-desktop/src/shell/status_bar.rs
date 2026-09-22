use eframe::egui::{self, Context};
use hv_pipeline::PipelineState;

use crate::app::HvBibleApp;
use crate::theme::{
    STATUS_ERROR, STATUS_SUCCESS, STATUS_WARNING,
};

pub fn show(ctx: &Context, app: &HvBibleApp) {
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(24.0)
        .frame(
            egui::Frame::none()
                .fill(crate::theme::bg_surface_sunken())
                .inner_margin(egui::Margin::symmetric(12.0, 4.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let status_color = match app.pipeline_state {
                    PipelineState::Stopped => STATUS_ERROR,
                    PipelineState::Listening | PipelineState::Silence => STATUS_WARNING,
                    PipelineState::Speaking => STATUS_SUCCESS,
                };
                metric(ui, app.status_label(), status_color);
                sep(ui);
                metric(ui, &app.selected_device, crate::theme::text_secondary());
                sep(ui);
                metric(ui, "48kHz / 24-bit", crate::theme::text_secondary());
                sep(ui);
                metric(ui, asr_label(app), crate::theme::text_secondary());
                sep(ui);
                metric(ui, "142ms", crate::theme::text_secondary());
                sep(ui);
                metric(ui, "1.24GB", crate::theme::text_secondary());
                sep(ui);
                metric(ui, "Queue 0", crate::theme::text_secondary());
                sep(ui);
                metric(ui, "00:42:18", crate::theme::text_secondary());
            });
        });
}

fn metric(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    ui.label(egui::RichText::new(text).size(10.0).color(color));
}

fn sep(ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("|").size(10.0).color(crate::theme::text_secondary()));
}

fn asr_label(app: &HvBibleApp) -> &'static str {
    match app.asr_info {
        crate::pipeline_integration::AsrEngineInfo::Mock => "Mock Engine",
        crate::pipeline_integration::AsrEngineInfo::SherpaCpu => "Zipformer CPU",
        crate::pipeline_integration::AsrEngineInfo::SherpaGpu => "Zipformer GPU",
    }
}
