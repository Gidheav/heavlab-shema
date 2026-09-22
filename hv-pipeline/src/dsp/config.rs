#[derive(Clone, Debug)]
pub struct DspConfig {
    pub hpf_enabled: bool,
    pub hpf_frequency: f32,
    pub hpf_slope: u8, // 6, 12, 18, 24
    pub lpf_enabled: bool,
    pub lpf_frequency: f32,
    pub lpf_slope: u8,
    pub gate_enabled: bool,
    pub gate_threshold_db: f32,
    pub gate_attack_ms: f32,
    pub gate_hold_ms: f32,
    pub gate_release_ms: f32,
    pub compressor_enabled: bool,
    pub compressor_threshold_db: f32,
    pub compressor_ratio: f32,
    pub compressor_attack_ms: f32,
    pub compressor_release_ms: f32,
    pub compressor_knee_db: f32,
    pub compressor_makeup_db: f32,
    pub agc_enabled: bool,
    pub agc_target_db: f32,
    pub agc_max_gain_db: f32,
    pub agc_attack_ms: f32,
    pub agc_release_ms: f32,
    pub limiter_enabled: bool,
    pub limiter_threshold_db: f32,
    pub limiter_release_ms: f32,
}

impl Default for DspConfig {
    fn default() -> Self {
        Self {
            hpf_enabled: true,
            hpf_frequency: 80.0,
            hpf_slope: 12, // 12 dB/oct is a good default
            lpf_enabled: true,
            lpf_frequency: 8000.0,
            lpf_slope: 12,
            gate_enabled: true,
            gate_threshold_db: -45.0,
            gate_attack_ms: 1.0,
            gate_hold_ms: 50.0,
            gate_release_ms: 100.0,
            compressor_enabled: true,
            compressor_threshold_db: -20.0,
            compressor_ratio: 3.0,
            compressor_attack_ms: 5.0,
            compressor_release_ms: 100.0,
            compressor_knee_db: 6.0,
            compressor_makeup_db: 0.0, // Automatically makeup is usually better handled individually or by AGC later
            agc_enabled: true,
            agc_target_db: -18.0,
            agc_max_gain_db: 20.0,
            agc_attack_ms: 10.0,
            agc_release_ms: 200.0,
            limiter_enabled: true,
            limiter_threshold_db: -1.0,
            limiter_release_ms: 50.0,
        }
    }
}
