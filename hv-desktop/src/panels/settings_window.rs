//! Professional settings window — separate OS viewport.
//! Layout: egui SidePanel (nav) + CentralPanel (content). OBS/Cursor style.

use eframe::egui::{self, Context, Frame, Margin, RichText, Sense, Stroke, Ui, Vec2};
use crate::app::HvBibleApp;
use crate::theme::{self, STATUS_SUCCESS, STATUS_WARNING};

struct NavEntry {
    icon:  &'static str,
    label: &'static str,
}

const NAV: &[NavEntry] = &[
    NavEntry { icon: "🖥",  label: "General"     },
    NavEntry { icon: "🎙",  label: "Audio"       },
    NavEntry { icon: "🤖",  label: "Recognition" },
    NavEntry { icon: "🎛",  label: "Processing"  },
    NavEntry { icon: "📖",  label: "Bible"       },
    NavEntry { icon: "⌨",  label: "Shortcuts"   },
    NavEntry { icon: "ℹ",  label: "About"       },
];

// ─── Entry point ──────────────────────────────────────────────────────────────

pub fn show(ctx: &Context, app: &mut HvBibleApp) {
    if !app.config.layout_state.settings_open {
        return;
    }

    let mut is_open = true;

    let vp_id = egui::ViewportId::from_hash_of("hv_settings");
    let builder = egui::ViewportBuilder::default()
        .with_title("Settings — HV-Bible Broadcast Engine")
        .with_inner_size([860.0, 600.0])
        .with_min_inner_size([720.0, 480.0])
        .with_decorations(true)
        .with_resizable(true);

    ctx.show_viewport_immediate(vp_id, builder, |ctx, _class| {
        if ctx.input(|i| i.viewport().close_requested()) {
            is_open = false;
        }
        theme::apply_visuals(ctx, app.dark_mode);

        // ── Left nav panel — fixed width, full height ──────────────────────
        egui::SidePanel::left("settings_nav")
            .exact_width(190.0)
            .resizable(false)
            .frame(Frame::none().fill(theme::bg_surface_sunken()))
            .show(ctx, |ui| {
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.label(
                        RichText::new("SETTINGS")
                            .size(9.5)
                            .color(theme::text_tertiary())
                            .strong(),
                    );
                });
                ui.add_space(12.0);

                for (idx, entry) in NAV.iter().enumerate() {
                    let active = app.config.layout_state.settings_tab == idx;
                    if nav_item(ui, entry.icon, entry.label, active) {
                        app.config.layout_state.settings_tab = idx;
                    }
                }

                // version string pinned to bottom
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.label(
                            RichText::new("v0.1.0 Beta")
                                .size(10.0)
                                .color(theme::text_tertiary()),
                        );
                    });
                });
            });

        // ── Main content area ──────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(Frame::none().fill(theme::bg_base()))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("settings_scroll")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let margin = Margin { left: 40.0, right: 40.0, top: 32.0, bottom: 40.0 };
                        Frame::none().inner_margin(margin).show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            match app.config.layout_state.settings_tab {
                                0 => page_general(ui, app),
                                1 => page_audio(ui, app),
                                2 => page_recognition(ui, app),
                                3 => page_processing(ui, app),
                                4 => page_bible(ui, app),
                                5 => page_shortcuts(ui),
                                6 => page_about(ui),
                                _ => {}
                            }
                        });
                    });
            });
    });

    app.config.layout_state.settings_open = is_open;
}

// ─── Nav item — returns true if clicked ──────────────────────────────────────

fn nav_item(ui: &mut Ui, icon: &str, label: &str, active: bool) -> bool {
    let accent = theme::accent();
    let bg     = if active { theme::accent_muted() } else { egui::Color32::TRANSPARENT };
    let fg     = if active { accent }               else { theme::text_secondary() };

    let full_w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(full_w, 38.0), Sense::click());

    if ui.is_rect_visible(rect) {
        // background fill (hover or active)
        let bg_draw = if resp.hovered() && !active { theme::bg_surface_raised() } else { bg };
        ui.painter().rect_filled(rect, 0.0, bg_draw);

        // left accent bar when active
        if active {
            let bar = egui::Rect::from_min_size(rect.min, Vec2::new(3.0, rect.height()));
            ui.painter().rect_filled(bar, 0.0, accent);
        }

        // icon + label centred vertically
        let cy = rect.center().y;
        ui.painter().text(
            egui::pos2(rect.left() + 18.0, cy),
            egui::Align2::LEFT_CENTER,
            icon,
            egui::FontId::proportional(15.0),
            fg,
        );
        ui.painter().text(
            egui::pos2(rect.left() + 44.0, cy),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(13.0),
            fg,
        );
    }

    resp.clicked()
}

