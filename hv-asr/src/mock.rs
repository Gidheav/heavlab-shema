//! Mock ASR engine for tests and pipeline development (no model files).

use hv_audio::AudioChunk;

use crate::traits::{AsrEngine, AsrError, Transcript};

/// Returns configurable canned transcripts. Used when a real model is not loaded.
#[derive(Debug)]
pub struct MockAsrEngine {
    name: String,
    responses: Vec<Transcript>,
    cursor: usize,
    emit_on_silence: bool,
}

impl MockAsrEngine {
    pub fn new() -> Self {
        Self {
            name: "mock".to_string(),
            responses: Vec::new(),
            cursor: 0,
            emit_on_silence: false,
        }
    }

    /// Cycle through the given transcripts on each qualifying `feed`.
    pub fn with_transcripts(responses: Vec<Transcript>) -> Self {
        Self {
            name: "mock".to_string(),
            responses,
            cursor: 0,
            emit_on_silence: false,
        }
    }

    /// Always emit this final transcript when the chunk has signal.
    pub fn with_text(text: impl Into<String>) -> Self {
        Self::with_transcripts(vec![Transcript::final_text(text, 1.0)])
    }

    pub fn emit_on_silence(mut self, enabled: bool) -> Self {
        self.emit_on_silence = enabled;
        self
    }
}

impl Default for MockAsrEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AsrEngine for MockAsrEngine {
    fn feed(&mut self, chunk: &AudioChunk) -> Result<Option<Transcript>, AsrError> {
        if self.responses.is_empty() {
            return Ok(None);
        }
        if !self.emit_on_silence && !chunk.has_signal() {
            return Ok(None);
        }
        let transcript = self.responses[self.cursor % self.responses.len()].clone();
        self.cursor = self.cursor.saturating_add(1);
        Ok(Some(transcript))
    }

    fn reset(&mut self) -> Result<(), AsrError> {
        self.cursor = 0;
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::MockAsrEngine;
    use crate::traits::{AsrEngine, Transcript};
    use hv_audio::AudioChunk;

    #[test]
    fn mock_returns_configured_transcripts() {
        let mut engine = MockAsrEngine::with_transcripts(vec![
            Transcript::partial("john three", 0.8),
            Transcript::final_text("john 3:16", 0.95),
        ]);
        assert_eq!(engine.name(), "mock");

        let silent = AudioChunk::new(vec![0; 160], 0);
        assert!(engine.feed(&silent).unwrap().is_none());

        let signal = AudioChunk::new(vec![100; 160], 10);
        let first = engine.feed(&signal).unwrap().expect("first transcript");
        assert_eq!(first.text, "john three");
        assert!(!first.is_final);

        let second = engine.feed(&signal).unwrap().expect("second transcript");
        assert_eq!(second.text, "john 3:16");
        assert!(second.is_final);
        assert!((second.confidence - 0.95).abs() < f32::EPSILON);
    }
}
