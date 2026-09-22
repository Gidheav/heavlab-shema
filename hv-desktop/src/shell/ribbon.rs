//! Ribbon bar — compact, Office-style toolbar directly below the title bar.
//!
//! Design rules:
//!  - Click-collapse only (˄ button). NO hover-to-expand.
//!  - Tab strip + body. No cards. Groups separated by thin vertical lines.
//!  - Total height: 21px tab + 48px body = 69px when expanded, 21px collapsed.

use eframe::egui::{self, Context};

use crate::app::HvBibleApp;
use crate::theme::{STATUS_ERROR, STATUS_SUCCESS};

const TAB_H: f32 = 21.0;
const BODY_H: f32 = 48.0;
const TITLE_BAR_H: f32 = 32.0;
const RESTORE_HANDLE_H: f32 = 4.0;

// ─── Persistent state ────────────────────────────────────────────────────────

#[derive(Default, Clone)]
pub struct RibbonState {
    pub active_tab: usize,
}

// ─── Entry point ─────────────────────────────────────────────────────────────

pub fn show(ctx: &Context, app: &mut HvBibleApp) {
    let sid = egui::Id::new("hvb_ribbon_v3");
    let mut state: RibbonState = ctx.data_mut(|d| d.get_temp(sid).unwrap_or_default());

    if app.config.layout_state.ribbon_collapsed {
        egui::TopBottomPanel::top("ribbon_bar")
            .exact_height(0.0)
            .frame(egui::Frame::none().inner_margin(egui::Margin::same(0.0)))
            .show(ctx, |_ui| {});
        show_restore_handle(ctx, app);
        ctx.data_mut(|d| d.insert_temp(sid, state));
        return;
    }

    egui::TopBottomPanel::top("ribbon_bar")
        .exact_height(TAB_H + BODY_H)
        .frame(
            egui::Frame::none()
                .fill(crate::theme::bg_surface())
                .stroke(egui::Stroke::new(1.0, crate::theme::border_subtle()))
                .inner_margin(egui::Margin::same(0.0)),
        )
        .show(ctx, |ui| {
            let full = ui.max_rect();

            // ── Tab strip ────────────────────────────────────────────────
            let tab_rect = egui::Rect::from_min_size(full.min, egui::vec2(full.width(), TAB_H));
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(tab_rect), |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.add_space(4.0);

                    let tabs = ["Home", "Broadcast", "Bible", "Audio", "Tools"];
                    let mut next = state.active_tab;
                    for (i, tab) in tabs.iter().enumerate() {
                        if tab_btn(ui, tab, state.active_tab == i) {
                            next = i;
                        }
                    }
                    state.active_tab = next;

                    // Collapse toggle — right side, click-only
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(6.0);
                        if ghost_icon(ui, "^").clicked() {
                            app.config.layout_state.ribbon_collapsed = true;
                        }
                    });
                });
            });

            // ── Body ─────────────────────────────────────────────────────
            let body_rect = egui::Rect::from_min_size(
                full.min + egui::vec2(0.0, TAB_H),
                egui::vec2(full.width(), BODY_H),
            );
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(body_rect), |ui| {
                // top divider
                ui.painter().hline(
                    body_rect.left()..=body_rect.right(),
                    body_rect.top(),
                    egui::Stroke::new(1.0, crate::theme::border_subtle()),
                );
                ui.horizontal_centered(|ui| {
                    ui.set_height(BODY_H);
                    ui.spacing_mut().item_spacing.x = 3.0;
                    ui.add_space(8.0);
                    match state.active_tab {
                        0 => tab_home(ui, app),
                        1 => tab_broadcast(ui, app),
                        2 => tab_bible(ui, app),
                        3 => tab_audio(ui, app),
                        _ => tab_tools(ui),
                    }
                });
            });
        });

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
            ui.painter()
                .rect_filled(rect, 0.0, crate::theme::accent_muted());
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "v",
                egui::FontId::proportional(9.0),
                crate::theme::text_primary(),
            );
            if response
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                app.config.layout_state.ribbon_collapsed = false;
            }
        });
}

// ─── Tab content ─────────────────────────────────────────────────────────────

