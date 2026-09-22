//! Error types for the Bible engine.
//!
//! Every failure mode returns a typed error — no panics on the hot path.

/// Errors from the Bible engine's runtime operations.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// The requested translation is not loaded.
    #[error("translation not found: {0}")]
    TranslationNotFound(String),

    /// The requested verse does not exist in the canonical versification.
    #[error("verse not found: {book} {chapter}:{verse}")]
    VerseNotFound {
        book: String,
        chapter: u16,
        verse: u16,
    },

    /// The data pack failed BLAKE3 checksum verification.
    #[error("corrupt data pack for translation: {translation}")]
    CorruptDataPack { translation: String },

    /// The data pack uses a format version this binary doesn't support.
    #[error("unsupported pack version: found {found}, expected {expected}")]
    UnsupportedPackVersion { found: u32, expected: u32 },

    /// The ASR model could not be loaded.
    #[error("model load failed: {0}")]
    ModelLoadFailed(String),

    /// The platform denied microphone access.
    #[error("microphone permission denied")]
    MicrophonePermissionDenied,

    /// `start_sermon_mode` was called while already listening.
    #[error("already listening")]
    AlreadyListening,

    /// `stop_sermon_mode` was called while not listening.
    #[error("not listening")]
    NotListening,

    /// An audio frame was dropped because the capture queue was full.
    #[error("audio frame dropped: {0}")]
    AudioBackpressure(String),

    /// An I/O error occurred during file operations.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    /// The book ID is outside the valid range (1–66).
    #[error("invalid book id: {0}")]
    InvalidBookId(u8),
}

/// Errors from the verse reference parser.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// The book name or alias was not recognized.
    #[error("unrecognized book name: {0}")]
    UnrecognizedBook(String),

    /// The chapter number is invalid or out of range.
    #[error("invalid chapter: {0}")]
    InvalidChapter(String),

    /// The verse number is invalid or out of range.
    #[error("invalid verse: {0}")]
    InvalidVerse(String),

    /// The input text could not be parsed as a complete verse reference.
    #[error("incomplete reference: {0}")]
    IncompleteReference(String),

    /// The input was empty or contained no recognizable tokens.
    #[error("empty input")]
    EmptyInput,
}
