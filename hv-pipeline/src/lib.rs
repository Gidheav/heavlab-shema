//! hv-pipeline — orchestrator wiring capture, VAD, ASR, and verse detection.

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod detector;
pub mod normalizer;
pub mod error;
pub mod events;
pub mod worker;
pub mod dsp;

pub use detector::{DetectedVerse, VerseDetector};
pub use error::PipelineError;
pub use events::{PipelineCommand, PipelineEvent, PipelineState};
pub use worker::Pipeline;
