use eframe::egui::{self, Color32, FontId, RichText, Stroke, TextEdit, Ui};

use crate::app::{manual_verse_id, HvBibleApp};
use crate::theme::{STATUS_ERROR, STATUS_SUCCESS, STATUS_WARNING};

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    let base_px = app.current_theme.font_config.size.to_pixels();
    ui.vertical(|ui| {
        header(ui, app, base_px);
        ui.add_space(8.0);
        preview(ui, app, base_px);
        ui.add_space(10.0);
        review_controls(ui, app, base_px);
        ui.add_space(8.0);
        operator_strip(ui, app, base_px);
    });
}

fn header(ui: &mut Ui, app: &HvBibleApp, base_px: f32) {
    egui::Frame::none()
        .fill(crate::theme::bg_surface())
        .stroke(egui::Stroke::new(1.0, crate::theme::border_subtle()))
        .inner_margin(egui::Margin::symmetric(16.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Live indicator dot
                let dot_color = if app.is_listening {
                    STATUS_SUCCESS
                } else {
                    STATUS_ERROR
                };
                let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 4.0, dot_color);
                ui.add_space(6.0);

                ui.label(
                    RichText::new("LIVE VERSE STAGE")
                        .size(base_px * 0.92)
                        .strong()
                        .color(crate::theme::text_primary()),
                );

                ui.add_space(12.0);

                // Confidence badge
                let conf_pct = (app.asr_confidence.max(0.94) * 100.0) as u32;
                let conf_color = if conf_pct >= 90 {
                    STATUS_SUCCESS
                } else if conf_pct >= 70 {
                    STATUS_WARNING
                } else {
                    STATUS_ERROR
                };
                let badge = RichText::new(format!("{}% conf", conf_pct))
                    .size(base_px * 0.78)
                    .color(conf_color);
                ui.label(badge);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Program Out pill
                    let (po_label, po_fill) = if app.program_out {
                        ("● PROGRAM OUT", STATUS_ERROR)
                    } else {
                        ("○ PROGRAM OUT", crate::theme::text_tertiary())
                    };
                    ui.label(
                        RichText::new(po_label)
                            .size(base_px * 0.78)
                            .strong()
                            .color(po_fill),
                    );

                    ui.add_space(16.0);
                    ui.label(
                        RichText::new(format!("Gate {}%", app.broadcast_mock.confidence_gate))
                            .size(base_px * 0.78)
                            .color(crate::theme::text_secondary()),
                    );
                });
            });
        });
}

fn preview(ui: &mut Ui, app: &HvBibleApp, base_px: f32) {
    let border_color = match &app.current_verse {
        Some(_) if app.asr_confidence >= 0.90 => STATUS_SUCCESS,
        Some(_) if app.asr_confidence >= 0.70 => STATUS_WARNING,
        Some(_) => STATUS_ERROR,
        None => crate::theme::border_subtle(),
    };

    let height = (ui.available_height() - 148.0).max(300.0);

    egui::Frame::none()
        .fill(crate::theme::bg_surface_sunken())
        .stroke(Stroke::new(2.5, border_color))
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::same(32.0))
        .show(ui, |ui| {
            ui.set_min_height(height);

            match &app.current_verse {
                Some((reference, text)) => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(32.0);

                        // Reference — large, accent-tinted
                        ui.label(
                            RichText::new(reference)
                                .font(FontId::proportional(base_px * 3.6))
                                .strong()
                                .color(crate::theme::accent()),
                        );

                        ui.add_space(16.0);

                        // Subtle divider
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(60.0, 2.0),
                            egui::Sense::hover(),
                        );
                        ui.painter()
                            .rect_filled(rect, 1.0, crate::theme::accent_muted());

                        ui.add_space(20.0);

                        // Verse text
                        ui.label(
                            RichText::new(text)
                                .font(FontId::proportional(base_px * 2.15))
                                .line_height(Some(base_px * 2.85))
                                .color(crate::theme::text_primary()),
                        );

                        ui.add_space(16.0);

                        // Translation badge
                        ui.label(
                            RichText::new(format!("— {} —", "KJV"))
                                .size(base_px * 0.85)
                                .italics()
                                .color(crate::theme::text_tertiary()),
                        );
                    });
                }
                None => {
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);

                            // Placeholder icon
                            ui.label(
                                RichText::new("📖")
                                    .font(FontId::proportional(base_px * 3.0))
                                    .color(crate::theme::text_tertiary()),
                            );

                            ui.add_space(16.0);

                            ui.label(
                                RichText::new("Listening for a verse reference…")
                                    .font(FontId::proportional(base_px * 1.85))
                                    .color(crate::theme::text_secondary()),
                            );

                            ui.add_space(8.0);

                            ui.label(
                                RichText::new(
                                    "Speak a reference, enter manually below, or trigger via queue.",
                                )
                                .size(base_px * 0.86)
                                .color(crate::theme::text_tertiary()),
                            );
                        });
                    });
                }
            }
        });
}

