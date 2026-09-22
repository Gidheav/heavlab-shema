//! Canonical audio types and capture traits.
//!
//! `AudioChunk` is the project-wide PCM type. Every other crate imports it from here.

use crossbeam_channel::{Receiver, Sender, TrySendError};
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// A chunk of 16 kHz, mono, signed 16-bit PCM audio.
#[derive(Debug, Clone)]
pub struct AudioChunk {
    /// PCM samples in signed 16-bit mono format.
    pub samples: Vec<i16>,
    /// Monotonic capture timestamp in milliseconds.
    pub timestamp_ms: u64,
}

impl AudioChunk {
    /// Create an audio chunk with its capture timestamp.
    pub fn new(samples: Vec<i16>, timestamp_ms: u64) -> Self {
        Self {
            samples,
            timestamp_ms,
        }
    }

    /// Chunk duration at the required 16 kHz sample rate.
    pub fn duration_ms(&self) -> u64 {
        (self.samples.len() as u64 * 1000) / 16_000
    }

    /// Whether any sample is nonzero.
    pub fn has_signal(&self) -> bool {
        self.samples.iter().any(|&sample| sample != 0)
    }
}

/// Enumerated input device for the UI dropdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Capture settings asserted by the caller.
#[derive(Debug, Clone, Copy)]
pub struct CaptureConfig {
    pub sample_rate_hz: u32,
    pub chunk_ms: u32,
    pub require_aec: bool,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            sample_rate_hz: 16_000,
            chunk_ms: 100,
            require_aec: false,
        }
    }
}

/// Failures from microphone capture. Never panics.
#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("capture error: {0}")]
    Capture(String),
    #[error("already listening")]
    AlreadyListening,
    #[error("not listening")]
    NotListening,
    #[error("channel error: {0}")]
    Channel(String),
    #[error("acoustic echo cancellation is unavailable")]
    AecUnavailable,
}

/// Platform microphone capture.
pub trait MicrophoneCapture: Send + Sync {
    fn start(&self) -> Result<Receiver<AudioChunk>, CaptureError>;

    fn start_with_aec(&self, aec_enabled: bool) -> Result<Receiver<AudioChunk>, CaptureError> {
        let _ = aec_enabled;
        self.start()
    }

    fn stop(&self);

    fn push_samples(&self, _samples: &[i16], _timestamp_ms: u64) -> Result<(), CaptureError> {
        Err(CaptureError::Channel(
            "capture does not accept pushed samples".to_string(),
        ))
    }

    fn sample_rate_hz(&self) -> u32 {
        16_000
    }
}

#[derive(Debug)]
struct CaptureState {
    active: bool,
    sender: Option<Sender<AudioChunk>>,
}

/// Capture endpoint fed by a platform shell (push PCM from the audio callback).
#[derive(Debug)]
pub struct PushCapture {
    config: CaptureConfig,
    state: Arc<Mutex<CaptureState>>,
}

impl PushCapture {
    pub fn new(config: CaptureConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(CaptureState {
                active: false,
                sender: None,
            })),
        }
    }

    pub fn config(&self) -> CaptureConfig {
        self.config
    }

    pub fn start_with_aec(&self, aec_enabled: bool) -> Result<Receiver<AudioChunk>, CaptureError> {
        if self.config.require_aec && !aec_enabled {
            return Err(CaptureError::AecUnavailable);
        }
        let mut state = self
            .state
            .lock()
            .map_err(|error| CaptureError::Capture(format!("capture lock error: {error}")))?;
        if state.active {
            return Err(CaptureError::AlreadyListening);
        }

        let chunk_ms = self.config.chunk_ms.max(1) as usize;
        let queue_capacity = 2_000usize.div_ceil(chunk_ms);
        let (sender, receiver) = crossbeam_channel::bounded(queue_capacity);
        state.sender = Some(sender);
        state.active = true;
        Ok(receiver)
    }

    pub fn push_samples(&self, samples: &[i16], timestamp_ms: u64) -> Result<(), CaptureError> {
        let state = self
            .state
            .lock()
            .map_err(|error| CaptureError::Capture(format!("capture lock error: {error}")))?;
        let sender = state.sender.as_ref().ok_or(CaptureError::NotListening)?;
        sender
            .try_send(AudioChunk::new(samples.to_vec(), timestamp_ms))
            .map_err(|error| match error {
                TrySendError::Full(_) => CaptureError::Channel(
                    "audio queue is full; frame dropped to protect callback thread".to_string(),
                ),
                TrySendError::Disconnected(_) => {
                    CaptureError::Channel("audio queue is disconnected".to_string())
                }
            })
    }
}

