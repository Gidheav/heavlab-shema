//! TOML settings persisted across sessions.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;

use crate::layout::layout_state::LayoutState;
use crate::paths;

/// Defaults for the fields added when the left column was rebuilt. Named
/// functions rather than literals so `#[serde(default = "...")]` and
/// [`AppConfig::default`] can never disagree about a starting value.
fn default_lpf_frequency() -> f32 {
    8_000.0
}
fn default_deesser_frequency() -> f32 {
    6_000.0
}
fn default_monitor_device() -> String {
    "System Default".to_string()
}
fn default_headphone_device() -> String {
    "System Default".to_string()
}
fn default_recording_format() -> String {
    "WAV 24-bit".to_string()
}
fn default_preset_name() -> String {
    "Church Service".to_string()
}
fn default_diagnostics_interval() -> u32 {
    1_000
}
fn default_sample_rate() -> String {
    "48 kHz".to_string()
}
fn default_bit_depth() -> String {
    "24-bit".to_string()
}
fn default_buffer_size() -> String {
    "256".to_string()
}
fn default_agc_target_db() -> f32 {
    -18.0
}
fn default_agc_max_gain_db() -> f32 {
    20.0
}

/// One stored console setup. Only the parameters an operator actually dials in
/// during setup — not the whole config, so a preset stays readable in the TOML.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct SavedPreset {
    pub name: String,
    pub gain_db: f32,
    pub agc_enabled: bool,
    pub aec_enabled: bool,
    pub noise_gate_enabled: bool,
    pub vad_sensitivity: f32,
    pub led_hardware: bool,
    pub led_screen: bool,
}

