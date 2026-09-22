use egui::{ComboBox, FontId, Stroke, TextEdit, Ui};

use crate::app::{manual_verse_id, HvBibleApp};
use crate::theme::{STATUS_SUCCESS, STATUS_WARNING, STATUS_ERROR, text_primary, text_secondary, text_tertiary, bg_surface_sunken, border_subtle, border_strong};

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    // Control bar: Translation, Font, Size
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Translation:").small().color(crate::theme::text_secondary()));
        ComboBox::from_id_salt("translation_select")
            .selected_text(app.translation.clone())
            .show_ui(ui, |ui| {
                let available_translations = app.store.loaded_translations();
                for code in ["KJV", "ESV", "NIV", "ASV"] {
                    let is_available = available_translations.contains(&code.to_string());
                    if is_available {
                        if ui.selectable_value(&mut app.translation, code.to_string(), code).changed() {
                            app.pipeline.send_command(hv_pipeline::PipelineCommand::SetTranslation(app.translation.clone()));

                            if let Some((reference, _text)) = &mut app.current_verse {
                                if let Ok(verse_ref) = bible_core::parser::resolve_text(reference) {
                                    if let Ok(g_index) = bible_core::canon::resolve_index(verse_ref.book, verse_ref.chapter, verse_ref.verse) {
                                        if let Ok(_new_text) = app.store.get_verse(&app.translation, g_index) {
                                            // TODO: Update verse text
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        ui.label(egui::RichText::new(code).color(crate::theme::text_tertiary()));
                    }
                }
            });

        ui.add_space(16.0);
        ui.label(egui::RichText::new("Font:").small().color(crate::theme::text_secondary()));
        ComboBox::from_id_salt("font_select")
            .selected_text("Serif")
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut "Serif".to_string(), "Serif".to_string(), "Serif");
                ui.selectable_value(&mut "Sans".to_string(), "Sans".to_string(), "Sans");
            });

        ui.add_space(16.0);
        ui.label(egui::RichText::new("Size:").small().color(crate::theme::text_secondary()));
        let mut font_size = 48.0_f32;
        ui.add(egui::Slider::new(&mut font_size, 24.0..=72.0).show_value(true));
    });

    ui.add_space(8.0);

    // Border state based on confidence
    let border_color = match &app.current_verse {
        Some(_) => {
            if app.asr_confidence >= 0.90 {
                STATUS_SUCCESS
            } else if app.asr_confidence >= 0.70 {
                STATUS_WARNING
            } else {
                STATUS_ERROR
            }
        }
        None => crate::theme::border_subtle(),
    };

    // Verse display area
    egui::Frame::none()
        .fill(crate::theme::bg_surface_sunken())
        .stroke(Stroke::new(2.0, border_color))
        .inner_margin(egui::Margin::same(32.0))
        .show(ui, |ui| {
            let alpha = app.verse_fade;
            let fade = |c: egui::Color32| {
                egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (alpha * 255.0) as u8)
            };

            match &app.current_verse {
                Some((reference, text)) => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(24.0);
                        ui.label(
                            egui::RichText::new(reference)
                                .font(FontId::proportional(48.0))
                                .strong()
                                .color(fade(crate::theme::text_primary())),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("────────")
                                .font(FontId::proportional(32.0))
                                .color(fade(crate::theme::text_secondary())),
                        );
                        ui.add_space(24.0);
                        ui.label(
                            egui::RichText::new(text)
                                .font(FontId::proportional(28.0))
                                .line_height(Some(36.0))
                                .color(fade(crate::theme::text_primary())),
                        );
                        ui.add_space(24.0);
                    });
                }
                None => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(48.0);
                        ui.label(
                            egui::RichText::new("Waiting for verse reference…")
                                .font(FontId::proportional(22.0))
                                .color(crate::theme::text_tertiary()),
                        );
                        ui.add_space(16.0);
                    });
                }
            }
        });

    ui.add_space(12.0);

    // Action buttons
    ui.horizontal(|ui| {
        if ui
            .add(
                egui::Button::new(egui::RichText::new("✓ Approve").strong())
                    .fill(STATUS_SUCCESS)
                    .min_size(egui::vec2(100.0, 28.0)),
            )
            .clicked()
        {
            app.approve_current();
        }

        if ui
            .add(
                egui::Button::new(egui::RichText::new("✗ Reject").strong())
                    .fill(STATUS_ERROR)
                    .min_size(egui::vec2(100.0, 28.0)),
            )
            .clicked()
        {
            app.reject_current();
        }

        if ui
            .add(
                egui::Button::new(egui::RichText::new("↻ Edit").strong())
                    .fill(crate::theme::border_strong())
                    .min_size(egui::vec2(100.0, 28.0)),
            )
            .clicked()
        {
            // TODO: Open edit dialog
        }

        if ui
            .add(
                egui::Button::new(egui::RichText::new("↩ Undo").strong())
                    .fill(crate::theme::border_strong())
                    .min_size(egui::vec2(100.0, 28.0)),
            )
            .clicked()
        {
            app.undo_verse();
        }

        if ui
            .add(
                egui::Button::new(egui::RichText::new("📋 Copy").strong())
                    .fill(crate::theme::border_strong())
                    .min_size(egui::vec2(100.0, 28.0)),
            )
            .clicked()
        {
            // TODO: Copy to clipboard
        }
    });

    ui.add_space(8.0);

    // Confidence meter
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Confidence:").small().color(crate::theme::text_secondary()));
        let confidence_width = (app.asr_confidence * 200.0) as f32;
        egui::Frame::none()
            .fill(crate::theme::bg_surface_sunken())
            .show(ui, |ui| {
                ui.add_space(4.0);
                let painter = ui.painter();
                let rect = ui.available_rect_before_wrap();
                painter.rect_filled(
                    egui::Rect::from_min_size(rect.min, egui::vec2(confidence_width, rect.height())),
                    0.0,
                    if app.asr_confidence >= 0.90 {
                        STATUS_SUCCESS
                    } else if app.asr_confidence >= 0.70 {
                        STATUS_WARNING
                    } else {
                        STATUS_ERROR
                    },
                );
                ui.add_space(4.0);
            });
        ui.label(
            egui::RichText::new(format!("{:>3.0}%", app.asr_confidence * 100.0))
                .small()
                .color(crate::theme::text_secondary()),
        );
        ui.add_space(16.0);
        ui.label(
            egui::RichText::new("Source: Auto-detect")
                .small()
                .color(crate::theme::text_secondary()),
        );
    });

    ui.add_space(12.0);

    // Manual entry
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("MANUAL").small().color(crate::theme::text_secondary()));
        let edit = TextEdit::singleline(&mut app.manual_input)
            .id(manual_verse_id())
            .hint_text("John 3:16")
            .desired_width(ui.available_width() - 72.0);
        let response = ui.add(edit);
        if app.focus_manual {
            response.request_focus();
            app.focus_manual = false;
        }
        if ui.button("Go").clicked()
            || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
        {
            app.submit_manual_verse();
        }
    });
}