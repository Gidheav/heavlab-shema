//! Studio Toolbar — A modern, beautiful horizontal control bar replacing the old Ribbon.

use eframe::egui::{self, Context, Frame, RichText, Sense, Stroke, Vec2};

use crate::app::HvBibleApp;
use crate::theme::{STATUS_ERROR, STATUS_SUCCESS};

const TOOLBAR_H: f32 = 64.0;
const RESTORE_HANDLE_H: f32 = 4.0;
const TITLE_BAR_H: f32 = 32.0;

// ─── Persistent state ────────────────────────────────────────────────────────

#[derive(Default, Clone)]
pub struct RibbonState {
    pub active_tab: usize,
}

// ─── Entry point ─────────────────────────────────────────────────────────────

pub fn show(ctx: &Context, app: &mut HvBibleApp) {
    let sid = egui::Id::new("hvb_toolbar");
    let mut state: RibbonState = ctx.data_mut(|d| d.get_temp(sid).unwrap_or_default());

    if app.config.layout_state.ribbon_collapsed {
        show_restore_handle(ctx, app);
        return;
    }

    egui::TopBottomPanel::top("studio_toolbar")
        .exact_height(TOOLBAR_H)
        .frame(
            Frame::none()
                .fill(crate::theme::bg_surface())
                .inner_margin(egui::Margin::symmetric(12.0, 0.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                // 1. Tab Selector (Left) - Now Beautiful Icon Cards
                tab_selector(ui, &mut state.active_tab);

                ui.add_space(16.0);
                divider(ui);
                ui.add_space(16.0);

                // 2. Tab Content - Tools as Cards
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    match state.active_tab {
                        0 => tab_home(ui, app),
                        1 => tab_broadcast(ui, app),
                        2 => tab_bible(ui, app),
                        3 => tab_audio(ui, app),
                        _ => tab_tools(ui, app),
                    }
                });

                // Collapse button far right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (rect, resp) = ui.allocate_exact_size(Vec2::new(32.0, TOOLBAR_H), Sense::click());
                    if ui.is_rect_visible(rect) {
                        let color = if resp.hovered() { crate::theme::text_primary() } else { crate::theme::text_tertiary() };
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "˄",
                            egui::FontId::proportional(16.0),
                            color,
                        );
                    }
                    if resp.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text("Collapse Toolbar").clicked() {
                        app.config.layout_state.ribbon_collapsed = true;
                    }
                });
            });
        });

    // Elegant bottom border separator
    egui::TopBottomPanel::top("toolbar_border")
        .exact_height(1.0)
        .frame(Frame::none().fill(crate::theme::border_subtle()))
        .show(ctx, |_| {});

    ctx.data_mut(|d| d.insert_temp(sid, state));
}

fn show_restore_handle(ctx: &Context, app: &mut HvBibleApp) {
    let screen = ctx.input(|i| i.screen_rect());
    egui::Area::new(egui::Id::new("ribbon_restore_handle"))
        .fixed_pos(egui::pos2(screen.left(), screen.top() + TITLE_BAR_H))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(screen.width(), RESTORE_HANDLE_H),
                egui::Sense::click(),
            );
            ui.painter().rect_filled(rect, 0.0, crate::theme::accent_muted());
            if response.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                app.config.layout_state.ribbon_collapsed = false;
            }
        });
}

// ─── Tab Content ─────────────────────────────────────────────────────────────

