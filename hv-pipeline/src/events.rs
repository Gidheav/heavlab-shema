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
    SetGain(f32),
    SetTranslation(String),
    SetDevice(String),
    SetNoiseGateThreshold(f32),
    EnableNoiseGate(bool),
    SetHighPassFreq(f32),
    EnableHighPass(bool),
    SetCompressorRatio(f32),
    EnableCompressor(bool),
}
