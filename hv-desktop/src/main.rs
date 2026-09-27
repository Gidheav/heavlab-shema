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

        assert_eq!(state.device.driver, "WASAPI");
        assert_eq!(state.routing.channels.len(), 2);
        assert_eq!(state.recording.active, false);
        assert_eq!(state.diagnostics.asr_queue, 0);
    }

    /// The processing chain and gain staging sections read live config and the
    /// real pipeline, not the mock, so the mock must not re-declare DSP state
    /// that could drift out of sync with `AppConfig`.
    #[test]
    fn audio_mock_does_not_duplicate_dsp_state() {
        let state = AudioControlState::mock_running();

        // Gain staging only mirrors the linear->dB projection; the HPF and
        // processor chain live on `AppConfig` and are driven by PipelineCommand.
        assert_eq!(state.gain.digital_trim_db, 0.0);
        assert!(state.gain.input_gain_db > 0.0);
    }

    /// Guards the "one editor per setting" rule for capture gain.
    ///
    /// `app.gain` is a *linear multiplier*; the pipeline clamps it to
    /// `0.0..=4.0` (worker.rs). Every UI surface must therefore speak
    /// decibels through `gain_db()` / `set_gain_db()`, otherwise a surface can
    /// display `+24.0 dB` while the pipeline silently clamps to `4.0x`
    /// (`+12.0 dB`). This test walks the real source files and fails if any
    /// module outside `app.rs` mutates the linear gain directly or bypasses
    /// the helpers.
    #[test]
    fn capture_gain_has_a_single_decibel_conversion_path() {
        let offenders: Vec<String> = [
            "src/panels/audio/gain.rs",
            "src/panels/audio/mod.rs",
            "src/panels/settings_window.rs",
            "src/shell/ribbon.rs",
            "src/shell/workspace.rs",
        ]
        .iter()
        .filter_map(|relative| {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
            let source = std::fs::read_to_string(&path).ok()?;
            let bad = source.contains("app.gain =") || source.contains("PipelineCommand::SetGain");
            bad.then(|| relative.to_string())
        })
        .collect();

        assert!(
            offenders.is_empty(),
            "these modules bypass gain_db()/set_gain_db() and can drift from the \
             audio console: {offenders:?}"
        );
    }

    /// The left console owns the audio controls; the ribbon AUDIO tab is
    /// read-only for everything it owns. If a duplicate editor reappears in the
    /// deck, the two surfaces can hold different values for one setting.
    #[test]
    fn ribbon_audio_tab_does_not_duplicate_console_controls() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/shell/ribbon.rs");
        let source = std::fs::read_to_string(&path).expect("read ribbon.rs");

        let start = source.find("fn tab_audio(").expect("tab_audio exists");
        let end = source[start..]
            .find("\nfn tab_tools(")
            .map(|offset| start + offset)
            .expect("tab_tools follows tab_audio");
        let body = &source[start..end];

        for forbidden in [
            "selected_device",  // device selection
            "devices",          // device list
            "gain_db",          // gain staging
            "set_gain_db",
            "hpf_enabled",      // signal chain
            "noise_gate_enabled",
            "monitoring.muted", // monitoring mute/solo
            "monitoring.solo",
        ] {
            assert!(
                !body.contains(forbidden),
                "ribbon AUDIO tab must not edit `{forbidden}` — that control is owned \
                 by the left audio console (panels::audio)"
            );
        }
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