// ─── Layout helpers ───────────────────────────────────────────────────────────

fn page_title(ui: &mut Ui, icon: &str, title: &str, subtitle: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(icon).size(26.0));
        ui.add_space(10.0);
        ui.vertical(|ui| {
            ui.add_space(2.0);
            ui.label(RichText::new(title).size(19.0).strong().color(theme::text_primary()));
            ui.label(RichText::new(subtitle).size(12.0).color(theme::text_tertiary()));
        });
    });
    ui.add_space(18.0);
    // full-width rule under title
    let y = ui.cursor().top();
    ui.painter().line_segment(
        [egui::pos2(ui.min_rect().left(), y), egui::pos2(ui.min_rect().right(), y)],
        Stroke::new(1.0, theme::border_subtle()),
    );
    ui.add_space(22.0);
}

fn section_header(ui: &mut Ui, title: &str) {
    ui.add_space(6.0);
    ui.label(RichText::new(title).size(10.5).color(theme::text_tertiary()).strong());
    let y = ui.cursor().top() + 2.0;
    ui.painter().line_segment(
        [egui::pos2(ui.min_rect().left(), y), egui::pos2(ui.min_rect().right(), y)],
        Stroke::new(1.0, theme::border_subtle()),
    );
    ui.add_space(12.0);
}

/// Two-column row: 220 px label+hint column left, widget column right.
fn setting_row(ui: &mut Ui, label: &str, hint: &str, add_widget: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_min_width(220.0);
            ui.set_max_width(220.0);
            ui.label(RichText::new(label).size(13.0).color(theme::text_primary()));
            if !hint.is_empty() {
                ui.add_space(1.0);
                ui.label(RichText::new(hint).size(11.0).color(theme::text_tertiary()));
            }
        });
        ui.add_space(24.0);
        ui.vertical(|ui| {
            add_widget(ui);
        });
    });
    ui.add_space(16.0);
}

// ─── Pages ───────────────────────────────────────────────────────────────────

