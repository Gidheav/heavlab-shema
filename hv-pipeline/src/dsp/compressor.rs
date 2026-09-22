use super::AudioProcessor;

pub struct Compressor {
    enabled: bool,
    threshold_db: f32,
    ratio: f32,
    knee_width_db: f32,
    makeup_gain_linear: f32,
    
    attack_coef: f32,
    release_coef: f32,
    
    envelope_db: f32,
}

impl Compressor {
    pub fn new(
        threshold_db: f32,
        ratio: f32,
        attack_ms: f32,
        release_ms: f32,
        knee_db: f32,
        makeup_db: f32,
        sample_rate: u32,
        enabled: bool,
    ) -> Self {
        let attack_samples = (attack_ms / 1000.0 * sample_rate as f32).max(1.0);
        let attack_coef = (-1.0 / attack_samples).exp();
        
        let release_samples = (release_ms / 1000.0 * sample_rate as f32).max(1.0);
        let release_coef = (-1.0 / release_samples).exp();

        Self {
            enabled,
            threshold_db,
            ratio,
            knee_width_db: knee_db,
            makeup_gain_linear: 10f32.powf(makeup_db / 20.0),
            attack_coef,
            release_coef,
            envelope_db: 0.0,
        }
    }
    
    fn gain_computer(&self, input_db: f32) -> f32 {
        if self.knee_width_db <= 0.0 {
            // Hard knee
            if input_db > self.threshold_db {
                self.threshold_db + (input_db - self.threshold_db) / self.ratio
            } else {
                input_db
            }
        } else {
            // Soft knee
            let over_threshold = input_level_db(input_db, self.threshold_db);
            
            if 2.0 * over_threshold.abs() <= self.knee_width_db {
                // In the knee
                input_db + (1.0 / self.ratio - 1.0) * (input_db - self.threshold_db + self.knee_width_db / 2.0).powi(2) / (2.0 * self.knee_width_db)
            } else if over_threshold > self.knee_width_db / 2.0 {
                // Above the knee
                self.threshold_db + (input_db - self.threshold_db) / self.ratio
            } else {
                // Below the knee
                input_db
            }
        }
    }
}

fn input_level_db(input_db: f32, threshold_db: f32) -> f32 {
    input_db - threshold_db
}

impl AudioProcessor for Compressor {
    fn process(&mut self, buffer: &mut [f32]) -> usize {
        for sample in buffer.iter_mut() {
            let input_abs = sample.abs();
            let input_db = if input_abs > 1e-5 {
                20.0 * input_abs.log10()
            } else {
                -100.0
            };
            
            let target_gain_db = self.gain_computer(input_db) - input_db;
            
            if target_gain_db < self.envelope_db {
                // Attack (gain reduction increases)
                self.envelope_db = self.attack_coef * self.envelope_db + (1.0 - self.attack_coef) * target_gain_db;
            } else {
                // Release (gain reduction decreases)
                self.envelope_db = self.release_coef * self.envelope_db + (1.0 - self.release_coef) * target_gain_db;
            }
            
            let gain_linear = 10f32.powf(self.envelope_db / 20.0);
            *sample *= gain_linear * self.makeup_gain_linear;
        }
        buffer.len()
    }

    fn reset(&mut self) {
        self.envelope_db = 0.0;
    }

    fn name(&self) -> &'static str {
        "Compressor"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compressor_reduces_gain_above_threshold() {
        let mut comp = Compressor::new(
            -20.0, 4.0, 1.0, 100.0, 0.0, 0.0, 16000, true
        );
        
        let mut buffer = vec![1.0; 1000]; // 0 dBFS signal
        comp.process(&mut buffer);
        
        // Target should be -20 + (20 / 4) = -15 dBFS
        let expected_linear = 10f32.powf(-15.0 / 20.0);
        let actual_linear = buffer[999].abs();
        
        assert!((actual_linear - expected_linear).abs() < 0.05, "Expected {}, got {}", expected_linear, actual_linear);
    }
}
