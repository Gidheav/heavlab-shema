//! TOML settings persisted across sessions.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;

use crate::layout::layout_state::LayoutState;
use crate::paths;

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
    pub vad_mode: String,
    pub vad_sensitivity: f32,
    pub noise_gate_enabled: bool,
    pub noise_gate_threshold: f32,
    pub hpf_enabled: bool,
    pub hpf_frequency: f32,
    pub compressor_enabled: bool,
    pub compressor_ratio: f32,
    pub use_gpu: bool,
    pub num_threads: usize,
    pub hotwords_enabled: bool,
    pub beam_size: usize,
    pub calibration_done: bool,
    #[serde(default, rename = "layoutState")]
    pub layout_state: LayoutState,
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
            vad_mode: "hybrid".to_string(),
            vad_sensitivity: 0.5,
            noise_gate_enabled: false,
            noise_gate_threshold: 0.005,
            hpf_enabled: false,
            hpf_frequency: 80.0,
            compressor_enabled: false,
            compressor_ratio: 4.0,
            use_gpu: false,
            num_threads: 4,
            hotwords_enabled: true,
            beam_size: 4,
            calibration_done: false,
            layout_state: LayoutState::default(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let path = paths::config_path();
        let Ok(raw) = fs::read_to_string(&path) else {
            return Self::default();
        };
        toml::from_str(&raw).unwrap_or_else(|error| {
            tracing::warn!("config parse failed ({path:?}): {error}");
            Self::default()
        })
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
"#;

        let config: AppConfig = toml::from_str(raw).expect("old config should parse");

        assert_eq!(config.layout_state.left_width, 340.0);
        assert_eq!(config.layout_state.right_width, 320.0);
        assert_eq!(config.layout_state.middle_bottom_height, 172.0);
    }
}