fn page_general(ui: &mut Ui, app: &mut HvBibleApp) {
    page_title(ui, "🖥", "General", "Appearance and interface preferences");

    section_header(ui, "APPEARANCE");

    setting_row(ui, "Color Mode", "Switch between dark and light interface theme", |ui| {
        ui.horizontal(|ui| {
            if radio_btn(ui, "Dark", app.dark_mode) {
                app.dark_mode = true;
                app.config.dark_mode = true;
                app.current_theme.mode = crate::theme::ThemeMode::Dark;
                let _ = app.config.save();
            }
            ui.add_space(8.0);
            if radio_btn(ui, "Light", !app.dark_mode) {
                app.dark_mode = false;
                app.config.dark_mode = false;
                app.current_theme.mode = crate::theme::ThemeMode::Light;
                let _ = app.config.save();
            }
        });
    });

    setting_row(ui, "Theme Profile", "Select a professional color palette", |ui| {
        egui::ComboBox::from_id_salt("theme_selector")
            .selected_text(&app.current_theme.id)
            .width(220.0)
            .show_ui(ui, |ui| {
                let themes = app.theme_registry.list_all();
                for theme_def in themes {
                    let is_sel = theme_def.id == app.current_theme.id;
                    if ui.selectable_label(is_sel, &theme_def.name).clicked() && !is_sel {
                        app.current_theme.id = theme_def.id.clone();
                        app.config.theme_id = theme_def.id.clone();
                        let _ = app.config.save();
                    }
                }
            });
    });

    ui.add_space(4.0);
    section_header(ui, "TYPOGRAPHY");

    setting_row(ui, "Font Family", "Primary interface typeface", |ui| {
        let mut selected_family = format!("{}", app.current_theme.font_config.family);
        egui::ComboBox::from_id_salt("font_family")
            .selected_text(&selected_family)
            .width(220.0)
            .show_ui(ui, |ui| {
                for family in ["Inter", "JetBrains Mono", "Fira Code", "Roboto", "Open Sans", "System"] {
                    if ui.selectable_value(&mut selected_family, family.to_string(), family).changed() {
                        let new_family = match family {
                            "Inter" => crate::theme::FontFamily::Inter,
                            "JetBrains Mono" => crate::theme::FontFamily::JetBrainsMono,
                            "Fira Code" => crate::theme::FontFamily::FiraCode,
                            "Roboto" => crate::theme::FontFamily::Roboto,
                            "Open Sans" => crate::theme::FontFamily::OpenSans,
                            "System" => crate::theme::FontFamily::System,
                            _ => crate::theme::FontFamily::Inter,
                        };
                        app.current_theme.font_config.family = new_family;
                        app.config.font_config = Some(app.current_theme.font_config.clone());
                        let _ = app.config.save();
                    }
                }
            });
    });

    setting_row(ui, "Font Style", "Weight and slant", |ui| {
        let mut selected_style = format!("{}", app.current_theme.font_config.style);
        egui::ComboBox::from_id_salt("font_style")
            .selected_text(&selected_style)
            .width(120.0)
            .show_ui(ui, |ui| {
                for style in ["Normal", "Italic", "Bold", "Bold Italic"] {
                    if ui.selectable_value(&mut selected_style, style.to_string(), style).changed() {
                        let new_style = match style {
                            "Normal" => crate::theme::FontStyle::Normal,
                            "Italic" => crate::theme::FontStyle::Italic,
                            "Bold" => crate::theme::FontStyle::Bold,
                            "Bold Italic" => crate::theme::FontStyle::BoldItalic,
                            _ => crate::theme::FontStyle::Normal,
                        };
                        app.current_theme.font_config.style = new_style;
                        app.config.font_config = Some(app.current_theme.font_config.clone());
                        let _ = app.config.save();
                    }
                }
            });
    });

    setting_row(ui, "UI Scale / Zoom", "Scale all text — shortcuts: Ctrl+= / Ctrl+- / Ctrl+0", |ui| {
        ui.horizontal(|ui| {
            let mut pct = (app.current_theme.font_config.ui_scale * 100.0).round() as i32;
            let changed = ui.add(
                egui::Slider::new(&mut pct, 50..=200)
                    .suffix("%")
                    .clamping(egui::SliderClamping::Always),
            ).changed();
            if changed {
                app.current_theme.font_config.ui_scale = (pct as f32 / 100.0 * 10.0).round() / 10.0;
                app.config.font_config = Some(app.current_theme.font_config.clone());
                let _ = app.config.save();
            }
            if ui.small_button("↺  100%").on_hover_text("Reset zoom to 100%").clicked() {
                app.current_theme.font_config.ui_scale = 1.0;
                app.config.font_config = Some(app.current_theme.font_config.clone());
                let _ = app.config.save();
            }
            let effective_px = (app.current_theme.font_config.size.to_pixels()
                * app.current_theme.font_config.ui_scale).round() as i32;
            ui.add_space(6.0);
            ui.label(
                RichText::new(format!("→ {}px effective", effective_px))
                    .size(11.0)
                    .color(theme::text_tertiary()),
            );
        });
    });

    setting_row(ui, "Base Font Size", "Foundation size before zoom is applied", |ui| {
        let mut selected_size = format!("{}", app.current_theme.font_config.size);
        egui::ComboBox::from_id_salt("font_size")
            .selected_text(&selected_size)
            .width(120.0)
            .show_ui(ui, |ui| {
                for size in ["10px", "12px", "14px", "16px", "18px", "20px", "24px"] {
                    if ui.selectable_value(&mut selected_size, size.to_string(), size).changed() {
                        let new_size = match size {
                            "10px" => crate::theme::FontSize::XSmall,
                            "12px" => crate::theme::FontSize::Small,
                            "14px" => crate::theme::FontSize::Medium,
                            "16px" => crate::theme::FontSize::Large,
                            "18px" => crate::theme::FontSize::XLarge,
                            "20px" => crate::theme::FontSize::XXLarge,
                            "24px" => crate::theme::FontSize::XXXLarge,
                            _ => crate::theme::FontSize::Medium,
                        };
                        app.current_theme.font_config.size = new_size;
                        app.config.font_config = Some(app.current_theme.font_config.clone());
                        let _ = app.config.save();
                    }
                }
            });
    });

    ui.add_space(4.0);
    section_header(ui, "STARTUP");

    setting_row(ui, "Auto-start Pipeline", "Automatically begin listening when the app opens", |ui| {
        let mut dummy = false;
        ui.add(toggle(&mut dummy));
    });

    setting_row(ui, "Restore Window Geometry", "Remember window size and position between sessions", |ui| {
        let mut dummy = true;
        ui.add(toggle(&mut dummy));
    });
}

