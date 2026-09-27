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

    // ── Left column contracts ────────────────────────────────────────────────
    //
    // The left column was rebuilt from scratch. These tests are why the rebuild
    // is safe to keep: each states a property the column must not lose, in a
    // form that fails loudly if a future change breaks it.

    /// Disclosure state must survive a restart, which means it belongs in the
    /// config rather than in egui's per-session memory.
    #[test]
    fn section_disclosure_state_is_persisted_in_the_config() {
        let config = crate::config::AppConfig::default();
        let mut state = config.layout_state.audio_sections;
        state.gain = !state.gain;
        state.led = !state.led;

        let layout = crate::layout::layout_state::LayoutState {
            audio_sections: state,
            ..config.layout_state.clone()
        };

        // Round-trip through TOML, which is what a restart actually does.
        let encoded = toml::to_string(&layout).expect("layout state serialises");
        let decoded: crate::layout::layout_state::LayoutState =
            toml::from_str(&encoded).expect("layout state deserialises");

        assert_eq!(decoded.audio_sections.gain, state.gain);
        assert_eq!(decoded.audio_sections.led, state.led);
    }

    /// Every setting the rebuild moved out of the left column must still be
    /// reachable, and persisted. A control that is nowhere is a lost feature.
    #[test]
    fn every_relocated_setting_is_still_persisted() {
        let config = crate::config::AppConfig::default();
        let body = toml::to_string(&config).expect("config serialises");

        for field in [
            // Relocated into Settings → Audio.
            "lpf_enabled",
            "lpf_frequency",
            "deesser_enabled",
            "monitor_output_device",
            "monitor_volume_db",
            "headphone_output_device",
            "headphone_volume_db",
            "capture_sample_rate",
            "capture_bit_depth",
            "capture_buffer_size",
            "preset_name",
            "saved_presets",
            "agc_target_db",
            "agc_max_gain_db",
            // Relocated into Settings → Advanced.
            "diagnostics_interval_ms",
            "experimental_features",
            // Relocated into Settings → Developer.
            "debug_overlay",
            "verbose_logging",
        ] {
            assert!(body.contains(field), "{field} is no longer persisted");
        }
    }

    /// A config written before the rebuild has no `audioSections` key at all. It
    /// must load with the documented defaults rather than fail to parse, or every
    /// existing operator's settings would be lost on upgrade.
    #[test]
    fn a_pre_rebuild_config_loads_with_default_sections() {
        let raw = r#"
audio_device = "Default Input"
gain = 1.0
translation = "KJV"
window_width = 1200.0
window_height = 800.0
dark_mode = true
asr_mode = "sherpa"
asr_model_size = "medium"
asr_model_id = "sherpa-zipformer-en"
vad_mode = "hybrid"
vad_sensitivity = 0.5
noise_gate_enabled = false
noise_gate_threshold = 0.005
hpf_enabled = false
hpf_frequency = 80.0
compressor_enabled = false
compressor_ratio = 4.0
use_gpu = false
num_threads = 4
hotwords_enabled = true
beam_size = 4
calibration_done = false
theme_id = "purple_graphite_dark"
theme_mode = "dark"

[layoutState]
leftCollapsed = false
leftWidth = 340.0
rightCollapsed = false
rightWidth = 320.0
middleBottomCollapsed = false
middleBottomHeight = 172.0
ribbonCollapsed = false
"#;

        let config: crate::config::AppConfig =
            toml::from_str(raw).expect("pre-rebuild config still parses");
        let sections = config.layout_state.audio_sections;

        // Input device and level start open: they are what an operator checks
        // before every service.
        assert!(sections.input_device);
        assert!(sections.input_level);
        // Everything else is deliberate.
        assert!(!sections.gain);
        assert!(!sections.voice);
        assert!(!sections.preset);
        assert!(!sections.led);
    }

    /// Every source file the left column is built from, by name. Shared by the
    /// two contracts below so they cannot drift apart.
    const COLUMN_SOURCES: &[&str] = &[
        "src/panels/audio/mod.rs",
        "src/panels/audio/device.rs",
        "src/panels/audio/level.rs",
        "src/panels/audio/gain.rs",
        "src/panels/audio/voice.rs",
        "src/panels/audio/preset.rs",
        "src/panels/audio/led.rs",
        "src/panels/audio/transport.rs",
        "src/components/column_controls.rs",
        "src/components/column_density.rs",
        "src/components/panel_section.rs",
        "src/components/channel_meter.rs",
    ];

    fn read_column_source(relative: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{relative} is unreadable: {error}"))
    }

    /// The column must offer exactly one editor per setting. If a control
    /// reappears on the main screen while Settings still owns it, the two
    /// surfaces can hold different values for the same knob — the exact failure
    /// the rebuild was meant to end.
    #[test]
    fn the_left_column_does_not_duplicate_settings_owned_by_settings() {
        let relocated = [
            "hpf_frequency",
            "lpf_frequency",
            "compressor_ratio",
            "deesser_frequency",
            "noise_gate_threshold",
            "routing.input",
            "routing.processing",
            "routing.phase",
            "monitoring.output_device",
            "monitoring.level_db",
            "recording.active",
            "recording.format",
            "diagnostics.cpu_percent",
            "diagnostics.jitter_ms",
            "capture_sample_rate",
            "capture_buffer_size",
        ];

        let mut offenders: Vec<String> = Vec::new();
        for relative in COLUMN_SOURCES {
            let source = read_column_source(relative);
            for setting in relocated {
                if source.contains(setting) {
                    offenders.push(format!("{relative} edits `{setting}`"));
                }
            }
        }

        assert!(
            offenders.is_empty(),
            "these controls belong in the Settings window, not the live surface: \
             {offenders:?}"
        );
    }

    /// The column must never take a fixed pixel width. A hardcoded width inside a
    /// resizable column is precisely how a horizontal scrollbar gets in, and the
    /// brief forbids one appearing under any circumstance.
    #[test]
    fn the_left_column_takes_no_fixed_element_widths() {
        let mut offenders: Vec<String> = Vec::new();
        for relative in COLUMN_SOURCES {
            for line in read_column_source(relative).lines() {
                let trimmed = line.trim_start();
                if trimmed.starts_with("//") {
                    continue;
                }
                // A literal first argument to `add_sized` / `vec2` is a width
                // the author chose instead of the width the column gave them.
                for marker in ["add_sized([", "add_sized(egui::vec2(", "vec2("] {
                    if let Some(rest) = trimmed.split(marker).nth(1) {
                        if rest.trim_start().starts_with(|c: char| c.is_ascii_digit()) {
                            offenders.push(format!("{relative}: {trimmed}"));
                        }
                    }
                }
            }
        }

        assert!(
            offenders.is_empty(),
            "these column elements hardcode a width instead of responding to the \
             parent: {offenders:?}"
        );
    }

    /// Renders the column's primitives inside a container of exactly `width`
    /// and reports the right edge the content actually reached.
    ///
    /// A widget that asked for more room than it was given makes `Ui::min_rect`
    /// grow past the container — which is the mechanism behind both a clipped
    /// control and a horizontal scrollbar. Asserting on `min_rect` catches the
    /// cause rather than the symptom.
    fn probe_column_width(width: f32) -> f32 {
        use crate::components::channel_meter::ChannelMeter;
        use crate::components::column_controls::{readout, value_bar, InlineButton, InlineToggle};
        use crate::components::column_density::ColumnDensity;
        use crate::components::panel_section::{PanelSection, SectionStatus};

        let context = egui::Context::default();
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(width, 900.0),
            )),
            ..Default::default()
        };

        let mut reached: f32 = 0.0;
        let _ = context.run(raw, |ctx| {
            egui::SidePanel::right("probe")
                .exact_width(width)
                .resizable(false)
                .frame(egui::Frame::none())
                .show(ctx, |ui| {
                    let density = ColumnDensity::from_width(ui.available_width());

                    let mut flag = false;
                    let mut open = true;

                    PanelSection::new("Input Level", "▤", &mut open, density)
                        .with_status(SectionStatus {
                            color: crate::theme::STATUS_ERROR,
                            label: "CLIP",
                        })
                        .show(ui, |ui| {
                            for channel in ["L", "R"] {
                                ChannelMeter {
                                    channel,
                                    rms_db: -3.5,
                                    peak_hold_db: -1.0,
                                    clipped: true,
                                    density,
                                }
                                .show(ui);
                            }
                            readout(ui, "Signal to noise", "30 dB");
                            value_bar(ui, 0.9, crate::theme::accent(), 6.0);
                            InlineToggle::new("AGC", &mut flag, density).show(ui);
                            InlineButton::new("Rescan", density).icon("↻").show(ui);
                        });

                    reached = ui.min_rect().right();
                });
        });
        reached
    }

    /// Acceptance criterion: the new left column demonstrates correct
    /// responsiveness at 200 px, 300 px, and 400 px. Nothing may exceed the
    /// width it was given at any of them — which is the definition of "no
    /// horizontal scrollbar ever appears".
    ///
    /// The lower bound matters as much as the upper one: a column that rendered
    /// nothing would sail past the overflow check, so each case also asserts the
    /// content actually used most of the width it was handed.
    #[test]
    fn the_column_never_exceeds_the_width_it_was_given() {
        for width in [200.0_f32, 300.0, 400.0] {
            let reached = probe_column_width(width);
            assert!(
                reached <= width + 0.5,
                "at {width} px the column reached {reached} px — it overflowed"
            );
            assert!(
                reached > width * 0.6,
                "at {width} px the column only reached {reached} px — \
                 it rendered less than the room it was given, so the overflow \
                 check above proved nothing"
            );
        }
    }

    /// The same must hold in icon-only mode, which is what a column narrower
    /// than 200 px collapses to.
    #[test]
    fn the_column_never_overflows_below_the_icon_only_breakpoint() {
        for width in [120.0_f32, 160.0, 199.0] {
            let reached = probe_column_width(width);
            assert!(
                reached <= width + 0.5,
                "at {width} px the column reached {reached} px — it overflowed"
            );
            assert!(
                reached > width * 0.6,
                "at {width} px the column only reached {reached} px — \
                 it rendered less than the room it was given"
            );
        }
    }
}
