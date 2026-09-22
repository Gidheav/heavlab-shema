use super::AudioProcessor;

pub struct DcBlocker {
    enabled: bool,
    r: f32,
    x_prev: f32,
    y_prev: f32,
}

impl DcBlocker {
    pub fn new() -> Self {
        Self {
            enabled: true,
            r: 0.995,
            x_prev: 0.0,
            y_prev: 0.0,
        }
    }
}

impl AudioProcessor for DcBlocker {
    fn process(&mut self, buffer: &mut [f32]) -> usize {
        for sample in buffer.iter_mut() {
            let x = *sample;
            // y[n] = x[n] - x[n-1] + R * y[n-1]
            let y = x - self.x_prev + self.r * self.y_prev;
            self.x_prev = x;
            self.y_prev = y;
            *sample = y;
        }
        buffer.len()
    }

    fn reset(&mut self) {
        self.x_prev = 0.0;
        self.y_prev = 0.0;
    }

    fn name(&self) -> &'static str {
        "DcBlocker"
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
    fn test_dc_blocker_removes_dc() {
        let mut blocker = DcBlocker::new();
        let mut buffer = vec![1.0; 1000]; // DC offset of 1.0
        
        blocker.process(&mut buffer);
        
        // After processing, the DC offset should be nearly 0
        let last_sample = buffer[999];
        assert!(last_sample.abs() < 0.01, "DC offset not removed, got {}", last_sample);
    }
}
