// audio/mod.rs — HV-Bible Audio Console
// A comprehensive, production-quality audio control panel for live church sermon capture.

mod diagnostics;
mod gain;
mod input_source;
mod led;
mod levels;
mod monitoring;
mod presets;
mod processing;
mod recording;
mod routing;
mod transport;
mod vad;

use eframe::egui::{self, Color32, ScrollArea, Stroke, Ui};

use crate::app::HvBibleApp;

// ── Public entry ────────────────────────────────────────────────────────────

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        // ── Transport bar (Pinned to bottom) ──────────────────────────────────
        transport_bar(ui, app);

        // ── Divider line ──────────────────────────────────────────────────────
        ui.add(egui::Separator::default().spacing(0.0));

        // ── Scrollable body (Takes remaining space) ───────────────────────────
        ScrollArea::vertical()
            .id_salt("audio_console_v2")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.set_min_width(ui.available_width());

                    panel_header(ui, app);
            ui.add_space(4.0);

            // ── STATUS STRIP ──────────────────────────────────────────────
            status_strip(ui, app);
            ui.add_space(6.0);

            // ── INPUT SOURCE ──────────────────────────────────────────────
            section(ui, "🎙  INPUT SOURCE", true, |ui| {
                input_source::show(ui, app);
            });

            // ── CHANNEL ROUTING ───────────────────────────────────────────
            section(ui, "⇄  CHANNEL ROUTING", true, |ui| {
                routing::show(ui, app);
            });

            // ── INPUT LEVELS ──────────────────────────────────────────────
            section(ui, "📊  INPUT LEVEL", true, |ui| {
                levels::show(ui, app);
            });

            // ── GAIN STAGING ──────────────────────────────────────────────
            section(ui, "🎚  GAIN STAGING", true, |ui| {
                gain::show(ui, app);
            });

            // ── PROCESSING CHAIN ──────────────────────────────────────────
            section(ui, "⚙  PROCESSING CHAIN", true, |ui| {
                processing::show(ui, app);
            });

            // ── VOICE DETECTION ───────────────────────────────────────────
            section(ui, "🎤  VOICE DETECTION", true, |ui| {
                vad::show(ui, app);
            });

            // ── MONITORING ────────────────────────────────────────────────
            section(ui, "🎧  MONITORING", true, |ui| {
                monitoring::show(ui, app);
            });

            // ── RECORDING ─────────────────────────────────────────────────
            section(ui, "⏺  RECORDING", true, |ui| {
                recording::show(ui, app);
            });

            // ── PRESETS ───────────────────────────────────────────────────
            section(ui, "💾  PRESETS", false, |ui| {
                presets::show(ui, app);
            });

            // ── LED STATUS ────────────────────────────────────────────────
            section(ui, "💡  LED STATUS", false, |ui| {
                led::show(ui, app);
            });

            // ── DIAGNOSTICS ───────────────────────────────────────────────
            section(ui, "🔬  DIAGNOSTICS", false, |ui| {
                diagnostics::show(ui, app);
            });

                });
            });
    });
}

// ── Panel header ────────────────────────────────────────────────────────────

fn panel_header(ui: &mut Ui, app: &mut HvBibleApp) {
    egui::Frame::none()
        .inner_margin(egui::Margin { left: 10.0, right: 10.0, top: 8.0, bottom: 4.0 })
        .fill(crate::theme::bg_surface())
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                // Title
                ui.label(
                    egui::RichText::new("AUDIO CONSOLE")
                        .size(12.0)
                        .strong()
                        .color(crate::theme::accent()),
                );
                ui.add_space(6.0);

                // Live status dot
                let dot_color = if app.is_listening {
                    crate::theme::STATUS_SUCCESS
                } else {
                    crate::theme::text_tertiary()
                };
                let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                ui.painter().circle_filled(dot_rect.center(), 4.0, dot_color);

                ui.label(
                    egui::RichText::new(app.status_label())
                        .size(11.0)
                        .color(dot_color),
                );

                // Right-side tools
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    icon_btn(ui, "⚙", "Audio settings");
                    icon_btn(ui, "📌", "Pin panel");
                    if icon_btn(ui, "—", "Collapse panel").clicked() {
                        app.config.layout_state.left_collapsed = true;
                    }
                });
            });

            ui.add_space(3.0);

            // Device row
            ui.horizontal(|ui| {
                let d = &app.audio_mock.device;
                ui.label(
                    egui::RichText::new(&d.name)
                        .size(11.0)
                        .color(crate::theme::text_primary()),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!(
                        "{} · {}kHz · {}-bit · {}smp ({:.1}ms)",
                        d.driver,
                        d.sample_rate / 1000,
                        d.bit_depth,
                        d.buffer_size,
                        d.buffer_latency_ms
                    ))
                    .size(10.0)
                    .color(crate::theme::text_secondary()),
                );
            });
        });
}

