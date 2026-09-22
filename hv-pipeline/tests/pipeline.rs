use std::sync::Arc;
use std::time::{Duration, Instant};

use bible_core::store::TranslationStore;
use crossbeam_channel::Receiver;
use hv_asr::{AsrEngine, AsrError, MockAsrEngine, Transcript};
use hv_audio::{AudioChunk, MicrophoneCapture, MockCapture};
use hv_pipeline::{Pipeline, PipelineCommand, PipelineEvent};
use hv_vad::EnergyVad;

struct SharedCapture(Arc<MockCapture>);

impl MicrophoneCapture for SharedCapture {
    fn start(
        &self,
    ) -> Result<crossbeam_channel::Receiver<AudioChunk>, hv_audio::CaptureError> {
        self.0.start()
    }

    fn stop(&self) {
        self.0.stop();
    }

    fn push_samples(
        &self,
        samples: &[i16],
        timestamp_ms: u64,
    ) -> Result<(), hv_audio::CaptureError> {
        self.0.push_samples(samples, timestamp_ms)
    }
}

fn loud_chunk(timestamp_ms: u64) -> AudioChunk {
    AudioChunk::new(vec![5000; 16_000], timestamp_ms)
}

fn wait_for(
    events: &Receiver<PipelineEvent>,
    timeout: Duration,
    mut pred: impl FnMut(&PipelineEvent) -> bool,
) -> Option<PipelineEvent> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match events.recv_timeout(remaining) {
            Ok(event) if pred(&event) => return Some(event),
            Ok(_) => {}
            Err(_) => return None,
        }
    }
    None
}

#[test]
fn pipeline_constructs_with_mocks() {
    let capture = Arc::new(MockCapture::new());
    let pipeline = Pipeline::new(
        Box::new(SharedCapture(Arc::clone(&capture))),
        Box::new(EnergyVad::default()),
        Box::new(MockAsrEngine::with_text("john 3:16")),
        Arc::new(TranslationStore::new()),
        "KJV".to_string(),
    );
    assert!(pipeline.is_ok());
}

#[test]
fn john_3_16_emits_verse_detected() {
    let capture = Arc::new(MockCapture::new());
    let pipeline = Pipeline::new(
        Box::new(SharedCapture(Arc::clone(&capture))),
        Box::new(EnergyVad::default()),
        Box::new(MockAsrEngine::with_text("john 3:16")),
        Arc::new(TranslationStore::new()),
        "KJV".to_string(),
    )
    .expect("pipeline");

    pipeline.send_command(PipelineCommand::Start);
    assert!(
        wait_for(pipeline.events(), Duration::from_secs(2), |e| {
            matches!(e, PipelineEvent::StateChanged(_))
        })
        .is_some(),
        "should start listening"
    );

    capture
        .inject(loud_chunk(0))
        .expect("inject speech");

    let event = wait_for(pipeline.events(), Duration::from_secs(2), |e| {
        matches!(e, PipelineEvent::VerseDetected { .. })
    })
    .expect("VerseDetected");

    match event {
        PipelineEvent::VerseDetected {
            reference,
            book,
            chapter,
            verse,
            ..
        } => {
            assert_eq!(book, 43);
            assert_eq!(chapter, 3);
            assert_eq!(verse, 16);
            assert!(
                reference.to_lowercase().contains("john"),
                "reference={reference}"
            );
        }
        other => panic!("unexpected {other:?}"),
    }
}

struct FailingAsr;

impl AsrEngine for FailingAsr {
    fn feed(&mut self, _chunk: &AudioChunk) -> Result<Option<Transcript>, AsrError> {
        Err(AsrError::InferenceError("forced failure".into()))
    }

    fn reset(&mut self) -> Result<(), AsrError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "failing"
    }
}

#[test]
fn worker_survives_asr_errors() {
    let capture = Arc::new(MockCapture::new());
    let pipeline = Pipeline::new(
        Box::new(SharedCapture(Arc::clone(&capture))),
        Box::new(EnergyVad::default()),
        Box::new(FailingAsr),
        Arc::new(TranslationStore::new()),
        "KJV".to_string(),
    )
    .expect("pipeline");

    pipeline.send_command(PipelineCommand::Start);
    capture.inject(loud_chunk(0)).expect("inject 1");

    assert!(
        wait_for(pipeline.events(), Duration::from_secs(2), |e| {
            matches!(e, PipelineEvent::Error(_))
        })
        .is_some(),
        "ASR error should be reported"
    );

    capture.inject(loud_chunk(1_000)).expect("inject 2");
    assert!(
        wait_for(pipeline.events(), Duration::from_secs(2), |e| {
            matches!(e, PipelineEvent::MeterUpdate { .. })
        })
        .is_some(),
        "worker must keep metering after ASR failure"
    );
}
