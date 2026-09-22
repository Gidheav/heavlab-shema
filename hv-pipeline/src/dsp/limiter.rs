use super::AudioProcessor;

pub struct Limiter {
    enabled: bool,
    threshold_linear: f32,
    release_coef: f32,
    envelope: f32,
}

impl Limiter {
    pub fn new(
        threshold_db: f32,
        release_ms: f32,
        sample_rate: u32,
        enabled: bool,
    ) -> Self {
        let release_samples = (release_ms / 1000.0 * sample_rate as f32).max(1.0);
        let release_coef = (-1.0 / release_samples).exp();

        Self {
            enabled,
            threshold_linear: 10f32.powf(threshold_db / 20.0),
            release_coef,
            envelope: 0.0,
        }
    }
}

impl AudioProcessor for Limiter {
    fn process(&mut self, buffer: &mut [f32]) -> usize {
        for sample in buffer.iter_mut() {
            let input_abs = sample.abs();
            
            // Peak envelope follower with instant attack
            if input_abs > self.envelope {
                self.envelope = input_abs;
            } else {
                self.envelope = self.release_coef * self.envelope + (1.0 - self.release_coef) * input_abs;
            }
            
            let mut gain = 1.0;
            if self.envelope > self.threshold_linear {
                gain = self.threshold_linear / self.envelope;
            }
            
            *sample *= gain;
            
            // Hard clip just in case to guarantee never exceeding threshold
            if *sample > self.threshold_linear {
                *sample = self.threshold_linear;
            } else if *sample < -self.threshold_linear {
                *sample = -self.threshold_linear;
            }
        }
        buffer.len()
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
    }

    fn name(&self) -> &'static str {
        "Limiter"
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
    fn test_limiter_prevents_clipping() {
        let threshold_db = -1.0;
        let mut limiter = Limiter::new(threshold_db, 50.0, 16000, true);
        
        let mut buffer = vec![1.0; 100]; // 0 dBFS signal (above -1 dBFS threshold)
        limiter.process(&mut buffer);
        
        let threshold_linear = 10f32.powf(threshold_db / 20.0);
        
        for &sample in buffer.iter() {
            assert!(sample.abs() <= threshold_linear + 1e-5);
        }
    }
}