fn page_audio(ui: &mut Ui, app: &mut HvBibleApp) {
    page_title(ui, "🎙", "Audio", "Capture device, levels, and live monitoring");

    section_header(ui, "INPUT DEVICE");

    setting_row(ui, "Microphone / Input", "System audio input source for capture", |ui| {
        egui::ComboBox::from_id_salt("s_audio_dev")
            .selected_text(&app.selected_device)
            .width(300.0)
            .show_ui(ui, |ui| {
                for dev in app.devices.clone() {
                    if ui.selectable_value(&mut app.selected_device, dev.clone(), &dev).clicked() {
                        app.restart_required = true;
                        app.pipeline.send_command(hv_pipeline::PipelineCommand::SetDevice(dev));
                    }
                }
            });
    });

    if app.restart_required {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.add_space(244.0);
            ui.colored_label(STATUS_WARNING, "⚠ Pipeline restart recommended to apply this change");
        });
        ui.add_space(6.0);
    }

    section_header(ui, "LEVELS");

    setting_row(ui, "Input Gain", "Scale the amplitude of the captured audio signal", |ui| {
        // Speak decibels, matching the audio console and the deck. Writing the
        // raw linear multiplier here would let this dialog disagree with every
        // other surface about what the gain actually is.
        ui.horizontal(|ui| {
            let mut gain_db = app.gain_db();
            let response = ui.add(
                egui::Slider::new(&mut gain_db, -60.0..=24.0)
                    .step_by(0.5)
                    .fixed_decimals(1)
                    .suffix(" dB"),
            );
            if response.changed() {
                app.set_gain_db(gain_db);
            }
            if ui
                .small_button("↺")
                .on_hover_text("Reset to 0.0 dB (unity)")
                .clicked()
            {
                app.set_gain_db(0.0);
            }
        });
    });

    section_header(ui, "LIVE LEVELS");
    ui.add_space(4.0);
    meter_bar(ui, "RMS ", app.meter_rms);
    ui.add_space(6.0);
    meter_bar(ui, "Peak", app.meter_peak);
}

fn meter_bar(ui: &mut Ui, label: &str, db: f32) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(11.0).color(theme::text_tertiary()).monospace());
        ui.add_space(8.0);
        let t   = ((db + 48.0) / 48.0).clamp(0.0, 1.0);
        let col = if db > -6.0 { theme::STATUS_ERROR }
                  else if db > -18.0 { STATUS_WARNING }
                  else { STATUS_SUCCESS };
        let (rect, _) = ui.allocate_exact_size(Vec2::new(220.0, 8.0), Sense::hover());
        ui.painter().rect_filled(rect, 2.0, theme::bg_surface_raised());
        let fill = egui::Rect::from_min_size(rect.min, Vec2::new(rect.width() * t, rect.height()));
        ui.painter().rect_filled(fill, 2.0, col);
        ui.add_space(10.0);
        ui.label(
            RichText::new(format!("{:.0} dB", db))
                .size(11.0)
                .color(theme::text_secondary())
                .monospace(),
        );
    });
}

