//! ASR engine abstraction.
//!
//! Defines the ASR engine boundary. Streaming sherpa-onnx zipformer/
//! transducer ASR and Silero VAD will plug into these traits behind the
//! optional `sherpa` feature; the default build has no native dependency.

use crate::capture::AudioChunk;
use crate::error::AsrError;

pub use crate::vad::{SimpleVad, Vad, VadConfig, VadState};

/// Trait for speech recognition engines.
///
/// Implementations process audio chunks and emit partial transcripts.
/// The engine must be `Send` to allow ownership by the `AudioWorker` thread.
pub trait AsrEngine: Send {
    /// Feed an audio chunk to the engine.
    ///
    /// Returns `Some(transcript)` when new text is available,
    /// `None` when the chunk was consumed but no new text is ready.
    fn feed(&mut self, chunk: &AudioChunk) -> Result<Option<String>, AsrError>;

    /// Reset the engine state (e.g., between utterances after a VAD endpoint).
    fn reset(&mut self);

    /// Get the language this engine is configured for.
    fn language(&self) -> &str;
}

/// Configuration for loading an ASR model.
#[derive(Debug, Clone)]
pub struct AsrModelConfig {
    /// Path to the encoder model file (ONNX).
    pub encoder_path: String,

    /// Path to the decoder model file (ONNX).
    pub decoder_path: String,

    /// Path to the joiner model file (ONNX).
    pub joiner_path: String,

    /// Path to the token vocabulary file.
    pub tokens_path: String,

    /// Language code (e.g., "en", "es", "fr").
    pub language: String,

    /// Number of threads for ONNX inference.
    pub num_threads: u32,

    /// Whether to enable the Silero VAD gate.
    pub enable_vad: bool,

    /// Path to the Silero VAD model (required if `enable_vad` is true).
    pub vad_model_path: Option<String>,

    /// VAD threshold: speech confidence above this value triggers ASR.
    pub vad_threshold: f32,

    /// VAD silence duration in seconds to trigger ASR sleep.
    pub vad_silence_duration: f32,
}

impl AsrModelConfig {
    /// Validate that the model configuration is complete.
    pub fn validate(&self) -> Result<(), AsrError> {
        if self.encoder_path.is_empty()
            || self.decoder_path.is_empty()
            || self.joiner_path.is_empty()
            || self.tokens_path.is_empty()
        {
            return Err(AsrError::ModelLoadFailed(
                "encoder, decoder, joiner, and tokens paths are required".to_string(),
            ));
        }
        if self.enable_vad && self.vad_model_path.is_none() {
            return Err(AsrError::VadLoadFailed(
                "vad model path is required when VAD is enabled".to_string(),
            ));
        }
        Ok(())
    }
}

impl Default for AsrModelConfig {
    fn default() -> Self {
        AsrModelConfig {
            encoder_path: String::new(),
            decoder_path: String::new(),
            joiner_path: String::new(),
            tokens_path: String::new(),
            language: "en".to_string(),
            num_threads: 2,
            enable_vad: true,
            vad_model_path: None,
            vad_threshold: 0.6,
            vad_silence_duration: 1.5,
        }
    }
}

// ── Mock ASR Engine for Development ─────────────────────────────────────

/// Mock ASR engine for development and testing.
///
/// This implementation simulates ASR behavior without actual model loading,
/// allowing development and testing of the pipeline infrastructure.
#[derive(Debug)]
pub struct MockAsrEngine {
    language: String,
    chunk_count: u64,
    last_transcript: Option<String>,
}

impl MockAsrEngine {
    /// Create a new mock ASR engine.
    pub fn new(language: String) -> Self {
        MockAsrEngine {
            language,
            chunk_count: 0,
            last_transcript: None,
        }
    }
}

impl AsrEngine for MockAsrEngine {
    fn feed(&mut self, chunk: &AudioChunk) -> Result<Option<String>, AsrError> {
        self.chunk_count += 1;

        // Simulate periodic transcript generation (every 10 chunks)
        if self.chunk_count % 10 == 0 && chunk.has_signal() {
            // Return mock transcript
            let transcript = match self.chunk_count % 30 {
                10 => "turn to john".to_string(),
                20 => "three sixteen".to_string(),
                _ => "and the lord said".to_string(),
            };
            if self.last_transcript.as_deref() == Some(transcript.as_str()) {
                Ok(None)
            } else {
                self.last_transcript = Some(transcript.clone());
                Ok(Some(transcript))
            }
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) {
        self.chunk_count = 0;
        self.last_transcript = None;
    }

    fn language(&self) -> &str {
        &self.language
    }
}

#[cfg(not(feature = "sherpa"))]
#[derive(Debug)]
pub struct SherpaAsrEngine {
    language: String,
}

#[cfg(not(feature = "sherpa"))]
impl SherpaAsrEngine {
    pub fn new(_config: AsrModelConfig) -> Result<Self, AsrError> {
        Err(AsrError::FeatureUnavailable(
            "sherpa-onnx ASR is not compiled in; build bible-asr with --features sherpa",
        ))
    }
}

#[cfg(not(feature = "sherpa"))]
impl AsrEngine for SherpaAsrEngine {
    fn feed(&mut self, _chunk: &AudioChunk) -> Result<Option<String>, AsrError> {
        Err(AsrError::FeatureUnavailable(
            "sherpa-onnx ASR is not available in this build",
        ))
    }

