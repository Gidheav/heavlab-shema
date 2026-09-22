use std::f32::consts::PI;

#[derive(Clone, Debug)]
pub struct Biquad {
    // Coefficients
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    
    // State (Direct Form 1)
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Biquad {
    pub fn new() -> Self {
        Self {
            a1: 0.0,
            a2: 0.0,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }
    
    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
    
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
                   - self.a1 * self.y1 - self.a2 * self.y2;
                   
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        
        output
    }
    
    /// Calculate Butterworth Low Pass coefficients
    pub fn set_lpf(&mut self, frequency: f32, sample_rate: f32) {
        let omega0 = 2.0 * PI * frequency / sample_rate;
        let sin_omega0 = omega0.sin();
        let cos_omega0 = omega0.cos();
        let alpha = sin_omega0 / std::f32::consts::SQRT_2; // Q = 1/sqrt(2) for Butterworth
        
        let a0 = 1.0 + alpha;
        let b0 = (1.0 - cos_omega0) / 2.0;
        let b1 = 1.0 - cos_omega0;
        let b2 = (1.0 - cos_omega0) / 2.0;
        let a1 = -2.0 * cos_omega0;
        let a2 = 1.0 - alpha;
        
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }
    
    /// Calculate Butterworth High Pass coefficients
    pub fn set_hpf(&mut self, frequency: f32, sample_rate: f32) {
        let omega0 = 2.0 * PI * frequency / sample_rate;
        let sin_omega0 = omega0.sin();
        let cos_omega0 = omega0.cos();
        let alpha = sin_omega0 / std::f32::consts::SQRT_2; // Q = 1/sqrt(2) for Butterworth
        
        let a0 = 1.0 + alpha;
        let b0 = (1.0 + cos_omega0) / 2.0;
        let b1 = -(1.0 + cos_omega0);
        let b2 = (1.0 + cos_omega0) / 2.0;
        let a1 = -2.0 * cos_omega0;
        let a2 = 1.0 - alpha;
        
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }
}
