use eframe::egui::{self, ComboBox, Ui};

use crate::app::HvBibleApp;

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    let mon = &mut app.audio_mock.monitoring;

    // Level slider
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Monitor Level")
                .size(11.0)
                .color(crate::theme::text_secondary()),
        );
        let w = (ui.available_width() - 70.0).clamp(40.0, 200.0);
        ui.add_sized(
            egui::vec2(w, 18.0),
            egui::Slider::new(&mut mon.level_db, -24.0..=12.0).show_value(false),
        );
        ui.label(
            egui::RichText::new(format!("{:+.1} dB", mon.level_db))
                .size(11.0)
                .monospace()
                .color(crate::theme::text_primary()),
        );
    });

    ui.add_space(6.0);

    // Controls
    ui.horizontal_wrapped(|ui| {
        toggle_btn(ui, "🔇 Mute", &mut mon.muted);
        ui.add_space(4.0);
        toggle_btn(ui, "🎧 Solo", &mut mon.solo);
        ui.add_space(4.0);

        // Test tone
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("♩ 1kHz Tone")
                        .size(11.0)
                        .color(crate::theme::text_primary()),
                )
                .fill(crate::theme::bg_surface_sunken()),
            )
            .clicked()
        {}
    });

    ui.add_space(4.0);

    // Output device
    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new("Output")
                .size(11.0)
                .color(crate::theme::text_secondary()),
        );
        let combo_width = (ui.available_width() - 8.0).clamp(80.0, 130.0);
        ComboBox::from_id_salt("monitor_out_v2")
            .selected_text(mon.output_device.clone())
            .width(combo_width)
            .show_ui(ui, |ui| {
                for item in ["Default", "Headphones", "Line Out", "Program Bus"] {
                    ui.selectable_value(&mut mon.output_device, item.to_string(), item);
                }
            });

        ui.add_space(8.0);

        ui.label(
            egui::RichText::new("Cue Bus")
                .size(11.0)
                .color(crate::theme::text_secondary()),
        );
        ui.label(
            egui::RichText::new(&mon.cue_bus)
                .size(11.0)
                .color(crate::theme::text_primary()),
        );
    });
}

fn toggle_btn(ui: &mut Ui, label: &str, active: &mut bool) {
    let fill = if *active {
        crate::theme::accent_muted()
    } else {
        crate::theme::bg_surface_sunken()
    };
    let text_color = if *active {
        crate::theme::accent()
    } else {
        crate::theme::text_primary()
    };
    if ui
        .add(
            egui::Button::new(egui::RichText::new(label).size(11.0).color(text_color))
                .fill(fill),
        )
        .clicked()
    {
        *active = !*active;
    }
}