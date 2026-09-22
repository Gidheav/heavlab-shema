//! Energy-based VAD (migrated from bible-asr `SimpleVad`).

use hv_audio::AudioChunk;

use crate::{Vad, VadConfig, VadState};

/// RMS-energy VAD for development and desktop fallback.
#[derive(Debug)]
pub struct EnergyVad {
    config: VadConfig,
    silence_started_ms: Option<u64>,
    speech_started_ms: Option<u64>,
}

impl EnergyVad {
    /// Create a detector from the full configuration.
    pub fn from_config(config: VadConfig) -> Self {
        Self {
            config,
            silence_started_ms: None,
            speech_started_ms: None,
        }
    }

    /// Convenience constructor.
    pub fn new(config: VadConfig) -> Self {
        Self::from_config(config)
    }

    /// RMS energy of a chunk. Kept identical to the bible-asr `SimpleVad` formula.
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

    fn silence_duration_ms(&self, now_ms: u64) -> u64 {
        self.silence_started_ms
            .map_or(0, |started| now_ms.saturating_sub(started))
    }

    fn speech_duration_ms(&self, now_ms: u64, chunk: &AudioChunk) -> u64 {
        match self.speech_started_ms {
            Some(started) => now_ms
                .saturating_sub(started)
                .max(chunk.duration_ms()),
            None => chunk.duration_ms(),
        }
    }
}

impl Default for EnergyVad {
    fn default() -> Self {
        Self::from_config(VadConfig::default())
    }
}

impl Vad for EnergyVad {
    fn process(&mut self, chunk: &AudioChunk) -> VadState {
        let timestamp_ms = chunk.timestamp_ms;
        if f64::from(Self::rms(chunk)) > self.config.energy_threshold {
            self.silence_started_ms = None;
            if self.speech_started_ms.is_none() {
                self.speech_started_ms = Some(timestamp_ms);
            }
            if self.speech_duration_ms(timestamp_ms, chunk) >= self.config.min_speech_ms {
                return VadState::Speech;
            }
            return VadState::Silence { duration_ms: 0 };
        }

        self.speech_started_ms = None;
        if self.silence_started_ms.is_none() {
            self.silence_started_ms = Some(timestamp_ms);
        }
        VadState::Silence {
            duration_ms: self.silence_duration_ms(timestamp_ms),
        }
    }

    fn reset(&mut self) {
        self.silence_started_ms = None;
        self.speech_started_ms = None;
    }

    fn config(&self) -> &VadConfig {
        &self.config
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::EnergyVad;
    use crate::{Vad, VadConfig, VadState};
    use hv_audio::AudioChunk;

    fn silent(timestamp_ms: u64) -> AudioChunk {
        AudioChunk::new(vec![0; 1600], timestamp_ms)
    }

    fn speech_block(timestamp_ms: u64, samples: usize, amplitude: i16) -> AudioChunk {
        AudioChunk::new(vec![amplitude; samples], timestamp_ms)
    }

    fn full_scale_sine(timestamp_ms: u64, samples: usize) -> AudioChunk {
        let mut pcm = Vec::with_capacity(samples);
        for i in 0..samples {
            let phase = 2.0 * std::f64::consts::PI * 440.0 * (i as f64) / 16_000.0;
            pcm.push((32767.0 * phase.sin()) as i16);
        }
        AudioChunk::new(pcm, timestamp_ms)
    }

    #[test]
    fn silence_input_is_silence() {
        let mut vad = EnergyVad::default();
        match vad.process(&silent(0)) {
            VadState::Silence { .. } => {}
            other => panic!("expected Silence, got {other:?}"),
        }
    }

    #[test]
    fn full_scale_sine_is_speech() {
        let mut vad = EnergyVad::default();
        // 16_000 samples = 1000 ms, above min_speech_ms (250).
        let state = vad.process(&full_scale_sine(0, 16_000));
        assert_eq!(state, VadState::Speech);
    }

    #[test]
    fn speech_then_silence_tracks_timeout() {
        let mut vad = EnergyVad::from_config(VadConfig::default());
        assert_eq!(
            vad.process(&speech_block(0, 16_000, 5000)),
            VadState::Speech
        );
        match vad.process(&silent(100)) {
            VadState::Silence { duration_ms } => assert_eq!(duration_ms, 0),
            other => panic!("expected start of silence, got {other:?}"),
        }
        match vad.process(&silent(1600)) {
            VadState::Silence { duration_ms } => {
                assert!(
                    duration_ms >= vad.config().silence_timeout_ms,
                    "duration_ms={duration_ms} timeout={}",
                    vad.config().silence_timeout_ms
                );
            }
            other => panic!("expected timed-out silence, got {other:?}"),
        }
    }

    #[test]
    fn threshold_changes_sensitivity() {
        let quiet = speech_block(0, 16_000, 50);
        let mut sensitive = EnergyVad::from_config(VadConfig {
            energy_threshold: 10.0,
            ..VadConfig::default()
        });
        let mut deaf = EnergyVad::from_config(VadConfig {
            energy_threshold: 1_000_000.0,
            ..VadConfig::default()
        });
        assert_eq!(sensitive.process(&quiet), VadState::Speech);
        match deaf.process(&quiet) {
            VadState::Silence { .. } => {}
            other => panic!("high threshold should stay silent, got {other:?}"),
        }
    }
}
