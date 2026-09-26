use eframe::egui::{self, Context, Ui};

use crate::app::HvBibleApp;
use crate::theme::{self, STATUS_SUCCESS, STATUS_WARNING, STATUS_ERROR, ThemeMode, Theme};

pub fn show(ctx: &Context, app: &mut HvBibleApp) {
    if !app.config.layout_state.settings_open {
        return;
    }

    let mut is_open = app.config.layout_state.settings_open;
    let viewport_id = egui::ViewportId::from_hash_of("settings_window");
    let builder = egui::ViewportBuilder::default()
        .with_title("Settings - HV-Bible")
        .with_inner_size([700.0, 550.0])
        .with_min_inner_size([500.0, 400.0])
        .with_decorations(true)
        .with_resizable(true);

    ctx.show_viewport_immediate(viewport_id, builder, |ctx, _class| {
        if ctx.input(|i| i.viewport().close_requested()) {
            is_open = false;
        }

        // We must apply the current theme to the new viewport context
        theme::apply_visuals(ctx, app.dark_mode);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Left sidebar for tabs
                ui.vertical(|ui| {
                    ui.set_width(140.0);
                    ui.add_space(8.0);
                    
                    let tabs = [
                        "General", "Audio", "ASR", "DSP", "Bible", "Hotkeys", "About",
                    ];
                    
                    for (i, &name) in tabs.iter().enumerate() {
                        let is_active = app.config.layout_state.settings_tab == i;
                        if ui.add(
                            egui::Button::new(name)
                                .fill(if is_active { crate::theme::accent() } else { crate::theme::bg_surface_sunken() })
                                .frame(false)
                        ).clicked() {
                            app.config.layout_state.settings_tab = i;
                        }
                        ui.add_space(4.0);
                    }
                });

                ui.separator();

                // Right content area
                egui::ScrollArea::vertical()
                    .id_salt("settings_scroll")
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.add_space(8.0);
                        
                        match app.config.layout_state.settings_tab {
                            0 => show_general(ui, app),
                            1 => show_audio(ui, app),
                            2 => show_asr(ui, app),
                            3 => show_dsp(ui, app),
                            4 => show_bible(ui, app),
                            5 => show_hotkeys(ui, app),
                            6 => show_about(ui),
                            _ => {}
                        }
                    });
            });
        });
    });

    app.config.layout_state.settings_open = is_open;
}

