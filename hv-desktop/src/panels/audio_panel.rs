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
                    // TODO: Open settings
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

                // CHANNEL ROUTING section
                CollapsibleSection::new("CHANNEL ROUTING").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Input:").small().color(crate::theme::text_secondary()));
                        ComboBox::from_id_salt("input_channels")
                            .selected_text("Stereo")
                            .width(100.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut "Stereo".to_string(), "Stereo".to_string(), "Stereo");
                                ui.selectable_value(&mut "Mono".to_string(), "Mono".to_string(), "Mono");
                            });

                        ui.add_space(16.0);
                        ui.label(egui::RichText::new("Processing:").small().color(crate::theme::text_secondary()));
                        ComboBox::from_id_salt("processing_mode")
                            .selected_text("Mono (L+R)")
                            .width(120.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut "Mono (L+R)".to_string(), "Mono (L+R)".to_string(), "Mono (L+R)");
                                ui.selectable_value(&mut "Mono (L)".to_string(), "Mono (L)".to_string(), "Mono (L)");
                                ui.selectable_value(&mut "Mono (R)".to_string(), "Mono (R)".to_string(), "Mono (R)");
                            });
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("L: ● Active").small().color(STATUS_SUCCESS));
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("R: ● Active").small().color(STATUS_SUCCESS));
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Phase: Normal").small().color(crate::theme::text_secondary()));
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

                // GAIN STAGING section
                CollapsibleSection::new("GAIN STAGING").show(ui, |ui| {
                    ui.label(egui::RichText::new("Input Gain").small().color(crate::theme::text_secondary()));
                    if ui.add(
                        egui::Slider::new(&mut app.gain, 0.0..=2.0)
                            .show_value(true)
                            .custom_formatter(|v, _| format!("{v:.1} dB")),
                    ).changed() {
                        app.pipeline.send_command(hv_pipeline::PipelineCommand::SetGain(app.gain));
                    }

                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("Digital Trim").small().color(crate::theme::text_secondary()));
                    let mut digital_trim = 0.0_f32;
                    ui.add(
                        egui::Slider::new(&mut digital_trim, -12.0..=12.0)
                            .show_value(true)
                            .custom_formatter(|v, _| format!("{v:.1} dB")),
                    );

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("High-Pass").small().color(crate::theme::text_secondary()));
                        ComboBox::from_id_salt("hpf_freq")
                            .selected_text("80 Hz")
                            .width(80.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut "Off".to_string(), "Off".to_string(), "Off");
                                ui.selectable_value(&mut "60 Hz".to_string(), "60 Hz".to_string(), "60 Hz");
                                ui.selectable_value(&mut "80 Hz".to_string(), "80 Hz".to_string(), "80 Hz");
                                ui.selectable_value(&mut "100 Hz".to_string(), "100 Hz".to_string(), "100 Hz");
                            });
                        ui.add_space(8.0);
                        let mut hpf_enabled = true;
                        ui.checkbox(&mut hpf_enabled, "Enabled");
                        ui.add_space(8.0);
                        ComboBox::from_id_salt("hpf_slope")
                            .selected_text("12 dB/oct")
                            .width(80.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut "6 dB/oct".to_string(), "6 dB/oct".to_string(), "6 dB/oct");
                                ui.selectable_value(&mut "12 dB/oct".to_string(), "12 dB/oct".to_string(), "12 dB/oct");
                                ui.selectable_value(&mut "24 dB/oct".to_string(), "24 dB/oct".to_string(), "24 dB/oct");
                            });
                    });
                });

                // PROCESSING section
                CollapsibleSection::new("PROCESSING").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let mut agc = true;
                        ui.checkbox(&mut agc, "AGC");
                        ui.add_space(8.0);
                        ComboBox::from_id_salt("agc_target")
                            .selected_text("-18 dB")
                            .width(70.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut "-18 dB".to_string(), "-18 dB".to_string(), "-18 dB");
                                ui.selectable_value(&mut "-12 dB".to_string(), "-12 dB".to_string(), "-12 dB");
                                ui.selectable_value(&mut "-24 dB".to_string(), "-24 dB".to_string(), "-24 dB");
                            });
                        ui.add_space(8.0);
                        ComboBox::from_id_salt("agc_attack")
                            .selected_text("10ms")
                            .width(60.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut "5ms".to_string(), "5ms".to_string(), "5ms");
                                ui.selectable_value(&mut "10ms".to_string(), "10ms".to_string(), "10ms");
                                ui.selectable_value(&mut "20ms".to_string(), "20ms".to_string(), "20ms");
                            });
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let mut gate = false;
                        ui.checkbox(&mut gate, "Gate");
                        ui.add_space(8.0);
                        ComboBox::from_id_salt("gate_threshold")
                            .selected_text("-45 dB")
                            .width(70.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut "-45 dB".to_string(), "-45 dB".to_string(), "-45 dB");
                                ui.selectable_value(&mut "-30 dB".to_string(), "-30 dB".to_string(), "-30 dB");
                                ui.selectable_value(&mut "-60 dB".to_string(), "-60 dB".to_string(), "-60 dB");
                            });
                        ui.add_space(8.0);
                        ComboBox::from_id_salt("gate_hold")
                            .selected_text("50ms")
                            .width(60.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut "25ms".to_string(), "25ms".to_string(), "25ms");
                                ui.selectable_value(&mut "50ms".to_string(), "50ms".to_string(), "50ms");
                                ui.selectable_value(&mut "100ms".to_string(), "100ms".to_string(), "100ms");
                            });
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let mut aec = true;
                        ui.checkbox(&mut aec, "AEC");
                        ui.add_space(8.0);
                        ComboBox::from_id_salt("aec_strength")
                            .selected_text("Medium")
                            .width(80.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut "Off".to_string(), "Off".to_string(), "Off");
                                ui.selectable_value(&mut "Low".to_string(), "Low".to_string(), "Low");
                                ui.selectable_value(&mut "Medium".to_string(), "Medium".to_string(), "Medium");
                                ui.selectable_value(&mut "High".to_string(), "High".to_string(), "High");
                            });
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let mut compressor = false;
                        ui.checkbox(&mut compressor, "Compressor");
                        ui.add_space(8.0);
                        ComboBox::from_id_salt("comp_ratio")
                            .selected_text("3:1")
                            .width(60.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut "2:1".to_string(), "2:1".to_string(), "2:1");
                                ui.selectable_value(&mut "3:1".to_string(), "3:1".to_string(), "3:1");
                                ui.selectable_value(&mut "4:1".to_string(), "4:1".to_string(), "4:1");
                            });
                        ui.add_space(8.0);
                        ComboBox::from_id_salt("comp_threshold")
                            .selected_text("-20 dB")
                            .width(70.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut "-12 dB".to_string(), "-12 dB".to_string(), "-12 dB");
                                ui.selectable_value(&mut "-20 dB".to_string(), "-20 dB".to_string(), "-20 dB");
                                ui.selectable_value(&mut "-30 dB".to_string(), "-30 dB".to_string(), "-30 dB");
                            });
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

                // MONITORING section
                CollapsibleSection::new("MONITORING").show(ui, |ui| {
                    ui.label(egui::RichText::new("Monitor Level").small().color(crate::theme::text_secondary()));
                    let mut monitor_level = 0.0_f32;
                    ui.add(
                        egui::Slider::new(&mut monitor_level, -20.0..=12.0)
                            .show_value(true)
                            .custom_formatter(|v, _| format!("{v:.1} dB")),
                    );

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let mut muted = false;
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("🔇 Mute").small())
                                    .fill(if muted { crate::theme::accent() } else { crate::theme::bg_surface_sunken() }),
                            )
                            .clicked()
                        {
                            // TODO: Toggle mute
                        }
                        ui.add_space(8.0);
                        let mut solo = false;
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("🎧 Solo").small())
                                    .fill(if solo { crate::theme::accent() } else { crate::theme::bg_surface_sunken() }),
                            )
                            .clicked()
                        {
                            // TODO: Toggle solo
                        }
                        ui.add_space(8.0);
                        if ui.small_button("🎵 Test Tone 1kHz").clicked() {
                            // TODO: Play test tone
                        }
                        ui.add_space(8.0);
                        ComboBox::from_id_salt("monitor_output")
                            .selected_text("Default")
                            .width(100.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut "Default".to_string(), "Default".to_string(), "Default");
                                ui.selectable_value(&mut "Headphones".to_string(), "Headphones".to_string(), "Headphones");
                            });
                    });
                });

                // RECORDING section
                CollapsibleSection::new("RECORDING").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let mut recording = false;
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("● Record").small())
                                    .fill(if recording { STATUS_ERROR } else { crate::theme::bg_surface_sunken() }),
                            )
                            .clicked()
                        {
                            // TODO: Toggle recording
                        }
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("00:00:00").monospace().color(crate::theme::text_secondary()));
                        ui.add_space(16.0);
                        let mut auto_record = false;
                        ui.checkbox(&mut auto_record, "Auto-record on start");
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Format:").small().color(crate::theme::text_secondary()));
                        ComboBox::from_id_salt("recording_format")
                            .selected_text("WAV 24-bit")
                            .width(100.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut "WAV 16-bit".to_string(), "WAV 16-bit".to_string(), "WAV 16-bit");
                                ui.selectable_value(&mut "WAV 24-bit".to_string(), "WAV 24-bit".to_string(), "WAV 24-bit");
                                ui.selectable_value(&mut "FLAC".to_string(), "FLAC".to_string(), "FLAC");
                            });
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Path: ~/sessions/sermon-001.wav").small().color(crate::theme::text_secondary()));
                        ui.add_space(8.0);
                        if ui.small_button("📁 Open Folder").clicked() {
                            // TODO: Open folder
                        }
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Disk: 412 GB free").small().color(crate::theme::text_secondary()));
                    });
                });

                // PRESET section
                CollapsibleSection::new("PRESET").show(ui, |ui| {
                    ComboBox::from_id_salt("preset_select")
                        .selected_text("Church Service")
                        .width(ui.available_width())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut "Church Service".to_string(), "Church Service".to_string(), "Church Service");
                            ui.selectable_value(&mut "Podcast".to_string(), "Podcast".to_string(), "Podcast");
                            ui.selectable_value(&mut "Conference".to_string(), "Conference".to_string(), "Conference");
                        });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui.small_button("💾 Save").clicked() {
                            // TODO: Save preset
                        }
                        ui.add_space(8.0);
                        if ui.small_button("💾 Save As…").clicked() {
                            // TODO: Save preset as
                        }
                        ui.add_space(8.0);
                        if ui.small_button("📋 Manage Presets…").clicked() {
                            // TODO: Open preset manager
                        }
                        ui.add_space(8.0);
                        if ui.small_button("↺ Reset").clicked() {
                            // TODO: Reset to defaults
                        }
                    });
                });

                // LED STATUS section
                CollapsibleSection::new("LED STATUS").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let status = match app.pipeline_state {
                            hv_pipeline::PipelineState::Stopped => Status::Error,
                            hv_pipeline::PipelineState::Listening | hv_pipeline::PipelineState::Silence => Status::Warning,
                            hv_pipeline::PipelineState::Speaking => Status::Success,
                        };
                        StatusPill::new(status, "").blink(app.led_blink).show(ui);
                        ui.label(
                            egui::RichText::new(app.status_label())
                                .strong()
                                .color(app.led_color),
                        );
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let mut hardware_led = true;
                        let mut screen_led = true;
                        let mut brightness = 80_f32;
                        ui.checkbox(&mut hardware_led, "Hardware LED: ON");
                        ui.add_space(8.0);
                        ui.checkbox(&mut screen_led, "Screen LED: ON");
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Brightness:").small().color(crate::theme::text_secondary()));
                        ui.add(egui::Slider::new(&mut brightness, 0.0..=100.0).show_value(true));
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui.small_button("🔆 Test LEDs").clicked() {
                            // TODO: Test LEDs
                        }
                        ui.add_space(8.0);
                        if ui.small_button("🎨 Configure Patterns…").clicked() {
                            // TODO: Open LED configuration
                        }
                    });
                });

                // DIAGNOSTICS section
                CollapsibleSection::new("DIAGNOSTICS").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("CPU 28%").small().color(crate::theme::text_secondary()));
                        ui.separator();
                        ui.label(egui::RichText::new("Audio Thread Jitter 0.4ms").small().color(crate::theme::text_secondary()));
                        ui.separator();
                        ui.label(egui::RichText::new("Dropped Frames 0").small().color(crate::theme::text_secondary()));
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Last Buffer 5.4ms").small().color(crate::theme::text_secondary()));
                        ui.separator();
                        ui.label(egui::RichText::new("ASR Queue 0").small().color(crate::theme::text_secondary()));
                        ui.separator();
                        ui.label(egui::RichText::new("Uptime 00:42:18").small().color(crate::theme::text_secondary()));
                    });

                    ui.add_space(4.0);
                    if ui.small_button("📊 Open Performance Monitor").clicked() {
                        // TODO: Open performance monitor
                    }
                });

                // Spacer before pinned bottom bar
                ui.add_space(16.0);
            });
        });
                });
        });
}