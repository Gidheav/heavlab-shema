use super::AudioProcessor;

#[derive(Clone, Copy, PartialEq)]
enum GateState {
    Closed,
    Opening,
    Open,
    Holding,
    Closing,
}

pub struct NoiseGate {
    enabled: bool,
    threshold_linear: f32,
    range_linear: f32,
    
    attack_coef: f32,
    release_coef: f32,
    
    hold_samples: usize,
    hold_counter: usize,
    
    state: GateState,
    envelope: f32, // current gain multiplier (range_linear to 1.0)
    sample_rate: u32,
}

impl NoiseGate {
    pub fn new(
        threshold_db: f32,
        attack_ms: f32,
        hold_ms: f32,
        release_ms: f32,
        range_db: f32,
        sample_rate: u32,
        enabled: bool,
    ) -> Self {
        let threshold_linear = 10f32.powf(threshold_db / 20.0);
        let range_linear = 10f32.powf(range_db / 20.0);
        
        // Time constants mapping to coefficient: exp(-1 / (time_in_samples))
        let attack_samples = (attack_ms / 1000.0 * sample_rate as f32).max(1.0);
        let attack_coef = (-1.0 / attack_samples).exp();
        
        let release_samples = (release_ms / 1000.0 * sample_rate as f32).max(1.0);
        let release_coef = (-1.0 / release_samples).exp();
        
        let hold_samples = (hold_ms / 1000.0 * sample_rate as f32) as usize;

        Self {
            enabled,
            threshold_linear,
            range_linear,
            attack_coef,
            release_coef,
            hold_samples,
            hold_counter: 0,
            state: GateState::Closed,
            envelope: range_linear,
            sample_rate,
        }
    }
}

impl AudioProcessor for NoiseGate {
    fn process(&mut self, buffer: &mut [f32]) -> usize {
        for sample in buffer.iter_mut() {
            let input_level = sample.abs();
            
            // State machine transitions
            let over_threshold = input_level >= self.threshold_linear;
            
            match self.state {
                GateState::Closed | GateState::Closing => {
                    if over_threshold {
                        self.state = GateState::Opening;
                    }
                }
                GateState::Open | GateState::Opening => {
                    if !over_threshold {
                        self.state = GateState::Holding;
                        self.hold_counter = 0;
                    }
                }
                GateState::Holding => {
                    if over_threshold {
                        self.state = GateState::Open;
                    } else {
                        self.hold_counter += 1;
                        if self.hold_counter >= self.hold_samples {
                            self.state = GateState::Closing;
                        }
                    }
                }
            }
            
            // Envelope generation
            let target_gain = match self.state {
                GateState::Opening | GateState::Open | GateState::Holding => 1.0,
                GateState::Closing | GateState::Closed => self.range_linear,
            };
            
            if target_gain > self.envelope {
                self.envelope = self.attack_coef * self.envelope + (1.0 - self.attack_coef) * target_gain;
            } else {
                self.envelope = self.release_coef * self.envelope + (1.0 - self.release_coef) * target_gain;
            }
            
            if (self.envelope - target_gain).abs() < 1e-4 {
                if self.state == GateState::Opening { self.state = GateState::Open; }
                if self.state == GateState::Closing { self.state = GateState::Closed; }
            }
            
            *sample *= self.envelope;
        }
        buffer.len()
    }

    fn reset(&mut self) {
        self.state = GateState::Closed;
        self.envelope = self.range_linear;
        self.hold_counter = 0;
    }

    fn name(&self) -> &'static str {
        "NoiseGate"
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
    fn test_noise_gate_attenuates_silence() {
        let sample_rate = 16000;
        let mut gate = NoiseGate::new(-40.0, 1.0, 10.0, 10.0, -80.0, sample_rate, true);
        
        let mut buffer = vec![10f32.powf(-50.0 / 20.0); 1000]; // -50 dB signal
        gate.process(&mut buffer);
        
        // After processing, it should be heavily attenuated
        let last_sample = buffer[999];
        assert!(last_sample < 10f32.powf(-120.0 / 20.0), "Signal was not gated");
    }
}