// ── Status strip ────────────────────────────────────────────────────────────

fn status_strip(ui: &mut Ui, app: &HvBibleApp) {
    egui::Frame::none()
        .inner_margin(egui::Margin::symmetric(10.0, 5.0))
        .fill(crate::theme::bg_surface_sunken())
        .stroke(Stroke::new(1.0, crate::theme::border_subtle()))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                // Live indicator
                status_chip(ui, "LIVE", app.is_listening, crate::theme::STATUS_SUCCESS);
                ui.add_space(4.0);

                // RMS
                let rms_color = meter_color(app.meter_rms);
                status_chip_colored(ui, &format!("{:.1} dB RMS", app.meter_rms), rms_color);
                ui.add_space(4.0);

                // Peak
                status_chip_colored(ui, &format!("{:.1} dB PK", app.meter_peak), crate::theme::text_secondary());
                ui.add_space(4.0);

                // Clip
                if app.is_clipping {
                    status_chip_colored(ui, "CLIP!", crate::theme::STATUS_ERROR);
                    ui.add_space(4.0);
                }

                // SNR
                status_chip_colored(
                    ui,
                    &format!("SNR {} dB", app.snr_db),
                    crate::theme::text_secondary(),
                );

                // Uptime - only show if we have enough width
                if ui.available_width() > 50.0 {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("⏱")
                            .size(10.0)
                            .color(crate::theme::text_tertiary()),
                    );
                    ui.label(
                        egui::RichText::new(&app.audio_mock.diagnostics.uptime)
                            .size(10.0)
                            .monospace()
                            .color(crate::theme::text_secondary()),
                    );
                }
            });
        });
}

fn status_chip(ui: &mut Ui, label: &str, active: bool, active_color: Color32) {
    let color = if active { active_color } else { crate::theme::text_tertiary() };
    let bg = if active {
        active_color.linear_multiply(0.15)
    } else {
        crate::theme::bg_surface_raised()
    };
    egui::Frame::none()
        .fill(bg)
        .stroke(Stroke::new(1.0, color.linear_multiply(0.4)))
        .rounding(3.0)
        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(label).size(10.0).strong().color(color));
        });
}

fn status_chip_colored(ui: &mut Ui, label: &str, color: Color32) {
    ui.label(egui::RichText::new(label).size(10.0).monospace().color(color));
}

// ── Collapsible section wrapper ─────────────────────────────────────────────

fn section(ui: &mut Ui, title: &str, default_open: bool, body: impl FnOnce(&mut Ui)) {
    let id = ui.make_persistent_id(title);
    let open = ui.data_mut(|d| *d.get_temp_mut_or(id, default_open));

    // Section header
    egui::Frame::none()
        .inner_margin(egui::Margin { left: 10.0, right: 8.0, top: 4.0, bottom: 4.0 })
        .fill(crate::theme::bg_surface_raised())
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            let resp = ui.horizontal(|ui| {
                let arrow = if open { "▾" } else { "▸" };
                ui.label(
                    egui::RichText::new(arrow)
                        .size(10.0)
                        .color(crate::theme::text_secondary()),
                );
                ui.label(
                    egui::RichText::new(title)
                        .size(11.0)
                        .strong()
                        .color(crate::theme::text_primary()),
                );
            });
            if resp.response.interact(egui::Sense::click()).clicked() {
                ui.data_mut(|d| d.insert_temp(id, !open));
            }
        });

    // Section body
    if open {
        egui::Frame::none()
            .inner_margin(egui::Margin { left: 10.0, right: 10.0, top: 6.0, bottom: 8.0 })
            .fill(crate::theme::bg_surface())
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                body(ui);
            });
    }

    // Separator
    ui.add(egui::Separator::default().spacing(0.0));
}

// ── Transport bar ────────────────────────────────────────────────────────────

