//! hv-vad — Voice activity detection.
//!
//! Decides WHEN someone is speaking. Does not capture audio or parse verses.

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod energy;

pub use energy::EnergyVad;

use hv_audio::AudioChunk;

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
}
