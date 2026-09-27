//! hv-vad — Voice activity detection.
//!
//! Decides WHEN someone is speaking. Does not capture audio or parse verses.

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod energy;

pub use energy::EnergyVad;

use hv_audio::AudioChunk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VadError {
    #[error("ONNX error: {0}")]
    Onnx(String),
    #[error("Invalid model path: {0}")]
    ModelPath(String),
}

/// Coarse detector output consumed by the pipeline worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadState {
    /// Speech is present and has lasted at least [`VadConfig::min_speech_ms`].
    Speech,
    /// Energy is below threshold; `duration_ms` is continuous silence so far.
    Silence { duration_ms: u64 },
}

/// Energy-VAD thresholds used by [`EnergyVad`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VadConfig {
    /// RMS energy that must be exceeded to count as speech.
    pub energy_threshold: f64,
    /// Silence duration after which ASR should sleep.
    pub silence_timeout_ms: u64,
    /// Speech must last at least this long before [`VadState::Speech`].
    pub min_speech_ms: u64,
}

impl VadConfig {
    /// RMS energy threshold for a 0.0–1.0 sensitivity.
    ///
    /// Swept geometrically so the mid-point lands exactly on the 1000.0 default
    /// and the response feels even across the slider in decibels, not in
    /// amplitude. 0.0 is permissive enough to catch a soft preacher; 1.0 is
    /// strict enough to ignore room tone.
    pub fn energy_threshold_for(sensitivity: f32) -> f64 {
        const MOST_PERMISSIVE: f64 = 250.0;
        const MOST_STRICT: f64 = 4_000.0;
        let sweep = sensitivity.clamp(0.0, 1.0) as f64;
        MOST_PERMISSIVE * (MOST_STRICT / MOST_PERMISSIVE).powf(sweep)
    }
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            energy_threshold: 1000.0,
            silence_timeout_ms: 1500,
            min_speech_ms: 250,
        }
    }
}

/// Voice-activity detector. Must be `Send` so the worker thread can own it.
pub trait Vad: Send + 'static {
    fn process(&mut self, chunk: &AudioChunk) -> VadState;
    fn reset(&mut self);
    fn config(&self) -> &VadConfig;
    /// Retune the detector live, 0.0 (permissive) .. 1.0 (strict).
    ///
    /// Defaults to a no-op so a detector with no notion of a continuous
    /// sensitivity knob keeps compiling; only the UI slider depends on it.
    fn set_sensitivity(&mut self, _sensitivity: f32) {}
}