fn tab_home(ui: &mut egui::Ui, app: &mut HvBibleApp) {
    group(ui, |ui| {
        if action_btn(ui, "▶  Start", STATUS_SUCCESS).clicked() && !app.is_listening {
            app.toggle_listening();
        }
        if action_btn(ui, "■  Stop", STATUS_ERROR).clicked() && app.is_listening {
            app.toggle_listening();
        }
    });
    group(ui, |ui| {
        if tool_btn(ui, "🔍").clicked() {}
        if tool_btn(ui, "Manual").clicked() {
            app.focus_manual = true;
        }
        if tool_btn(ui, "↩").clicked() {
            app.undo_verse();
        }
        if tool_btn(ui, "✔").clicked() {
            app.approve_current();
        }
    });
    group(ui, |ui| {
        if tool_btn(ui, "Program Out").clicked() {
            app.program_out = !app.program_out;
        }
        if tool_btn(ui, "Lower Third").clicked() {}
    });
    group(ui, |ui| {
        ui.label(
            egui::RichText::new("Translation")
                .size(10.5)
                .color(crate::theme::text_tertiary()),
        );
        egui::ComboBox::from_id_salt("rb_trans")
            .selected_text(app.translation.as_str())
            .width(72.0)
            .show_ui(ui, |ui| {
                for t in ["KJV", "NIV", "ESV", "NASB", "NLT", "NKJV", "MSG", "AMP"] {
                    ui.selectable_value(&mut app.translation, t.to_string(), t);
                }
            });
    });
    // Theme toggle — last item, no trailing separator
    let lbl = if app.dark_mode {
        "  Light Mode  "
    } else {
        "  Dark Mode  "
    };
    if tool_btn(ui, lbl).clicked() {
        app.dark_mode = !app.dark_mode;
    }
}

fn tab_broadcast(ui: &mut egui::Ui, app: &mut HvBibleApp) {
    group(ui, |ui| {
        action_btn(ui, "● Go Live", STATUS_ERROR);
        if tool_btn(ui, "Emergency Clear").clicked() {}
        if tool_btn(ui, "Fallback Graphic").clicked() {}
    });
    group(ui, |ui| {
        if tool_btn(ui, "Program Out").clicked() {
            app.program_out = !app.program_out;
        }
        if tool_btn(ui, "Lower Third").clicked() {}
        if tool_btn(ui, "Clean Feed").clicked() {}
    });
    group(ui, |ui| {
        for mode in ["Window", "NDI", "SDI", "OBS"] {
            if tool_btn(ui, mode).clicked() {}
        }
    });
}

fn tab_bible(ui: &mut egui::Ui, app: &mut HvBibleApp) {
    group(ui, |ui| {
        if tool_btn(ui, "🔍").clicked() {}
        if tool_btn(ui, "Manual Entry").clicked() {
            app.focus_manual = true;
        }
        if tool_btn(ui, "Cross-Reference").clicked() {}
        if tool_btn(ui, "Comparison").clicked() {}
    });
    group(ui, |ui| {
        ui.label(
            egui::RichText::new("Translation")
                .size(10.5)
                .color(crate::theme::text_tertiary()),
        );
        egui::ComboBox::from_id_salt("rb_bible_trans")
            .selected_text(app.translation.as_str())
            .width(72.0)
            .show_ui(ui, |ui| {
                for t in ["KJV", "NIV", "ESV", "NASB", "NLT", "NKJV", "MSG", "AMP"] {
                    ui.selectable_value(&mut app.translation, t.to_string(), t);
                }
            });
        if tool_btn(ui, "Download").clicked() {}
    });
}

fn tab_audio(ui: &mut egui::Ui, app: &mut HvBibleApp) {
    group(ui, |ui| {
        if action_btn(ui, "▶  Start ASR", STATUS_SUCCESS).clicked() && !app.is_listening {
            app.toggle_listening();
        }
        if action_btn(ui, "■  Stop ASR", STATUS_ERROR).clicked() && app.is_listening {
            app.toggle_listening();
        }
    });
    group(ui, |ui| {
        ui.label(
            egui::RichText::new("Input Gain")
                .size(10.5)
                .color(crate::theme::text_tertiary()),
        );
        ui.add(
            egui::Slider::new(&mut app.gain, -24.0..=24.0)
                .suffix(" dB")
                .clamping(egui::SliderClamping::Always),
        );
    });
    group(ui, |ui| {
        ui.label(
            egui::RichText::new("Device")
                .size(10.5)
                .color(crate::theme::text_tertiary()),
        );
        egui::ComboBox::from_id_salt("rb_audio_dev")
            .selected_text(app.selected_device.chars().take(22).collect::<String>())
            .width(160.0)
            .show_ui(ui, |ui| {
                for dev in app.devices.clone() {
                    let sel = app.selected_device == dev;
                    if ui.selectable_label(sel, &dev).clicked() {
                        app.selected_device = dev.clone();
                        app.pipeline
                            .send_command(hv_pipeline::PipelineCommand::SetDevice(dev));
                    }
                }
            });
    });
}

