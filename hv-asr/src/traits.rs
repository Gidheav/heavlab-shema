//! ASR engine boundary types.

use std::path::Path;

use hv_audio::AudioChunk;
use thiserror::Error;

/// Incremental or final transcript emitted by an engine.
#[derive(Debug, Clone, PartialEq)]
pub struct Transcript {
    pub text: String,
    pub is_final: bool,
    pub confidence: f32,
}

impl Transcript {
    pub fn partial(text: impl Into<String>, confidence: f32) -> Self {
        Self {
            text: text.into(),
            is_final: false,
            confidence,
        }
    }

    pub fn final_text(text: impl Into<String>, confidence: f32) -> Self {
        Self {
            text: text.into(),
            is_final: true,
            confidence,
        }
    }
}

/// Files and runtime options for a streaming transducer model.
#[derive(Debug, Clone)]
pub struct AsrConfig {
    pub encoder_path: String,
    pub decoder_path: String,
    pub joiner_path: String,
    pub tokens_path: String,
    pub num_threads: usize,
    pub use_gpu: bool,
    pub language: String,
    pub hotwords_file: Option<String>,
    pub decoding_method: String,
    pub max_active_paths: usize,
    pub blank_penalty: f32,
}

impl Default for AsrConfig {
    fn default() -> Self {
        Self {
            encoder_path: String::new(),
            decoder_path: String::new(),
            joiner_path: String::new(),
            tokens_path: String::new(),
            num_threads: 2,
            use_gpu: false,
            language: "en".to_string(),
            hotwords_file: None,
            decoding_method: "modified_beam_search".to_string(),
            max_active_paths: 4,
            blank_penalty: 2.0,
        }
    }
}

impl AsrConfig {
    /// Validate that required model files exist. Never panics.
    pub fn validate(&self) -> Result<(), AsrError> {
        let required = [
            ("encoder", self.encoder_path.as_str()),
            ("decoder", self.decoder_path.as_str()),
            ("joiner", self.joiner_path.as_str()),
            ("tokens", self.tokens_path.as_str()),
        ];
        for (label, path) in required {
            if path.is_empty() {
                return Err(AsrError::InvalidConfig(format!(
                    "{label} path is required"
                )));
            }
            if !Path::new(path).is_file() {
                return Err(AsrError::ModelNotFound {
                    label: label.to_string(),
                    path: path.to_string(),
                });
            }
        }
        if self.num_threads == 0 {
            return Err(AsrError::InvalidConfig(
                "num_threads must be at least 1".to_string(),
            ));
        }
        Ok(())
    }
}

/// Streaming speech-recognition engine owned by the worker thread.
pub trait AsrEngine: Send + 'static {
    fn feed(&mut self, chunk: &AudioChunk) -> Result<Option<Transcript>, AsrError>;
    fn reset(&mut self) -> Result<(), AsrError>;
    fn name(&self) -> &str;
}

/// Typed ASR failures. Callers must handle these; nothing here panics.
#[derive(Debug, Error)]
pub enum AsrError {
    #[error("invalid ASR config: {0}")]
    InvalidConfig(String),
    #[error("ASR model {label} not found at {path}")]
    ModelNotFound { label: String, path: String },
    #[error("model load failed: {0}")]
    ModelLoadFailed(String),
    #[error("inference error: {0}")]
    InferenceError(String),
    #[error("feature unavailable: {0}")]
    FeatureUnavailable(&'static str),
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::AsrConfig;

    #[test]
    fn asr_config_rejects_missing_paths() {
        let config = AsrConfig {
            encoder_path: "does-not-exist-encoder.onnx".to_string(),
            decoder_path: "does-not-exist-decoder.onnx".to_string(),
            joiner_path: "does-not-exist-joiner.onnx".to_string(),
            tokens_path: "does-not-exist-tokens.txt".to_string(),
            ..AsrConfig::default()
        };
        let err = config.validate().expect_err("missing files must error");
        let msg = err.to_string();
        assert!(
            msg.contains("not found") || msg.contains("required"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn asr_config_rejects_empty_paths() {
        let err = AsrConfig::default()
            .validate()
            .expect_err("empty paths must error");
        assert!(err.to_string().contains("required"));
    }
}
