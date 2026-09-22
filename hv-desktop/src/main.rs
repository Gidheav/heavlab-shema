//! HV-Bible - Broadcast Engine desktop shell.

mod app;
mod components;
mod config;
mod layout;
mod mock;
mod panels;
mod paths;
mod pipeline_integration;
mod shell;
mod shortcuts;
mod theme;

use app::HvBibleApp;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    if let Err(error) = bible_rt::apply_process_tuning() {
        tracing::warn!("runtime tuning skipped: {error}");
    }

    let config = config::AppConfig::load();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("HV-Bible - Broadcast Engine")
            .with_decorations(false)
            .with_inner_size([
                config.window_width.max(1440.0),
                config.window_height.max(900.0),
            ])
            .with_min_inner_size([1280.0, 760.0]),
        vsync: true,
        ..Default::default()
    };

    eframe::run_native(
        "HV-Bible - Broadcast Engine",
        options,
        Box::new(|cc| Ok(Box::new(HvBibleApp::new(cc)))),
    )
}

#[cfg(test)]
mod ui_contract_tests {
    use crate::layout::workspace_preset::WorkspacePreset;
    use crate::mock::audio_state::AudioControlState;
    use crate::theme;

    #[test]
    fn audio_control_mock_exposes_professional_section_contract() {
        let state = AudioControlState::mock_running();

        assert_eq!(state.sections().len(), 11);
        assert_eq!(state.device.driver, "WASAPI");
        assert_eq!(state.routing.channels.len(), 2);
        assert_eq!(state.recording.active, false);
        assert_eq!(state.diagnostics.asr_queue, 0);
    }

    #[test]
    fn default_workspace_prioritizes_large_operator_stage() {
        let preset = WorkspacePreset::default_broadcast();

        assert!(preset.center_weight > preset.left_weight);
        assert!(preset.center_weight > preset.right_weight);
        assert_eq!(preset.right_tabs, ["Run Sheet", "Log", "Queue", "Detected"]);
        assert_eq!(preset.bottom_tabs, ["Transcript", "Metrics", "Events"]);
    }

    #[test]
    fn default_theme_uses_purple_graphite_accent() {
        assert_eq!(theme::THEME_NAME, "Purple Graphite");
        let accent = theme::accent();
        assert!(accent.b() > accent.g());
        assert!(accent.r() > accent.g());
    }
}
