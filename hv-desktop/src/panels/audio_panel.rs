use egui::{ComboBox, ScrollArea, Ui};

use crate::app::HvBibleApp;
use crate::components::{CollapsibleSection, MeterBar, StatusPill};
use crate::components::status_pill::Status;
use crate::theme::{self, accent, STATUS_SUCCESS, STATUS_WARNING, STATUS_ERROR, text_primary, text_secondary, text_inverse, bg_surface_sunken, border_subtle};

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    egui::TopBottomPanel::bottom("audio_transport_bar")
        .frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(0.0, 8.0)))
        .show_inside(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let button = if app.is_listening {
                    egui::Button::new(egui::RichText::new("⏸ Pause").strong().color(crate::theme::text_inverse()))
                        .fill(crate::theme::accent())
                        .min_size(egui::vec2(80.0, 28.0))
                } else {
                    egui::Button::new(egui::RichText::new("▶ Start").strong().color(crate::theme::text_inverse()))
                        .fill(STATUS_SUCCESS)
                        .min_size(egui::vec2(80.0, 28.0))
                };
                if ui.add(button).clicked() {
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

                ui.add_space(8.0);
                let muted = false;
                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("🔇 Mute").small())
                            .fill(if muted { crate::theme::accent() } else { crate::theme::bg_surface_sunken() })
                            .min_size(egui::vec2(80.0, 28.0)),
                    )
                    .clicked()
                {
                    // TODO: Toggle mute
                }

                ui.add_space(8.0);
                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("⚙ Settings").small())
                            .fill(crate::theme::bg_surface_sunken())
                            .min_size(egui::vec2(80.0, 28.0)),
                    )
                    .clicked()
                {
                    app.config.layout_state.settings_open = true;
                }
            });

            if let Some(error) = &app.last_error {
                ui.add_space(8.0);
                ui.colored_label(theme::STATUS_ERROR, error);
            }
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(10.0, 8.0)))
        .show_inside(ui, |ui| {
            ScrollArea::vertical()
                .id_source("audio_panel_scroll")
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        // Panel title
                        ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("AUDIO CONTROL").color(crate::theme::accent()).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("—").clicked() {
                            // TODO: Collapse panel
                        }
                        if ui.small_button("📌").clicked() {
                            // TODO: Pin panel
                        }
                        if ui.small_button("⚙").clicked() {
                            // TODO: Open panel settings
                        }
                    });
                });
                ui.add_space(8.0);

                // INPUT SOURCE section
                CollapsibleSection::new("INPUT SOURCE")
                    .with_badge("ACTIVE")
                    .show(ui, |ui| {
                        ComboBox::from_id_salt("audio_device")
                            .selected_text(app.selected_device.clone())
                            .width(ui.available_width())
                            .show_ui(ui, |ui| {
                                let devices = app.devices.clone();
                                for device in devices {
                                    if ui.selectable_value(&mut app.selected_device, device.clone(), &device).changed() {
                                        app.pipeline.send_command(hv_pipeline::PipelineCommand::SetDevice(device));
                                    }
                                }
                            });

                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("WASAPI  ·  48kHz  ·  24-bit  ·  Buffer 256 (5.3 ms)")
                                .small()
                                .color(crate::theme::text_secondary()),
                        );

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if ui.small_button("🔄 Refresh").clicked() {
                                // TODO: Refresh device list
                            }
                            if ui.small_button("🔊 Test").clicked() {
                                // TODO: Test input
                            }
                            if ui.small_button("🎚 Calibrate").clicked() {
                                // TODO: Calibrate gain
                            }
                            if ui.small_button("📄 Device Info").clicked() {
                                // TODO: Open device info
                            }
                        });
                    });

                // INPUT LEVEL section
                CollapsibleSection::new("INPUT LEVEL")
                    .with_badge("RMS PK")
                    .show(ui, |ui| {
                        egui::Frame::none()
                            .fill(crate::theme::bg_surface_sunken())
                            .stroke(egui::Stroke::new(1.0, crate::theme::border_subtle()))
                            .show(ui, |ui| {
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new("L").small().color(crate::theme::text_secondary()));
                                        MeterBar::new(app.meter_rms, app.meter_peak, app.is_clipping).show(ui);
                                        ui.label(
                                            egui::RichText::new(format!("{:>6.1} dB", app.meter_rms))
                                                .small()
                                                .color(crate::theme::text_secondary()),
                                        );
                                    });
                                    ui.add_space(8.0);
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new("R").small().color(crate::theme::text_secondary()));
                                        MeterBar::new(app.meter_rms, app.meter_peak, app.is_clipping).show(ui);
                                        ui.label(
                                            egui::RichText::new(format!("{:>6.1} dB", app.meter_rms))
                                                .small()
                                                .color(crate::theme::text_secondary()),
                                        );
                                    });
                                });
                                ui.add_space(4.0);
                            });

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("Peak: {:>6.1} dB", app.meter_peak))
                                    .small()
                                    .color(crate::theme::text_secondary()),
                            );
                            ui.separator();
                            let clip_text = if app.is_clipping { "●" } else { "○" };
                            ui.label(
                                egui::RichText::new(format!("Clip: {}", clip_text))
                                    .small()
                                    .color(if app.is_clipping { STATUS_ERROR } else { crate::theme::text_secondary() }),
                            );
                            ui.separator();
                            ui.label(
                                egui::RichText::new(format!("SNR: {} dB", app.snr_db))
                                    .small()
                                    .color(crate::theme::text_secondary()),
                            );
                            ui.separator();
                            ui.label(
                                egui::RichText::new("LUFS: -22.4")
                                    .small()
                                    .color(crate::theme::text_secondary()),
                            );
                        });

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ComboBox::from_id_salt("hold_duration")
                                .selected_text("3s")
                                .width(60.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut "1s".to_string(), "1s".to_string(), "1s");
                                    ui.selectable_value(&mut "3s".to_string(), "3s".to_string(), "3s");
                                    ui.selectable_value(&mut "5s".to_string(), "5s".to_string(), "5s");
                                });
                            ui.add_space(8.0);
                            if ui.small_button("Reset Peak").clicked() {
                                // TODO: Reset peak
                            }
                            ui.add_space(8.0);
                            if ui.small_button("Clear Clip").clicked() {
                                // TODO: Clear clip
                            }
                        });
                    });

                // VOICE ACTIVITY DETECTION section
                CollapsibleSection::new("VOICE ACTIVITY DETECTION")
                    .with_badge("● ACTIVE")
                    .show(ui, |ui| {
                        egui::Frame::none()
                            .fill(crate::theme::bg_surface_sunken())
                            .stroke(egui::Stroke::new(1.0, crate::theme::border_subtle()))
                            .show(ui, |ui| {
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    let status = match app.pipeline_state {
                                        hv_pipeline::PipelineState::Stopped => Status::Error,
                                        hv_pipeline::PipelineState::Listening | hv_pipeline::PipelineState::Silence => Status::Warning,
                                        hv_pipeline::PipelineState::Speaking => Status::Success,
                                    };
                                    StatusPill::new(status, "").blink(app.led_blink).show(ui);
                                    ui.vertical(|ui| {
                                        let vad_status = match app.pipeline_state {
                                            hv_pipeline::PipelineState::Stopped => "OFF",
                                            hv_pipeline::PipelineState::Listening => "ARMED",
                                            hv_pipeline::PipelineState::Silence => "IDLE",
                                            hv_pipeline::PipelineState::Speaking => "ACTIVE",
                                        };
                                        ui.label(
                                            egui::RichText::new(vad_status)
                                                .strong()
                                                .color(app.led_color),
                                        );
                                        ui.label(
                                            egui::RichText::new(format!("Confidence: {:.2}", app.asr_confidence))
                                                .small()
                                                .color(crate::theme::text_secondary()),
                                        );
                                    });
                                });
                                ui.add_space(4.0);
                            });

                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Sensitivity").small().color(crate::theme::text_secondary()));
                        let mut sensitivity = 0.6_f32;
                        ui.add(egui::Slider::new(&mut sensitivity, 0.0..=1.0).show_value(true));

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Wake threshold: 0.60")
                                    .small()
                                    .color(crate::theme::text_secondary()),
                            );
                            ui.add_space(16.0);
                            ui.label(
                                egui::RichText::new("Sleep after: 1.5s")
                                    .small()
                                    .color(crate::theme::text_secondary()),
                            );
                            ui.add_space(16.0);
                            ui.label(
                                egui::RichText::new("Min speech: 250ms")
                                    .small()
                                    .color(crate::theme::text_secondary()),
                            );
                        });
                    });

                // Spacer before pinned bottom bar
                ui.add_space(16.0);
            });
        });
                });
        });
}