    fn reset(&mut self) {}

    fn language(&self) -> &str {
        &self.language
    }
}

// ── Sherpa-ONNX Native Implementation ───────────────────────────────────

#[cfg(feature = "sherpa")]
pub struct SherpaAsrEngine {
    language: String,
    recognizer: sherpa_onnx::OnlineRecognizer,
    stream: sherpa_onnx::OnlineStream,
    last_text: String,
}

#[cfg(feature = "sherpa")]
impl std::fmt::Debug for SherpaAsrEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SherpaAsrEngine")
            .field("language", &self.language)
            .field("last_text", &self.last_text)
            .finish()
    }
}

#[cfg(feature = "sherpa")]
impl SherpaAsrEngine {
    pub fn new(config: AsrModelConfig) -> Result<Self, AsrError> {
        config.validate()?;

        let mut recognizer_config = sherpa_onnx::OnlineRecognizerConfig::default();
        recognizer_config.model_config.transducer.encoder = Some(config.encoder_path);
        recognizer_config.model_config.transducer.decoder = Some(config.decoder_path);
        recognizer_config.model_config.transducer.joiner = Some(config.joiner_path);
        recognizer_config.model_config.tokens = Some(config.tokens_path);
        recognizer_config.model_config.num_threads = config.num_threads as i32;
        
        let recognizer = sherpa_onnx::OnlineRecognizer::create(&recognizer_config)
            .ok_or_else(|| AsrError::ModelLoadFailed("Failed to create Sherpa-ONNX recognizer".into()))?;
            
        let stream = recognizer.create_stream();

        Ok(Self {
            language: config.language,
            recognizer,
            stream,
            last_text: String::new(),
        })
    }
}

#[cfg(feature = "sherpa")]
impl AsrEngine for SherpaAsrEngine {
    fn feed(&mut self, chunk: &AudioChunk) -> Result<Option<String>, AsrError> {
        let samples_f32: Vec<f32> = chunk.samples.iter().map(|&s| s as f32 / 32768.0).collect();
        
        self.stream.accept_waveform(16000, &samples_f32);

        while self.recognizer.is_ready(&self.stream) {
            self.recognizer.decode(&self.stream);
        }

        if let Some(result) = self.recognizer.get_result(&self.stream) {
            let text = result.text.to_lowercase();
            
            if text.is_empty() || text == self.last_text {
                Ok(None)
            } else {
                self.last_text = text.clone();
                Ok(Some(text))
            }
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) {
        self.stream = self.recognizer.create_stream();
        self.last_text.clear();
    }

    fn language(&self) -> &str {
        &self.language
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::err_expect)]
mod tests {
    use super::*;

    #[test]
    fn mock_asr_engine_basic() {
        let mut engine = MockAsrEngine::new("en".to_string());
        assert_eq!(engine.language(), "en");

        // Feed chunks without signal
        let silent_chunk = AudioChunk::new(vec![0i16; 1600], 1000);
        assert!(engine.feed(&silent_chunk).unwrap().is_none());

        // Feed chunks with signal
        let signal_chunk = AudioChunk::new(vec![100i16; 1600], 2000);
        let mut saw_transcript = false;
        for _ in 0..10 {
            if engine.feed(&signal_chunk).unwrap().is_some() {
                saw_transcript = true;
            }
        }
        assert!(saw_transcript);
    }

    #[test]
    fn model_validation_rejects_missing_paths() {
        assert!(matches!(
            AsrModelConfig::default().validate(),
            Err(AsrError::ModelLoadFailed(_))
        ));
    }

    #[test]
    fn model_validation_rejects_missing_vad_model() {
        let mut config = AsrModelConfig {
            encoder_path: "encoder".to_string(),
            decoder_path: "decoder".to_string(),
            joiner_path: "joiner".to_string(),
            tokens_path: "tokens".to_string(),
            ..AsrModelConfig::default()
        };
        assert!(matches!(config.validate(), Err(AsrError::VadLoadFailed(_))));
        config.enable_vad = false;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn sherpa_engine_is_honest_when_unavailable() {
        let error = SherpaAsrEngine::new(AsrModelConfig::default())
            .err()
            .expect("test error");
        assert!(matches!(error, AsrError::FeatureUnavailable(_)));
    }
}
