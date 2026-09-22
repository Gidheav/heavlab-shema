//! ASR error types.

/// Errors from the ASR pipeline.
#[derive(Debug, thiserror::Error)]
pub enum AsrError {
    /// The ASR model could not be loaded from the given path.
    #[error("model load failed: {0}")]
    ModelLoadFailed(String),

    /// The platform denied microphone access.
    #[error("microphone permission denied")]
    MicrophonePermissionDenied,

    /// An error occurred during audio capture.
    #[error("capture error: {0}")]
    CaptureError(String),

    /// The VAD model could not be loaded.
    #[error("vad model load failed: {0}")]
    VadLoadFailed(String),

    /// An error occurred during speech recognition inference.
    #[error("inference error: {0}")]
    InferenceError(String),

    /// The worker is already running.
    #[error("already listening")]
    AlreadyListening,

    /// The worker is not running.
    #[error("not listening")]
    NotListening,

    /// Channel communication error.
    #[error("channel error: {0}")]
    ChannelError(String),

    /// Platform capture did not assert that hardware AEC is enabled.
    #[error("acoustic echo cancellation is unavailable")]
    AecUnavailable,

    /// An optional native feature was not compiled.
    #[error("feature unavailable: {0}")]
    FeatureUnavailable(&'static str),
}
