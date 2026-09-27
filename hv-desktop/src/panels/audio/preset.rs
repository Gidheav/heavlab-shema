//! preset.rs — Preset section.
//!
//! A picker and two small buttons. Recalling a console setup is an operating
//! decision; building one is a setup decision and belongs in Settings.

use eframe::egui::{self, Ui};

use crate::app::HvBibleApp;
use crate::components::column_controls::{column_width, InlineButton};
use crate::components::column_density::{ColumnDensity, UNIT};
use crate::panels::settings_window::PRESET_NAMES;

pub fn show(ui: &mut Ui, app: &mut HvBibleApp, density: ColumnDensity) {
    if !density.shows_labels() {
        // Icon-only: a single button that recalls the default preset. Enough to
        // keep the column operable without a word of text.
        if InlineButton::new("Preset", density)
            .icon("⌘")
            .active(app.config.saved_presets.iter().any(|preset| {
                preset.name == app.config.preset_name
            }))
            .show(ui)
            .clicked()
        {
            apply_default(app);
        }
        return;
    }

    let name = app.config.preset_name.clone();
    // Snapshot the library: the picker writes into `config.preset_name` while
    // iterating the stored presets, so the two borrows cannot overlap.
    let stored: Vec<String> = app
        .config
        .saved_presets
        .iter()
        .map(|preset| preset.name.clone())
        .collect();

    egui::ComboBox::from_id_salt("audio_preset_pick")
        .selected_text(name.clone())
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            for option in PRESET_NAMES {
                if ui
                    .selectable_value(&mut app.config.preset_name, (*option).to_string(), *option)
                    .changed()
                {
                    app.save_config();
                }
            }
            // Presets the operator has actually stored, so the picker and the
            // library can never disagree about what exists.
            for stored_name in &stored {
                if ui
                    .selectable_value(
                        &mut app.config.preset_name,
                        stored_name.clone(),
                        stored_name,
                    )
                    .changed()
                {
                    app.apply_preset(stored_name);
                }
            }
        });

    ui.add_space(density.row_gap());

    // Two buttons, sharing the width. The library row opens Settings on the
    // page that owns preset management, so there is exactly one editor.
    let per_column = column_width(ui.available_width(), 2, UNIT);
    ui.horizontal(|ui| {
        if ui
            .add_sized(
                [per_column, density.row_height()],
                egui::Button::new(egui::RichText::new("💾  Save").size(11.0)),
            )
            .on_hover_text("Store the current console under this name")
            .clicked()
        {
            let name = app.config.preset_name.clone();
            app.save_preset(&name);
        }

        if ui
            .add_sized(
                [per_column, density.row_height()],
                egui::Button::new(egui::RichText::new("📂  Manage").size(11.0)),
            )
            .on_hover_text("Open the preset library in Settings")
            .clicked()
        {
            app.config.layout_state.settings_tab = crate::panels::settings_window::TAB_AUDIO;
            app.config.layout_state.settings_open = true;
        }
    });
}

/// Recalls the preset a new session starts on.
fn apply_default(app: &mut HvBibleApp) {
    let default = app.config.default_preset_name.clone();
    app.apply_preset(&default);
}

/// The header light: a dot when a stored preset is the one in use, so the
/// operator can tell a recalled console from a hand-tuned one.
pub fn status(app: &HvBibleApp) -> Option<(egui::Color32, &'static str)> {
    if app
        .config
        .saved_presets
        .iter()
        .any(|preset| preset.name == app.config.preset_name)
    {
        return Some((crate::theme::accent(), "SAVED"));
    }
    None
}
