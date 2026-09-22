//! bible-ffi — UniFFI boundary layer.
//!
//! This is the ONLY crate in the workspace allowed to use `uniffi` macros.
//! It depends on `bible-core` and `bible-asr` and exposes the public
//! API surface for Swift (iOS) and Kotlin (Android) consumption.
//!
//! Phase 4: UniFFI proc-macro approach.

use std::sync::{Arc, Mutex};

use bible_asr::worker::{AudioWorker, TranscriptHandler, WorkerConfig};
use bible_core::error::{EngineError as CoreEngineError, ParseError};
use bible_core::store::TranslationStore;
use bible_core::types::VerseRef as CoreVerseRef;

/// UniFFI-compatible verse reference wrapper.
#[derive(Debug, Clone, uniffi::Record)]
pub struct VerseRef {
    pub book: u8,
    pub chapter: u16,
    pub verse: u16,
}

impl From<CoreVerseRef> for VerseRef {
    fn from(core: CoreVerseRef) -> Self {
        VerseRef {
            book: core.book.as_u8(),
            chapter: core.chapter,
            verse: core.verse,
        }
    }
}

impl From<VerseRef> for CoreVerseRef {
    fn from(ffi: VerseRef) -> Self {
        CoreVerseRef {
            book: bible_core::types::BookId::new(ffi.book).unwrap(),
            chapter: ffi.chapter,
            verse: ffi.verse,
        }
    }
}

/// UniFFI-compatible error type wrapping bible-core errors.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum EngineError {
    /// The requested translation is not loaded.
    #[error("translation not found: {0}")]
    TranslationNotFound(String),

    /// The requested verse does not exist in the canonical versification.
    #[error("verse not found: {0} {1}:{2}")]
    VerseNotFound(String, u16, u16),

    /// The data pack failed BLAKE3 checksum verification.
    #[error("corrupt data pack for translation: {0}")]
    CorruptDataPack(String),

    /// The data pack uses a format version this binary doesn't support.
    #[error("unsupported pack version: found {0}, expected {1}")]
    UnsupportedPackVersion(u32, u32),

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
    Io(String),

    /// The book ID is outside the valid range (1–66).
    #[error("invalid book id: {0}")]
    InvalidBookId(u8),
}

impl From<CoreEngineError> for EngineError {
    fn from(err: CoreEngineError) -> Self {
        match err {
            CoreEngineError::TranslationNotFound(s) => EngineError::TranslationNotFound(s),
            CoreEngineError::VerseNotFound { book, chapter, verse } => {
                EngineError::VerseNotFound(book, chapter, verse)
            }
            CoreEngineError::CorruptDataPack { translation } => {
                EngineError::CorruptDataPack(translation)
            }
            CoreEngineError::UnsupportedPackVersion { found, expected } => {
                EngineError::UnsupportedPackVersion(found, expected)
            }
            CoreEngineError::ModelLoadFailed(s) => EngineError::ModelLoadFailed(s),
            CoreEngineError::MicrophonePermissionDenied => EngineError::MicrophonePermissionDenied,
            CoreEngineError::AlreadyListening => EngineError::AlreadyListening,
            CoreEngineError::NotListening => EngineError::NotListening,
            CoreEngineError::AudioBackpressure(s) => EngineError::AudioBackpressure(s),
            CoreEngineError::Io(e) => EngineError::Io(e.to_string()),
            CoreEngineError::InvalidBookId(id) => EngineError::InvalidBookId(id),
        }
    }
}

/// The main Bible engine — thread-safe, owns all state.
///
/// This struct is `Send + Sync` and is the single entry point
/// for all operations from the mobile shell.
#[derive(uniffi::Object)]
pub struct BibleEngine {
    /// Translation data store.
    store: TranslationStore,

    /// Audio worker for sermon mode.
    worker: Mutex<AudioWorker>,
}

impl BibleEngine {
    /// Create an engine with the scripted ASR used by desktop development and tests.
    ///
    /// This constructor is intentionally not part of the UniFFI surface. Mobile
    /// shells must use [`BibleEngine::new`], which never fabricates transcripts.
    pub fn new_with_mock_asr() -> Self {
        let config = WorkerConfig {
            use_mock_asr: true,
            ..WorkerConfig::default()
        };
        BibleEngine {
            store: TranslationStore::new(),
            worker: Mutex::new(AudioWorker::new(config)),
        }
    }
}