impl Default for PushCapture {
    fn default() -> Self {
        Self::new(CaptureConfig::default())
    }
}

impl MicrophoneCapture for PushCapture {
    fn start_with_aec(&self, aec_enabled: bool) -> Result<Receiver<AudioChunk>, CaptureError> {
        PushCapture::start_with_aec(self, aec_enabled)
    }

    fn start(&self) -> Result<Receiver<AudioChunk>, CaptureError> {
        if self.config.require_aec {
            return Err(CaptureError::AecUnavailable);
        }
        self.start_with_aec(false)
    }

    fn stop(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.active = false;
            state.sender = None;
        }
    }

    fn push_samples(&self, samples: &[i16], timestamp_ms: u64) -> Result<(), CaptureError> {
        PushCapture::push_samples(self, samples, timestamp_ms)
    }

    fn sample_rate_hz(&self) -> u32 {
        self.config.sample_rate_hz
    }
}

/// Mock capture for desktop development and pipeline tests.
#[derive(Debug)]
pub struct MockCapture {
    state: Arc<Mutex<CaptureState>>,
}

impl MockCapture {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(CaptureState {
                active: false,
                sender: None,
            })),
        }
    }

    pub fn inject(&self, chunk: AudioChunk) -> Result<(), CaptureError> {
        let state = self
            .state
            .lock()
            .map_err(|error| CaptureError::Capture(format!("capture lock error: {error}")))?;
        state
            .sender
            .as_ref()
            .ok_or(CaptureError::NotListening)?
            .send(chunk)
            .map_err(|error| {
                CaptureError::Channel(format!("mock audio queue disconnected: {error}"))
            })
    }
}

impl Default for MockCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl MicrophoneCapture for MockCapture {
    fn start(&self) -> Result<Receiver<AudioChunk>, CaptureError> {
        let mut state = self
            .state
            .lock()
            .map_err(|error| CaptureError::Capture(format!("capture lock error: {error}")))?;
        if state.active {
            return Err(CaptureError::AlreadyListening);
        }
        let (sender, receiver) = crossbeam_channel::unbounded();
        state.sender = Some(sender);
        state.active = true;
        Ok(receiver)
    }

    fn stop(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.active = false;
            state.sender = None;
        }
    }

    fn push_samples(&self, samples: &[i16], timestamp_ms: u64) -> Result<(), CaptureError> {
        self.inject(AudioChunk::new(samples.to_vec(), timestamp_ms))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn mock_capture_injects_chunks() {
        let capture = MockCapture::new();
        assert!(matches!(
            capture.inject(AudioChunk::new(vec![1], 0)),
            Err(CaptureError::NotListening)
        ));
        let receiver = capture.start().expect("start");
        capture.inject(AudioChunk::new(vec![5], 7)).expect("inject");
        assert_eq!(receiver.recv().expect("chunk").timestamp_ms, 7);
        capture.stop();
        assert!(matches!(
            capture.inject(AudioChunk::new(vec![1], 8)),
            Err(CaptureError::NotListening)
        ));
    }

    #[test]
    fn push_capture_reports_backpressure_without_blocking() {
        let capture = PushCapture::new(CaptureConfig {
            chunk_ms: 1000,
            require_aec: false,
            ..CaptureConfig::default()
        });
        let _receiver = capture.start().expect("start");
        let mut full = false;
        for timestamp in 0..100 {
            if capture.push_samples(&[1], timestamp).is_err() {
                full = true;
                break;
            }
        }
        assert!(full);
    }
}