fn show_general(ui: &mut Ui, app: &mut HvBibleApp) {
    ui.heading("General Settings");
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.label("Theme:");
        let mut is_dark = app.dark_mode;
        if ui.radio_value(&mut is_dark, true, "Dark").clicked() {
            app.dark_mode = true;
        }
        if ui.radio_value(&mut is_dark, false, "Light").clicked() {
            app.dark_mode = false;
        }
    });

    ui.add_space(16.0);
    ui.heading("Premium Themes");
    ui.add_space(8.0);

    ui.label("Select from 12+ premium themes");
    ui.add_space(4.0);

    // Theme category filter
    let mut selected_category = "All".to_string();
    ui.horizontal(|ui| {
        ui.label("Category:");
        let categories = [
            "All", "Professional", "Elegant", "Vibrant", "Minimal", "Broadcast", "Nature", "Retro", "Cyberpunk",
        ];
        egui::ComboBox::from_id_salt("theme_category")
            .selected_text(&selected_category)
            .width(150.0)
            .show_ui(ui, |ui| {
                for category in categories {
                    if ui.selectable_value(&mut selected_category, category.to_string(), category).changed() {
                        // Category changed
                    }
                }
            });
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    // Theme selection grid
    let all_themes = app.theme_registry.list_all();
    let filtered_themes: Vec<_> = if selected_category == "All" {
        all_themes
    } else {
        all_themes
            .into_iter()
            .filter(|t| t.category.to_string() == selected_category)
            .collect()
    };

    let columns = 3;
    let row_height = 80.0;
    
    // Store selected theme for apply button
    let mut selected_theme_id: Option<String> = None;
    
    egui::Grid::new("theme_grid")
        .num_columns(columns)
        .spacing([16.0, 16.0])
        .show(ui, |ui| {
            for theme in filtered_themes {
                let is_selected = app.current_theme.id == theme.id;
                let is_premium = theme.is_premium;
                
                ui.vertical(|ui| {
                    // Theme preview box
                    let frame = egui::Frame::none()
                        .fill(if is_selected {
                            crate::theme::accent()
                        } else {
                            crate::theme::bg_surface_raised()
                        })
                        .stroke(if is_selected {
                            egui::Stroke::new(2.0, crate::theme::accent())
                        } else {
                            egui::Stroke::new(1.0, crate::theme::border_subtle())
                        })
                        .rounding(4.0);
                    
                    frame.show(ui, |ui| {
                        ui.set_min_size(egui::vec2(180.0, row_height));
                        ui.vertical_centered(|ui| {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&theme.name).strong().size(14.0));
                                if is_premium {
                                    ui.label(egui::RichText::new("⭐").color(crate::theme::accent()));
                                }
                            });
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(&theme.description).small().color(crate::theme::text_secondary()));
                            ui.add_space(4.0);
                            
                            // Color preview
                            ui.horizontal(|ui| {
                                ui.label("Colors:");
                                let dark_preview = theme.dark_colors.to_color32();
                                let preview = egui::Frame::none()
                                    .fill(dark_preview.bg_base)
                                    .stroke(egui::Stroke::new(1.0, dark_preview.accent))
                                    .rounding(2.0);
                                preview.show(ui, |ui| {
                                    ui.set_min_size(egui::vec2(20.0, 20.0));
                                });
                                
                                let light_preview = theme.light_colors.to_color32();
                                let preview = egui::Frame::none()
                                    .fill(light_preview.bg_base)
                                    .stroke(egui::Stroke::new(1.0, light_preview.accent))
                                    .rounding(2.0);
                                preview.show(ui, |ui| {
                                    ui.set_min_size(egui::vec2(20.0, 20.0));
                                });
                            });
                        });
                    });
                    
                    if ui.add_enabled(true, egui::Button::new(if is_selected { "Selected" } else { "Select" }).small()).clicked() {
                        selected_theme_id = Some(theme.id.clone());
                    }
                });
                ui.end_row();
            }
        });

    ui.add_space(16.0);
    
    // Apply theme button
    ui.horizontal(|ui| {
        ui.add_space(100.0);
        if ui.add_enabled(selected_theme_id.is_some(), egui::Button::new("Apply Selected Theme").min_size(egui::vec2(200.0, 30.0))).clicked() {
            if let Some(theme_id) = selected_theme_id {
                let new_theme = Theme::new(theme_id.clone(), app.current_theme.mode);
                if let Some(definition) = app.theme_registry.get(&theme_id) {
                    new_theme.apply(ui.ctx(), definition);
                    app.current_theme = new_theme;
                    app.config.theme_id = theme_id.clone();
                    let _ = app.config.save();
                }
            }
        }
    });

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    ui.heading("Theme Mode");
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        let mut current_mode = match app.current_theme.mode {
            ThemeMode::Dark => "Dark".to_string(),
            ThemeMode::Light => "Light".to_string(),
            ThemeMode::FollowSystem => "Follow System".to_string(),
        };
        
        egui::ComboBox::from_id_salt("theme_mode")
            .selected_text(&current_mode)
            .width(150.0)
            .show_ui(ui, |ui| {
                if ui.selectable_value(&mut current_mode, "Dark".to_string(), "Dark").changed() {
                    app.current_theme.mode = ThemeMode::Dark;
                    app.config.theme_mode = "dark".to_string();
                    if let Some(definition) = app.theme_registry.get(&app.current_theme.id) {
                        app.current_theme.apply(ui.ctx(), definition);
                        let _ = app.config.save();
                    }
                }
                if ui.selectable_value(&mut current_mode, "Light".to_string(), "Light").changed() {
                    app.current_theme.mode = ThemeMode::Light;
                    app.config.theme_mode = "light".to_string();
                    if let Some(definition) = app.theme_registry.get(&app.current_theme.id) {
                        app.current_theme.apply(ui.ctx(), definition);
                        let _ = app.config.save();
                    }
                }
                if ui.selectable_value(&mut current_mode, "Follow System".to_string(), "Follow System").changed() {
                    app.current_theme.mode = ThemeMode::FollowSystem;
                    app.config.theme_mode = "follow_system".to_string();
                    if let Some(definition) = app.theme_registry.get(&app.current_theme.id) {
                        app.current_theme.apply(ui.ctx(), definition);
                        let _ = app.config.save();
                    }
                }
            });
    });

    ui.add_space(16.0);
    ui.heading("Font Customization");
    ui.add_space(4.0);

    ui.label("Customize fonts, sizes, and UI scaling");
    ui.add_space(8.0);

    // Font Family
    ui.horizontal(|ui| {
        ui.label("Font Family:");
        ui.add_space(8.0);
        let mut selected_family = format!("{}", app.current_theme.font_config.family);
        egui::ComboBox::from_id_salt("font_family")
            .selected_text(&selected_family)
            .width(150.0)
            .show_ui(ui, |ui| {
                for family in [
                    "Inter", "JetBrains Mono", "Fira Code", "Roboto", "Open Sans", "System"
                ] {
                    if ui.selectable_value(&mut selected_family, family.to_string(), family).changed() {
                        // Update font family
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
                        let _ = app.config.save();
                    }
                }
            });
    });

    ui.add_space(12.0);

    // Font Size
    ui.horizontal(|ui| {
        ui.label("Font Size:");
        ui.add_space(8.0);
        let mut selected_size = format!("{}", app.current_theme.font_config.size);
        egui::ComboBox::from_id_salt("font_size")
            .selected_text(&selected_size)
            .width(120.0)
            .show_ui(ui, |ui| {
                for size in ["10px", "12px", "14px", "16px", "18px", "20px", "24px"] {
                    if ui.selectable_value(&mut selected_size, size.to_string(), size).changed() {
                        // Update font size
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
                        let _ = app.config.save();
                    }
                }
            });
    });

    ui.add_space(12.0);

    // Font Style
    ui.horizontal(|ui| {
        ui.label("Font Style:");
        ui.add_space(8.0);
        let mut selected_style = format!("{}", app.current_theme.font_config.style);
        egui::ComboBox::from_id_salt("font_style")
            .selected_text(&selected_style)
            .width(120.0)
            .show_ui(ui, |ui| {
                for style in ["Normal", "Italic", "Bold", "Bold Italic"] {
                    if ui.selectable_value(&mut selected_style, style.to_string(), style).changed() {
                        // Update font style
                        let new_style = match style {
                            "Normal" => crate::theme::FontStyle::Normal,
                            "Italic" => crate::theme::FontStyle::Italic,
                            "Bold" => crate::theme::FontStyle::Bold,
                            "Bold Italic" => crate::theme::FontStyle::BoldItalic,
                            _ => crate::theme::FontStyle::Normal,
                        };
                        app.current_theme.font_config.style = new_style;
                        let _ = app.config.save();
                    }
                }
            });
    });

    ui.add_space(12.0);

    // UI Scale
    ui.horizontal(|ui| {
        ui.label("UI Scale:");
        ui.add_space(8.0);
        ui.add(egui::Slider::new(&mut app.current_theme.font_config.ui_scale, 0.75..=2.0)
            .step_by(0.05)
            .text("x"));
        if ui.button("Reset").clicked() {
            app.current_theme.font_config.ui_scale = 1.0;
            let _ = app.config.save();
        }
    });

    ui.add_space(8.0);
    ui.colored_label(STATUS_WARNING, "Font changes may require application restart to take full effect");

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    ui.heading("Theme Import/Export");
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        if ui.button("Export Current Theme").clicked() {
            // Export theme functionality
            if let Some(definition) = app.theme_registry.get(&app.current_theme.id) {
                let export = crate::theme::ThemeExport::from_definition(definition);
                let theme_path = crate::paths::data_dir().join("themes").join(format!("{}.json", app.current_theme.id));
                if let Some(parent) = theme_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                match export.save_to_file(&theme_path) {
                    Ok(_) => {
                        ui.colored_label(STATUS_SUCCESS, "Theme exported successfully!");
                    }
                    Err(e) => {
                        ui.colored_label(STATUS_ERROR, format!("Failed to export theme: {}", e));
                    }
                }
            }
        }
        ui.add_space(8.0);
        if ui.button("Import Theme").clicked() {
            // Import theme functionality - would need file dialog
            ui.label("Theme import requires file dialog implementation");
        }
    });
}

