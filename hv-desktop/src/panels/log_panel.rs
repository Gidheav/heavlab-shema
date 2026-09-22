use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use egui::{Align, Layout, ScrollArea, Ui};

use crate::app::{HvBibleApp, SermonLogEntry};
use crate::paths;

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    ui.horizontal(|ui| {
        ui.heading(egui::RichText::new("SERMON LOG").color(crate::theme::accent()).strong());
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{} entries", app.sermon_log.len()))
                    .small()
                    .color(crate::theme::text_secondary()),
            );
        });
    });
    ui.add_space(8.0);

    ui.add(
        egui::TextEdit::singleline(&mut app.log_filter)
            .hint_text("🔍")
            .desired_width(ui.available_width()),
    );
    ui.add_space(8.0);

    ScrollArea::vertical()
        .id_salt("sermon_log_scroll")
        .auto_shrink([false, false])
        .max_height(ui.available_height() - 52.0)
        .show(ui, |ui| {
            let filter = app.log_filter.to_lowercase();
            let mut clicked: Option<usize> = None;
            for (index, entry) in app.sermon_log.iter().enumerate().rev() {
                if !filter.is_empty() && !entry.reference.to_lowercase().contains(&filter) {
                    continue;
                }
                let mark = if entry.approved { "✓" } else { " " };
                let label = format!("{}  {}  {}", entry.timestamp, entry.reference, mark);
                let rich = if entry.approved {
                    egui::RichText::new(label).color(egui::Color32::GREEN)
                } else {
                    egui::RichText::new(label).color(crate::theme::text_primary())
                };
                if ui.selectable_label(false, rich).clicked() {
                    clicked = Some(index);
                }
            }
            if let Some(index) = clicked {
                app.restore_log_entry(index);
            }
        });

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui.button("EXPORT").clicked() {
            export_log(&app.sermon_log, "sermon-log.txt");
        }
    });
}

fn export_log(entries: &[SermonLogEntry], filename: &str) {
    let path = paths::data_dir().join(filename);
    if let Some(parent) = path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            tracing::warn!("export mkdir failed: {error}");
            return;
        }
    }
    let mut body = String::from("HV-Bible sermon log\n===================\n\n");
    for entry in entries {
        let mark = if entry.approved {
            "approved"
        } else {
            "pending"
        };
        body.push_str(&format!(
            "{}  {}  ({})\n",
            entry.timestamp, entry.reference, mark
        ));
    }
    if let Err(error) = fs::write(&path, body) {
        tracing::warn!("export failed ({path:?}): {error}");
    } else {
        tracing::info!("exported sermon log to {path:?}");
    }
}

pub fn format_clock() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let s = secs % 86400;
    let h = s / 3600;
    let m = (s % 3600) / 60;
    let sec = s % 60;
    format!("{h:02}:{m:02}:{sec:02}")
}