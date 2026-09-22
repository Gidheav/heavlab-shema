//! Audio capture boundary.
//!
//! The iOS and Android shells own AVAudioEngine/AudioRecord and platform AEC.
//! They push converted 16 kHz mono i16 frames through [`PushCapture`].

use crate::error::AsrError;
use crossbeam_channel::{Receiver, Sender, TrySendError};
use std::sync::{Arc, Mutex};

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

    /// Return the chunk duration at the required 16 kHz sample rate.
    pub fn duration_ms(&self) -> u64 {
        (self.samples.len() as u64 * 1000) / 16_000
    }

    /// Return whether any sample is nonzero.
    pub fn has_signal(&self) -> bool {
        self.samples.iter().any(|&sample| sample != 0)
    }
}

/// Capture settings asserted by the platform shell.
#[derive(Debug, Clone, Copy)]
pub struct CaptureConfig {
    /// Input sample rate expected from the platform shell.
    pub sample_rate_hz: u32,
    /// Target duration of each pushed chunk.
    pub chunk_ms: u32,
    /// Whether the shell must assert platform AEC before starting.
    pub require_aec: bool,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            sample_rate_hz: 16_000,
            chunk_ms: 100,
            require_aec: true,
        }
    }
}

/// Platform microphone capture.
pub trait MicrophoneCapture: Send + Sync {
    /// Start capture without an AEC assertion.
    fn start(&self) -> Result<Receiver<AudioChunk>, AsrError>;

    /// Start capture after the caller asserts whether platform AEC is enabled.
    ///
    /// Implementations that do not require an AEC assertion delegate to
    /// [`MicrophoneCapture::start`].
    fn start_with_aec(&self, aec_enabled: bool) -> Result<Receiver<AudioChunk>, AsrError> {
        let _ = aec_enabled;
        self.start()
    }

    /// Stop capture and release its active audio queue.
    fn stop(&self);

    /// Push a frame without blocking the platform audio callback.
    ///
    /// Implementations use a bounded queue of approximately two seconds of
    /// audio. A full or disconnected queue returns [`AsrError::ChannelError`]
    /// rather than blocking indefinitely.
    fn push_samples(&self, _samples: &[i16], _timestamp_ms: u64) -> Result<(), AsrError> {
        Err(AsrError::ChannelError(
            "capture does not accept pushed samples".to_string(),
        ))
    }

    /// Return the sample rate expected by this capture implementation.
    fn sample_rate_hz(&self) -> u32 {
        16_000
    }
}

#[derive(Debug)]
struct CaptureState {
    active: bool,
    sender: Option<Sender<AudioChunk>>,
}

/// Capture endpoint fed by the iOS/Android shell.
#[derive(Debug)]
pub struct PushCapture {
    config: CaptureConfig,
    state: Arc<Mutex<CaptureState>>,
}