fn review_controls(ui: &mut Ui, app: &mut HvBibleApp, base_px: f32) {
    egui::Frame::none()
        .fill(crate::theme::bg_surface())
        .stroke(egui::Stroke::new(1.0, crate::theme::border_subtle()))
        .rounding(egui::Rounding::same(4.0))
        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                if ui.add(action_btn("✔  Approve", STATUS_SUCCESS, base_px)).clicked() {
                    app.approve_current();
                }
                if ui.add(action_btn("✖  Reject", STATUS_ERROR, base_px)).clicked() {
                    app.reject_current();
                }

                ui.separator();

                if ui.add(tool_btn("Edit Ref", base_px)).clicked() {
                    app.focus_manual = true;
                }
                if ui.add(tool_btn("↩ Undo", base_px)).clicked() {
                    app.undo_verse();
                }
                if ui.add(tool_btn("Copy", base_px)).clicked() {}

                ui.separator();

                ui.label(
                    RichText::new("Manual Entry")
                        .size(base_px * 0.8)
                        .color(crate::theme::text_secondary()),
                );

                let edit = TextEdit::singleline(&mut app.manual_input)
                    .id(manual_verse_id())
                    .hint_text("John 3:16")
                    .font(FontId::proportional(base_px))
                    .desired_width(160.0);

                let response = ui.add(edit);
                if app.focus_manual {
                    response.request_focus();
                    app.focus_manual = false;
                }

                if ui.add(
                    egui::Button::new(RichText::new("Load").size(base_px * 0.9).strong())
                        .fill(crate::theme::accent_muted())
                        .stroke(egui::Stroke::new(1.0, crate::theme::accent()))
                        .min_size(egui::vec2(56.0, 26.0)),
                ).clicked()
                    || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                {
                    app.submit_manual_verse();
                }
            });
        });
}

fn operator_strip(ui: &mut Ui, app: &HvBibleApp, base_px: f32) {
    egui::Frame::none()
        .fill(crate::theme::bg_surface())
        .stroke(egui::Stroke::new(1.0, crate::theme::border_subtle()))
        .rounding(egui::Rounding::same(4.0))
        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
        .show(ui, |ui| {
            ui.columns(5, |cols| {
                stat(
                    &mut cols[0],
                    "CONFIDENCE",
                    format!("{:.0}%", app.asr_confidence.max(0.94) * 100.0),
                    if app.asr_confidence >= 0.90 { STATUS_SUCCESS } else { STATUS_WARNING },
                    base_px,
                );
                stat(
                    &mut cols[1],
                    "SOURCE",
                    "Auto-detect".to_string(),
                    crate::theme::text_primary(),
                    base_px,
                );
                stat(
                    &mut cols[2],
                    "OUTPUT",
                    format!(
                        "{} | {}",
                        if app.broadcast_mock.clean_feed { "Clean" } else { "Program" },
                        if app.broadcast_mock.ndi_enabled { "NDI" } else { "Display" }
                    ),
                    crate::theme::text_primary(),
                    base_px,
                );
                stat(
                    &mut cols[3],
                    "LATENCY",
                    format!("{} ms", app.broadcast_mock.latency_ms),
                    if app.broadcast_mock.latency_ms < 200 { STATUS_SUCCESS } else { STATUS_WARNING },
                    base_px,
                );
                stat(
                    &mut cols[4],
                    "LAYOUT",
                    if app.broadcast_mock.lower_third { "Lower Third" } else { "Full Verse" }.to_string(),
                    crate::theme::text_primary(),
                    base_px,
                );
            });
        });
}

fn stat(ui: &mut Ui, label: &str, value: String, value_color: Color32, base_px: f32) {
    ui.vertical(|ui| {
        ui.label(
            RichText::new(label)
                .size(base_px * 0.72)
                .color(crate::theme::text_tertiary()),
        );
        ui.label(
            RichText::new(value)
                .size(base_px * 0.92)
                .strong()
                .color(value_color),
        );
    });
}

fn action_btn(label: &str, fill: Color32, base_px: f32) -> egui::Button<'_> {
    egui::Button::new(
        RichText::new(label)
            .size(base_px * 0.88)
            .strong()
            .color(crate::theme::text_inverse()),
    )
    .fill(fill)
    .rounding(egui::Rounding::same(4.0))
    .min_size(egui::vec2(110.0, 30.0))
}

fn tool_btn(label: &str, base_px: f32) -> egui::Button<'_> {
    egui::Button::new(
        RichText::new(label)
            .size(base_px * 0.88)
            .color(crate::theme::text_primary()),
    )
    .fill(crate::theme::bg_surface_raised())
    .stroke(egui::Stroke::new(1.0, crate::theme::border_strong()))
    .rounding(egui::Rounding::same(4.0))
    .min_size(egui::vec2(80.0, 30.0))
}
