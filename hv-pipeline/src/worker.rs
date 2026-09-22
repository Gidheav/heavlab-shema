//! Dedicated std::thread worker. No async runtime.

use std::sync::Arc;
use std::thread;

use bible_core::store::TranslationStore;
use crossbeam_channel::{select, Receiver, Sender};
use hv_asr::AsrEngine;
use hv_audio::{AudioChunk, AudioMeter, MicrophoneCapture, NoiseGate, HighPassFilter, Compressor};
use hv_vad::{Vad, VadState};

use crate::detector::VerseDetector;
use crate::error::PipelineError;
use crate::events::{PipelineCommand, PipelineEvent, PipelineState};

const MIN_TRANSCRIPT_CONFIDENCE: f32 = 0.6;

/// Handle held by the UI. The worker thread owns capture, VAD, ASR, and the detector.
pub struct Pipeline {
    command_tx: Sender<PipelineCommand>,
    event_rx: Receiver<PipelineEvent>,
    worker_thread: Option<thread::JoinHandle<()>>,
}

impl Pipeline {
    pub fn new(
        capture: Box<dyn MicrophoneCapture>,
        vad: Box<dyn Vad>,
        engine: Box<dyn AsrEngine>,
        store: Arc<TranslationStore>,
        translation: String,
    ) -> Result<Self, PipelineError> {
        let (command_tx, command_rx) = crossbeam_channel::bounded(32);
        let (event_tx, event_rx) = crossbeam_channel::bounded(256);

        let worker_thread = thread::Builder::new()
            .name("hv-pipeline".to_string())
            .spawn(move || {
                worker_loop(capture, vad, engine, store, translation, command_rx, event_tx);
            })
            .map_err(|error| PipelineError::Spawn(error.to_string()))?;

        Ok(Self {
            command_tx,
            event_rx,
            worker_thread: Some(worker_thread),
        })
    }

    pub fn events(&self) -> &Receiver<PipelineEvent> {
        &self.event_rx
    }

    pub fn send_command(&self, cmd: PipelineCommand) {
        if let Err(error) = self.command_tx.try_send(cmd) {
            tracing::warn!("pipeline command dropped: {error}");
        }
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        // Send stop command just in case, though the channel dropping will break the loop anyway
        let _ = self.command_tx.try_send(PipelineCommand::Stop);
        
        // Dropping command_tx will close the channel, causing worker loop to exit
        if let Some(thread) = self.worker_thread.take() {
            tracing::info!("Joining pipeline worker thread...");
            let _ = thread.join();
            tracing::info!("Pipeline worker thread joined.");
        }
    }
}

fn emit(tx: &Sender<PipelineEvent>, event: PipelineEvent) {
    if tx.try_send(event).is_err() {
        // UI is stalled or gone; keep the audio thread moving.
    }
}

fn apply_gain(chunk: &mut AudioChunk, gain: f32) {
    if (gain - 1.0).abs() < f32::EPSILON {
        return;
    }
    for sample in &mut chunk.samples {
        let scaled = f32::from(*sample) * gain;
        *sample = scaled.clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16;
    }
}

