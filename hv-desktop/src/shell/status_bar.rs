use eframe::egui::{self, Color32, Context, RichText};
use hv_pipeline::PipelineState;

use crate::app::HvBibleApp;
use crate::theme::{STATUS_ERROR, STATUS_SUCCESS, STATUS_WARNING};

pub fn show(ctx: &Context, app: &HvBibleApp) {
    let base_px = app.current_theme.font_config.size.to_pixels();
    let label_size = (base_px * 0.72).max(9.5);

    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(26.0)
        .frame(
            egui::Frame::none()
                .fill(crate::theme::bg_surface_sunken())
                .stroke(egui::Stroke::new(1.0, crate::theme::border_subtle()))
                .inner_margin(egui::Margin::symmetric(12.0, 4.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;

                // Pipeline status — color-coded pill
                let (status_label, status_color) = match app.pipeline_state {
                    PipelineState::Stopped => ("● STOPPED", STATUS_ERROR),
                    PipelineState::Listening | PipelineState::Silence => ("● ARMED", STATUS_WARNING),
                    PipelineState::Speaking => ("● SPEAKING", STATUS_SUCCESS),
                };
                ui.label(
                    RichText::new(status_label)
                        .size(label_size)
                        .strong()
                        .color(status_color),
                );

                sep(ui, label_size);
                metric(ui, &app.selected_device.chars().take(28).collect::<String>(), label_size, crate::theme::text_secondary());
                sep(ui, label_size);
                metric(ui, "48kHz / 24-bit", label_size, crate::theme::text_secondary());
                sep(ui, label_size);
                metric(ui, asr_label(app), label_size, crate::theme::text_secondary());

                // Right-aligned metrics
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;

                    // Session timer placeholder
                    metric(ui, "00:42:18", label_size, crate::theme::text_tertiary());
                    sep(ui, label_size);
                    metric(ui, "Queue: 0", label_size, crate::theme::text_secondary());
                    sep(ui, label_size);
                    metric(ui, "1.24 GB", label_size, crate::theme::text_secondary());
                    sep(ui, label_size);

                    // Latency — color-coded
                    let latency = 142u32;
                    let lat_color = if latency < 200 { STATUS_SUCCESS } else { STATUS_WARNING };
                    metric(ui, &format!("{latency}ms"), label_size, lat_color);
                });
            });
        });
}

fn metric(ui: &mut egui::Ui, text: &str, size: f32, color: Color32) {
    ui.label(RichText::new(text).size(size).color(color));
}

fn sep(ui: &mut egui::Ui, size: f32) {
    ui.label(
        RichText::new("│")
            .size(size)
            .color(crate::theme::border_strong()),
    );
}

fn asr_label(app: &HvBibleApp) -> &'static str {
    match app.asr_info {
        crate::pipeline_integration::AsrEngineInfo::Mock => "Mock ASR",
        crate::pipeline_integration::AsrEngineInfo::SherpaCpu => "Zipformer CPU",
        crate::pipeline_integration::AsrEngineInfo::SherpaGpu => "Zipformer GPU",
    }
}
