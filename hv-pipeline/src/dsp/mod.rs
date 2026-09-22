use self::config::DspConfig;
use self::meter::{MeterReading, RmsCalculator};

pub mod config;
pub mod biquad;
pub mod dc_blocker;
pub mod high_pass;
pub mod low_pass;
pub mod noise_gate;
pub mod compressor;
pub mod agc;
pub mod limiter;
pub mod meter;

/// All DSP processors implement this trait.
/// Process audio in-place for zero-allocation hot path.
pub trait AudioProcessor: Send {
    /// Process a buffer of f32 samples in-place. Returns the number of valid samples.
    fn process(&mut self, buffer: &mut [f32]) -> usize;
    
    /// Reset internal state (called on device change or session restart)
    fn reset(&mut self);
    
    /// Name for debugging/logging
    fn name(&self) -> &'static str;
    
    /// Whether this processor is currently enabled
    fn is_enabled(&self) -> bool;
    
    /// Enable/disable without destroying state
    fn set_enabled(&mut self, enabled: bool);
}

pub struct DspChain {
    processors: Vec<Box<dyn AudioProcessor>>,
    meter: RmsCalculator,
}

impl DspChain {
    pub fn new(config: &DspConfig, sample_rate: u32) -> Self {
        let dc_blocker = Box::new(dc_blocker::DcBlocker::new());
        let high_pass = Box::new(high_pass::HighPassFilter::new(
            config.hpf_frequency, 
            config.hpf_slope, 
            sample_rate, 
            config.hpf_enabled
        ));
        let low_pass = Box::new(low_pass::LowPassFilter::new(
            config.lpf_frequency, 
            config.lpf_slope,
            sample_rate, 
            config.lpf_enabled
        ));
        let noise_gate = Box::new(noise_gate::NoiseGate::new(
            config.gate_threshold_db,
            config.gate_attack_ms,
            config.gate_hold_ms,
            config.gate_release_ms,
            -80.0, // range_db
            sample_rate,
            config.gate_enabled
        ));
        let compressor = Box::new(compressor::Compressor::new(
            config.compressor_threshold_db,
            config.compressor_ratio,
            config.compressor_attack_ms,
            config.compressor_release_ms,
            config.compressor_knee_db,
            config.compressor_makeup_db,
            sample_rate,
            config.compressor_enabled
        ));
        let agc = Box::new(agc::Agc::new(
            config.agc_target_db,
            config.agc_max_gain_db,
            config.agc_attack_ms,
            config.agc_release_ms,
            sample_rate,
            config.agc_enabled
        ));
        let limiter = Box::new(limiter::Limiter::new(
            config.limiter_threshold_db,
            config.limiter_release_ms,
            sample_rate,
            config.limiter_enabled
        ));

        let processors: Vec<Box<dyn AudioProcessor>> = vec![
            dc_blocker,
            high_pass,
            low_pass,
            noise_gate,
            compressor,
            agc,
            limiter,
        ];

        Self {
            processors,
            meter: RmsCalculator::new(10.0, sample_rate), // 10ms integration time for PPM
        }
    }
    
    /// Process a buffer through the entire chain
    pub fn process(&mut self, buffer: &mut [f32]) {
        for proc in &mut self.processors {
            if proc.is_enabled() {
                proc.process(buffer);
            }
        }
        self.meter.update(buffer);
    }
    
    /// Get current meter readings (called from UI thread)
    pub fn meter(&self) -> MeterReading {
        self.meter.read()
    }
    
    /// Update config at runtime
    pub fn update_config(&mut self, config: &DspConfig) {
        // Implementation left for later, or we can just reconstruct the chain since it's cheap,
        // but state resets might not be desirable. Ideally we'd update each processor.
        // For now, we will add update_config to the processors as needed.
    }
}
