//! Voice activity detection and gating.

use crate::capture::AudioChunk;

/// The coarse state emitted by a voice activity detector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadState {
    /// Speech is currently present.
    Speaking,
    /// Silence has lasted long enough to sleep ASR.
    Silence,
    /// Silence is present but has not reached the sleep threshold.
    Transitioning,
}

/// VAD thresholds used by the worker.
#[derive(Debug, Clone, Copy)]
pub struct VadConfig {
    /// RMS threshold used by [`SimpleVad`].
    pub speech_threshold: f32,
    /// Silence before ASR is put to sleep.
    pub silence_ms: u64,
    /// Continuous silence before the UI is notified.
    pub pause_notify_ms: u64,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            speech_threshold: 1000.0,
            silence_ms: 1500,
            pause_notify_ms: 30_000,
        }
    }
}

/// A detector that gates ASR based on chunk RMS energy.
pub trait Vad: Send {
    fn process(&mut self, chunk: &AudioChunk, timestamp_ms: u64) -> VadState;
    fn reset(&mut self);
}

/// Simple energy-based VAD for development and desktop fallback.
#[derive(Debug)]
pub struct SimpleVad {
    config: VadConfig,
    silence_started_ms: Option<u64>,
    state: VadState,
}

impl SimpleVad {
    /// Create a VAD from the full configuration.
    pub fn from_config(config: VadConfig) -> Self {
        Self {
            config,
            silence_started_ms: None,
            state: VadState::Silence,
        }
    }

    /// Compatibility constructor for existing callers.
    pub fn new(threshold: f32, silence_duration_ms: u64) -> Self {
        Self::from_config(VadConfig {
            speech_threshold: threshold,
            silence_ms: silence_duration_ms,
            ..VadConfig::default()
        })
    }

    /// Current state.
    pub fn state(&self) -> VadState {
        self.state
    }

    /// Process a chunk using this VAD.
    ///
    /// This inherent shim keeps callers that used `SimpleVad` before the
    /// trait split source-compatible without requiring a trait import.
    pub fn process(&mut self, chunk: &AudioChunk, timestamp_ms: u64) -> VadState {
        <Self as Vad>::process(self, chunk, timestamp_ms)
    }

    /// Reset this VAD.
    pub fn reset(&mut self) {
        <Self as Vad>::reset(self);
    }

    /// Continuous silence duration at the supplied timestamp.
    pub fn silence_duration_ms(&self, now_ms: u64) -> u64 {
        self.silence_started_ms
            .map_or(0, |started| now_ms.saturating_sub(started))
    }

    fn rms(chunk: &AudioChunk) -> f32 {
        if chunk.samples.is_empty() {
            return 0.0;
        }
        let sum_sq: u64 = chunk
            .samples
            .iter()
            .map(|&sample| {
                let value = i64::from(sample);
                (value * value) as u64
            })
            .sum();
        (sum_sq as f64 / chunk.samples.len() as f64).sqrt() as f32
    }
}

impl Vad for SimpleVad {
    fn process(&mut self, chunk: &AudioChunk, timestamp_ms: u64) -> VadState {
        if Self::rms(chunk) > self.config.speech_threshold {
            self.silence_started_ms = None;
            self.state = VadState::Speaking;
            return self.state;
        }

        if self.silence_started_ms.is_none() {
            self.silence_started_ms = Some(timestamp_ms);
        }
        let silence_duration = self.silence_duration_ms(timestamp_ms);
        if silence_duration >= self.config.silence_ms {
            self.state = VadState::Silence;
        } else {
            self.state = VadState::Transitioning;
        }
        self.state
    }

    fn reset(&mut self) {
        self.silence_started_ms = None;
        self.state = VadState::Silence;
    }
}

impl Default for SimpleVad {
    fn default() -> Self {
        Self::from_config(VadConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::{SimpleVad, VadConfig, VadState};
    use crate::capture::AudioChunk;

    fn silent(timestamp_ms: u64) -> AudioChunk {
        AudioChunk::new(vec![0; 1600], timestamp_ms)
    }

    fn speech(timestamp_ms: u64) -> AudioChunk {
        AudioChunk::new(vec![5000; 1600], timestamp_ms)
    }

    #[test]
    fn threshold_crossing() {
        let mut vad = SimpleVad::new(1000.0, 1500);
        assert_eq!(vad.process(&silent(0), 0), VadState::Transitioning);
        assert_eq!(vad.process(&speech(100), 100), VadState::Speaking);
    }

    #[test]
    fn transition_becomes_silence_after_threshold() {
        let mut vad = SimpleVad::new(1000.0, 1500);
        vad.process(&speech(1000), 1000);
        assert_eq!(vad.process(&silent(1100), 1100), VadState::Transitioning);
        assert_eq!(vad.process(&silent(2600), 2600), VadState::Silence);
    }

    #[test]
    fn silence_duration_accumulates_and_resets_on_speech() {
        let mut vad = SimpleVad::from_config(VadConfig::default());
        vad.process(&silent(500), 500);
        assert_eq!(vad.silence_duration_ms(2500), 2000);
        vad.process(&speech(3000), 3000);
        assert_eq!(vad.silence_duration_ms(4000), 0);
    }

    #[test]
    fn decreasing_timestamps_do_not_panic() {
        let mut vad = SimpleVad::new(1000.0, 1500);
        vad.process(&speech(2000), 2000);
        assert_eq!(vad.process(&silent(1000), 1000), VadState::Transitioning);
        assert_eq!(vad.silence_duration_ms(500), 0);
    }
}