#[uniffi::export]
impl BibleEngine {
    /// Create a new BibleEngine instance.
    #[uniffi::constructor]
    pub fn new() -> Self {
        let config = WorkerConfig {
            use_mock_asr: false,
            ..WorkerConfig::default()
        };
        BibleEngine {
            store: TranslationStore::new(),
            worker: Mutex::new(AudioWorker::new(config)),
        }
    }

    /// Look up a verse by translation, book name, chapter, and verse number.
    ///
    /// The book name can be any recognized alias (e.g., "John", "Jn", "joh").
    ///
    /// # Errors
    /// - `EngineError::TranslationNotFound` if the translation isn't loaded
    /// - `EngineError::VerseNotFound` if the reference is invalid
    pub fn lookup(
        &self,
        translation: String,
        book: String,
        chapter: u16,
        verse: u16,
    ) -> Result<String, EngineError> {
        // Resolve the book alias to a BookId
        let book_id = bible_core::canon::resolve_book_alias(&book).map_err(|_| {
            EngineError::VerseNotFound(book.clone(), chapter, verse)
        })?;

        // Resolve to a global verse index
        let index = bible_core::canon::resolve_index(book_id, chapter, verse)?;

        // Look up the verse text in the loaded translation
        self.store.get_verse(&translation, index).map_err(EngineError::from)
    }

    /// Resolve a spoken or written text string to a verse reference.
    ///
    /// # Examples
    /// - `"John 3:16"` → VerseRef { book: 43, chapter: 3, verse: 16 }
    /// - `"First Corinthians 13:4"` → VerseRef { book: 46, chapter: 13, verse: 4 }
    ///
    /// # Errors
    /// - `EngineError` wrapping a `ParseError` if the text can't be parsed
    pub fn resolve_spoken(&self, _language: String, text: String) -> Result<VerseRef, EngineError> {
        bible_core::parser::resolve_text(&text)
            .map(|core| core.into())
            .map_err(|e| match e {
                ParseError::UnrecognizedBook(b) => EngineError::VerseNotFound(b, 0, 0),
                ParseError::InvalidChapter(c) => EngineError::VerseNotFound(
                    String::new(),
                    c.parse().unwrap_or(0),
                    0,
                ),
                ParseError::InvalidVerse(v) => EngineError::VerseNotFound(
                    String::new(),
                    0,
                    v.parse().unwrap_or(0),
                ),
                ParseError::IncompleteReference(r) => EngineError::VerseNotFound(r, 0, 0),
                ParseError::EmptyInput => EngineError::VerseNotFound("(empty)".to_string(), 0, 0),
            })
    }

    /// Start sermon mode — live ASR transcription with verse detection.
    ///
    /// The `handler` callback fires on the ASR worker thread (not the
    /// main thread). Marshaling to the UI thread is the caller's
    /// responsibility (e.g., `DispatchQueue.main.async` in Swift).
    /// `aec_enabled` must be true when the platform shell has enabled
    /// hardware acoustic echo cancellation.
    ///
    /// # Errors
    /// - `EngineError::AlreadyListening` if already in sermon mode
    /// - `EngineError::ModelLoadFailed` if the ASR model can't be loaded
    pub fn start_sermon_mode(
        &self,
        _language: String,
        aec_enabled: bool,
        handler: Box<dyn VerseDetectedHandler>,
    ) -> Result<(), EngineError> {
        let mut worker = self
            .worker
            .lock()
            .map_err(|_| EngineError::ModelLoadFailed("worker lock poisoned".to_string()))?;

        // Wrap the VerseDetectedHandler in a TranscriptHandler adapter
        let adapter = Arc::new(HandlerAdapter { inner: handler });

        worker
            .start_with_aec(adapter, aec_enabled)
            .map_err(|e| match e {
                bible_asr::AsrError::AlreadyListening => EngineError::AlreadyListening,
                bible_asr::AsrError::FeatureUnavailable(message) => {
                    EngineError::ModelLoadFailed(message.to_string())
                }
                bible_asr::AsrError::AecUnavailable => EngineError::ModelLoadFailed(e.to_string()),
                other => EngineError::ModelLoadFailed(other.to_string()),
            })
    }

