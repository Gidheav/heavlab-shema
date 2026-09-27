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
use crate::theme::{self, STATUS_ERROR, STATUS_INFO, STATUS_SUCCESS, STATUS_WARNING, ThemeRegistry, Theme, ThemeMode};

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
    /// True while the mic is open but the chain is held. Distinct from
    /// `is_listening`: the device stays hot, nothing is being recognised.
    pub is_paused: bool,
    /// Per-channel peak hold, decayed once per frame. Lives here rather than
    /// in the meter component because it is a running measurement, not a
    /// drawing.
    pub channel_peak_hold: [f32; 2],
    /// Latched clip flags, cleared by the operator from the Input Level section.
    pub channel_clip: [bool; 2],
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
    pub available_models: Vec<hv_asr::ModelInfo>,
    pub selected_model_id: String,
    pub restart_required: bool,
    pub theme_registry: ThemeRegistry,
    pub current_theme: Theme,
    pub theme_preview: Option<crate::theme::ThemePreview>,
    blink_clock: Instant,
    /// When the previous frame was drawn, so the peak hold can fall at a real
    /// rate instead of an assumed one.
    last_frame: Instant,
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
            asr_model_id: config.asr_model_id.clone(),
            vad_mode: config.vad_mode.clone(),
            vad_sensitivity: config.vad_sensitivity,
            noise_gate_enabled: config.noise_gate_enabled,
            noise_gate_threshold: config.noise_gate_threshold,
            agc_enabled: config.agc_enabled,
            aec_enabled: config.aec_enabled,
            hpf_enabled: config.hpf_enabled,
            hpf_frequency: config.hpf_frequency,
            compressor_enabled: config.compressor_enabled,
            compressor_ratio: config.compressor_ratio,
            num_threads: config.num_threads,
            hotwords_enabled: config.hotwords_enabled,
            beam_size: config.beam_size,
        };

        let (pipeline, store, asr_info) =
            match pipeline_integration::create_pipeline(pipeline_config, None) {
                Ok((p, s, i)) => (p, s, i),
                Err(e) => panic!("Failed to initialize pipeline: {e}"),
            };

        let mut audio_mock = AudioControlState::mock_running();
        audio_mock.device.name = selected_device.clone();

        // Scan available models
        let data_path = crate::paths::data_dir();
        let models_path = data_path.join("models");
        let available_models = hv_asr::scan_models(&models_path);
        let selected_model_id = config.asr_model_id.clone();

        // Initialize theme system
        let theme_registry = ThemeRegistry::new();
        let theme_mode = match config.theme_mode.as_str() {
            "light" => ThemeMode::Light,
            "follow_system" => ThemeMode::FollowSystem,
            _ => ThemeMode::Dark,
        };
        let mut current_theme = Theme::new(config.theme_id.clone(), theme_mode);
        if let Some(font_config) = config.font_config.clone() {
            current_theme.font_config = font_config;
        }
        
        // Apply initial theme
        if let Some(definition) = theme_registry.get(&current_theme.id) {
            current_theme.apply(&cc.egui_ctx, definition);
        }

        Self {
            current_verse: None,
            transcript: String::new(),
            sermon_log: Vec::new(),
            meter_rms: -48.0,
            meter_peak: -48.0,
            is_paused: false,
            channel_peak_hold: [-48.0, -48.0],
            channel_clip: [false, false],
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
            available_models,
            selected_model_id,
            restart_required: false,
            theme_registry,
            current_theme,
            theme_preview: None,
            blink_clock: Instant::now(),
            last_frame: Instant::now(),
            dark_mode: config.dark_mode,
        }
    }

    /// Linear capture gain expressed in decibels (0 dB = unity).
    /// `gain` is a multiplier; the UI and the config file both speak dB.
    pub fn gain_db(&self) -> f32 {
        if self.gain <= f32::EPSILON {
            -60.0
        } else {
            (20.0 * self.gain.log10()).clamp(-60.0, 24.0)
        }
    }

    /// Apply a decibels value to the linear capture gain and forward it to the
    /// pipeline. Clamped to the -60 dB .. +24 dB capture range.
    pub fn set_gain_db(&mut self, db: f32) {
        let db = db.clamp(-60.0, 24.0);
        self.gain = 10.0_f32.powf(db / 20.0);
        self.pipeline
            .send_command(hv_pipeline::PipelineCommand::SetGain(self.gain));
    }

    pub fn toggle_listening(&mut self) {
        if self.is_listening {
            self.stop_capture();
        } else {
            self.start_capture();
        }
    }

    /// Opens the capture device. The mic goes live; the chain starts consuming.
    pub fn start_capture(&mut self) {
        if self.is_listening {
            return;
        }
        self.is_listening = true;
        self.is_paused = false;
        self.pipeline
            .send_command(hv_pipeline::PipelineCommand::Start);
        self.pipeline_state = PipelineState::Listening;
        self.last_error = None;
        self.refresh_led();
    }

    /// Holds the chain without closing the device, so resuming costs nothing.
    pub fn pause_capture(&mut self) {
        if !self.is_listening || self.is_paused {
            return;
        }
        self.is_paused = true;
        self.pipeline
            .send_command(hv_pipeline::PipelineCommand::Pause);
    }

    /// Releases a pause. No-op when the pipeline was never started.
    pub fn resume_capture(&mut self) {
        if !self.is_paused {
            return;
        }
        self.is_paused = false;
        self.pipeline
            .send_command(hv_pipeline::PipelineCommand::Resume);
    }

    /// Flips between paused and running, the way a transport's pause button does.
    pub fn toggle_pause(&mut self) {
        if self.is_paused {
            self.resume_capture();
        } else if self.is_listening {
            self.pause_capture();
        } else {
            self.start_capture();
        }
    }

    /// Closes the capture device and zeroes the meters.
    pub fn stop_capture(&mut self) {
        self.is_listening = false;
        self.is_paused = false;
        self.pipeline
            .send_command(hv_pipeline::PipelineCommand::Stop);
        self.pipeline_state = PipelineState::Stopped;
        self.meter_rms = -48.0;
        self.meter_peak = -48.0;
        self.channel_peak_hold = [-48.0, -48.0];
        self.refresh_led();
    }

    /// Re-scans the host for capture devices. Backs the Input Device section's
    /// refresh button; picks up a device plugged in after launch.
    pub fn refresh_devices(&mut self) {
        self.devices = match hv_audio::CpalCapture::list_devices() {
            Ok(devices) => devices.into_iter().map(|device| device.name).collect(),
            Err(error) => {
                tracing::warn!("device refresh failed: {error}");
                self.last_error = Some(format!("Could not read audio devices: {error}"));
                return;
            }
        };
        if self.devices.is_empty() {
            self.devices.push("Default Input".to_string());
        }
        if !self.devices.iter().any(|name| *name == self.selected_device) {
            self.selected_device = self.devices[0].clone();
        }
        self.audio_mock.device.name = self.selected_device.clone();
    }

    /// The host audio API the capture driver reports, for the driver badge.
    pub fn host_driver(&self) -> &'static str {
        if cfg!(target_os = "windows") {
            "WASAPI"
        } else if cfg!(target_os = "macos") {
            "CoreAudio"
        } else if cfg!(target_os = "linux") {
            "ALSA"
        } else {
            "ASIO"
        }
    }

    /// Bypasses a capture processor and persists the choice.
    fn send_processor(&mut self, command: hv_pipeline::PipelineCommand) {
        self.pipeline.send_command(command);
        self.save_config();
    }

    /// Writes the config to disk. Every persisted field goes through here so
    /// one failure is logged once, in one place.
    pub fn save_config(&mut self) {
        if let Err(error) = self.config.save() {
            tracing::warn!("config save failed: {error}");
        }
    }

    pub fn set_hpf_enabled(&mut self, enabled: bool) {
        self.config.hpf_enabled = enabled;
        self.send_processor(hv_pipeline::PipelineCommand::EnableHighPass(enabled));
    }

    pub fn set_compressor_enabled(&mut self, enabled: bool) {
        self.config.compressor_enabled = enabled;
        self.send_processor(hv_pipeline::PipelineCommand::EnableCompressor(enabled));
    }

    /// Low-pass and de-esser are configured per venue and have no live command;
    /// the operator sets them once in Settings, so persisting is all that is
    /// needed here.
    pub fn set_lpf_enabled(&mut self, enabled: bool) {
        self.config.lpf_enabled = enabled;
        self.save_config();
    }

    pub fn set_deesser_enabled(&mut self, enabled: bool) {
        self.config.deesser_enabled = enabled;
        self.save_config();
    }

    pub fn set_agc_enabled(&mut self, enabled: bool) {
        self.config.agc_enabled = enabled;
        self.send_processor(hv_pipeline::PipelineCommand::EnableAgc(enabled));
    }

    pub fn set_aec_enabled(&mut self, enabled: bool) {
        self.config.aec_enabled = enabled;
        self.send_processor(hv_pipeline::PipelineCommand::EnableAec(enabled));
    }

    pub fn set_noise_gate_enabled(&mut self, enabled: bool) {
        self.config.noise_gate_enabled = enabled;
        self.send_processor(hv_pipeline::PipelineCommand::EnableNoiseGate(enabled));
    }

    /// Retunes the live voice-activity detector. No restart needed.
    pub fn set_vad_sensitivity(&mut self, sensitivity: f32) {
        let sensitivity = sensitivity.clamp(0.0, 1.0);
        self.config.vad_sensitivity = sensitivity;
        self.pipeline
            .send_command(hv_pipeline::PipelineCommand::SetVadSensitivity(sensitivity));
        let _ = self.config.save();
    }

    /// Opens the Settings window on the Audio page — the gear in the pinned bar.
    pub fn open_audio_settings(&mut self) {
        self.config.layout_state.settings_tab = crate::panels::settings_window::TAB_AUDIO;
        self.config.layout_state.settings_open = true;
    }

    /// Hands the meters to the Input Level section for one frame.
    ///
    /// The pipeline reports a single downmixed pair of values, so both channels
    /// carry the same level. The peak hold is per channel so the two strips
    /// still fall independently, which is what the operator expects to see.
    pub fn sample_meters(&mut self) {
        let level = [self.meter_rms, self.meter_rms];
        for channel in 0..2 {
            self.channel_peak_hold[channel] = self.channel_peak_hold[channel]
                .max(level[channel])
                .max(self.meter_peak);
            if self.is_clipping {
                self.channel_clip[channel] = true;
            }
        }
    }

    /// Lets the peak hold fall at 20 dB per second, the usual PPM behaviour.
    pub fn decay_peak_hold(&mut self, dt_seconds: f32) {
        let drop = 20.0 * dt_seconds;
        for channel in &mut self.channel_peak_hold {
            *channel = (*channel - drop).max(-48.0);
        }
    }

    /// Clears the latched clip indicators. The operator acknowledges the clip
    /// and the lights go out; without this the dot stays red all service.
    pub fn clear_clip(&mut self) {
        self.channel_clip = [false, false];
    }

    // ── Presets ─────────────────────────────────────────────────────────────

    /// A snapshot of the parameters an operator dials in during setup.
    pub fn current_preset(&self, name: &str) -> crate::config::SavedPreset {
        crate::config::SavedPreset {
            name: name.to_string(),
            gain_db: self.gain_db(),
            agc_enabled: self.config.agc_enabled,
            aec_enabled: self.config.aec_enabled,
            noise_gate_enabled: self.config.noise_gate_enabled,
            vad_sensitivity: self.config.vad_sensitivity,
            led_hardware: self.audio_mock.led.hardware_enabled,
            led_screen: self.audio_mock.led.screen_enabled,
        }
    }

    /// Stores the live console under `name`, replacing any preset with that name.
    pub fn save_preset(&mut self, name: &str) {
        let preset = self.current_preset(name);
        match self
            .config
            .saved_presets
            .iter_mut()
            .find(|stored| stored.name == name)
        {
            Some(existing) => *existing = preset,
            None => self.config.saved_presets.push(preset),
        }
        self.config.preset_name = name.to_string();
        self.save_config();
    }

    pub fn delete_preset(&mut self, name: &str) {
        self.config.saved_presets.retain(|stored| stored.name != name);
        self.save_config();
    }

    /// Names of the presets the operator has actually saved, for the pickers.
    pub fn stored_preset_names(&self) -> Vec<String> {
        self.config
            .saved_presets
            .iter()
            .map(|preset| preset.name.clone())
            .collect()
    }

    /// Applies a stored preset: writes the config, then tells the pipeline about
    /// every value it owns. Toggling a bypass is only useful if the chain
    /// actually changes, so the commands go out here rather than on save.
    pub fn apply_preset(&mut self, name: &str) {
        let Some(preset) = self
            .config
            .saved_presets
            .iter()
            .find(|stored| stored.name == name)
            .cloned()
        else {
            return;
        };

        self.set_gain_db(preset.gain_db);
        self.set_agc_enabled(preset.agc_enabled);
        self.set_aec_enabled(preset.aec_enabled);
        self.set_noise_gate_enabled(preset.noise_gate_enabled);
        self.set_vad_sensitivity(preset.vad_sensitivity);
        self.audio_mock.led.hardware_enabled = preset.led_hardware;
        self.audio_mock.led.screen_enabled = preset.led_screen;
        self.config.preset_name = name.to_string();
        self.save_config();
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
        if self.is_paused {
            return "PAUSED";
        }
        match self.pipeline_state {
            PipelineState::Stopped => "STANDBY",
            PipelineState::Listening => "LISTENING",
            PipelineState::Silence => "SILENCE",
            PipelineState::Speaking => "SPEAKING",
        }
    }

    pub fn vad_label(&self) -> &'static str {
        if self.is_paused {
            return "Paused";
        }
        match self.pipeline_state {
            PipelineState::Speaking => "Speaking",
            PipelineState::Silence => "Silence",
            PipelineState::Listening => "Armed",
            PipelineState::Stopped => "Off",
        }
    }

    pub fn change_model(&mut self, model_id: String) {
        if self.selected_model_id != model_id {
            self.selected_model_id = model_id.clone();
            self.config.asr_model_id = model_id;
            let _ = self.config.save();
            self.restart_required = true;
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
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32().min(0.5);
        self.last_frame = now;
        self.decay_peak_hold(dt);
        self.sample_meters();

        self.audio_mock.device.name = self.selected_device.clone();
        self.audio_mock.device.driver = self.host_driver().to_string();
        self.audio_mock.level.left_db = self.meter_rms;
        self.audio_mock.level.right_db = self.meter_rms;
        self.audio_mock.level.peak_db = self.meter_peak;
        self.audio_mock.level.clip = self.is_clipping;
        self.audio_mock.level.snr_db = self.snr_db;
        self.audio_mock.vad.status = self.vad_label().to_string();
        self.audio_mock.vad.confidence = self.asr_confidence.max(0.90);
        self.audio_mock.vad.sensitivity = self.config.vad_sensitivity;
        self.audio_mock.gain.input_gain_db = self.gain_db();
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
        if self.pipeline_state != PipelineState::Stopped {
            ctx.request_repaint_after(std::time::Duration::from_millis(33)); // ~30 FPS
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(200)); // ~5 FPS when stopped
        }
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
        if let Some(definition) = self.theme_registry.get(&self.current_theme.id) {
            self.current_theme.apply(ctx, definition);
        } else {
            theme::apply_visuals(ctx, self.dark_mode);
        }

        // ── Single unified zoom system ──────────────────────────────────────
        // set_zoom_factor natively scales egui's entire geometry (panels, paddings).
        // text_styles are derived from base_size. They scale automatically
        // because set_zoom_factor multiplies the final render size.
        let zoom = self.current_theme.font_config.ui_scale.clamp(0.5, 2.0);
        ctx.set_zoom_factor(zoom);

        let base_px = self.current_theme.font_config.size.to_pixels();
        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(eframe::egui::TextStyle::Body,     eframe::egui::FontId::proportional(base_px));
        style.text_styles.insert(eframe::egui::TextStyle::Button,   eframe::egui::FontId::proportional(base_px));
        style.text_styles.insert(eframe::egui::TextStyle::Small,    eframe::egui::FontId::proportional(base_px * 0.85));
        style.text_styles.insert(eframe::egui::TextStyle::Monospace, eframe::egui::FontId::monospace(base_px * 0.9));
        style.text_styles.insert(eframe::egui::TextStyle::Heading,  eframe::egui::FontId::proportional(base_px * 1.25));
        ctx.set_style(style);
        self.poll_events();
        shortcuts::handle(ctx, self);
        self.tick_animation(ctx);
        self.sync_mock_state();

        shell::title_bar::show(ctx, self);
        shell::ribbon::show(ctx, self);
        shell::status_bar::show(ctx, self);
        shell::workspace::show(ctx, self);
        shell::program_out::show(ctx, self);
        
        crate::panels::settings_window::show(ctx, self);

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