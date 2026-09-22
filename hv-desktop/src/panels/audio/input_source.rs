use eframe::egui::{self, ComboBox, Ui};

use crate::app::HvBibleApp;
use crate::panels::audio::labeled_slider;

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    // Device selector
    ComboBox::from_id_salt("audio_device_v2")
        .selected_text(app.selected_device.clone())
        .show_ui(ui, |ui| {
            for device in app.devices.clone() {
                if ui
                    .selectable_value(&mut app.selected_device, device.clone(), &device)
                    .changed()
                {
                    app.pipeline
                        .send_command(hv_pipeline::PipelineCommand::SetDevice(device));
                }
            }
        });

    ui.add_space(6.0);

    // Device info row
    let d = &app.audio_mock.device;
    ui.horizontal_wrapped(|ui| {
        tag(ui, &d.driver);
        tag(ui, &format!("{}kHz", d.sample_rate / 1000));
        tag(ui, &format!("{}-bit", d.bit_depth));
        tag(ui, &format!("Buf {}", d.buffer_size));
        tag(ui, &format!("{:.1} ms", d.buffer_latency_ms));
    });

    ui.add_space(4.0);

    // Health indicator
    let health_color = match d.health.as_str() {
        "Active" => crate::theme::STATUS_SUCCESS,
        "Warning" => crate::theme::STATUS_WARNING,
        _ => crate::theme::STATUS_ERROR,
    };
    ui.horizontal(|ui| {
        let (dot, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
        ui.painter().circle_filled(dot.center(), 4.0, health_color);
        ui.label(
            egui::RichText::new(&d.health)
                .size(11.0)
                .color(health_color),
        );
    });

    ui.add_space(6.0);

    // Channel status
    ui.horizontal_wrapped(|ui| {
        for ch in &app.audio_mock.routing.channels {
            let color = if ch.active {
                crate::theme::STATUS_SUCCESS
            } else {
                crate::theme::STATUS_ERROR
            };
            ui.colored_label(color, &ch.id);
            ui.label(
                egui::RichText::new(format!("{:.1}dB", ch.peak_db))
                    .size(10.0)
                    .monospace()
                    .color(crate::theme::text_secondary()),
            );
            ui.add_space(8.0);
        }
        ui.label(
            egui::RichText::new(format!("Phase: {}", app.audio_mock.routing.phase))
                .size(10.0)
                .color(crate::theme::text_secondary()),
        );
    });

    ui.add_space(6.0);

    // Input gain quick slider
    labeled_slider(
        ui,
        "Input Gain",
        &mut app.gain,
        -20.0,
        20.0,
        "dB",
    );

    ui.add_space(4.0);

    // Action buttons
    ui.horizontal_wrapped(|ui| {
        for (icon, label, tip) in [
            ("🔄", "Refresh", "Refresh device list"),
            ("🔊", "Test", "Play test tone"),
            ("🎚", "Calibrate", "Auto-calibrate gain"),
            ("ℹ", "Info", "Show device details"),
        ] {
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(format!("{} {}", icon, label))
                            .size(11.0)
                            .color(crate::theme::text_primary()),
                    )
                    .fill(crate::theme::bg_surface_sunken()),
                )
                .on_hover_text(tip)
                .clicked()
            {}
        }
    });
}

fn tag(ui: &mut Ui, label: &str) {
    egui::Frame::none()
        .fill(crate::theme::bg_surface_sunken())
        .rounding(3.0)
        .inner_margin(egui::Margin::symmetric(5.0, 2.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(label)
                    .size(10.0)
                    .monospace()
                    .color(crate::theme::text_primary()),
            );
        });
}