fn page_recognition(ui: &mut Ui, app: &mut HvBibleApp) {
    page_title(ui, "🤖", "Recognition", "ASR engine, model selection, and inference tuning");

    section_header(ui, "MODEL");

    let model_label = app.available_models.iter()
        .find(|m| m.id == app.selected_model_id)
        .map(|m| format!("{} [{}]", m.id, m.language))
        .unwrap_or_else(|| "— none selected —".to_string());

    setting_row(ui, "Active Model", "The ASR model loaded for transcription", |ui| {
        egui::ComboBox::from_id_salt("s_asr_model")
            .selected_text(&model_label)
            .width(300.0)
            .show_ui(ui, |ui| {
                for m in app.available_models.clone() {
                    let lbl = format!("{} — {} ({} MB)", m.display_name, m.language, m.size_mb);
                    let is_sel = m.id == app.selected_model_id;
                    if ui.add_enabled(m.available, egui::SelectableLabel::new(is_sel, lbl)).clicked()
                        && !is_sel
                    {
                        app.change_model(m.id.clone());
                    }
                }
            });
    });

    if app.restart_required {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.add_space(244.0);
            ui.colored_label(STATUS_WARNING, "⚠ Restart required to load the new model");
        });
        ui.add_space(6.0);
    }

    section_header(ui, "INFERENCE");

    setting_row(ui, "Beam Size", "Wider search = more accurate but slower (1–10)", |ui| {
        ui.add(egui::Slider::new(&mut app.config.beam_size, 1..=10).clamping(egui::SliderClamping::Always));
    });

    setting_row(ui, "GPU Acceleration", "Use CUDA / DirectML for inference — requires restart", |ui| {
        let changed = ui.add(toggle(&mut app.config.use_gpu)).changed();
        if changed { app.restart_required = true; }
        if app.config.use_gpu {
            ui.add_space(8.0);
            ui.label(RichText::new("ENABLED").size(10.0).color(STATUS_SUCCESS).strong());
        }
    });

    setting_row(ui, "Worker Threads", "CPU threads for inference (0 = auto-detect)", |ui| {
        ui.add(egui::Slider::new(&mut app.config.num_threads, 0..=16));
    });

    section_header(ui, "ENGINE INFO");

    let backend = match app.asr_info {
        crate::pipeline_integration::AsrEngineInfo::Mock      => "Mock Engine (no real audio)",
        crate::pipeline_integration::AsrEngineInfo::SherpaCpu => "Sherpa-ONNX Zipformer  (CPU)",
        crate::pipeline_integration::AsrEngineInfo::SherpaGpu => "Sherpa-ONNX Zipformer  (GPU)",
    };

    egui::Grid::new("eng_info").num_columns(2).spacing([28.0, 6.0]).show(ui, |ui| {
        kv(ui, "Backend", backend);
        if let Some(m) = app.available_models.iter().find(|m| m.id == app.selected_model_id) {
            kv(ui, "Model ID",  &m.id);
            kv(ui, "Language",  &m.language);
            kv(ui, "Streaming", if m.is_streaming { "Yes" } else { "No" });
            kv(ui, "Disk Size", &format!("{} MB", m.size_mb));
        }
    });
}

fn page_processing(ui: &mut Ui, app: &mut HvBibleApp) {
    page_title(ui, "🎛", "Processing", "Real-time DSP chain applied to audio before recognition");

    section_header(ui, "NOISE GATE");

    setting_row(ui, "Noise Gate", "Silence audio below a set amplitude level", |ui| {
        if ui.add(toggle(&mut app.config.noise_gate_enabled)).changed() {
            app.pipeline.send_command(
                hv_pipeline::PipelineCommand::EnableNoiseGate(app.config.noise_gate_enabled),
            );
        }
    });

    if app.config.noise_gate_enabled {
        setting_row(ui, "Threshold", "Amplitude below which audio is silenced", |ui| {
            if ui.add(
                egui::Slider::new(&mut app.config.noise_gate_threshold, 0.0001..=0.05)
                    .logarithmic(true)
                    .fixed_decimals(4),
            ).changed() {
                app.pipeline.send_command(
                    hv_pipeline::PipelineCommand::SetNoiseGateThreshold(app.config.noise_gate_threshold),
                );
            }
        });
    }

    section_header(ui, "HIGH-PASS FILTER");

    setting_row(ui, "High-Pass Filter", "Remove low-frequency rumble (HVAC, mic handling)", |ui| {
        if ui.add(toggle(&mut app.config.hpf_enabled)).changed() {
            app.pipeline.send_command(
                hv_pipeline::PipelineCommand::EnableHighPass(app.config.hpf_enabled),
            );
        }
    });

    if app.config.hpf_enabled {
        setting_row(ui, "Cutoff Frequency", "Attenuate all audio below this frequency", |ui| {
            if ui.add(
                egui::Slider::new(&mut app.config.hpf_frequency, 20.0..=300.0)
                    .suffix(" Hz")
                    .fixed_decimals(0),
            ).changed() {
                app.pipeline.send_command(
                    hv_pipeline::PipelineCommand::SetHighPassFreq(app.config.hpf_frequency),
                );
            }
        });
    }

    section_header(ui, "DYNAMIC COMPRESSOR");

    setting_row(ui, "Compressor", "Level out loud and quiet passages for better recognition", |ui| {
        if ui.add(toggle(&mut app.config.compressor_enabled)).changed() {
            app.pipeline.send_command(
                hv_pipeline::PipelineCommand::EnableCompressor(app.config.compressor_enabled),
            );
        }
    });

    if app.config.compressor_enabled {
        setting_row(ui, "Ratio", "1:1 = bypass  ·  20:1 = hard limiter", |ui| {
            if ui.add(
                egui::Slider::new(&mut app.config.compressor_ratio, 1.0..=20.0)
                    .fixed_decimals(1)
                    .suffix(":1"),
            ).changed() {
                app.pipeline.send_command(
                    hv_pipeline::PipelineCommand::SetCompressorRatio(app.config.compressor_ratio),
                );
            }
        });
    }
}

