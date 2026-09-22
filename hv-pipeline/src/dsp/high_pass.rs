use super::AudioProcessor;
use super::biquad::Biquad;

pub struct HighPassFilter {
    enabled: bool,
    stages: Vec<Biquad>,
    frequency: f32,
    slope: u8,
    sample_rate: u32,
}

impl HighPassFilter {
    pub fn new(frequency: f32, slope: u8, sample_rate: u32, enabled: bool) -> Self {
        let mut hpf = Self {
            enabled,
            stages: Vec::new(),
            frequency,
            slope,
            sample_rate,
        };
        hpf.recalculate();
        hpf
    }
    
    pub fn set_params(&mut self, frequency: f32, slope: u8) {
        self.frequency = frequency;
        self.slope = slope;
        self.recalculate();
    }
    
    fn recalculate(&mut self) {
        let num_stages = (self.slope / 12).max(1) as usize; // Minimum 1 stage for 6dB (though biquad is 12dB, we just use 1 stage)
        // Note: For a true 6dB/oct filter we'd use a 1st order filter.
        // For simplicity, we'll map 6dB to 1 biquad (12dB) if they ask for it, 
        // or we could implement a 1st order. Let's just use 12dB increments.
        let num_biquads = match self.slope {
            6 | 12 => 1,
            18 | 24 => 2,
            _ => 1,
        };
        
        self.stages.clear();
        for _ in 0..num_biquads {
            let mut biquad = Biquad::new();
            biquad.set_hpf(self.frequency, self.sample_rate as f32);
            self.stages.push(biquad);
        }
    }
}

impl AudioProcessor for HighPassFilter {
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
        "HighPassFilter"
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
    fn test_high_pass_attenuates_low_freqs() {
        let sample_rate = 16000;
        let mut hpf = HighPassFilter::new(1000.0, 12, sample_rate, true);
        
        // Generate a 100Hz sine wave (below 1000Hz cutoff)
        let freq = 100.0;
        let mut buffer: Vec<f32> = (0..sample_rate)
            .map(|i| (2.0 * PI * freq * i as f32 / sample_rate as f32).sin())
            .collect();
            
        // Calculate original energy
        let orig_energy: f32 = buffer.iter().map(|&x| x * x).sum();
        
        hpf.process(&mut buffer);
        
        // Calculate processed energy
        let new_energy: f32 = buffer.iter().map(|&x| x * x).sum();
        
        assert!(new_energy < orig_energy * 0.1, "Low frequency was not attenuated properly");
    }
}