fn worker_loop(
    mut capture: Box<dyn MicrophoneCapture>,
    mut vad: Box<dyn Vad>,
    mut engine: Box<dyn AsrEngine>,
    store: Arc<TranslationStore>,
    translation: String,
    command_rx: Receiver<PipelineCommand>,
    event_tx: Sender<PipelineEvent>,
) {
    let mut meter = AudioMeter::new();
    let mut detector = VerseDetector::new(store, translation);
    let mut gain = 1.0_f32;
    let mut noise_gate_enabled = false;
    let mut noise_gate = NoiseGate::default();
    let mut hpf_enabled = false;
    let mut hpf = HighPassFilter::default();
    let mut compressor_enabled = false;
    let mut compressor = Compressor::default();
    
    let mut audio_rx: Option<Receiver<AudioChunk>> = None;
    let mut last_state = PipelineState::Stopped;
    
    let mut consecutive_reconnects = 0;
    let mut last_disconnect = std::time::Instant::now();

    loop {
        let audio_owned = audio_rx.clone();
        if let Some(rx) = audio_owned {
            select! {
                recv(command_rx) -> msg => {
                    match msg {
                        Ok(cmd) => handle_command(
                            cmd,
                            &mut capture,
                            &mut vad,
                            &mut engine,
                            &mut detector,
                            &mut meter,
                            &mut gain,
                            &mut noise_gate_enabled,
                            &mut noise_gate,
                            &mut hpf_enabled,
                            &mut hpf,
                            &mut compressor_enabled,
                            &mut compressor,
                            &mut audio_rx,
                            &event_tx,
                            &mut last_state,
                        ),
                        Err(_) => break,
                    }
                }
                recv(rx) -> msg => {
                    match msg {
                        Ok(chunk) => process_chunk(
                            chunk,
                            gain,
                            &mut meter,
                            &mut vad,
                            &mut engine,
                            &mut detector,
                            noise_gate_enabled,
                            &mut noise_gate,
                            hpf_enabled,
                            &mut hpf,
                            compressor_enabled,
                            &mut compressor,
                            &event_tx,
                            &mut last_state,
                        ),
                        Err(_) => {
                            // Only emit error and stop if we haven't already stopped capture purposefully
                            if audio_rx.is_some() {
                                capture.stop();
                                audio_rx = None;
                                set_state(&event_tx, &mut last_state, PipelineState::Stopped);

                                if last_disconnect.elapsed().as_secs() > 5 {
                                    consecutive_reconnects = 0;
                                }
                                last_disconnect = std::time::Instant::now();

                                if consecutive_reconnects >= 3 {
                                    tracing::error!("Too many consecutive disconnects. Aborting reconnect.");
                                    emit(&event_tx, PipelineEvent::Error("Audio capture failed. Please select a valid device and press START.".into()));
                                } else {
                                    consecutive_reconnects += 1;
                                    tracing::warn!("Audio capture channel disconnected. Attempting to reconnect ({})...", consecutive_reconnects);
                                    emit(&event_tx, PipelineEvent::Error("Audio disconnected. Reconnecting...".into()));
                                    
                                    // Sleep a bit before reconnecting
                                    std::thread::sleep(std::time::Duration::from_secs(1));
                                    if let Ok(rx) = capture.start() {
                                        audio_rx = Some(rx);
                                        set_state(&event_tx, &mut last_state, PipelineState::Listening);
                                        tracing::info!("Reconnected successfully.");
                                    } else {
                                        emit(&event_tx, PipelineEvent::Error("Audio capture failed. Please select a valid device and press START.".into()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            match command_rx.recv() {
                Ok(cmd) => handle_command(
                    cmd,
                    &mut capture,
                    &mut vad,
                    &mut engine,
                    &mut detector,
                    &mut meter,
                    &mut gain,
                    &mut noise_gate_enabled,
                    &mut noise_gate,
                    &mut hpf_enabled,
                    &mut hpf,
                    &mut compressor_enabled,
                    &mut compressor,
                    &mut audio_rx,
                    &event_tx,
                    &mut last_state,
                ),
                Err(_) => break,
            }
        }
    }

    capture.stop();
}

#[allow(clippy::too_many_arguments)]
fn handle_command(
    cmd: PipelineCommand,
    capture: &mut Box<dyn MicrophoneCapture>,
    vad: &mut Box<dyn Vad>,
    engine: &mut Box<dyn AsrEngine>,
    detector: &mut VerseDetector,
    meter: &mut AudioMeter,
    gain: &mut f32,
    noise_gate_enabled: &mut bool,
    noise_gate: &mut NoiseGate,
    hpf_enabled: &mut bool,
    hpf: &mut HighPassFilter,
    compressor_enabled: &mut bool,
    compressor: &mut Compressor,
    audio_rx: &mut Option<Receiver<AudioChunk>>,
    event_tx: &Sender<PipelineEvent>,
    last_state: &mut PipelineState,
) {
    match cmd {
        PipelineCommand::Start => {
            if audio_rx.is_some() {
                return;
            }
            match capture.start() {
                Ok(rx) => {
                    vad.reset();
                    meter.reset();
                    detector.reset();
                    if let Err(error) = engine.reset() {
                        emit(event_tx, PipelineEvent::Error(format!("ASR reset: {error}")));
                    }
                    *audio_rx = Some(rx);
                    set_state(event_tx, last_state, PipelineState::Listening);
                }
                Err(error) => {
                    emit(event_tx, PipelineEvent::Error(error.to_string()));
                    set_state(event_tx, last_state, PipelineState::Stopped);
                }
            }
        }
        PipelineCommand::Stop => {
            capture.stop();
            *audio_rx = None;
            vad.reset();
            detector.reset();
            meter.reset();
            set_state(event_tx, last_state, PipelineState::Stopped);
        }
        PipelineCommand::SetGain(value) => {
            *gain = value.clamp(0.0, 4.0);
        }
        PipelineCommand::SetTranslation(translation) => {
            detector.set_translation(translation);
        }
        PipelineCommand::SetDevice(device_name) => {
            capture.stop();
            *audio_rx = None;
            vad.reset();
            meter.reset();
            detector.reset();
            set_state(event_tx, last_state, PipelineState::Stopped);
            
            *capture = Box::new(hv_audio::CpalCapture::with_device(device_name));
            
            match capture.start() {
                Ok(rx) => {
                    if let Err(error) = engine.reset() {
                        emit(event_tx, PipelineEvent::Error(format!("ASR reset: {error}")));
                    }
                    *audio_rx = Some(rx);
                    set_state(event_tx, last_state, PipelineState::Listening);
                }
                Err(error) => {
                    emit(event_tx, PipelineEvent::Error(format!("Failed to start new device: {error}")));
                }
            }
        }
        PipelineCommand::SetNoiseGateThreshold(t) => {
            noise_gate.threshold = t;
        }
        PipelineCommand::EnableNoiseGate(enable) => {
            *noise_gate_enabled = enable;
        }
        PipelineCommand::SetHighPassFreq(freq) => {
            hpf.frequency = freq;
        }
        PipelineCommand::EnableHighPass(enable) => {
            *hpf_enabled = enable;
        }
        PipelineCommand::SetCompressorRatio(r) => {
            compressor.ratio = r;
        }
        PipelineCommand::EnableCompressor(enable) => {
            *compressor_enabled = enable;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn process_chunk(
    mut chunk: AudioChunk,
    gain: f32,
    meter: &mut AudioMeter,
    vad: &mut Box<dyn Vad>,
    engine: &mut Box<dyn AsrEngine>,
    detector: &mut VerseDetector,
    noise_gate_enabled: bool,
    noise_gate: &mut NoiseGate,
    hpf_enabled: bool,
    hpf: &mut HighPassFilter,
    compressor_enabled: bool,
    compressor: &mut Compressor,
    event_tx: &Sender<PipelineEvent>,
    last_state: &mut PipelineState,
) {
    apply_gain(&mut chunk, gain);
    
    if hpf_enabled {
        hpf.process(&mut chunk);
    }
    if noise_gate_enabled {
        noise_gate.process(&mut chunk);
    }
    if compressor_enabled {
        compressor.process(&mut chunk);
    }

    let reading = meter.process(&chunk);
    emit(
        event_tx,
        PipelineEvent::MeterUpdate {
            rms_db: reading.rms_db,
            peak_db: reading.peak_db,
            is_clipping: reading.is_clipping,
        },
    );

    let vad_state = vad.process(&chunk);
    match vad_state {
        VadState::Speech => set_state(event_tx, last_state, PipelineState::Speaking),
        VadState::Silence { .. } => set_state(event_tx, last_state, PipelineState::Silence),
    }

    if matches!(vad_state, VadState::Silence { .. }) {
        return;
    }

    match engine.feed(&chunk) {
        Ok(Some(transcript)) => {
            emit(
                event_tx,
                PipelineEvent::Transcript {
                    text: transcript.text.clone(),
                    is_final: transcript.is_final,
                    confidence: transcript.confidence,
                },
            );
            if transcript.confidence >= MIN_TRANSCRIPT_CONFIDENCE {
                if let Some(hit) = detector.push_transcript(&transcript.text) {
                    emit(
                        event_tx,
                        PipelineEvent::VerseDetected {
                            reference: hit.reference,
                            text: hit.text,
                            book: hit.book.as_u8(),
                            chapter: hit.chapter,
                            verse: hit.verse,
                        },
                    );
                }
            }
        }
        Ok(None) => {}
        Err(error) => {
            tracing::warn!("ASR inference error (capture continues): {error}");
            emit(event_tx, PipelineEvent::Error(format!("ASR: {error}")));
        }
    }
}

fn set_state(tx: &Sender<PipelineEvent>, last: &mut PipelineState, next: PipelineState) {
    if *last != next {
        *last = next;
        emit(tx, PipelineEvent::StateChanged(next));
    }
}
