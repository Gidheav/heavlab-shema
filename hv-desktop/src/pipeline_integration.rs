//! Real audio pipeline integration using hv-pipeline.

use std::sync::Arc;

use bible_core::store::TranslationStore;
use hv_asr::{AsrConfig, AsrEngine, MockAsrEngine};
use hv_audio::{CpalCapture, MicrophoneCapture};
use hv_pipeline::Pipeline;
use hv_vad::{EnergyVad, VadConfig};

use crate::paths::data_dir;

/// Configuration for the real audio pipeline.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub audio_device: Option<String>,
    pub translation: String,
    pub use_mock_asr: bool,
    pub use_gpu: bool,
    pub asr_mode: String,
    pub asr_model_size: String,
    pub vad_mode: String,
    pub vad_sensitivity: f32,
    pub noise_gate_enabled: bool,
    pub noise_gate_threshold: f32,
    pub hpf_enabled: bool,
    pub hpf_frequency: f32,
    pub compressor_enabled: bool,
    pub compressor_ratio: f32,
    pub num_threads: usize,
    pub hotwords_enabled: bool,
    pub beam_size: usize,
}

/// Information about the active ASR engine for status display.
#[derive(Debug, Clone, PartialEq)]
pub enum AsrEngineInfo {
    Mock,
    SherpaCpu,
    SherpaGpu,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            audio_device: None,
            translation: "KJV".to_string(),
            use_mock_asr: true,
            use_gpu: false,
            asr_mode: "sherpa".to_string(),
            asr_model_size: "medium".to_string(),
            vad_mode: "hybrid".to_string(),
            vad_sensitivity: 0.5,
            noise_gate_enabled: false,
            noise_gate_threshold: 0.005,
            hpf_enabled: false,
            hpf_frequency: 80.0,
            compressor_enabled: false,
            compressor_ratio: 4.0,
            num_threads: 4,
            hotwords_enabled: true,
            beam_size: 4,
        }
    }
}

/// Create a real pipeline with actual audio capture, VAD, ASR, and verse detection.
pub fn create_pipeline(
    config: PipelineConfig,
) -> Result<(Pipeline, Arc<TranslationStore>, AsrEngineInfo), String> {
    // Set up audio capture
    let capture: Box<dyn MicrophoneCapture> = if let Some(device_name) = config.audio_device {
        Box::new(CpalCapture::with_device(device_name))
    } else {
        Box::new(CpalCapture::new())
    };

    // Set up VAD
    let mut vad_config = VadConfig::default();
    vad_config.energy_threshold = match config.vad_sensitivity {
        s if s < 0.3 => 1500.0,
        s if s > 0.7 => 500.0,
        _ => 1000.0,
    };
    let vad: Box<dyn hv_vad::Vad> = Box::new(EnergyVad::new(vad_config));

    // Set up ASR engine
    let (engine, asr_info): (Box<dyn AsrEngine>, AsrEngineInfo) = if config.use_mock_asr {
        (Box::new(MockAsrEngine::new()), AsrEngineInfo::Mock)
    } else {
        // Configure real ASR with model paths
        let data_path = data_dir();
        // Updated model directory to match the 2023-06-26 Zipformer model
        let model_path = data_path
            .join("models")
            .join("sherpa-onnx-streaming-zipformer-en-2023-06-26");

        let asr_config = AsrConfig {
            encoder_path: model_path
                .join("encoder-epoch-99-avg-1-chunk-16-left-128.onnx")
                .to_string_lossy()
                .to_string(),
            decoder_path: model_path
                .join("decoder-epoch-99-avg-1-chunk-16-left-128.onnx")
                .to_string_lossy()
                .to_string(),
            joiner_path: model_path
                .join("joiner-epoch-99-avg-1-chunk-16-left-128.onnx")
                .to_string_lossy()
                .to_string(),
            tokens_path: model_path.join("tokens.txt").to_string_lossy().to_string(),
            num_threads: config.num_threads,
            use_gpu: config.use_gpu,
            language: "en".to_string(),
            hotwords_file: if config.hotwords_enabled {
                let hotwords_path = data_path.join("hotwords.txt");
                if !hotwords_path.exists() {
                    let _ = hv_asr::hotwords::generate_hotwords_file(&hotwords_path);
                }
                Some(hotwords_path.to_string_lossy().to_string())
            } else {
                None
            },
            decoding_method: "modified_beam_search".to_string(),
            max_active_paths: config.beam_size,
            blank_penalty: 2.0,
        };

        match asr_config.validate() {
            Ok(_) => match hv_asr::SherpaAsrEngine::new(asr_config) {
                Ok(engine) => {
                    let info = if config.use_gpu {
                        AsrEngineInfo::SherpaGpu
                    } else {
                        AsrEngineInfo::SherpaCpu
                    };
                    (Box::new(engine), info)
                }
                Err(error) => {
                    tracing::error!(
                        "Failed to create sherpa ASR engine: {error}, falling back to mock"
                    );
                    (Box::new(MockAsrEngine::new()), AsrEngineInfo::Mock)
                }
            },
            Err(error) => {
                tracing::error!("ASR config validation failed: {error}, using mock ASR");
                (Box::new(MockAsrEngine::new()), AsrEngineInfo::Mock)
            }
        }
    };

    // Set up translation store
    let data_path = data_dir();
    let packs_path = data_path.join("packs");
    let store = Arc::new(TranslationStore::new());
    // Load translations
    let translations = ["KJV", "ESV", "NIV", "ASV"];
    for name in translations {
        let bible_path = packs_path.join(format!("{}.bible.bin", name));
        let offsets_path = packs_path.join(format!("{}.offsets.bin", name));
        if bible_path.exists() && offsets_path.exists() {
            if let Err(error) = store.load_translation(name, &bible_path, &offsets_path) {
                tracing::warn!("Failed to load {} translation: {}", name, error);
            }
        } else {
            tracing::warn!("Missing pack for translation: {}", name);
        }
    }

    // Create pipeline
    let pipeline = Pipeline::new(capture, vad, engine, Arc::clone(&store), config.translation)
        .map_err(|error| format!("Failed to create pipeline: {error}"))?;

    pipeline.send_command(hv_pipeline::PipelineCommand::SetNoiseGateThreshold(config.noise_gate_threshold));
    pipeline.send_command(hv_pipeline::PipelineCommand::SetHighPassFreq(config.hpf_frequency));
    pipeline.send_command(hv_pipeline::PipelineCommand::SetCompressorRatio(config.compressor_ratio));
    // pipeline.send_command(hv_pipeline::PipelineCommand::EnableProcessing(true)); // Optional, let user toggle it


    Ok((pipeline, store, asr_info))
}
