use super::AudioProcessor;
use super::biquad::Biquad;

pub struct LowPassFilter {
    enabled: bool,
    stages: Vec<Biquad>,
    frequency: f32,
    slope: u8,
    sample_rate: u32,
}

impl LowPassFilter {
    pub fn new(frequency: f32, slope: u8, sample_rate: u32, enabled: bool) -> Self {
        let mut lpf = Self {
            enabled,
            stages: Vec::new(),
            frequency,
            slope,
            sample_rate,
        };
        lpf.recalculate();
        lpf
    }
    
    pub fn set_params(&mut self, frequency: f32, slope: u8) {
        self.frequency = frequency;
        self.slope = slope;
        self.recalculate();
    }
    
    fn recalculate(&mut self) {
        let num_biquads = match self.slope {
            6 | 12 => 1,
            18 | 24 => 2,
            _ => 1,
        };
        
        self.stages.clear();
        for _ in 0..num_biquads {
            let mut biquad = Biquad::new();
            biquad.set_lpf(self.frequency, self.sample_rate as f32);
            self.stages.push(biquad);
        }
    }
}

impl AudioProcessor for LowPassFilter {
    fn process(&mut self, buffer: &mut [f32]) -> usize {
        for sample in buffer.iter_mut() {
            let mut s = *sample;
            for stage in self.stages.iter_mut() {
                s = stage.process_sample(s);
            }
            *sample = s;
        }
        buffer.len()
    }

    fn reset(&mut self) {
        for stage in self.stages.iter_mut() {
            stage.reset();
        }
    }

    fn name(&self) -> &'static str {
        "LowPassFilter"
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
    use std::f32::consts::PI;

    #[test]
    fn test_low_pass_attenuates_high_freqs() {
        let sample_rate = 16000;
        let mut lpf = LowPassFilter::new(1000.0, 12, sample_rate, true);
        
        // Generate a 4000Hz sine wave (above 1000Hz cutoff)
        let freq = 4000.0;
        let mut buffer: Vec<f32> = (0..sample_rate)
            .map(|i| (2.0 * PI * freq * i as f32 / sample_rate as f32).sin())
            .collect();
            
        let orig_energy: f32 = buffer.iter().map(|&x| x * x).sum();
        lpf.process(&mut buffer);
        let new_energy: f32 = buffer.iter().map(|&x| x * x).sum();
        
        assert!(new_energy < orig_energy * 0.1, "High frequency was not attenuated properly");
    }
}
