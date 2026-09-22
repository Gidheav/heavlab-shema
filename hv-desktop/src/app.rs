use std::time::Instant;

use eframe::egui::{Color32, Context, Id};
use hv_pipeline::{PipelineEvent, PipelineState};

use crate::config::AppConfig;
use crate::layout::workspace_preset::WorkspacePreset;
use crate::mock::audio_state::AudioControlState;
use crate::mock::broadcast_state::BroadcastState;
use crate::pipeline_integration::{self, PipelineConfig};
use crate::shell;
use crate::shortcuts;
use crate::theme::{self, STATUS_ERROR, STATUS_INFO, STATUS_SUCCESS, STATUS_WARNING, text_inverse};

pub fn manual_verse_id() -> Id {
    Id::new("manual_verse")
}

#[derive(Debug, Clone)]
pub struct SermonLogEntry {
    pub timestamp: String,
    pub reference: String,
    pub text: String,
    pub approved: bool,
}

pub struct HvBibleApp {
    pub current_verse: Option<(String, String)>,
    pub transcript: String,
    pub sermon_log: Vec<SermonLogEntry>,
    pub meter_rms: f32,
    pub meter_peak: f32,
    pub is_listening: bool,
    pub pipeline_state: PipelineState,
    pub led_color: Color32,
    pub led_blink: bool,
    pub gain: f32,
    pub translation: String,
    pub selected_device: String,
    pub devices: Vec<String>,
    pub manual_input: String,
    pub focus_manual: bool,
    pub log_filter: String,
    pub verse_fade: f32,
    pub asr_confidence: f32,
    pub snr_db: i32,
    pub is_clipping: bool,
    pub last_error: Option<String>,
    pub program_out: bool,
    pub active_right_tab: usize,
    pub active_bottom_tab: usize,
    pub window_maximized: bool,
    pub audio_mock: AudioControlState,
    pub broadcast_mock: BroadcastState,
    pub workspace_preset: WorkspacePreset,
    pub pipeline: hv_pipeline::Pipeline,
    pub store: std::sync::Arc<bible_core::store::TranslationStore>,
    pub asr_info: crate::pipeline_integration::AsrEngineInfo,
    verse_history: Vec<(String, String)>,
    pub config: AppConfig,
    blink_clock: Instant,
    pub dark_mode: bool,
}

