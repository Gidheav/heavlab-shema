//! Typed pipeline failures.

use hv_audio::CaptureError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("failed to start pipeline worker: {0}")]
    Spawn(String),
    #[error(transparent)]
    Capture(#[from] CaptureError),
}
