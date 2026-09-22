pub struct MeterReading {
    pub rms_dbfs: f32,
    pub peak_dbfs: f32,
}

pub struct RmsCalculator {
    integration_samples: usize,
    square_sum: f32,
    samples_collected: usize,
    current_rms_dbfs: f32,
    current_peak_dbfs: f32,
    window_peak: f32,
}

impl RmsCalculator {
    pub fn new(integration_ms: f32, sample_rate: u32) -> Self {
        let integration_samples = (integration_ms / 1000.0 * sample_rate as f32) as usize;
        Self {
            integration_samples: integration_samples.max(1),
            square_sum: 0.0,
            samples_collected: 0,
            current_rms_dbfs: -100.0,
            current_peak_dbfs: -100.0,
            window_peak: 0.0,
        }
    }

    pub fn update(&mut self, buffer: &[f32]) {
        for &sample in buffer {
            self.square_sum += sample * sample;
            
            let abs_sample = sample.abs();
            if abs_sample > self.window_peak {
                self.window_peak = abs_sample;
            }
            
            self.samples_collected += 1;
            
            if self.samples_collected >= self.integration_samples {
                let mean_square = self.square_sum / self.integration_samples as f32;
                let rms = mean_square.sqrt();
                self.current_rms_dbfs = if rms > 1e-5 {
                    20.0 * rms.log10()
                } else {
                    -100.0
                };
                
                self.current_peak_dbfs = if self.window_peak > 1e-5 {
                    20.0 * self.window_peak.log10()
                } else {
                    -100.0
                };
                
                self.square_sum = 0.0;
                self.samples_collected = 0;
                self.window_peak = 0.0;
            }
        }
    }

    pub fn read(&self) -> MeterReading {
        MeterReading {
            rms_dbfs: self.current_rms_dbfs,
            peak_dbfs: self.current_peak_dbfs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rms_calculation() {
        let mut meter = RmsCalculator::new(10.0, 16000); // 160 samples per window
        let mut buffer = vec![0.5; 160]; // Square wave of amplitude 0.5 has RMS 0.5
        
        meter.update(&buffer);
        let reading = meter.read();
        
        let expected_db = 20.0 * 0.5f32.log10(); // ~ -6.02 dB
        assert!((reading.rms_dbfs - expected_db).abs() < 0.1);
        assert!((reading.peak_dbfs - expected_db).abs() < 0.1);
    }
}