impl Default for SavedPreset {
    fn default() -> Self {
        Self {
            name: "Custom".to_string(),
            gain_db: 0.0,
            agc_enabled: false,
            aec_enabled: false,
            noise_gate_enabled: false,
            vad_sensitivity: 0.5,
            led_hardware: true,
            led_screen: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub audio_device: Option<String>,
    pub gain: f32,
    pub translation: String,
    pub window_width: f32,
    pub window_height: f32,
    pub dark_mode: bool,
    pub asr_mode: String,
    pub asr_model_size: String,
    pub asr_model_id: String,
    pub vad_mode: String,
    pub vad_sensitivity: f32,
    pub noise_gate_enabled: bool,
    pub noise_gate_threshold: f32,
    pub hpf_enabled: bool,
    pub hpf_frequency: f32,
    pub compressor_enabled: bool,
    pub compressor_ratio: f32,
    // ── Left-column processors, owned by panels::audio::gain ──────────────────
    #[serde(default)]
    pub agc_enabled: bool,
    #[serde(default)]
    pub aec_enabled: bool,
    /// Where the AGC rides. A per-venue setting, so it lives on the config
    /// rather than on each saved preset.
    #[serde(default = "default_agc_target_db")]
    pub agc_target_db: f32,
    #[serde(default = "default_agc_max_gain_db")]
    pub agc_max_gain_db: f32,

    // ── Relocated out of the left column into Settings → Audio ───────────────
    #[serde(default)]
    pub lpf_enabled: bool,
    #[serde(default = "default_lpf_frequency")]
    pub lpf_frequency: f32,
    #[serde(default)]
    pub deesser_enabled: bool,
    #[serde(default = "default_deesser_frequency")]
    pub deesser_frequency: f32,
    #[serde(default = "default_monitor_device")]
    pub monitor_output_device: String,
    #[serde(default)]
    pub monitor_volume_db: f32,
    #[serde(default = "default_headphone_device")]
    pub headphone_output_device: String,
    #[serde(default)]
    pub headphone_volume_db: f32,
    #[serde(default = "default_recording_format")]
    pub recording_format: String,
    #[serde(default)]
    pub recording_auto: bool,
    /// Divider the operator's saved presets are stored under. See
    /// `panels::audio::preset`.
    #[serde(default = "default_preset_name")]
    pub preset_name: String,
    #[serde(default = "default_preset_name")]
    pub default_preset_name: String,
    #[serde(default)]
    pub saved_presets: Vec<SavedPreset>,
    #[serde(default = "default_sample_rate")]
    pub capture_sample_rate: String,
    #[serde(default = "default_bit_depth")]
    pub capture_bit_depth: String,
    #[serde(default = "default_buffer_size")]
    pub capture_buffer_size: String,

    // ── Relocated to Settings → Advanced ─────────────────────────────────────
    #[serde(default = "default_diagnostics_interval")]
    pub diagnostics_interval_ms: u32,
    #[serde(default)]
    pub experimental_features: bool,

    // ── Relocated to Settings → Developer ────────────────────────────────────
    #[serde(default)]
    pub debug_overlay: bool,
    #[serde(default)]
    pub verbose_logging: bool,

    pub use_gpu: bool,
    pub num_threads: usize,
    pub hotwords_enabled: bool,
    pub beam_size: usize,
    pub calibration_done: bool,
    #[serde(default, rename = "layoutState")]
    pub layout_state: LayoutState,
    #[serde(default)]
    pub theme_id: String,
    #[serde(default)]
    pub theme_mode: String, // "dark", "light", "follow_system"
    #[serde(default)]
    pub font_config: Option<crate::theme::FontConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            audio_device: None,
            gain: 1.0,
            translation: "KJV".to_string(),
            window_width: 1200.0,
            window_height: 800.0,
            dark_mode: true,
            asr_mode: "sherpa".to_string(),
            asr_model_size: "medium".to_string(),
            asr_model_id: "sherpa-zipformer-en".to_string(),
            vad_mode: "hybrid".to_string(),
            vad_sensitivity: 0.5,
            noise_gate_enabled: false,
            noise_gate_threshold: 0.005,
            hpf_enabled: false,
            hpf_frequency: 80.0,
            compressor_enabled: false,
            compressor_ratio: 4.0,
            agc_enabled: false,
            aec_enabled: false,
            agc_target_db: default_agc_target_db(),
            agc_max_gain_db: default_agc_max_gain_db(),
            lpf_enabled: false,
            lpf_frequency: default_lpf_frequency(),
            deesser_enabled: false,
            deesser_frequency: default_deesser_frequency(),
            monitor_output_device: default_monitor_device(),
            monitor_volume_db: 0.0,
            headphone_output_device: default_headphone_device(),
            headphone_volume_db: 0.0,
            recording_format: default_recording_format(),
            recording_auto: false,
            preset_name: default_preset_name(),
            default_preset_name: default_preset_name(),
            saved_presets: Vec::new(),
            capture_sample_rate: default_sample_rate(),
            capture_bit_depth: default_bit_depth(),
            capture_buffer_size: default_buffer_size(),
            diagnostics_interval_ms: default_diagnostics_interval(),
            experimental_features: false,
            debug_overlay: false,
            verbose_logging: false,
            use_gpu: false,
            num_threads: 4,
            hotwords_enabled: false,
            beam_size: 4,
            calibration_done: false,
            layout_state: LayoutState::default(),
            theme_id: "purple_graphite_dark".to_string(),
            theme_mode: "dark".to_string(),
            font_config: None,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let path = paths::config_path();
        let Ok(raw) = fs::read_to_string(&path) else {
            return Self::default();
        };
        let mut config: Self = toml::from_str(&raw).unwrap_or_else(|error| {
            tracing::warn!("config parse failed ({path:?}): {error}");
            Self::default()
        });
        
        // Ensure settings window is not persisted as open across restarts
        config.layout_state.settings_open = false;
        config
    }

    pub fn save(&self) -> Result<(), io::Error> {
        let path = paths::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let body = toml::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(path, body)
    }
}

#[cfg(test)]
mod tests {
    use super::AppConfig;

    #[test]
    fn default_config_contains_required_layout_state() {
        let config = AppConfig::default();

        assert_eq!(config.layout_state.left_width, 340.0);
        assert_eq!(config.layout_state.right_width, 320.0);
        assert_eq!(config.layout_state.middle_bottom_height, 172.0);
        assert!(!config.layout_state.left_collapsed);
        assert!(!config.layout_state.right_collapsed);
        assert!(!config.layout_state.middle_bottom_collapsed);
        assert!(!config.layout_state.ribbon_collapsed);
    }

    #[test]
    fn existing_config_without_layout_state_uses_layout_defaults() {
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
"#;

        let config: AppConfig = toml::from_str(raw).expect("old config should parse");

        assert_eq!(config.layout_state.left_width, 340.0);
        assert_eq!(config.layout_state.right_width, 320.0);
        assert_eq!(config.layout_state.middle_bottom_height, 172.0);
    }
}