fn page_bible(ui: &mut Ui, app: &mut HvBibleApp) {
    page_title(ui, "📖", "Bible", "Translation, detection, and vocabulary biasing");

    section_header(ui, "TRANSLATION");

    setting_row(ui, "Bible Translation", "The translation shown to the operator on detections", |ui| {
        egui::ComboBox::from_id_salt("s_translation")
            .selected_text(&app.translation)
            .width(180.0)
            .show_ui(ui, |ui| {
                for t in ["KJV", "WEB", "ASV", "ESV", "NIV", "NLT"] {
                    if ui.selectable_value(&mut app.translation, t.to_string(), t).clicked() {
                        app.pipeline.send_command(
                            hv_pipeline::PipelineCommand::SetTranslation(t.to_string()),
                        );
                    }
                }
            });
    });

    section_header(ui, "VOCABULARY BIASING");

    setting_row(
        ui,
        "Hotwords / Biasing",
        "Boost probability of Bible book names and phrases",
        |ui| {
            ui.add(toggle(&mut app.config.hotwords_enabled));
        },
    );

    ui.horizontal(|ui| {
        ui.add_space(244.0);
        ui.label(
            RichText::new(
                "Validated against model BPE token vocabulary to prevent encoding mismatches.\
                 Requires pipeline restart.",
            )
            .size(11.0)
            .color(theme::text_tertiary()),
        );
    });
}

fn page_shortcuts(ui: &mut Ui) {
    page_title(ui, "⌨", "Shortcuts", "Keyboard bindings for broadcast operator actions");

    section_header(ui, "PIPELINE");
    shortcut(ui, "Start / Stop Listening", "F5  or  Ctrl+S");
    shortcut(ui, "Pause Capture",          "F6");
    shortcut(ui, "Mute Monitor",           "F7");

    ui.add_space(8.0);
    section_header(ui, "VERSE MANAGEMENT");
    shortcut(ui, "Approve Verse",      "Space");
    shortcut(ui, "Reject / Clear",     "Escape");
    shortcut(ui, "Undo Last Push",     "Ctrl+Z");
    shortcut(ui, "Manual Verse Entry", "Ctrl+M");
    shortcut(ui, "Verse Search",       "Ctrl+F");

    ui.add_space(8.0);
    section_header(ui, "WINDOWS & NAVIGATION");
    shortcut(ui, "Program Out Window", "F11");
    shortcut(ui, "Open Settings",      "Ctrl+,");
    shortcut(ui, "Quick Slots 1–9",    "Ctrl+1  …  Ctrl+9");
    shortcut(ui, "Close Settings",     "Escape  /  Alt+F4");

    ui.add_space(8.0);
    section_header(ui, "ZOOM & DISPLAY");
    shortcut(ui, "Zoom In",            "Ctrl+=  or  Ctrl++");
    shortcut(ui, "Zoom Out",           "Ctrl+-");
    shortcut(ui, "Reset Zoom (100%)",  "Ctrl+0");
}