fn show_audio(ui: &mut Ui, app: &mut HvBibleApp) {
    ui.heading("Audio Capture");
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.label("Input Device:");
        egui::ComboBox::from_id_salt("audio_device_combo")
            .selected_text(&app.selected_device)
            .show_ui(ui, |ui| {
                for dev in &app.devices {
                    if ui
                        .selectable_value(&mut app.selected_device, dev.clone(), dev)
                        .clicked()
                    {
                        app.restart_required = true;
                    }
                }
            });
    });

    if app.restart_required {
        ui.add_space(8.0);
        ui.colored_label(STATUS_WARNING, "Restart audio pipeline to apply device changes.");
    }

    ui.add_space(16.0);
    ui.horizontal(|ui| {
        ui.label("Input Gain:");
        ui.add(egui::Slider::new(&mut app.gain, 0.1..=10.0).text("x"));
    });
}

fn show_asr(ui: &mut Ui, app: &mut HvBibleApp) {
    ui.heading("Speech Recognition (ASR)");
    ui.add_space(8.0);

    ui.label("ASR Model Selection");
    ui.add_space(4.0);
    
    egui::ComboBox::from_id_salt("asr_model_combo")
        .selected_text({
            app.available_models
                .iter()
                .find(|m| m.id == app.selected_model_id)
                .map(|m| m.display_name.clone())
                .unwrap_or_else(|| "Select model...".to_string())
        })
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            let mut selected_model_id = None;
            for model in &app.available_models {
                let is_selected = model.id == app.selected_model_id;
                let is_available = model.available;
                
                let label = if is_available {
                    format!("{} ({}, {} MB)", model.display_name, model.language, model.size_mb)
                } else {
                    format!("{} (Not available)", model.display_name)
                };

                if ui.add_enabled(is_available, egui::SelectableLabel::new(is_selected, label)).changed() {
                    selected_model_id = Some(model.id.clone());
                }
            }
            
            if let Some(model_id) = selected_model_id {
                app.change_model(model_id);
            }
        });

    if app.restart_required {
        ui.add_space(8.0);
        ui.colored_label(STATUS_WARNING, "⚠ Model change requires application restart to take effect.");
    }

    ui.add_space(16.0);
    
    if let Some(current_model) = app.available_models.iter().find(|m| m.id == app.selected_model_id) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Engine Type:").small().color(crate::theme::text_secondary()));
            ui.label(egui::RichText::new(format!("{:?}", current_model.engine_type)).small());
            ui.add_space(16.0);
            ui.label(egui::RichText::new("Streaming:").small().color(crate::theme::text_secondary()));
            ui.label(egui::RichText::new(if current_model.is_streaming { "Yes" } else { "No" }).small());
        });
        ui.add_space(16.0);
    }

    ui.separator();
    ui.add_space(8.0);
    
    ui.label("Advanced Settings");
    ui.add_space(4.0);
    
    ui.horizontal(|ui| {
        ui.label("Beam Size:");
        ui.add(egui::Slider::new(&mut app.config.beam_size, 1..=10));
    });

    ui.add_space(8.0);
    ui.checkbox(&mut app.config.use_gpu, "Enable GPU Acceleration (CUDA/DirectML)");
    ui.label(
        egui::RichText::new("Requires restart and compatible GPU")
            .color(crate::theme::text_tertiary())
            .small(),
    );
}

