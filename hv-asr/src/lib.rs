//! hv-asr — Automatic speech recognition.
//!
//! Decides WHAT is being said. Does not capture audio or parse Bible verses.

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod mock;
pub mod sherpa;
pub mod traits;
pub mod hotwords;

pub use mock::MockAsrEngine;
pub use sherpa::SherpaAsrEngine;
pub use traits::{AsrConfig, AsrEngine, AsrError, Transcript};
