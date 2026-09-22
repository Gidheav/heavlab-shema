//! Real-time VU / peak metering. Intended for the DSP thread.

use crate::capture::AudioChunk;

const FULL_SCALE: f32 = 32768.0;
const MIN_DB: f32 = -96.0;
const CLIP_THRESHOLD: i16 = (0.95 * 32767.0) as i16;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeterReading {
    /// RMS level in dBFS (-96.0 to 0.0).
    pub rms_db: f32,
    /// Peak level in dBFS.
    pub peak_db: f32,
    /// True if any sample is at or above 0.95 * i16::MAX.
    pub is_clipping: bool,
}

#[derive(Debug, Clone)]
pub struct AudioMeter {
    peak_hold: f32,
    decay_rate: f32,
}

impl AudioMeter {
    pub fn new() -> Self {
        Self {
            peak_hold: 0.0,
            decay_rate: 0.85,
        }
    }

    pub fn process(&mut self, chunk: &AudioChunk) -> MeterReading {
        if chunk.samples.is_empty() {
            self.peak_hold *= self.decay_rate;
            return MeterReading {
                rms_db: MIN_DB,
                peak_db: to_db(self.peak_hold),
                is_clipping: false,
            };
        }

        let mut sum_sq: f64 = 0.0;
        let mut peak: f32 = 0.0;
        let mut is_clipping = false;
        for &sample in &chunk.samples {
            let abs = sample.unsigned_abs() as f32;
            if abs > peak {
                peak = abs;
            }
            if sample.saturating_abs() >= CLIP_THRESHOLD {
                is_clipping = true;
            }
            let value = f64::from(sample);
            sum_sq += value * value;
        }

        let rms = (sum_sq / chunk.samples.len() as f64).sqrt() as f32;
        if peak > self.peak_hold {
            self.peak_hold = peak;
        } else {
            self.peak_hold *= self.decay_rate;
        }

        MeterReading {
            rms_db: to_db(rms),
            peak_db: to_db(self.peak_hold),
            is_clipping,
        }
    }

    pub fn reset(&mut self) {
        self.peak_hold = 0.0;
    }
}

impl Default for AudioMeter {
    fn default() -> Self {
        Self::new()
    }
}

fn to_db(linear: f32) -> f32 {
    if linear <= 0.0 {
        return MIN_DB;
    }
    (20.0 * (linear / FULL_SCALE).log10()).clamp(MIN_DB, 0.0)
}

#[cfg(test)]
mod tests {
    use super::{AudioMeter, MIN_DB};
    use crate::capture::AudioChunk;

    #[test]
    fn silence_is_minus_96_db() {
        let mut meter = AudioMeter::new();
        let reading = meter.process(&AudioChunk::new(vec![0; 1600], 0));
        assert!((reading.rms_db - MIN_DB).abs() < 0.01);
        assert!(!reading.is_clipping);
    }

    #[test]
    fn full_scale_is_near_zero_db() {
        let mut meter = AudioMeter::new();
        let reading = meter.process(&AudioChunk::new(vec![i16::MAX; 1600], 0));
        assert!(
            reading.rms_db > -0.05 && reading.rms_db <= 0.0,
            "rms_db={}",
            reading.rms_db
        );
        assert!(reading.is_clipping);
    }

    #[test]
    fn half_scale_is_about_minus_six_db() {
        let mut meter = AudioMeter::new();
        let reading = meter.process(&AudioChunk::new(vec![16384; 1600], 0));
        assert!(
            (reading.rms_db + 6.02).abs() < 0.15,
            "rms_db={}",
            reading.rms_db
        );
        assert!(!reading.is_clipping);
    }
}
