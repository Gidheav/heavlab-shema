//! bible-asr — ASR pipeline for real-time Bible verse detection.
//!
//! Wraps sherpa-onnx for speech recognition with OS-specific audio
//! capture behind a trait. No async runtime — threading uses
//! `std::thread` + `crossbeam-channel` only.
//!
//! Phase 3: streaming sherpa-onnx (zipformer transducer) and Silero VAD
//! plug into the `AsrEngine` and `Vad` trait boundaries when enabled.
#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod capture;
#[cfg(feature = "desktop-audio")]
pub mod capture_cpal;
pub mod engine;
pub mod error;
pub mod ring;
pub mod vad;
pub mod worker;

pub use capture::{
    create_capture, AudioChunk, CaptureConfig, MicrophoneCapture, MockCapture, PushCapture,
};
#[cfg(feature = "desktop-audio")]
pub use capture_cpal::CpalCapture;
pub use engine::{AsrEngine, AsrModelConfig, MockAsrEngine, SherpaAsrEngine};
pub use error::AsrError;
pub use ring::AudioRing;
pub use vad::{SimpleVad, Vad, VadConfig, VadState};
pub use worker::AudioWorker;

// Re-export platform-specific implementations when available
#[cfg(target_os = "ios")]
pub use capture::PushCapture as IosCapture;

#[cfg(target_os = "android")]
pub use capture::PushCapture as AndroidCapture;
