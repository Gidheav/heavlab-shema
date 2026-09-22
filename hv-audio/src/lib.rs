//! hv-audio — capture, ring buffer, and metering.

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod capture;
#[cfg(feature = "desktop")]
pub mod capture_cpal;
pub mod meter;
pub mod ring;
pub mod dsp;

pub use capture::{
    AudioChunk, AudioDevice, CaptureConfig, CaptureError, MicrophoneCapture, MockCapture,
    PushCapture,
};
#[cfg(feature = "desktop")]
pub use capture_cpal::CpalCapture;
pub use meter::{AudioMeter, MeterReading};
pub use ring::AudioRing;
pub use dsp::{NoiseGate, HighPassFilter, Compressor};