impl PushCapture {
    /// Create a shell-fed capture endpoint.
    pub fn new(config: CaptureConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(CaptureState {
                active: false,
                sender: None,
            })),
        }
    }

    /// Return the capture settings used by this endpoint.
    pub fn config(&self) -> CaptureConfig {
        self.config
    }

    /// Start after the shell has asserted whether platform AEC is enabled.
    pub fn start_with_aec(&self, aec_enabled: bool) -> Result<Receiver<AudioChunk>, AsrError> {
        if self.config.require_aec && !aec_enabled {
            return Err(AsrError::AecUnavailable);
        }
        let mut state = self
            .state
            .lock()
            .map_err(|error| AsrError::CaptureError(format!("capture lock error: {error}")))?;
        if state.active {
            return Err(AsrError::AlreadyListening);
        }

        let chunk_ms = self.config.chunk_ms.max(1) as usize;
        // Bound the queue to approximately two seconds of audio.
        let queue_capacity = 2_000usize.div_ceil(chunk_ms);
        let (sender, receiver) = crossbeam_channel::bounded(queue_capacity);
        state.sender = Some(sender);
        state.active = true;
        Ok(receiver)
    }

    /// Queue PCM samples without blocking the platform callback thread.
    pub fn push_samples(&self, samples: &[i16], timestamp_ms: u64) -> Result<(), AsrError> {
        let state = self
            .state
            .lock()
            .map_err(|error| AsrError::CaptureError(format!("capture lock error: {error}")))?;
        let sender = state.sender.as_ref().ok_or(AsrError::NotListening)?;
        sender
            .try_send(AudioChunk::new(samples.to_vec(), timestamp_ms))
            .map_err(|error| match error {
                TrySendError::Full(_) => AsrError::ChannelError(
                    "audio queue is full; frame dropped to protect callback thread".to_string(),
                ),
                TrySendError::Disconnected(_) => {
                    AsrError::ChannelError("audio queue is disconnected".to_string())
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
    fn start_with_aec(&self, aec_enabled: bool) -> Result<Receiver<AudioChunk>, AsrError> {
        PushCapture::start_with_aec(self, aec_enabled)
    }

    fn start(&self) -> Result<Receiver<AudioChunk>, AsrError> {
        if self.config.require_aec {
            return Err(AsrError::AecUnavailable);
        }
        self.start_with_aec(false)
    }

    fn stop(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.active = false;
            state.sender = None;
        }
    }

    fn push_samples(&self, samples: &[i16], timestamp_ms: u64) -> Result<(), AsrError> {
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

    /// Inject a chunk into an active mock capture.
    pub fn inject(&self, chunk: AudioChunk) -> Result<(), AsrError> {
        let state = self
            .state
            .lock()
            .map_err(|error| AsrError::CaptureError(format!("capture lock error: {error}")))?;
        state
            .sender
            .as_ref()
            .ok_or(AsrError::NotListening)?
            .send(chunk)
            .map_err(|error| {
                AsrError::ChannelError(format!("mock audio queue disconnected: {error}"))
            })
    }
}

impl Default for MockCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl MicrophoneCapture for MockCapture {
    fn start(&self) -> Result<Receiver<AudioChunk>, AsrError> {
        let mut state = self
            .state
            .lock()
            .map_err(|error| AsrError::CaptureError(format!("capture lock error: {error}")))?;
        if state.active {
            return Err(AsrError::AlreadyListening);
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

    fn push_samples(&self, samples: &[i16], timestamp_ms: u64) -> Result<(), AsrError> {
        self.inject(AudioChunk::new(samples.to_vec(), timestamp_ms))
    }
}

/// Construct the platform capture boundary.
pub fn create_capture(config: CaptureConfig) -> Box<dyn MicrophoneCapture> {
    #[cfg(any(target_os = "ios", target_os = "android"))]
    {
        Box::new(PushCapture::new(config))
    }
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    {
        let _ = config;
        Box::new(MockCapture::new())
    }
}

#[cfg(target_os = "ios")]
pub use PushCapture as IosCapture;
#[cfg(target_os = "android")]
pub use PushCapture as AndroidCapture;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn push_capture_requires_aec_and_forwards_timestamp() {
        let capture = PushCapture::new(CaptureConfig::default());
        assert!(matches!(capture.start(), Err(AsrError::AecUnavailable)));
        assert!(matches!(
            capture.start_with_aec(false),
            Err(AsrError::AecUnavailable)
        ));
        let receiver = capture.start_with_aec(true).expect("start");
        capture.push_samples(&[1, 2, 3], 42).expect("push");
        let chunk = receiver.recv().expect("chunk");
        assert_eq!(chunk.samples, [1, 2, 3]);
        assert_eq!(chunk.timestamp_ms, 42);
    }

    #[test]
    fn push_capture_rejects_double_start_and_push_before_start() {
        let capture = PushCapture::new(CaptureConfig {
            require_aec: false,
            ..CaptureConfig::default()
        });
        assert!(matches!(
            capture.push_samples(&[1], 0),
            Err(AsrError::NotListening)
        ));
        let _receiver = capture.start().expect("start");
        assert!(matches!(capture.start(), Err(AsrError::AlreadyListening)));
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

    #[test]
    fn mock_capture_injects_chunks() {
        let capture = MockCapture::new();
        assert!(matches!(
            capture.inject(AudioChunk::new(vec![1], 0)),
            Err(AsrError::NotListening)
        ));
        let receiver = capture.start().expect("start");
        capture.inject(AudioChunk::new(vec![5], 7)).expect("inject");
        assert_eq!(receiver.recv().expect("chunk").timestamp_ms, 7);
        capture.stop();
        assert!(matches!(
            capture.inject(AudioChunk::new(vec![1], 8)),
            Err(AsrError::NotListening)
        ));
    }
}