    /// Push one converted PCM frame from the platform shell.
    ///
    /// A full capture queue returns `EngineError::AudioBackpressure`; shells
    /// should treat that as a transient dropped frame and continue pushing.
    pub fn push_audio(&self, samples: Vec<i16>, timestamp_ms: u64) -> Result<(), EngineError> {
        let worker = self
            .worker
            .lock()
            .map_err(|_| EngineError::ModelLoadFailed("worker lock poisoned".to_string()))?;
        worker
            .push_audio(&samples, timestamp_ms)
            .map_err(|error| match error {
                bible_asr::AsrError::NotListening => EngineError::NotListening,
                bible_asr::AsrError::ChannelError(message) => {
                    EngineError::AudioBackpressure(message)
                }
                bible_asr::AsrError::AecUnavailable
                | bible_asr::AsrError::FeatureUnavailable(_) => {
                    EngineError::ModelLoadFailed(error.to_string())
                }
                other => EngineError::ModelLoadFailed(other.to_string()),
            })
    }

    /// Stop sermon mode.
    ///
    /// # Errors
    /// - `EngineError::NotListening` if not currently in sermon mode
    pub fn stop_sermon_mode(&self) -> Result<(), EngineError> {
        let mut worker = self
            .worker
            .lock()
            .map_err(|_| EngineError::ModelLoadFailed("worker lock poisoned".to_string()))?;

        worker.stop().map_err(|e| match e {
            bible_asr::AsrError::NotListening => EngineError::NotListening,
            other => EngineError::ModelLoadFailed(other.to_string()),
        })
    }

    /// Get the engine version (semver).
    ///
    /// Used by the native shell to check data-pack compatibility.
    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

impl Default for BibleEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Callback trait for verse detection during sermon mode.
///
/// Implemented by foreign code (Swift/Kotlin) to receive callbacks
/// from the Rust ASR pipeline.
#[uniffi::export(callback_interface)]
pub trait VerseDetectedHandler: Send + Sync {
    /// Called when a verse reference is detected in speech.
    fn on_verse_detected(&self, verse_ref: VerseRef, text: String) -> Result<(), String>;

    /// Called with partial transcript text (for live display).
    fn on_partial_transcript(&self, text: String) -> Result<(), String>;

    /// Called when an error occurs in the ASR pipeline.
    fn on_error(&self, error: String) -> Result<(), String>;

    /// Called once after prolonged continuous silence.
    fn on_listening_paused(&self) {}

    /// Called when speech resumes after a pause notification.
    fn on_speech_resumed(&self) {}
}

/// Adapter bridging `VerseDetectedHandler` to `TranscriptHandler`.
struct HandlerAdapter {
    inner: Box<dyn VerseDetectedHandler>,
}

impl TranscriptHandler for HandlerAdapter {
    fn on_transcript(&self, text: String) {
        // Try to resolve the transcript as a verse reference
        if let Ok(core_verse_ref) = bible_core::parser::resolve_text(&text) {
            let verse_ref: VerseRef = core_verse_ref.into();
            let _ = self.inner.on_verse_detected(verse_ref, text.clone());
        }
        let _ = self.inner.on_partial_transcript(text);
    }

    fn on_error(&self, error: String) {
        let _ = self.inner.on_error(error);
    }

    fn on_listening_state_changed(&self, is_active: bool) {
        // Optional: could be exposed as another callback method
        let _ = (is_active,);
    }

    fn on_listening_paused(&self) {
        self.inner.on_listening_paused();
    }

    fn on_speech_resumed(&self) {
        self.inner.on_speech_resumed();
    }
}

uniffi::setup_scaffolding!();

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use bible_asr::capture::{CaptureConfig, PushCapture};
    use bible_asr::engine::MockAsrEngine;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    struct NoopHandler;

    impl VerseDetectedHandler for NoopHandler {
        fn on_verse_detected(&self, _verse_ref: VerseRef, _text: String) -> Result<(), String> {
            Ok(())
        }

        fn on_partial_transcript(&self, _text: String) -> Result<(), String> {
            Ok(())
        }

        fn on_error(&self, _error: String) -> Result<(), String> {
            Ok(())
        }
    }

    struct BlockingHandler {
        entered: Arc<AtomicBool>,
        release: Arc<AtomicBool>,
    }

