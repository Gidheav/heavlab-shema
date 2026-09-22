//! Dedicated thread wiring capture, ring, VAD, ASR, and callbacks.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle, ThreadId};

use crossbeam_channel::{select, Receiver, Sender};

use crate::capture::{create_capture, CaptureConfig, MicrophoneCapture};
use crate::engine::{AsrEngine, AsrModelConfig, MockAsrEngine, SherpaAsrEngine};
use crate::error::AsrError;
use crate::ring::AudioRing;
use crate::vad::{SimpleVad, VadConfig, VadState};

/// Receives partial ASR output and lifecycle notifications.
pub trait TranscriptHandler: Send + Sync {
    fn on_transcript(&self, text: String);
    fn on_error(&self, error: String);
    fn on_listening_state_changed(&self, is_active: bool);

    /// Called once after continuous silence reaches the pause threshold.
    fn on_listening_paused(&self) {}

    /// Called when speech resumes after a pause notification.
    fn on_speech_resumed(&self) {}
}

/// Configuration for an [`AudioWorker`].
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub asr_config: AsrModelConfig,
    pub use_mock_asr: bool,
    pub capture: CaptureConfig,
    pub vad: VadConfig,
    pub ring_capacity_samples: usize,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            asr_config: AsrModelConfig::default(),
            use_mock_asr: true,
            capture: CaptureConfig {
                require_aec: cfg!(any(target_os = "ios", target_os = "android")),
                ..CaptureConfig::default()
            },
            vad: VadConfig::default(),
            ring_capacity_samples: 160_000,
        }
    }
}

/// Dedicated worker thread for the hot audio path.
pub struct AudioWorker {
    running: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
    worker_thread_id: Option<ThreadId>,
    pending_join: Option<JoinHandle<()>>,
    pending_join_thread_id: Option<ThreadId>,
    stop_sender: Option<Sender<()>>,
    config: WorkerConfig,
    active_capture: Option<Arc<dyn MicrophoneCapture>>,
    ring: Arc<Mutex<AudioRing>>,
}

