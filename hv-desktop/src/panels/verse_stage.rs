use eframe::egui::{self, FontId, Stroke, TextEdit, Ui};

use crate::app::{manual_verse_id, HvBibleApp};
use crate::theme::{
    STATUS_ERROR, STATUS_SUCCESS, STATUS_WARNING,
};

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    ui.vertical(|ui| {
        header(ui, app);
        ui.add_space(10.0);
        preview(ui, app);
        ui.add_space(10.0);
        review_controls(ui, app);
        ui.add_space(10.0);
        operator_strip(ui, app);
    });
}

fn header(ui: &mut Ui, app: &HvBibleApp) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("LIVE VERSE STAGE")
                .size(13.0)
                .strong()
                .color(crate::theme::text_primary()),
        );
        ui.label(
            egui::RichText::new(format!("Gate {}%", app.broadcast_mock.confidence_gate))
                .size(11.0)
                .color(crate::theme::text_secondary()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(if app.program_out {
                    "PROGRAM OPEN"
                } else {
                    "PROGRAM CLOSED"
                })
                .size(11.0)
                .color(crate::theme::accent()),
            );
            ui.label(
                egui::RichText::new(app.broadcast_mock.program_output)
                    .size(11.0)
                    .color(crate::theme::text_secondary()),
            );
        });
    });
}

fn preview(ui: &mut Ui, app: &HvBibleApp) {
    let border_color = match &app.current_verse {
        Some(_) if app.asr_confidence >= 0.90 => STATUS_SUCCESS,
        Some(_) if app.asr_confidence >= 0.70 => STATUS_WARNING,
        Some(_) => STATUS_ERROR,
        None => crate::theme::border_subtle(),
    };

    let height = (ui.available_height() - 128.0).max(330.0);
    egui::Frame::none()
        .fill(crate::theme::bg_surface_sunken())
        .stroke(Stroke::new(2.0, border_color))
        .inner_margin(egui::Margin::same(34.0))
        .show(ui, |ui| {
            ui.set_min_height(height);
            match &app.current_verse {
                Some((reference, text)) => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(42.0);
                        ui.label(
                            egui::RichText::new(reference)
                                .font(FontId::proportional(54.0))
                                .strong()
                                .color(crate::theme::text_primary()),
                        );
                        ui.add_space(18.0);
                        ui.label(
                            egui::RichText::new(text)
                                .font(FontId::proportional(32.0))
                                .line_height(Some(42.0))
                                .color(crate::theme::text_primary()),
                        );
                    });
                }
                None => {
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("Waiting for verse reference")
                                    .font(FontId::proportional(28.0))
                                    .color(crate::theme::text_tertiary()),
                            );
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(
                                    "Manual entry, queue trigger, or ASR detection will load here.",
                                )
                                .size(12.0)
                                .color(crate::theme::text_secondary()),
                            );
                        });
                    });
                }
            }
        });
}

fn review_controls(ui: &mut Ui, app: &mut HvBibleApp) {
    ui.horizontal_wrapped(|ui| {
        if ui.add(primary_button("✔", STATUS_SUCCESS)).clicked() {
            app.approve_current();
        }
        if ui.add(primary_button("Reject", STATUS_ERROR)).clicked() {
            app.reject_current();
        }
        if ui.add(secondary_button("Edit Reference")).clicked() {
            app.focus_manual = true;
        }
        if ui.add(secondary_button("Undo Push")).clicked() {
            app.undo_verse();
        }
        if ui.add(secondary_button("Copy")).clicked() {}

        ui.separator();
        ui.label(
            egui::RichText::new("Manual")
                .size(11.0)
                .color(crate::theme::text_secondary()),
        );
        let edit = TextEdit::singleline(&mut app.manual_input)
            .id(manual_verse_id())
            .hint_text("John 3:16")
            .desired_width(150.0);
        let response = ui.add(edit);
        if app.focus_manual {
            response.request_focus();
            app.focus_manual = false;
        }
        if ui.button("Load").clicked()
            || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
        {
            app.submit_manual_verse();
        }
    });
}

fn operator_strip(ui: &mut Ui, app: &HvBibleApp) {
    egui::Frame::none()
        .fill(crate::theme::bg_surface())
        .stroke(egui::Stroke::new(1.0, crate::theme::border_subtle()))
        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
        .show(ui, |ui| {
            ui.columns(4, |columns| {
                stat(
                    &mut columns[0],
                    "Confidence",
                    format!("{:.0}%", app.asr_confidence.max(0.94) * 100.0),
                );
                stat(&mut columns[1], "Source", "Auto-detect".to_string());
                stat(
                    &mut columns[2],
                    "Output",
                    format!(
                        "{} | {}",
                        if app.broadcast_mock.clean_feed {
                            "Clean"
                        } else {
                            "Program"
                        },
                        if app.broadcast_mock.ndi_enabled {
                            "NDI"
                        } else {
                            "Display"
                        }
                    ),
                );
                stat(
                    &mut columns[3],
                    "Latency",
                    format!(
                        "{} ms | {}",
                        app.broadcast_mock.latency_ms,
                        if app.broadcast_mock.lower_third {
                            "Lower third"
                        } else {
                            "Full verse"
                        }
                    ),
                );
            });
        });
}

fn stat(ui: &mut Ui, label: &str, value: String) {
    ui.label(egui::RichText::new(label).size(10.0).color(crate::theme::text_secondary()));
    ui.label(
        egui::RichText::new(value)
            .size(13.0)
            .strong()
            .color(crate::theme::text_primary()),
    );
}

fn primary_button(label: &'static str, fill: egui::Color32) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(label)
            .strong()
            .color(crate::theme::text_inverse()),
    )
    .fill(fill)
    .min_size(egui::vec2(104.0, 30.0))
}

fn secondary_button(label: &'static str) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(label).color(crate::theme::text_primary()))
        .fill(crate::theme::bg_surface())
        .stroke(egui::Stroke::new(1.0, crate::theme::border_strong()))
        .min_size(egui::vec2(104.0, 30.0))
}
