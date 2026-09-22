pub const AUDIO_SECTION_IDS: [&str; 11] = [
    "Input Source",
    "Routing Matrix",
    "Input Level",
    "Gain Staging",
    "Processing Chain",
    "Voice Detection",
    "Monitoring",
    "Recording",
    "Presets",
    "LED Status",
    "Diagnostics",
];

#[derive(Debug, Clone)]
pub struct AudioControlState {
    pub device: AudioDeviceState,
    pub routing: RoutingState,
    pub level: LevelState,
    pub gain: GainState,
    pub processing: ProcessingState,
    pub vad: VadState,
    pub monitoring: MonitoringState,
    pub recording: RecordingState,
    pub preset: String,
    pub led: LedState,
    pub diagnostics: AudioDiagnosticsState,
}

impl AudioControlState {
    pub fn mock_running() -> Self {
        Self {
            device: AudioDeviceState {
                name: "Microphone Array (Realtek(R) Audio)".to_string(),
                driver: "WASAPI".to_string(),
                sample_rate: 48_000,
                bit_depth: 24,
                buffer_size: 256,
                buffer_latency_ms: 5.3,
                health: "Active".to_string(),
            },
            routing: RoutingState {
                input: "Stereo".to_string(),
                processing: "Mono (L+R)".to_string(),
                channels: vec![
                    ChannelState {
                        id: "L".to_string(),
                        active: true,
                        peak_db: -8.9,
                    },
                    ChannelState {
                        id: "R".to_string(),
                        active: true,
                        peak_db: -9.1,
                    },
                ],
                phase: "Normal".to_string(),
            },
            level: LevelState {
                left_db: -17.6,
                right_db: -17.4,
                peak_db: -8.9,
                clip: false,
                snr_db: 30,
                lufs: -22.4,
                meter_mode: "RMS".to_string(),
                hold_seconds: 3,
            },
            gain: GainState {
                input_gain_db: 2.0,
                digital_trim_db: 0.0,
                hpf_enabled: true,
                hpf_frequency: "80 Hz".to_string(),
                hpf_slope: "12 dB/oct".to_string(),
            },
            processing: ProcessingState {
                agc: ProcessorRow::enabled("AGC", "Target -18 dB", "Attack 10 ms"),
                gate: ProcessorRow::disabled("Gate", "Threshold -45 dB", "Hold 50 ms"),
                aec: ProcessorRow::enabled("AEC", "Strength Medium", "Echo return -31 dB"),
                compressor: ProcessorRow::disabled("Compressor", "Ratio 3:1", "Threshold -20 dB"),
                limiter: ProcessorRow::enabled("Limiter", "Ceiling -1 dB", "Release 80 ms"),
            },
            vad: VadState {
                status: "Active".to_string(),
                confidence: 0.90,
                silence_seconds: 0.3,
                silence_threshold_seconds: 1.5,
                speech_seconds: 0.2,
                min_speech_seconds: 0.25,
                sensitivity: 0.60,
            },
            monitoring: MonitoringState {
                level_db: 0.0,
                muted: false,
                solo: false,
                output_device: "Default".to_string(),
                cue_bus: "Operator".to_string(),
            },
            recording: RecordingState {
                active: false,
                elapsed: "00:00:00".to_string(),
                auto_record: false,
                format: "WAV 24-bit".to_string(),
                path: "~/sessions/sermon-001.wav".to_string(),
                disk_free_gb: 412,
            },
            preset: "Church Service".to_string(),
            led: LedState {
                status: "Speaking".to_string(),
                hardware_enabled: true,
                screen_enabled: true,
                brightness_percent: 80.0,
            },
            diagnostics: AudioDiagnosticsState {
                cpu_percent: 28,
                jitter_ms: 0.4,
                dropped_frames: 0,
                last_buffer_ms: 5.4,
                asr_queue: 0,
                uptime: "00:42:18".to_string(),
                watchdog: "Healthy".to_string(),
            },
        }
    }

    pub fn sections(&self) -> &'static [&'static str] {
        &AUDIO_SECTION_IDS
    }
}

#[derive(Debug, Clone)]
pub struct AudioDeviceState {
    pub name: String,
    pub driver: String,
    pub sample_rate: u32,
    pub bit_depth: u16,
    pub buffer_size: u16,
    pub buffer_latency_ms: f32,
    pub health: String,
}

#[derive(Debug, Clone)]
pub struct RoutingState {
    pub input: String,
    pub processing: String,
    pub channels: Vec<ChannelState>,
    pub phase: String,
}

#[derive(Debug, Clone)]
pub struct ChannelState {
    pub id: String,
    pub active: bool,
    pub peak_db: f32,
}

#[derive(Debug, Clone)]
pub struct LevelState {
    pub left_db: f32,
    pub right_db: f32,
    pub peak_db: f32,
    pub clip: bool,
    pub snr_db: i32,
    pub lufs: f32,
    pub meter_mode: String,
    pub hold_seconds: u8,
}

#[derive(Debug, Clone)]
pub struct GainState {
    pub input_gain_db: f32,
    pub digital_trim_db: f32,
    pub hpf_enabled: bool,
    pub hpf_frequency: String,
    pub hpf_slope: String,
}

#[derive(Debug, Clone)]
pub struct ProcessingState {
    pub agc: ProcessorRow,
    pub gate: ProcessorRow,
    pub aec: ProcessorRow,
    pub compressor: ProcessorRow,
    pub limiter: ProcessorRow,
}

#[derive(Debug, Clone)]
pub struct ProcessorRow {
    pub name: String,
    pub enabled: bool,
    pub primary: String,
    pub secondary: String,
}

impl ProcessorRow {
    fn enabled(name: &str, primary: &str, secondary: &str) -> Self {
        Self {
            name: name.to_string(),
            enabled: true,
            primary: primary.to_string(),
            secondary: secondary.to_string(),
        }
    }

    fn disabled(name: &str, primary: &str, secondary: &str) -> Self {
        Self {
            name: name.to_string(),
            enabled: false,
            primary: primary.to_string(),
            secondary: secondary.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VadState {
    pub status: String,
    pub confidence: f32,
    pub silence_seconds: f32,
    pub silence_threshold_seconds: f32,
    pub speech_seconds: f32,
    pub min_speech_seconds: f32,
    pub sensitivity: f32,
}

#[derive(Debug, Clone)]
pub struct MonitoringState {
    pub level_db: f32,
    pub muted: bool,
    pub solo: bool,
    pub output_device: String,
    pub cue_bus: String,
}

#[derive(Debug, Clone)]
pub struct RecordingState {
    pub active: bool,
    pub elapsed: String,
    pub auto_record: bool,
    pub format: String,
    pub path: String,
    pub disk_free_gb: u32,
}

#[derive(Debug, Clone)]
pub struct LedState {
    pub status: String,
    pub hardware_enabled: bool,
    pub screen_enabled: bool,
    pub brightness_percent: f32,
}

#[derive(Debug, Clone)]
pub struct AudioDiagnosticsState {
    pub cpu_percent: u8,
    pub jitter_ms: f32,
    pub dropped_frames: u32,
    pub last_buffer_ms: f32,
    pub asr_queue: u32,
    pub uptime: String,
    pub watchdog: String,
}
