use super::AudioProcessor;

pub struct Agc {
    enabled: bool,
    target_linear: f32,
    max_gain_linear: f32,
    attack_coef: f32,
    release_coef: f32,
    
    envelope: f32,
    current_gain: f32,
}

impl Agc {
    pub fn new(
        target_db: f32,
        max_gain_db: f32,
        attack_ms: f32,
        release_ms: f32,
        sample_rate: u32,
        enabled: bool,
    ) -> Self {
        let attack_samples = (attack_ms / 1000.0 * sample_rate as f32).max(1.0);
        let attack_coef = (-1.0 / attack_samples).exp();
        
        let release_samples = (release_ms / 1000.0 * sample_rate as f32).max(1.0);
        let release_coef = (-1.0 / release_samples).exp();

        Self {
            enabled,
            target_linear: 10f32.powf(target_db / 20.0),
            max_gain_linear: 10f32.powf(max_gain_db / 20.0),
            attack_coef,
            release_coef,
            envelope: 0.0,
            current_gain: 1.0,
        }
    }
}

impl AudioProcessor for Agc {
    fn process(&mut self, buffer: &mut [f32]) -> usize {
        for sample in buffer.iter_mut() {
            let input_abs = sample.abs();
            
            // Envelope follower for input level
            if input_abs > self.envelope {
                self.envelope = self.attack_coef * self.envelope + (1.0 - self.attack_coef) * input_abs;
            } else {
                self.envelope = self.release_coef * self.envelope + (1.0 - self.release_coef) * input_abs;
            }
            
            let mut target_gain = if self.envelope > 1e-5 {
                self.target_linear / self.envelope
            } else {
                self.max_gain_linear
            };
            
            target_gain = target_gain.clamp(1.0, self.max_gain_linear);
            
            // Smoothly adjust current gain towards target gain using same release/attack behavior but reversed
            // since this is a gain multiplier
            if target_gain < self.current_gain {
                // Attacking (reducing gain)
                self.current_gain = self.attack_coef * self.current_gain + (1.0 - self.attack_coef) * target_gain;
            } else {
                // Releasing (increasing gain)
                self.current_gain = self.release_coef * self.current_gain + (1.0 - self.release_coef) * target_gain;
            }
            
            *sample *= self.current_gain;
        }
        buffer.len()
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
        self.current_gain = 1.0;
    }

    fn name(&self) -> &'static str {
        "Agc"
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
    fn test_agc_boosts_signal() {
        let mut agc = Agc::new(-18.0, 20.0, 10.0, 200.0, 16000, true);
        
        let input_db = -40.0;
        let mut buffer = vec![10f32.powf(input_db / 20.0); 16000]; // 1 second of -40dB signal
        
        agc.process(&mut buffer);
        
        // After 1 second, it should have boosted it significantly
        let last_sample = buffer[15999].abs();
        let target_linear = 10f32.powf(-20.0 / 20.0); // clamped by max gain of 20dB
        
        assert!(last_sample > target_linear * 0.9);
    }
}