fn tab_tools(ui: &mut egui::Ui) {
    group(ui, |ui| {
        if tool_btn(ui, "Health Check").clicked() {}
        if tool_btn(ui, "Metrics").clicked() {}
        if tool_btn(ui, "Export Logs").clicked() {}
    });
    group(ui, |ui| {
        if tool_btn(ui, "Preferences").clicked() {}
        if tool_btn(ui, "Shortcuts").clicked() {}
    });
}

// ─── Layout helpers ──────────────────────────────────────────────────────────

/// Renders group content and appends a vertical separator line.
fn group(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    add_contents(ui);
    ui.add_space(8.0);
    ui.separator(); // vertical line in horizontal layout
    ui.add_space(8.0);
}

// ─── Widget helpers ──────────────────────────────────────────────────────────

/// Tab button in the tab strip — ghost, active tab gets filled bg + accent underline.
fn tab_btn(ui: &mut egui::Ui, label: &str, active: bool) -> bool {
    ui.scope(|ui| {
        ui.visuals_mut().button_frame = false;
        ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
        ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
        ui.visuals_mut().widgets.hovered.bg_fill = crate::theme::bg_surface_raised();
        ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::NONE;

        let resp = ui.add(
            egui::Button::new(
                egui::RichText::new(format!("  {}  ", label))
                    .size(12.0)
                    .color(if active {
                        crate::theme::text_primary()
                    } else {
                        crate::theme::text_secondary()
                    }),
            )
            .fill(if active {
                crate::theme::bg_surface_raised()
            } else {
                egui::Color32::TRANSPARENT
            })
            .stroke(egui::Stroke::NONE)
            .min_size(egui::vec2(0.0, TAB_H)),
        );

        if active {
            let r = resp.rect;
            ui.painter().line_segment(
                [
                    egui::pos2(r.left() + 5.0, r.bottom()),
                    egui::pos2(r.right() - 5.0, r.bottom()),
                ],
                egui::Stroke::new(2.0, crate::theme::accent()),
            );
        }
        resp.clicked()
    })
    .inner
}

/// Tiny ghost icon button (collapse chevron).
fn ghost_icon(ui: &mut egui::Ui, icon: &str) -> egui::Response {
    ui.scope(|ui| {
        ui.visuals_mut().button_frame = false;
        ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
        ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
        ui.visuals_mut().widgets.hovered.bg_fill = crate::theme::bg_surface_raised();
        ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::NONE;
        ui.add(
            egui::Button::new(
                egui::RichText::new(icon)
                    .size(12.0)
                    .color(crate::theme::text_tertiary()),
            )
            .fill(egui::Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE)
            .min_size(egui::vec2(22.0, TAB_H)),
        )
    })
    .inner
}

/// Coloured primary action button (Start/Stop/Go Live etc.).
fn action_btn(ui: &mut egui::Ui, label: &str, fill: egui::Color32) -> egui::Response {
    ui.scope(|ui| {
        ui.spacing_mut().button_padding = egui::vec2(10.0, 5.0);
        ui.add(
            egui::Button::new(
                egui::RichText::new(label)
                    .size(12.5)
                    .color(crate::theme::text_inverse())
                    .strong(),
            )
            .fill(fill)
            .min_size(egui::vec2(80.0, 30.0)),
        )
    })
    .inner
}

/// Subtle tool button — raised bg, accent on hover. Theme-aware text.
fn tool_btn(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.scope(|ui| {
        ui.visuals_mut().widgets.inactive.bg_fill = crate::theme::bg_surface_raised();
        ui.visuals_mut().widgets.inactive.bg_stroke =
            egui::Stroke::new(1.0, crate::theme::border_subtle());
        ui.visuals_mut().widgets.inactive.rounding = egui::Rounding::same(3.0);
        ui.visuals_mut().widgets.hovered.bg_fill = crate::theme::accent_muted();
        ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::new(1.0, crate::theme::accent());
        ui.visuals_mut().widgets.hovered.rounding = egui::Rounding::same(3.0);
        ui.spacing_mut().button_padding = egui::vec2(8.0, 4.0);
        // No explicit .color() so egui uses override_text_color → theme-aware
        ui.button(egui::RichText::new(label).size(12.5))
    })
    .inner
}