fn transport_bar(ui: &mut Ui, app: &mut HvBibleApp) {
    egui::Frame::none()
        .fill(crate::theme::bg_surface_raised())
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                // Start / Pause
                let (icon, fill) = if app.is_listening {
                    ("⏸  Pause", crate::theme::accent())
                } else {
                    ("▶  Start", crate::theme::STATUS_SUCCESS)
                };
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(icon)
                                .size(12.0)
                                .strong()
                                .color(crate::theme::text_inverse()),
                        )
                        .fill(fill)
                        .min_size(egui::vec2(0.0, 30.0)),
                    )
                    .clicked()
                {
                    app.toggle_listening();
                }

                // Stop (only when active)
                if app.is_listening {
                    ui.add_space(4.0);
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("⏹  Stop")
                                    .size(12.0)
                                    .strong()
                                    .color(crate::theme::text_inverse()),
                            )
                            .fill(crate::theme::STATUS_ERROR)
                            .min_size(egui::vec2(0.0, 30.0)),
                        )
                        .clicked()
                    {
                        app.toggle_listening();
                    }
                }

                ui.add_space(8.0);

                // Mute
                let muted = app.audio_mock.monitoring.muted;
                let mute_fill = if muted {
                    crate::theme::accent_muted()
                } else {
                    crate::theme::bg_surface_sunken()
                };
                let mute_label = if muted { "🔇 Muted" } else { "🔊 Mute" };
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(mute_label)
                                .size(11.0)
                                .color(crate::theme::text_primary()),
                        )
                        .fill(mute_fill)
                        .min_size(egui::vec2(0.0, 30.0)),
                    )
                    .clicked()
                {
                    app.audio_mock.monitoring.muted = !muted;
                }

                // Right side — REC indicator + recording
                // We don't use right_to_left so it wraps properly in a horizontal_wrapped
                ui.add_space(8.0);
                let rec = app.audio_mock.recording.active;
                let rec_fill = if rec {
                    crate::theme::STATUS_ERROR
                } else {
                    crate::theme::bg_surface_sunken()
                };
                let rec_label = if rec { "⏺ Stop Rec" } else { "⏺ Record" };
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(rec_label)
                                .size(11.0)
                                .color(if rec {
                                    crate::theme::text_inverse()
                                } else {
                                    crate::theme::text_primary()
                                }),
                        )
                        .fill(rec_fill)
                        .min_size(egui::vec2(0.0, 30.0)),
                    )
                    .clicked()
                {
                    app.audio_mock.recording.active = !rec;
                }

                if rec {
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(&app.audio_mock.recording.elapsed)
                            .size(12.0)
                            .monospace()
                            .color(crate::theme::STATUS_ERROR),
                    );
                }
            });
        });
}

// ── Shared helpers ───────────────────────────────────────────────────────────

pub(super) fn icon_btn(ui: &mut Ui, icon: &str, tooltip: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(icon).size(11.0))
            .fill(Color32::TRANSPARENT)
            .min_size(egui::vec2(22.0, 22.0))
            .stroke(Stroke::NONE),
    )
    .on_hover_text(tooltip)
}

/// Row: label on left, value right-aligned, monospace.
pub(super) fn data_row(ui: &mut Ui, label: &str, value: impl Into<String>) {
    ui.horizontal(|ui| {
        ui.set_min_width(ui.available_width());
        ui.label(
            egui::RichText::new(label)
                .size(11.0)
                .color(crate::theme::text_secondary()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(value.into())
                    .size(11.0)
                    .monospace()
                    .color(crate::theme::text_primary()),
            );
        });
    });
}

pub(super) fn bar(ui: &mut Ui, label: &str, value: f32, readout: String) {
    ui.horizontal(|ui| {
        let label_width = if ui.available_width() < 250.0 { 50.0 } else { 60.0 };
        ui.add_sized(
            egui::vec2(label_width, 10.0),
            egui::Label::new(
                egui::RichText::new(label)
                    .size(11.0)
                    .color(crate::theme::text_secondary())
            ),
        );
        let bar = egui::ProgressBar::new(value.clamp(0.0, 1.0))
            .fill(crate::theme::accent())
            .show_percentage();
        // Custom readout formatting
        let text = egui::RichText::new(readout)
            .size(11.0)
            .monospace()
            .color(crate::theme::text_primary());
        
        let w = (ui.available_width() - label_width).clamp(30.0, 400.0);
        ui.add_sized(
            egui::vec2(w, 12.0),
            bar.text(text)
        );
    });
}

/// Small labeled slider in a horizontal layout.
pub(super) fn labeled_slider(ui: &mut Ui, label: &str, value: &mut f32, min: f32, max: f32, unit: &str) {
    ui.horizontal(|ui| {
        let label_width = if ui.available_width() < 250.0 { 60.0 } else { 80.0 };
        ui.add_sized(
            egui::vec2(label_width, 18.0),
            egui::Label::new(
                egui::RichText::new(label)
                    .size(11.0)
                    .color(crate::theme::text_secondary()),
            ),
        );
        let w = (ui.available_width() - 50.0).clamp(30.0, 400.0);
        ui.add_sized(
            egui::vec2(w, 18.0),
            egui::Slider::new(value, min..=max).show_value(false),
        );
        ui.label(
            egui::RichText::new(format!("{:+.1} {}", *value, unit))
                .size(11.0)
                .monospace()
                .color(crate::theme::text_primary()),
        );
    });
}

pub(super) fn small_tool(ui: &mut Ui, label: &str, tooltip: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(label).size(10.0))
            .min_size(egui::vec2(28.0, 22.0))
            .stroke(egui::Stroke::new(1.0, crate::theme::border_subtle())),
    )
    .on_hover_text(tooltip)
}

fn meter_color(db: f32) -> Color32 {
    if db > -3.0 {
        crate::theme::STATUS_ERROR
    } else if db > -12.0 {
        crate::theme::STATUS_WARNING
    } else {
        crate::theme::STATUS_SUCCESS
    }
}
