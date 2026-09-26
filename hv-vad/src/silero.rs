use crate::{Vad, VadConfig, VadError, VadState};
use hv_audio::AudioChunk;
use ort::{Session, SessionOutputs};
use std::path::Path;

pub struct SileroVad {
    session: Session,
    config: VadConfig,
    h: ndarray::Array3<f32>,
    c: ndarray::Array3<f32>,
    speech_duration: u64,
    silence_duration: u64,
    is_speech: bool,
    threshold: f32,
}

impl SileroVad {
    pub fn new(model_path: &Path, config: VadConfig) -> Result<Self, VadError> {
        if !model_path.exists() {
            return Err(VadError::ModelPath(format!(
                "Silero VAD model not found at {}",
                model_path.display()
            )));
        }

        let session = Session::builder()
            .map_err(|e: ort::Error| VadError::Onnx(e.to_string()))?
            .with_optimization_level(ort::GraphOptimizationLevel::Level1)
            .map_err(|e: ort::Error| VadError::Onnx(e.to_string()))?
            .with_intra_threads(1)
            .map_err(|e: ort::Error| VadError::Onnx(e.to_string()))?
            .commit_from_file(model_path)
            .map_err(|e: ort::Error| VadError::Onnx(e.to_string()))?;

        Ok(Self {
            session,
            config,
            h: ndarray::Array3::<f32>::zeros((2, 1, 64)),
            c: ndarray::Array3::<f32>::zeros((2, 1, 64)),
            speech_duration: 0,
            silence_duration: 0,
            is_speech: false,
            threshold: 0.5,
        })
    }
}

impl Vad for SileroVad {
    fn process(&mut self, chunk: &AudioChunk) -> VadState {
        let chunk_ms = chunk.duration_ms();

        // Convert i16 samples to f32 normalized
        let mut f32_samples = Vec::with_capacity(chunk.samples.len());
        for &sample in &chunk.samples {
            f32_samples.push(sample as f32 / 32768.0);
        }

        // Inputs for Silero VAD v4
        let input_tensor = ndarray::Array2::from_shape_vec((1, f32_samples.len()), f32_samples).unwrap_or_else(|_| ndarray::Array2::zeros((1, 1)));
        let sr_tensor = ndarray::Array1::from_vec(vec![16000_i64]);

        let prob = (|| -> Result<f32, ort::Error> {
            let input_val = ort::Value::from_array(input_tensor)?;
            let sr_val = ort::Value::from_array(sr_tensor)?;
            let h_val = ort::Value::from_array(self.h.clone())?;
            let c_val = ort::Value::from_array(self.c.clone())?;

            let inputs = ort::inputs![
                "input" => input_val,
                "sr" => sr_val,
                "h" => h_val,
                "c" => c_val,
            ]?;

            let outputs = self.session.run(inputs)?;

            let prob = outputs["output"]
                .try_extract_tensor::<f32>()?
                .into_dimensionality::<ndarray::Ix2>()
                .map(|t| t[[0, 0]])
                .unwrap_or(0.0);

            if let Ok(hn) = outputs["hn"].try_extract_tensor::<f32>() {
                self.h.assign(&hn);
            }
            if let Ok(cn) = outputs["cn"].try_extract_tensor::<f32>() {
                self.c.assign(&cn);
            }
            
            Ok(prob)
        })().unwrap_or(0.0);

        if prob >= self.threshold {
            self.speech_duration += chunk_ms;
            self.silence_duration = 0;
            if self.speech_duration >= self.config.min_speech_ms {
                self.is_speech = true;
            }
        } else {
            self.silence_duration += chunk_ms;
            if self.silence_duration >= self.config.silence_timeout_ms {
                self.is_speech = false;
                self.speech_duration = 0;
            }
        }

        if self.is_speech {
            VadState::Speech
        } else {
            VadState::Silence {
                duration_ms: self.silence_duration,
            }
        }
    }

    fn reset(&mut self) {
        self.h = ndarray::Array3::<f32>::zeros((2, 1, 64));
        self.c = ndarray::Array3::<f32>::zeros((2, 1, 64));
        self.speech_duration = 0;
        self.silence_duration = 0;
        self.is_speech = false;
    }

    fn config(&self) -> &VadConfig {
        &self.config
    }
}