fn page_about(ui: &mut Ui) {
    page_title(ui, "ℹ", "About", "Version info, open-source credits, and system diagnostics");

    section_header(ui, "APPLICATION");
    egui::Grid::new("about_app").num_columns(2).spacing([32.0, 8.0]).show(ui, |ui| {
        kv(ui, "Application", "HV-Bible Broadcast Engine");
        kv(ui, "Version",     "0.1.0  (Beta)");
        kv(ui, "Build",       env!("CARGO_PKG_VERSION"));
        kv(ui, "UI Toolkit",  "egui 0.29 + eframe");
        kv(ui, "ASR Runtime", "Sherpa-ONNX / ONNX Runtime 2.0");
        kv(ui, "VAD Engine",  "Silero VAD v4");
        kv(ui, "License",     "Proprietary — All rights reserved");
    });

    ui.add_space(20.0);
    section_header(ui, "SYSTEM");
    egui::Grid::new("about_sys").num_columns(2).spacing([32.0, 8.0]).show(ui, |ui| {
        kv(ui, "OS",   std::env::consts::OS);
        kv(ui, "Arch", std::env::consts::ARCH);
    });

    ui.add_space(20.0);
    section_header(ui, "OPEN-SOURCE COMPONENTS");
    ui.label(
        RichText::new(
            "egui · eframe · cpal · ort · sherpa-onnx · crossbeam · serde · toml · tracing · bible-core",
        )
        .size(12.0)
        .color(theme::text_secondary()),
    );
}

// ─── Reusable widgets ─────────────────────────────────────────────────────────

/// iOS-style animated toggle switch.  Use with `ui.add(toggle(&mut val))`.
fn toggle(on: &mut bool) -> impl egui::Widget + '_ {
    move |ui: &mut Ui| {
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(40.0, 22.0), Sense::click());
        if resp.clicked() { *on = !*on; }

        let t      = ui.ctx().animate_bool_with_time(resp.id, *on, 0.15);
        let accent = theme::accent();
        let bg     = egui::Color32::from_rgb(
            lerp_u8(0x36, accent.r(), t),
            lerp_u8(0x30, accent.g(), t),
            lerp_u8(0x46, accent.b(), t),
        );
        let r = rect.height() / 2.0;
        ui.painter().rect_filled(rect, r, bg);
        let cx = rect.left() + r + t * (rect.width() - r * 2.0);
        ui.painter().circle_filled(egui::pos2(cx, rect.center().y), r - 2.0, egui::Color32::WHITE);
        resp
    }
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t.clamp(0.0, 1.0)) as u8
}

/// Filled pill-style toggle button. Returns `true` if clicked.
fn radio_btn(ui: &mut Ui, label: &str, active: bool) -> bool {
    let accent = theme::accent();
    let fill   = if active { accent }               else { theme::bg_surface_raised() };
    let fg     = if active { theme::text_inverse() } else { theme::text_secondary()  };
    ui.add(
        egui::Button::new(RichText::new(label).size(12.5).color(fg))
            .fill(fill)
            .stroke(Stroke::new(1.0, if active { accent } else { theme::border_subtle() }))
            .rounding(5.0)
            .min_size(Vec2::new(72.0, 30.0)),
    ).clicked()
}

/// Shortcut row: action on left, keybind chip on right.
fn shortcut(ui: &mut Ui, action: &str, keys: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(action).size(13.0).color(theme::text_primary()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            Frame::none()
                .fill(theme::bg_surface_raised())
                .stroke(Stroke::new(1.0, theme::border_subtle()))
                .rounding(4.0)
                .inner_margin(Margin::symmetric(8.0, 3.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(keys)
                            .size(11.5)
                            .color(theme::text_secondary())
                            .monospace(),
                    );
                });
        });
    });
    ui.add_space(6.0);
}

/// Key-value pair for Grid layouts.
fn kv(ui: &mut Ui, key: &str, val: &str) {
    ui.label(RichText::new(key).size(12.0).color(theme::text_tertiary()));
    ui.label(RichText::new(val).size(12.0).color(theme::text_primary()));
    ui.end_row();
}