fn tab_home(ui: &mut egui::Ui, app: &mut HvBibleApp) {
    group(ui, |ui| {
        if !app.is_listening {
            if tool_card(ui, "▶", "Start", Some(STATUS_SUCCESS)).clicked() {
                app.toggle_listening();
            }
        } else {
            if tool_card(ui, "■", "Stop", Some(STATUS_ERROR)).clicked() {
                app.toggle_listening();
            }
        }
    });

    group(ui, |ui| {
        if tool_card(ui, "⌨", "Manual", None).clicked() {
            app.focus_manual = true;
        }
        if tool_card(ui, "↩", "Undo", None).clicked() {
            app.undo_verse();
        }
        if tool_card(ui, "✔", "Approve", None).clicked() {
            app.approve_current();
        }
    });

    group(ui, |ui| {
        let p_out_color = if app.program_out { Some(crate::theme::accent()) } else { None };
        if tool_card(ui, "📺", "PGM Out", p_out_color).clicked() {
            app.program_out = !app.program_out;
        }
    });

    group(ui, |ui| {
        widget_container(ui, 70.0, "Translation", |ui| {
            egui::ComboBox::from_id_salt("t_trans")
                .selected_text(app.translation.as_str())
                .width(60.0)
                .show_ui(ui, |ui| {
                    for t in ["KJV", "NIV", "ESV", "NASB", "NLT", "NKJV", "MSG", "AMP"] {
                        if ui.selectable_value(&mut app.translation, t.to_string(), t).changed() {
                            app.pipeline.send_command(hv_pipeline::PipelineCommand::SetTranslation(t.to_string()));
                        }
                    }
                });
        });
    });

    if tool_card(ui, "⚙", "Settings", None).clicked() {
        app.config.layout_state.settings_open = true;
    }
}

fn tab_broadcast(ui: &mut egui::Ui, app: &mut HvBibleApp) {
    group(ui, |ui| {
        tool_card(ui, "●", "Go Live", Some(STATUS_ERROR));
        tool_card(ui, "⚠", "Emergency", None);
    });

    group(ui, |ui| {
        let p_out_color = if app.program_out { Some(crate::theme::accent()) } else { None };
        if tool_card(ui, "📺", "PGM Out", p_out_color).clicked() {
            app.program_out = !app.program_out;
        }
        tool_card(ui, "🏷", "L3rd", None);
        tool_card(ui, "🧹", "Clean", None);
    });
}

fn tab_bible(ui: &mut egui::Ui, app: &mut HvBibleApp) {
    group(ui, |ui| {
        tool_card(ui, "🔍", "Search", None);
        if tool_card(ui, "⌨", "Manual", None).clicked() {
            app.focus_manual = true;
        }
        tool_card(ui, "🔗", "X-Ref", None);
    });
    
    group(ui, |ui| {
        widget_container(ui, 70.0, "Translation", |ui| {
            egui::ComboBox::from_id_salt("t_b_trans")
                .selected_text(app.translation.as_str())
                .width(60.0)
                .show_ui(ui, |ui| {
                    for t in ["KJV", "NIV", "ESV", "NASB", "NLT", "NKJV", "MSG", "AMP"] {
                        if ui.selectable_value(&mut app.translation, t.to_string(), t).changed() {
                            app.pipeline.send_command(hv_pipeline::PipelineCommand::SetTranslation(t.to_string()));
                        }
                    }
                });
        });
    });
}

fn tab_audio(ui: &mut egui::Ui, app: &mut HvBibleApp) {
    group(ui, |ui| {
        if !app.is_listening {
            if tool_card(ui, "▶", "Start ASR", Some(STATUS_SUCCESS)).clicked() {
                app.toggle_listening();
            }
        } else {
            if tool_card(ui, "■", "Stop ASR", Some(STATUS_ERROR)).clicked() {
                app.toggle_listening();
            }
        }
    });

    group(ui, |ui| {
        widget_container(ui, 100.0, "Gain", |ui| {
            ui.add(
                egui::Slider::new(&mut app.gain, -24.0..=24.0)
                    .suffix(" dB")
                    .clamping(egui::SliderClamping::Always)
            );
        });
    });

    group(ui, |ui| {
        widget_container(ui, 170.0, "Device", |ui| {
            egui::ComboBox::from_id_salt("t_audio_dev")
                .selected_text(app.selected_device.chars().take(22).collect::<String>())
                .width(160.0)
                .show_ui(ui, |ui| {
                    for dev in app.devices.clone() {
                        let sel = app.selected_device == dev;
                        if ui.selectable_label(sel, &dev).clicked() {
                            app.selected_device = dev.clone();
                            app.pipeline.send_command(hv_pipeline::PipelineCommand::SetDevice(dev));
                        }
                    }
                });
        });
    });
}

fn tab_tools(ui: &mut egui::Ui, _app: &mut HvBibleApp) {
    group(ui, |ui| {
        tool_card(ui, "🩺", "Health", None);
        tool_card(ui, "📊", "Metrics", None);
        tool_card(ui, "📄", "Export", None);
    });
}

