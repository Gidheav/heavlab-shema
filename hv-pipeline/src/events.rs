//! Streamed updates from the pipeline worker to the UI thread.

#[derive(Debug, Clone, PartialEq)]
pub enum PipelineEvent {
    Transcript {
        text: String,
        is_final: bool,
        confidence: f32,
    },
    VerseDetected {
        reference: String,
        text: String,
        book: u8,
        chapter: u16,
        verse: u16,
    },
    MeterUpdate {
        rms_db: f32,
        peak_db: f32,
        is_clipping: bool,
    },
    StateChanged(PipelineState),
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PipelineState {
    #[default]
    Stopped,
    Listening,
    Silence,
    Speaking,
}

#[derive(Debug, Clone)]
pub enum PipelineCommand {
    Start,
    Stop,
    /// Hold the mic open but stop feeding the chain — no metering, no
    /// recognition. The device stays hot so resuming is instant.
    Pause,
    Resume,
    SetGain(f32),
    SetTranslation(String),
    SetDevice(String),
    SetNoiseGateThreshold(f32),
    EnableNoiseGate(bool),
    SetHighPassFreq(f32),
    EnableHighPass(bool),
    SetCompressorRatio(f32),
    EnableCompressor(bool),
    /// Automatic gain control — envelope leveler on the capture stream.
    EnableAgc(bool),
    /// Acoustic feedback / echo suppression on the capture stream.
    EnableAec(bool),
    /// Voice-activity sensitivity, 0.0 (permissive) .. 1.0 (strict).
    SetVadSensitivity(f32),
}