fn show_dsp(ui: &mut Ui, app: &mut HvBibleApp) {
    ui.heading("Digital Signal Processing");
    ui.add_space(8.0);

    ui.checkbox(&mut app.config.noise_gate_enabled, "Noise Gate");
    if app.config.noise_gate_enabled {
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.label("Threshold:");
            ui.add(egui::Slider::new(&mut app.config.noise_gate_threshold, 0.0001..=0.05).logarithmic(true));
        });
    }

    ui.add_space(8.0);
    ui.checkbox(&mut app.config.hpf_enabled, "High-Pass Filter");
    if app.config.hpf_enabled {
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.label("Cutoff Frequency (Hz):");
            ui.add(egui::Slider::new(&mut app.config.hpf_frequency, 20.0..=200.0));
        });
    }

    ui.add_space(8.0);
    ui.checkbox(&mut app.config.compressor_enabled, "Compressor");
    if app.config.compressor_enabled {
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.label("Ratio:");
            ui.add(egui::Slider::new(&mut app.config.compressor_ratio, 1.0..=20.0));
        });
    }
}

fn show_bible(ui: &mut Ui, app: &mut HvBibleApp) {
    ui.heading("Bible & Language");
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.label("Translation:");
        egui::ComboBox::from_id_salt("translation_combo")
            .selected_text(&app.translation)
            .show_ui(ui, |ui| {
                for t in ["KJV", "WEB", "ASV"] {
                    ui.selectable_value(&mut app.translation, t.to_string(), t);
                }
            });
    });

    ui.add_space(8.0);
    ui.checkbox(&mut app.config.hotwords_enabled, "Enable Bible Vocabulary Biasing");
    ui.label(
        egui::RichText::new("Improves recognition of Bible names (requires restart).")
            .color(theme::text_tertiary())
            .size(10.0),
    );
}

fn show_hotkeys(ui: &mut Ui, _app: &mut HvBibleApp) {
    ui.heading("Keyboard Shortcuts");
    ui.add_space(8.0);
    
    egui::Grid::new("hotkeys_grid").num_columns(2).spacing([40.0, 8.0]).show(ui, |ui| {
        ui.label("Start / Stop Audio");
        ui.label(egui::RichText::new("Ctrl + Space").strong());
        ui.end_row();

        ui.label("Accept Detection");
        ui.label(egui::RichText::new("Enter").strong());
        ui.end_row();

        ui.label("Reject / Clear");
        ui.label(egui::RichText::new("Escape").strong());
        ui.end_row();

        ui.label("Manual Verse Input");
        ui.label(egui::RichText::new("Ctrl + K").strong());
        ui.end_row();

        ui.label("Settings");
        ui.label(egui::RichText::new("Ctrl + ,").strong());
        ui.end_row();
    });
}

fn show_about(ui: &mut Ui) {
    ui.heading("About HV-Bible");
    ui.add_space(8.0);
    ui.label(egui::RichText::new("Broadcast Engine").strong());
    ui.label("Version 0.1.0 (Beta)");
    ui.add_space(16.0);
    
    ui.label("Built with egui, ONNX Runtime, and Rust.");
}