// ─── UI Components ───────────────────────────────────────────────────────────

fn tab_selector(ui: &mut egui::Ui, active_tab: &mut usize) {
    let tabs = [
        ("🏠", "Home"),
        ("📡", "Broadcast"),
        ("📖", "Bible"),
        ("🎙", "Audio"),
        ("🛠", "Tools"),
    ];
    
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0; 
        for (i, &(icon, name)) in tabs.iter().enumerate() {
            let active = *active_tab == i;
            if icon_card_tab(ui, icon, name, active) {
                *active_tab = i;
            }
        }
    });
}

fn icon_card_tab(ui: &mut egui::Ui, icon: &str, label: &str, active: bool) -> bool {
    let size = egui::vec2(68.0, 64.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    
    if ui.is_rect_visible(rect) {
        let is_hovered = resp.hovered();
        
        let bg_color = if active {
            crate::theme::accent_muted()
        } else if is_hovered {
            crate::theme::bg_surface_raised()
        } else {
            egui::Color32::TRANSPARENT
        };
        
        let stroke = if active {
            egui::Stroke::new(1.0, crate::theme::accent())
        } else if is_hovered {
            egui::Stroke::new(1.0, crate::theme::border_subtle())
        } else {
            egui::Stroke::NONE
        };
        
        ui.painter().rect(rect, 2.0, bg_color, stroke);
        
        let icon_color = if active { crate::theme::accent() } else { crate::theme::text_primary() };
        ui.painter().text(
            rect.center() - egui::vec2(0.0, 6.0),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(20.0),
            icon_color,
        );
        
        let text_color = if active { crate::theme::text_primary() } else { crate::theme::text_secondary() };
        ui.painter().text(
            rect.center() + egui::vec2(0.0, 13.0),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(11.0),
            text_color,
        );
    }
    
    resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

fn tool_card(ui: &mut egui::Ui, icon: &str, label: &str, active_color: Option<egui::Color32>) -> egui::Response {
    let size = egui::vec2(60.0, 56.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    
    if ui.is_rect_visible(rect) {
        let is_hovered = resp.hovered();
        
        let bg_color = if let Some(c) = active_color {
            if is_hovered { c.linear_multiply(0.8) } else { c }
        } else if is_hovered {
            crate::theme::bg_surface_raised()
        } else {
            crate::theme::bg_base()
        };
        
        let stroke = if active_color.is_none() {
            egui::Stroke::new(1.0, crate::theme::border_subtle())
        } else {
            egui::Stroke::NONE
        };
        
        ui.painter().rect(rect, 2.0, bg_color, stroke);
        
        let icon_color = if active_color.is_some() { crate::theme::text_inverse() } else { crate::theme::text_primary() };
        ui.painter().text(
            rect.center() - egui::vec2(0.0, 6.0),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(18.0),
            icon_color,
        );
        
        let text_color = if active_color.is_some() { crate::theme::text_inverse() } else { crate::theme::text_secondary() };
        ui.painter().text(
            rect.center() + egui::vec2(0.0, 13.0),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(11.0),
            text_color,
        );
    }
    
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn divider(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 48.0), Sense::hover());
    ui.painter().line_segment(
        [rect.center_top(), rect.center_bottom()],
        Stroke::new(1.0, crate::theme::border_subtle())
    );
}

fn group(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal_centered(|ui| {
        add_contents(ui);
    });
    ui.add_space(8.0);
    divider(ui);
    ui.add_space(8.0);
}

fn widget_container(ui: &mut egui::Ui, width: f32, label: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    let size = egui::vec2(width, 56.0);
    ui.allocate_exact_size(size, egui::Sense::hover());
    
    let mut child_ui = ui.new_child(egui::UiBuilder::new().layout(egui::Layout::top_down(egui::Align::Center)));
    child_ui.spacing_mut().item_spacing.y = 4.0;
    
    // Top padding to vertically center the 2 items in the 56px height.
    child_ui.add_space(10.0);
    
    child_ui.label(RichText::new(label).size(10.0).color(crate::theme::text_tertiary()));
    add_contents(&mut child_ui);
}
