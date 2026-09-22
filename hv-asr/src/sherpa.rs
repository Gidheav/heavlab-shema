//! Sherpa-ONNX streaming Zipformer engine.

use crate::traits::{AsrConfig, AsrEngine, AsrError, Transcript};
use hv_audio::AudioChunk;

#[cfg(not(feature = "sherpa"))]
#[derive(Debug)]
pub struct SherpaAsrEngine {
    language: String,
}

#[cfg(not(feature = "sherpa"))]
impl SherpaAsrEngine {
    pub fn new(config: AsrConfig) -> Result<Self, AsrError> {
        config.validate()?;
        let _ = config.use_gpu;
        Err(AsrError::FeatureUnavailable(
            "sherpa-onnx ASR is not compiled in; build hv-asr with --features sherpa",
        ))
    }
}

#[cfg(not(feature = "sherpa"))]
impl AsrEngine for SherpaAsrEngine {
    fn feed(&mut self, _chunk: &AudioChunk) -> Result<Option<Transcript>, AsrError> {
        Err(AsrError::FeatureUnavailable(
            "sherpa-onnx ASR is not available in this build",
        ))
    }

    fn reset(&mut self) -> Result<(), AsrError> {
        Ok(())
    }

    fn name(&self) -> &str {
        &self.language
    }
}

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
    pub fn new(config: AsrConfig) -> Result<Self, AsrError> {
        config.validate()?;

        let mut recognizer_config = sherpa_onnx::OnlineRecognizerConfig::default();
        recognizer_config.model_config.transducer.encoder = Some(config.encoder_path);
        recognizer_config.model_config.transducer.decoder = Some(config.decoder_path);
        recognizer_config.model_config.transducer.joiner = Some(config.joiner_path);
        recognizer_config.model_config.tokens = Some(config.tokens_path);
        recognizer_config.model_config.num_threads = config.num_threads as i32;
        if config.use_gpu {
            // Set GPU provider based on platform (Windows: DirectML, macOS: CoreML, Linux: CUDA)
            let provider = if cfg!(target_os = "windows") {
                "dml"
            } else if cfg!(target_os = "macos") {
                "coreml"
            } else {
                "cuda"
            };
            recognizer_config.model_config.provider = Some(provider.to_string());
        }

        recognizer_config.hotwords_file = config.hotwords_file;
        recognizer_config.decoding_method = Some(config.decoding_method);
        recognizer_config.max_active_paths = config.max_active_paths as i32;
        recognizer_config.blank_penalty = config.blank_penalty;

        let recognizer = sherpa_onnx::OnlineRecognizer::create(&recognizer_config).ok_or_else(
            || AsrError::ModelLoadFailed("failed to create sherpa-onnx recognizer".into()),
        )?;
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
    fn feed(&mut self, chunk: &AudioChunk) -> Result<Option<Transcript>, AsrError> {
        let samples_f32: Vec<f32> = chunk
            .samples
            .iter()
            .map(|&sample| sample as f32 / 32768.0)
            .collect();

        self.stream.accept_waveform(16_000, &samples_f32);

        while self.recognizer.is_ready(&self.stream) {
            self.recognizer.decode(&self.stream);
        }

        let is_endpoint = self.recognizer.is_endpoint(&self.stream);
        let Some(result) = self.recognizer.get_result(&self.stream) else {
            return Ok(None);
        };
        
        if is_endpoint {
            self.recognizer.reset(&self.stream);
        }
        
        let text = result.text.to_lowercase();
        if text.is_empty() || text == self.last_text {
            return Ok(None);
        }
        
        self.last_text = text.clone();
        
        // A heuristic for confidence since the Rust bindings don't expose it yet.
        // We use a baseline of 0.88 + up to 0.1 depending on text length, 
        // maxing out at 0.98. Longer texts often have higher confidence if not rejected.
        let length_bonus = (text.len() as f32 * 0.005).min(0.10);
        let base_conf = if is_endpoint { 0.90 } else { 0.85 };
        let confidence = (base_conf + length_bonus).min(0.99);
        
        Ok(Some(Transcript {
            text,
            is_final: is_endpoint,
            confidence, 
        }))
    }

    fn reset(&mut self) -> Result<(), AsrError> {
        self.stream = self.recognizer.create_stream();
        self.last_text.clear();
        Ok(())
    }

    fn name(&self) -> &str {
        "sherpa-zipformer"
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::SherpaAsrEngine;
    use crate::traits::{AsrConfig, AsrError};

    #[test]
    fn sherpa_new_errors_when_model_files_missing() {
        let config = AsrConfig {
            encoder_path: "missing-encoder.onnx".to_string(),
            decoder_path: "missing-decoder.onnx".to_string(),
            joiner_path: "missing-joiner.onnx".to_string(),
            tokens_path: "missing-tokens.txt".to_string(),
            ..AsrConfig::default()
        };
        let err = SherpaAsrEngine::new(config).expect_err("missing models must not panic");
        match err {
            AsrError::ModelNotFound { .. } | AsrError::InvalidConfig(_) => {}
            other => panic!("expected missing-file error, got {other:?}"),
        }
    }
}