impl AudioWorker {
    pub fn new(config: WorkerConfig) -> Self {
        let ring_capacity_samples = config.ring_capacity_samples;
        Self {
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
            worker_thread_id: None,
            pending_join: None,
            pending_join_thread_id: None,
            stop_sender: None,
            config,
            active_capture: None,
            ring: Arc::new(Mutex::new(AudioRing::new(ring_capacity_samples))),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(WorkerConfig::default())
    }

    /// Start using the configured platform/default capture and ASR engine.
    pub fn start(&mut self, handler: Arc<dyn TranscriptHandler>) -> Result<(), AsrError> {
        self.start_with_aec(handler, false)
    }

    /// Start using the configured capture after the shell's AEC assertion.
    pub fn start_with_aec(
        &mut self,
        handler: Arc<dyn TranscriptHandler>,
        aec_enabled: bool,
    ) -> Result<(), AsrError> {
        let capture: Arc<dyn MicrophoneCapture> = Arc::from(create_capture(self.config.capture));
        let engine: Box<dyn AsrEngine> = if self.config.use_mock_asr {
            Box::new(MockAsrEngine::new(self.config.asr_config.language.clone()))
        } else {
            Box::new(SherpaAsrEngine::new(self.config.asr_config.clone())?)
        };
        self.start_with(handler, capture, engine, aec_enabled)
    }

    /// Start with injectable capture and engine implementations.
    pub fn start_with(
        &mut self,
        handler: Arc<dyn TranscriptHandler>,
        capture: Arc<dyn MicrophoneCapture>,
        engine: Box<dyn AsrEngine>,
        aec_enabled: bool,
    ) -> Result<(), AsrError> {
        self.reap_pending_join();
        self.reap_finished_thread();
        if self.running.load(Ordering::SeqCst) {
            return Err(AsrError::AlreadyListening);
        }

        let audio_rx = capture.start_with_aec(aec_enabled)?;
        let (stop_tx, stop_rx) = crossbeam_channel::bounded(1);
        let running = Arc::clone(&self.running);
        let ring = Arc::clone(&self.ring);
        let config = self.config.clone();
        let capture_for_thread = Arc::clone(&capture);
        running.store(true, Ordering::SeqCst);
        let running_for_thread = Arc::clone(&running);
        let handle = thread::Builder::new()
            .name("asr-worker".to_string())
            .spawn(move || {
                Self::worker_loop(
                    handler,
                    stop_rx,
                    audio_rx,
                    &running_for_thread,
                    config,
                    capture_for_thread,
                    engine,
                    ring,
                );
            })
            .map_err(|error| {
                running.store(false, Ordering::SeqCst);
                capture.stop();
                AsrError::CaptureError(error.to_string())
            })?;
        let worker_thread_id = handle.thread().id();
        self.active_capture = Some(capture);
        self.thread_handle = Some(handle);
        self.worker_thread_id = Some(worker_thread_id);
        self.stop_sender = Some(stop_tx);
        Ok(())
    }

    /// Forward PCM supplied by a platform shell to the active capture.
    pub fn push_audio(&self, samples: &[i16], timestamp_ms: u64) -> Result<(), AsrError> {
        self.active_capture
            .as_ref()
            .ok_or(AsrError::NotListening)?
            .push_samples(samples, timestamp_ms)
    }

    /// Stop listening; calls from handler callbacks are supported.
    ///
    /// When called on the worker thread, this returns without joining that
    /// thread. It is reaped by the next `start*` call or when this worker is
    /// dropped.
    pub fn stop(&mut self) -> Result<(), AsrError> {
        if !self.running.load(Ordering::SeqCst) {
            return Err(AsrError::NotListening);
        }
        self.running.store(false, Ordering::SeqCst);
        if let Some(sender) = self.stop_sender.take() {
            let _ = sender.send(());
        }
        if let Some(capture) = self.active_capture.take() {
            capture.stop();
        }
        self.clear_ring();
        if let Some(handle) = self.thread_handle.take() {
            let worker_thread_id = self.worker_thread_id.take();
            if worker_thread_id == Some(thread::current().id()) {
                self.pending_join = Some(handle);
                self.pending_join_thread_id = worker_thread_id;
            } else {
                let _ = handle.join();
            }
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    fn clear_ring(&self) {
        if let Ok(mut ring) = self.ring.lock() {
            ring.clear();
        }
    }

    fn reap_pending_join(&mut self) {
        let Some(handle) = self.pending_join.take() else {
            return;
        };
        let thread_id = self.pending_join_thread_id.take();
        if thread_id == Some(thread::current().id()) {
            self.pending_join = Some(handle);
            self.pending_join_thread_id = thread_id;
        } else {
            let _ = handle.join();
        }
    }

    fn reap_finished_thread(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }
        let Some(handle) = self.thread_handle.take() else {
            return;
        };
        let thread_id = self.worker_thread_id.take();
        if thread_id == Some(thread::current().id()) {
            self.pending_join = Some(handle);
            self.pending_join_thread_id = thread_id;
        } else {
            let _ = handle.join();
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn worker_loop(
        handler: Arc<dyn TranscriptHandler>,
        stop_rx: Receiver<()>,
        audio_rx: Receiver<crate::capture::AudioChunk>,
        running: &AtomicBool,
        config: WorkerConfig,
        capture: Arc<dyn MicrophoneCapture>,
        mut asr_engine: Box<dyn AsrEngine>,
        ring: Arc<Mutex<AudioRing>>,
    ) {
        handler.on_listening_state_changed(true);
        let mut vad = SimpleVad::from_config(config.vad);
        let mut asr_active = false;
        let mut pause_notified = false;
        loop {
            select! {
                recv(audio_rx) -> result => {
                    let chunk = match result {
                        Ok(chunk) => chunk,
                        Err(_) => {
                            handler.on_error("audio capture channel disconnected".to_string());
                            break;
                        }
                    };
                    if let Ok(mut audio_ring) = ring.lock() {
                        audio_ring.push_samples(&chunk.samples);
                    }
                    let vad_state = vad.process(&chunk, chunk.timestamp_ms);
                    if vad_state == VadState::Speaking {
                        if pause_notified {
                            handler.on_speech_resumed();
                            pause_notified = false;
                        }
                        if !asr_active {
                            asr_active = true;
                        }
                    } else if vad.silence_duration_ms(chunk.timestamp_ms)
                        >= config.vad.pause_notify_ms
                        && !pause_notified
                    {
                        handler.on_listening_paused();
                        pause_notified = true;
                    }

                    match vad_state {
                        VadState::Speaking | VadState::Transitioning if asr_active => {
                            match asr_engine.feed(&chunk) {
                                Ok(Some(transcript)) => handler.on_transcript(transcript),
                                Ok(None) => {}
                                Err(error) => handler.on_error(format!("ASR inference error: {error}")),
                            }
                        }
                        VadState::Silence if asr_active => {
                            asr_active = false;
                            asr_engine.reset();
                        }
                        _ => {}
                    }
                }
                recv(stop_rx) -> _ => break,
            }
            if !running.load(Ordering::SeqCst) {
                break;
            }
        }
        capture.stop();
        if let Ok(mut audio_ring) = ring.lock() {
            audio_ring.clear();
        }
        handler.on_listening_state_changed(false);
        running.store(false, Ordering::SeqCst);
    }
}

impl Default for AudioWorker {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl Drop for AudioWorker {
    fn drop(&mut self) {
        if self.is_running() {
            let _ = self.stop();
        }
        self.reap_pending_join();
        self.reap_finished_thread();
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::field_reassign_with_default
)]
mod tests {
    use super::*;
    use crate::capture::{AudioChunk, CaptureConfig, MockCapture, PushCapture};
    use std::sync::atomic::AtomicUsize;
    use std::time::{Duration, Instant};

    struct TestHandler {
        transcripts: AtomicUsize,
        errors: AtomicUsize,
        paused: AtomicUsize,
        resumed: AtomicUsize,
    }

    impl TestHandler {
        fn new() -> Self {
            Self {
                transcripts: AtomicUsize::new(0),
                errors: AtomicUsize::new(0),
                paused: AtomicUsize::new(0),
                resumed: AtomicUsize::new(0),
            }
        }
    }

    impl TranscriptHandler for TestHandler {
        fn on_transcript(&self, _text: String) {
            self.transcripts.fetch_add(1, Ordering::SeqCst);
        }
        fn on_error(&self, _error: String) {
            self.errors.fetch_add(1, Ordering::SeqCst);
        }
        fn on_listening_state_changed(&self, _is_active: bool) {}
        fn on_listening_paused(&self) {
            self.paused.fetch_add(1, Ordering::SeqCst);
        }
        fn on_speech_resumed(&self) {
            self.resumed.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct SelfStoppingHandler {
        worker: Arc<Mutex<AudioWorker>>,
        completed: Sender<bool>,
    }

    impl TranscriptHandler for SelfStoppingHandler {
        fn on_transcript(&self, _text: String) {}

        fn on_error(&self, _error: String) {}

        fn on_listening_state_changed(&self, _is_active: bool) {}

        fn on_listening_paused(&self) {
            let stopped = self
                .worker
                .lock()
                .map(|mut worker| worker.stop().is_ok())
                .unwrap_or(false);
            let _ = self.completed.send(stopped);
        }
    }

    fn start_test_worker(
        config: WorkerConfig,
    ) -> (AudioWorker, Arc<MockCapture>, Arc<TestHandler>) {
        let capture = Arc::new(MockCapture::new());
        let handler = Arc::new(TestHandler::new());
        let mut worker = AudioWorker::new(config);
        worker
            .start_with(
                Arc::clone(&handler) as Arc<dyn TranscriptHandler>,
                Arc::clone(&capture) as Arc<dyn MicrophoneCapture>,
                Box::new(MockAsrEngine::new("en".to_string())),
                false,
            )
            .expect("worker start");
        (worker, capture, handler)
    }

    fn wait_until(mut condition: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while !condition() {
            assert!(Instant::now() < deadline, "worker condition timed out");
            thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn silence_produces_no_transcripts() {
        let (mut worker, capture, handler) = start_test_worker(WorkerConfig::default());
        for timestamp in (0..2_000).step_by(100) {
            capture
                .inject(AudioChunk::new(vec![0; 1600], timestamp))
                .expect("inject");
        }
        wait_until(|| {
            worker
                .ring
                .lock()
                .map(|ring| ring.len() == 32_000)
                .unwrap_or(false)
        });
        assert_eq!(handler.transcripts.load(Ordering::SeqCst), 0);
        worker.stop().expect("stop");
    }

    #[test]
    fn speech_reaches_asr_and_ring_clears_on_stop() {
        let mut config = WorkerConfig::default();
        config.ring_capacity_samples = 3200;
        let (mut worker, capture, handler) = start_test_worker(config);
        for timestamp in (0..1_000).step_by(100) {
            capture
                .inject(AudioChunk::new(vec![5000; 1600], timestamp))
                .expect("inject");
        }
        wait_until(|| handler.transcripts.load(Ordering::SeqCst) > 0);
        assert!(handler.transcripts.load(Ordering::SeqCst) > 0);
        assert!(worker.ring.lock().expect("ring").len() <= 3200);
        worker.stop().expect("stop");
        assert_eq!(worker.ring.lock().expect("ring").len(), 0);
        assert!(worker
            .ring
            .lock()
            .expect("ring")
            .backing_store()
            .iter()
            .all(|sample| *sample == 0));
    }

    #[test]
    fn pause_notification_fires_once_and_rearms() {
        let mut config = WorkerConfig::default();
        config.vad.pause_notify_ms = 30_000;
        let (mut worker, capture, handler) = start_test_worker(config);
        capture
            .inject(AudioChunk::new(vec![5000; 1600], 0))
            .expect("inject");
        capture
            .inject(AudioChunk::new(vec![0; 1600], 100))
            .expect("inject");
        capture
            .inject(AudioChunk::new(vec![0; 1600], 30_100))
            .expect("inject");
        capture
            .inject(AudioChunk::new(vec![0; 1600], 60_100))
            .expect("inject");
        capture
            .inject(AudioChunk::new(vec![5000; 1600], 60_200))
            .expect("inject");
        wait_until(|| {
            handler.paused.load(Ordering::SeqCst) == 1
                && handler.resumed.load(Ordering::SeqCst) == 1
        });
        assert_eq!(handler.paused.load(Ordering::SeqCst), 1);
        assert_eq!(handler.resumed.load(Ordering::SeqCst), 1);
        worker.stop().expect("stop");
    }

    #[test]
    fn callback_can_stop_worker_without_self_joining() {
        let capture_config = CaptureConfig {
            require_aec: false,
            ..CaptureConfig::default()
        };
        let capture = Arc::new(PushCapture::new(capture_config));
        let worker = Arc::new(Mutex::new(AudioWorker::with_defaults()));
        let (completed_tx, completed_rx) = crossbeam_channel::bounded(1);
        let handler = Arc::new(SelfStoppingHandler {
            worker: Arc::clone(&worker),
            completed: completed_tx,
        });

        {
            let mut worker_guard = worker.lock().expect("worker lock");
            worker_guard
                .start_with(
                    Arc::clone(&handler) as Arc<dyn TranscriptHandler>,
                    Arc::clone(&capture) as Arc<dyn MicrophoneCapture>,
                    Box::new(MockAsrEngine::new("en".to_string())),
                    false,
                )
                .expect("worker start");
        }
        capture
            .push_samples(&[5000; 1600], 0)
            .expect("speech injection");
        capture
            .push_samples(&[0; 1600], 100)
            .expect("silence injection");
        capture
            .push_samples(&[0; 1600], 30_100)
            .expect("pause injection");

        assert!(
            completed_rx
                .recv_timeout(Duration::from_secs(2))
                .expect("callback completion"),
            "callback stop should succeed"
        );
        wait_until(|| {
            !worker
                .lock()
                .map(|worker| worker.is_running())
                .unwrap_or(true)
        });

        let restart_capture = Arc::new(MockCapture::new());
        let restart_handler = Arc::new(TestHandler::new());
        let mut worker_guard = worker.lock().expect("worker lock");
        worker_guard
            .start_with(
                Arc::clone(&restart_handler) as Arc<dyn TranscriptHandler>,
                Arc::clone(&restart_capture) as Arc<dyn MicrophoneCapture>,
                Box::new(MockAsrEngine::new("en".to_string())),
                false,
            )
            .expect("worker restart");
        assert!(worker_guard.is_running());
        worker_guard.stop().expect("worker stop");
    }

    #[test]
    fn capture_failure_is_reported_and_worker_stops() {
        struct FailingCapture;
        impl MicrophoneCapture for FailingCapture {
            fn start(&self) -> Result<Receiver<AudioChunk>, AsrError> {
                Err(AsrError::CaptureError("test failure".to_string()))
            }
            fn stop(&self) {}
        }
        let handler = Arc::new(TestHandler::new());
        let mut worker = AudioWorker::with_defaults();
        let worker_error = worker
            .start_with(
                Arc::clone(&handler) as Arc<dyn TranscriptHandler>,
                Arc::new(FailingCapture),
                Box::new(MockAsrEngine::new("en".to_string())),
                false,
            )
            .expect_err("capture start should fail");
        assert!(matches!(worker_error, AsrError::CaptureError(_)));
        assert!(!worker.is_running());
        assert_eq!(handler.errors.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn push_capture_worker_requires_aec_assertion() {
        let capture_config = CaptureConfig::default();
        let mut config = WorkerConfig::default();
        config.capture = capture_config;
        let handler = Arc::new(TestHandler::new());
        let capture = Arc::new(PushCapture::new(capture_config));
        let mut worker = AudioWorker::new(config.clone());
        let result = worker.start_with(
            Arc::clone(&handler) as Arc<dyn TranscriptHandler>,
            Arc::clone(&capture) as Arc<dyn MicrophoneCapture>,
            Box::new(MockAsrEngine::new("en".to_string())),
            false,
        );
        assert!(matches!(result, Err(AsrError::AecUnavailable)));
        assert!(!worker.is_running());

        let capture = Arc::new(PushCapture::new(capture_config));
        let mut worker = AudioWorker::new(config);
        worker
            .start_with(
                Arc::clone(&handler) as Arc<dyn TranscriptHandler>,
                capture,
                Box::new(MockAsrEngine::new("en".to_string())),
                true,
            )
            .expect("AEC assertion should allow capture start");
        assert!(worker.is_running());
        worker.stop().expect("stop");
    }
}
