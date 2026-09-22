//! Simple Digital Signal Processing (DSP) for audio enhancement.

use crate::AudioChunk;

pub struct NoiseGate {
    pub threshold: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    current_gain: f32,
}

impl Default for NoiseGate {
    fn default() -> Self {
        Self {
            threshold: 0.005, // Linear amplitude (approx -46 dBFS)
            attack_ms: 10.0,
            release_ms: 100.0,
            current_gain: 1.0,
        }
    }
}

impl NoiseGate {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            ..Default::default()
        }
    }

    pub fn process(&mut self, chunk: &mut AudioChunk) {
        // Attack/release coefficients for 16kHz
        let sample_rate = 16000.0;
        let attack_coef = (-(1000.0 / (self.attack_ms * sample_rate))).exp();
        let release_coef = (-(1000.0 / (self.release_ms * sample_rate))).exp();

        for sample in &mut chunk.samples {
            let s_f32 = *sample as f32 / 32768.0;
            let abs_s = s_f32.abs();
            
            let target_gain = if abs_s > self.threshold { 1.0 } else { 0.0 };
            
            if target_gain > self.current_gain {
                self.current_gain = attack_coef * self.current_gain + (1.0 - attack_coef) * target_gain;
            } else {
                self.current_gain = release_coef * self.current_gain + (1.0 - release_coef) * target_gain;
            }
            
            *sample = (s_f32 * self.current_gain * 32767.0) as i16;
        }
    }
}

pub struct HighPassFilter {
    pub frequency: f32,
    x1: f32,
    y1: f32,
}

impl Default for HighPassFilter {
    fn default() -> Self {
        Self::new(80.0) // 80Hz default
    }
}

impl HighPassFilter {
    pub fn new(frequency: f32) -> Self {
        Self {
            frequency,
            x1: 0.0,
            y1: 0.0,
        }
    }

    pub fn process(&mut self, chunk: &mut AudioChunk) {
        let sample_rate = 16000.0;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * self.frequency);
        let dt = 1.0 / sample_rate;
        let alpha = rc / (rc + dt);

        for sample in &mut chunk.samples {
            let x0 = *sample as f32;
            let y0 = alpha * (self.y1 + x0 - self.x1);
            
            self.x1 = x0;
            self.y1 = y0;
            
            *sample = y0.clamp(-32768.0, 32767.0) as i16;
        }
    }
}

pub struct Compressor {
    pub threshold: f32,
    pub ratio: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    pub makeup_gain: f32,
    current_gain: f32,
}

impl Default for Compressor {
    fn default() -> Self {
        Self {
            threshold: 0.1, // approx -20 dBFS
            ratio: 4.0,
            attack_ms: 5.0,
            release_ms: 50.0,
            makeup_gain: 1.0,
            current_gain: 1.0,
        }
    }
}

impl Compressor {
    pub fn process(&mut self, chunk: &mut AudioChunk) {
        let sample_rate = 16000.0;
        let attack_coef = (-(1000.0 / (self.attack_ms * sample_rate))).exp();
        let release_coef = (-(1000.0 / (self.release_ms * sample_rate))).exp();

        for sample in &mut chunk.samples {
            let s_f32 = *sample as f32 / 32768.0;
            let abs_s = s_f32.abs();
            
            let target_gain = if abs_s > self.threshold {
                let over = abs_s - self.threshold;
                let compressed = over / self.ratio;
                (self.threshold + compressed) / abs_s
            } else {
                1.0
            };
            
            if target_gain < self.current_gain {
                self.current_gain = attack_coef * self.current_gain + (1.0 - attack_coef) * target_gain;
            } else {
                self.current_gain = release_coef * self.current_gain + (1.0 - release_coef) * target_gain;
            }
            
            *sample = (s_f32 * self.current_gain * self.makeup_gain * 32767.0).clamp(-32768.0, 32767.0) as i16;
        }
    }
}