    impl VerseDetectedHandler for BlockingHandler {
        fn on_verse_detected(&self, _verse_ref: VerseRef, _text: String) -> Result<(), String> {
            Ok(())
        }

        fn on_partial_transcript(&self, _text: String) -> Result<(), String> {
            self.entered.store(true, Ordering::SeqCst);
            while !self.release.load(Ordering::SeqCst) {
                thread::yield_now();
            }
            Ok(())
        }

        fn on_error(&self, _error: String) -> Result<(), String> {
            Ok(())
        }
    }

    struct ReleaseOnDrop(Arc<AtomicBool>);

    impl Drop for ReleaseOnDrop {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    #[test]
    fn engine_creation() {
        let engine = BibleEngine::new();
        assert!(!engine.version().is_empty());
        let _ = engine;
    }

    #[test]
    fn production_engine_does_not_start_with_mock_asr() {
        let result =
            BibleEngine::new().start_sermon_mode("en".to_string(), true, Box::new(NoopHandler));
        assert!(matches!(
            result,
            Err(EngineError::ModelLoadFailed(message))
                if message.contains("sherpa-onnx ASR is not compiled in")
        ));
    }

    #[test]
    fn explicit_mock_engine_can_start_for_development() {
        let engine = BibleEngine::new_with_mock_asr();
        engine
            .start_sermon_mode("en".to_string(), true, Box::new(NoopHandler))
            .expect("mock ASR should start");
        engine.stop_sermon_mode().expect("stop mock ASR");
    }

    #[test]
    fn resolve_spoken_john_3_16() {
        let engine = BibleEngine::new();
        let result = engine.resolve_spoken("en".to_string(), "John 3:16".to_string());
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert_eq!(vr.book, 43);
        assert_eq!(vr.chapter, 3);
        assert_eq!(vr.verse, 16);
    }

    #[test]
    fn resolve_spoken_empty_fails() {
        let engine = BibleEngine::new();
        let result = engine.resolve_spoken("en".to_string(), "".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn push_audio_before_start_fails() {
        let engine = BibleEngine::new();
        assert!(matches!(
            engine.push_audio(vec![0; 1600], 0),
            Err(EngineError::NotListening)
        ));
    }

    #[test]
    fn push_audio_maps_full_capture_queue_to_backpressure() {
        let engine = BibleEngine::new_with_mock_asr();
        let entered = Arc::new(AtomicBool::new(false));
        let release = Arc::new(AtomicBool::new(false));
        let _release_guard = ReleaseOnDrop(Arc::clone(&release));
        let handler = Box::new(BlockingHandler {
            entered: Arc::clone(&entered),
            release: Arc::clone(&release),
        });
        let adapter = Arc::new(HandlerAdapter { inner: handler });
        let capture = Arc::new(PushCapture::new(CaptureConfig {
            chunk_ms: 100,
            require_aec: false,
            ..CaptureConfig::default()
        }));
        {
            let mut worker = engine.worker.lock().expect("worker lock");
            worker
                .start_with(
                    adapter,
                    Arc::clone(&capture) as Arc<dyn bible_asr::MicrophoneCapture>,
                    Box::new(MockAsrEngine::new("en".to_string())),
                    true,
                )
                .expect("start worker");
        }

        for timestamp in 0..10 {
            engine
                .push_audio(vec![5_000; 1_600], timestamp * 100)
                .expect("push speech");
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        while !entered.load(Ordering::SeqCst) {
            assert!(Instant::now() < deadline, "worker did not enter callback");
            thread::sleep(Duration::from_millis(5));
        }

        for timestamp in 0..20 {
            engine
                .push_audio(vec![0; 1_600], 1_000 + timestamp * 100)
                .expect("fill capture queue");
        }
        let result = engine.push_audio(vec![0; 1_600], 3_000);
        assert!(matches!(result, Err(EngineError::AudioBackpressure(_))));

        release.store(true, Ordering::SeqCst);
        engine.stop_sermon_mode().expect("stop worker");
    }

    #[test]
    fn version_is_semver() {
        let v = BibleEngine::new().version();
        let parts: Vec<&str> = v.split('.').collect();
        assert_eq!(parts.len(), 3, "Version '{}' is not semver", v);
    }

    #[test]
    fn stop_without_start_fails() {
        let engine = BibleEngine::new();
        let result = engine.stop_sermon_mode();
        assert!(result.is_err());
    }
}