//! Model registry for discovering and cataloging available ASR models.

use std::path::Path;

/// Describes one ASR model available on disk.
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub engine_type: EngineType,
    pub language: String,
    pub is_streaming: bool,
    pub size_mb: u64,
    pub path: std::path::PathBuf,
    pub available: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EngineType {
    SherpaZipformer,
    SherpaWhisper,
    WhisperCpp,
    Moonshine,
    Mock,
}

/// Scan data/models/ and return all available models.
pub fn scan_models(models_dir: &Path) -> Vec<ModelInfo> {
    let mut models = Vec::new();

    // Sherpa Zipformer EN (streaming)
    let zipformer_en_path = models_dir.join("sherpa-onnx-streaming-zipformer-en-2023-06-26");
    let zipformer_en = ModelInfo {
        id: "sherpa-zipformer-en".to_string(),
        display_name: "Zipformer EN (Streaming)".to_string(),
        engine_type: EngineType::SherpaZipformer,
        language: "en".to_string(),
        is_streaming: true,
        size_mb: calculate_dir_size(&zipformer_en_path),
        path: zipformer_en_path.clone(),
        available: check_sherpa_zipformer_files(&zipformer_en_path),
    };
    models.push(zipformer_en);

    // Sherpa Whisper Tiny EN
    let whisper_tiny_path = models_dir.join("sherpa-onnx-whisper-tiny.en");
    let whisper_tiny = ModelInfo {
        id: "sherpa-whisper-tiny-en".to_string(),
        display_name: "Sherpa Whisper Tiny EN".to_string(),
        engine_type: EngineType::SherpaWhisper,
        language: "en".to_string(),
        is_streaming: false,
        size_mb: calculate_dir_size(&whisper_tiny_path),
        path: whisper_tiny_path.clone(),
        available: check_sherpa_whisper_files(&whisper_tiny_path),
    };
    models.push(whisper_tiny);

    // Whisper Tiny EN (whisper.cpp format)
    let whisper_cpp_tiny_path = models_dir.join("whisper/ggml-tiny.en.bin");
    let whisper_cpp_tiny = ModelInfo {
        id: "whisper-cpp-tiny-en".to_string(),
        display_name: "Whisper Tiny EN (ggml)".to_string(),
        engine_type: EngineType::WhisperCpp,
        language: "en".to_string(),
        is_streaming: false,
        size_mb: calculate_file_size(&whisper_cpp_tiny_path),
        path: whisper_cpp_tiny_path.clone(),
        available: whisper_cpp_tiny_path.is_file(),
    };
    models.push(whisper_cpp_tiny);

    // Whisper Small EN (whisper.cpp format)
    let whisper_cpp_small_en_path = models_dir.join("whisper/ggml-small.en.bin");
    let whisper_cpp_small_en = ModelInfo {
        id: "whisper-cpp-small-en".to_string(),
        display_name: "Whisper Small EN (ggml)".to_string(),
        engine_type: EngineType::WhisperCpp,
        language: "en".to_string(),
        is_streaming: false,
        size_mb: calculate_file_size(&whisper_cpp_small_en_path),
        path: whisper_cpp_small_en_path.clone(),
        available: whisper_cpp_small_en_path.is_file(),
    };
    models.push(whisper_cpp_small_en);

    // Whisper Small Multilingual (whisper.cpp format)
    let whisper_cpp_small_path = models_dir.join("whisper/ggml-small.bin");
    let whisper_cpp_small = ModelInfo {
        id: "whisper-cpp-small".to_string(),
        display_name: "Whisper Small Multilingual (ggml)".to_string(),
        engine_type: EngineType::WhisperCpp,
        language: "multilingual".to_string(),
        is_streaming: false,
        size_mb: calculate_file_size(&whisper_cpp_small_path),
        path: whisper_cpp_small_path.clone(),
        available: whisper_cpp_small_path.is_file(),
    };
    models.push(whisper_cpp_small);

    // Moonshine Tiny
    let moonshine_tiny_path = models_dir.join("moonshine-tiny");
    let moonshine_tiny = ModelInfo {
        id: "moonshine-tiny".to_string(),
        display_name: "Moonshine Tiny".to_string(),
        engine_type: EngineType::Moonshine,
        language: "en".to_string(),
        is_streaming: false,
        size_mb: calculate_dir_size(&moonshine_tiny_path),
        path: moonshine_tiny_path.clone(),
        available: check_moonshine_files(&moonshine_tiny_path),
    };
    models.push(moonshine_tiny);

    models
}

fn check_sherpa_zipformer_files(path: &Path) -> bool {
    path.join("encoder-epoch-99-avg-1-chunk-16-left-128.onnx").is_file()
        && path.join("decoder-epoch-99-avg-1-chunk-16-left-128.onnx").is_file()
        && path.join("joiner-epoch-99-avg-1-chunk-16-left-128.onnx").is_file()
        && path.join("tokens.txt").is_file()
}

fn check_sherpa_whisper_files(path: &Path) -> bool {
    path.join("tiny.en-encoder.onnx").is_file()
        && path.join("tiny.en-decoder.onnx").is_file()
        && path.join("tiny.en-tokens.txt").is_file()
}

fn check_moonshine_files(path: &Path) -> bool {
    path.join("preprocess.onnx").is_file()
        && path.join("encode.onnx").is_file()
        && path.join("cached_decode.onnx").is_file()
        && path.join("uncached_decode.onnx").is_file()
}

fn calculate_dir_size(path: &Path) -> u64 {
    if !path.is_dir() {
        return 0;
    }

    let mut total_size = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }
    }
    total_size / (1024 * 1024) // Convert to MB
}

fn calculate_file_size(path: &Path) -> u64 {
    if let Ok(metadata) = path.metadata() {
        metadata.len() / (1024 * 1024) // Convert to MB
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_models_returns_expected_models() {
        let temp_dir = std::env::temp_dir();
        let models = scan_models(&temp_dir);
        
        // Should return all model definitions even if files don't exist
        assert!(!models.is_empty());
        
        // Check that IDs are unique
        let ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
        let unique_ids: std::collections::HashSet<_> = ids.into_iter().collect();
        assert_eq!(unique_ids.len(), models.len());
    }

    #[test]
    fn test_model_info_structure() {
        let model = ModelInfo {
            id: "test-model".to_string(),
            display_name: "Test Model".to_string(),
            engine_type: EngineType::Mock,
            language: "en".to_string(),
            is_streaming: false,
            size_mb: 100,
            path: Path::new("/test/path").to_path_buf(),
            available: true,
        };

        assert_eq!(model.id, "test-model");
        assert_eq!(model.display_name, "Test Model");
        assert_eq!(model.engine_type, EngineType::Mock);
        assert!(model.available);
    }
}