impl HvBibleApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config = AppConfig::load();
        theme::apply_visuals(&cc.egui_ctx, config.dark_mode);

        let devices = match hv_audio::CpalCapture::list_devices() {
            Ok(devs) => devs.into_iter().map(|d| d.name).collect::<Vec<String>>(),
            Err(_) => vec!["Default Input".to_string()],
        };

        let selected_device = config.audio_device.clone().unwrap_or_else(|| {
            devices
                .first()
                .cloned()
                .unwrap_or_else(|| "Default Input".to_string())
        });

        let pipeline_config = PipelineConfig {
            audio_device: Some(selected_device.clone()),
            translation: config.translation.clone(),
            use_mock_asr: config.asr_mode == "mock",
            use_gpu: config.use_gpu,
            asr_mode: config.asr_mode.clone(),
            asr_model_size: config.asr_model_size.clone(),
            vad_mode: config.vad_mode.clone(),
            vad_sensitivity: config.vad_sensitivity,
            noise_gate_enabled: config.noise_gate_enabled,
            noise_gate_threshold: config.noise_gate_threshold,
            hpf_enabled: config.hpf_enabled,
            hpf_frequency: config.hpf_frequency,
            compressor_enabled: config.compressor_enabled,
            compressor_ratio: config.compressor_ratio,
            num_threads: config.num_threads,
            hotwords_enabled: config.hotwords_enabled,
            beam_size: config.beam_size,
        };

        let (pipeline, store, asr_info) =
            match pipeline_integration::create_pipeline(pipeline_config) {
                Ok((p, s, i)) => (p, s, i),
                Err(e) => panic!("Failed to initialize pipeline: {e}"),
            };

        let mut audio_mock = AudioControlState::mock_running();
        audio_mock.device.name = selected_device.clone();

        Self {
            current_verse: None,
            transcript: String::new(),
            sermon_log: Vec::new(),
            meter_rms: -48.0,
            meter_peak: -48.0,
            is_listening: false,
            pipeline_state: PipelineState::Stopped,
            led_color: STATUS_ERROR,
            led_blink: true,
            gain: config.gain,
            translation: config.translation.clone(),
            selected_device,
            devices,
            manual_input: String::new(),
            focus_manual: false,
            log_filter: String::new(),
            verse_fade: 1.0,
            asr_confidence: 0.0,
            snr_db: 0,
            is_clipping: false,
            last_error: None,
            program_out: false,
            active_right_tab: 0,
            active_bottom_tab: 0,
            window_maximized: false,
            audio_mock,
            broadcast_mock: BroadcastState::mock_live(),
            workspace_preset: WorkspacePreset::default_broadcast(),
            pipeline,
            store,
            asr_info,
            verse_history: Vec::new(),
            config: config.clone(),
            blink_clock: Instant::now(),
            dark_mode: config.dark_mode,
        }
    }

    pub fn toggle_listening(&mut self) {
        self.is_listening = !self.is_listening;
        if self.is_listening {
            self.pipeline
                .send_command(hv_pipeline::PipelineCommand::Start);
            self.pipeline_state = PipelineState::Listening;
            self.last_error = None;
        } else {
            self.pipeline
                .send_command(hv_pipeline::PipelineCommand::Stop);
            self.pipeline_state = PipelineState::Stopped;
            self.meter_rms = -48.0;
            self.meter_peak = -48.0;
        }
        self.refresh_led();
    }

    pub fn approve_current(&mut self) {
        let Some((reference, text)) = self.current_verse.clone() else {
            return;
        };
        self.sermon_log.push(SermonLogEntry {
            timestamp: crate::panels::log_panel::format_clock(),
            reference,
            text,
            approved: true,
        });
        self.led_color = crate::theme::text_inverse();
        self.led_blink = false;
    }

    pub fn reject_current(&mut self) {
        if let Some(verse) = self.current_verse.take() {
            self.verse_history.push(verse);
        }
        self.verse_fade = 0.0;
    }

    pub fn undo_verse(&mut self) {
        if let Some(previous) = self.verse_history.pop() {
            if let Some(current) = self.current_verse.take() {
                self.verse_history.push(current);
            }
            self.current_verse = Some(previous);
            self.verse_fade = 0.0;
        }
    }

    pub fn submit_manual_verse(&mut self) {
        let reference = self.manual_input.trim().to_string();
        if reference.is_empty() {
            return;
        }
        self.manual_input.clear();
        if let Some(current) = self.current_verse.take() {
            self.verse_history.push(current);
        }

        let mut text = String::from("Manual override");
        let mut canonical_ref = reference.clone();

        if let Ok(verse_ref) = bible_core::parser::resolve_text(&reference) {
            if let Ok(g_index) =
                bible_core::canon::resolve_index(verse_ref.book, verse_ref.chapter, verse_ref.verse)
            {
                if let Ok(verse_text) = self.store.get_verse(&self.translation, g_index) {
                    text = verse_text;
                    canonical_ref = format!(
                        "{} {}:{}",
                        bible_core::canon::book_name(verse_ref.book),
                        verse_ref.chapter,
                        verse_ref.verse
                    );
                }
            }
        }

        self.current_verse = Some((canonical_ref.clone(), text.clone()));
        self.verse_fade = 0.0;
        self.led_color = crate::theme::text_inverse();
        self.led_blink = false;
        self.sermon_log.push(SermonLogEntry {
            timestamp: crate::panels::log_panel::format_clock(),
            reference: canonical_ref,
            text,
            approved: true,
        });
    }

    pub fn restore_log_entry(&mut self, index: usize) {
        let Some(entry) = self.sermon_log.get(index) else {
            return;
        };
        if let Some(current) = self.current_verse.take() {
            self.verse_history.push(current);
        }
        self.current_verse = Some((entry.reference.clone(), entry.text.clone()));
        self.verse_fade = 0.0;
    }

    pub fn select_log_hotkey(&mut self, slot: usize) {
        if slot == 0 {
            return;
        }
        let index = self.sermon_log.len().saturating_sub(slot);
        if index < self.sermon_log.len() {
            self.restore_log_entry(index);
        }
    }

    pub fn status_label(&self) -> &'static str {
        match self.pipeline_state {
            PipelineState::Stopped => "STANDBY",
            PipelineState::Listening => "LISTENING",
            PipelineState::Silence => "SILENCE",
            PipelineState::Speaking => "SPEAKING",
        }
    }

    pub fn vad_label(&self) -> &'static str {
        match self.pipeline_state {
            PipelineState::Speaking => "Speaking",
            PipelineState::Silence => "Silence",
            PipelineState::Listening => "Armed",
            PipelineState::Stopped => "Off",
        }
    }

    fn poll_events(&mut self) {
        while let Ok(event) = self.pipeline.events().try_recv() {
            match event {
                PipelineEvent::Transcript {
                    text,
                    is_final: _,
                    confidence,
                } => {
                    if !self.transcript.is_empty() {
                        self.transcript.push(' ');
                    }
                    self.transcript.push_str(&text);
                    if self.transcript.len() > 400 {
                        let drain = self.transcript.len() - 400;
                        self.transcript.drain(..drain);
                    }
                    self.asr_confidence = confidence;
                }
                PipelineEvent::VerseDetected {
                    reference, text, ..
                } => {
                    if let Some(current) = self.current_verse.take() {
                        self.verse_history.push(current);
                    }
                    self.current_verse = Some((reference.clone(), text.clone()));
                    self.verse_fade = 0.0;
                    self.led_color = STATUS_INFO;
                    self.led_blink = false;
                    self.sermon_log.push(SermonLogEntry {
                        timestamp: crate::panels::log_panel::format_clock(),
                        reference,
                        text,
                        approved: false,
                    });
                }
                PipelineEvent::MeterUpdate {
                    rms_db,
                    peak_db,
                    is_clipping,
                } => {
                    self.meter_rms = rms_db;
                    self.meter_peak = peak_db;
                    self.is_clipping = is_clipping;
                    self.snr_db = (rms_db + 48.0).clamp(0.0, 48.0) as i32;
                }
                PipelineEvent::StateChanged(state) => {
                    self.pipeline_state = state;
                    self.refresh_led();
                }
                PipelineEvent::Error(message) => {
                    self.last_error = Some(message);
                    self.led_color = STATUS_ERROR;
                    self.led_blink = false;
                }
            }
        }
    }

    fn refresh_led(&mut self) {
        match self.pipeline_state {
            PipelineState::Stopped => {
                self.led_color = STATUS_ERROR;
                self.led_blink = true;
            }
            PipelineState::Listening | PipelineState::Silence => {
                self.led_color = STATUS_WARNING;
                self.led_blink = false;
            }
            PipelineState::Speaking => {
                self.led_color = STATUS_SUCCESS;
                self.led_blink = false;
            }
        }
    }

    fn sync_mock_state(&mut self) {
        self.audio_mock.device.name = self.selected_device.clone();
        self.audio_mock.level.left_db = self.meter_rms;
        self.audio_mock.level.right_db = self.meter_rms;
        self.audio_mock.level.peak_db = self.meter_peak;
        self.audio_mock.level.clip = self.is_clipping;
        self.audio_mock.level.snr_db = self.snr_db;
        self.audio_mock.vad.status = self.vad_label().to_string();
        self.audio_mock.vad.confidence = self.asr_confidence.max(0.90);
        self.audio_mock.gain.input_gain_db = self.gain;
        self.audio_mock.led.status = self.status_label().to_string();
    }

    fn tick_animation(&mut self, ctx: &Context) {
        self.verse_fade = (self.verse_fade + ctx.input(|i| i.stable_dt) * 2.4).min(1.0);
        if self.led_blink && self.blink_clock.elapsed().as_millis() > 400 {
            self.blink_clock = Instant::now();
        }
        let phase_off = self.led_blink && (self.blink_clock.elapsed().as_millis() / 400) % 2 == 1;
        if self.pipeline_state == PipelineState::Stopped {
            self.led_blink = true;
            self.led_color = if phase_off {
                STATUS_ERROR.linear_multiply(0.25)
            } else {
                STATUS_ERROR
            };
        }
        ctx.request_repaint();
    }

    fn persist_window(&mut self, ctx: &Context) {
        let rect = ctx.input(|i| i.screen_rect());
        self.config.window_width = rect.width();
        self.config.window_height = rect.height();
        self.config.gain = self.gain;
        self.config.translation = self.translation.clone();
        self.config.audio_device = Some(self.selected_device.clone());
    }
}

impl eframe::App for HvBibleApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        theme::apply_visuals(ctx, self.dark_mode);
        self.poll_events();
        shortcuts::handle(ctx, self);
        self.tick_animation(ctx);
        self.sync_mock_state();

        shell::title_bar::show(ctx, self);
        shell::ribbon::show(ctx, self);
        shell::status_bar::show(ctx, self);
        shell::workspace::show(ctx, self);
        shell::program_out::show(ctx, self);

        self.persist_window(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.config.translation = self.translation.clone();
        self.config.audio_device = Some(self.selected_device.clone());
        self.config.gain = self.gain;
        self.config.dark_mode = self.dark_mode;

        if let Err(error) = self.config.save() {
            tracing::warn!("failed to save config: {error}");
        }
    }
}