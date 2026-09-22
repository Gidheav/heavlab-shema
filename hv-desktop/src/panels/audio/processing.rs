use eframe::egui::{self, Ui};

use crate::app::HvBibleApp;
use crate::config::AppConfig;
use hv_pipeline::events::PipelineCommand;

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    // Header hint
    ui.label(
        egui::RichText::new("Toggle DSP processors in the chain. Active processors run top-to-bottom.")
            .size(10.0)
            .color(crate::theme::text_secondary()),
    );
    ui.add_space(6.0);

    // High-Pass Filter
    let mut hpf_enabled = app.config.hpf_enabled;
    if processor_row(ui, 0, &mut hpf_enabled, "High-Pass Filter", format!("Freq: {}Hz", app.config.hpf_frequency).as_str(), "Slope: 12dB/oct") {
        app.config.hpf_enabled = hpf_enabled;
        app.pipeline.send_command(PipelineCommand::EnableHighPass(hpf_enabled));
        let _ = app.config.save();
    }

    // Noise Gate
    let mut gate_enabled = app.config.noise_gate_enabled;
    if processor_row(ui, 1, &mut gate_enabled, "Noise Gate", format!("Threshold: {:.3}", app.config.noise_gate_threshold).as_str(), "Attack: 10ms | Release: 100ms") {
        app.config.noise_gate_enabled = gate_enabled;
        app.pipeline.send_command(PipelineCommand::EnableNoiseGate(gate_enabled));
        let _ = app.config.save();
    }

    // Compressor
    let mut comp_enabled = app.config.compressor_enabled;
    if processor_row(ui, 2, &mut comp_enabled, "Compressor", format!("Ratio: {}:1", app.config.compressor_ratio).as_str(), "Threshold: -20dBFS") {
        app.config.compressor_enabled = comp_enabled;
        app.pipeline.send_command(PipelineCommand::EnableCompressor(comp_enabled));
        let _ = app.config.save();
    }
}

// Returns true if the toggle was clicked
fn processor_row(ui: &mut Ui, index: usize, enabled: &mut bool, name: &str, primary: &str, secondary: &str) -> bool {
    let mut changed = false;
    egui::Frame::none()
        .fill(if *enabled {
            crate::theme::bg_surface_raised()
        } else {
            crate::theme::bg_surface_sunken()
        })
        .rounding(4.0)
        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                // Index
                ui.label(
                    egui::RichText::new(format!("{}", index + 1))
                        .size(10.0)
                        .color(crate::theme::text_tertiary()),
                );

                // Toggle
                if ui.checkbox(enabled, "").changed() {
                    changed = true;
                }

                // Name
                ui.label(
                    egui::RichText::new(name)
                        .size(12.0)
                        .strong()
                        .color(crate::theme::text_primary()),
                );

                ui.add_space(8.0);

                // Primary params
                ui.label(
                    egui::RichText::new(primary)
                        .size(10.0)
                        .color(crate::theme::text_secondary()),
                );
                ui.add_space(4.0);

                // Secondary params
                ui.label(
                    egui::RichText::new(secondary)
                        .size(10.0)
                        .color(crate::theme::text_secondary()),
                );
                ui.add_space(4.0);

                // ON/OFF badge
                let (badge_text, badge_color) = if *enabled {
                    ("ON", crate::theme::STATUS_SUCCESS)
                } else {
                    ("OFF", crate::theme::text_tertiary())
                };
                ui.label(
                    egui::RichText::new(badge_text)
                        .size(10.0)
                        .strong()
                        .color(badge_color),
                );
            });
        });
    ui.add_space(2.0);
    changed
}
