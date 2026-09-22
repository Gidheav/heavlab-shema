#![deny(clippy::unwrap_used, clippy::expect_used)]

#[cfg(feature = "desktop-audio")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
#[cfg(feature = "desktop-audio")]
use crossbeam_channel::{unbounded, Receiver, Sender};
#[cfg(feature = "desktop-audio")]
use crate::capture::{AudioChunk, MicrophoneCapture};
#[cfg(feature = "desktop-audio")]
use crate::error::AsrError;
#[cfg(feature = "desktop-audio")]
use std::sync::{Arc, Mutex};

/// Target sample rate for the ASR engine (16 kHz).
#[cfg(feature = "desktop-audio")]
const TARGET_SAMPLE_RATE: u32 = 16_000;

/// Number of 16 kHz samples per output chunk (~100 ms).
#[cfg(feature = "desktop-audio")]
const TARGET_CHUNK_SAMPLES: usize = 1600;

#[cfg(feature = "desktop-audio")]
#[derive(Default)]
pub struct CpalCapture {
    active_flag: Arc<Mutex<bool>>,
}

#[cfg(feature = "desktop-audio")]
impl CpalCapture {
    pub fn new() -> Self {
        Self {
            active_flag: Arc::new(Mutex::new(false)),
        }
    }

    /// Build an input stream that captures at the device's native rate,
    /// resamples to 16 kHz mono in the callback, and sends chunks of
    /// `TARGET_CHUNK_SAMPLES` samples.
    fn build_stream<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        sender: Sender<AudioChunk>,
    ) -> Result<cpal::Stream, String>
    where
        T: cpal::Sample + cpal::SizedSample,
        i16: cpal::FromSample<T>,
    {
        let channels = config.channels as usize;
        let native_rate = config.sample_rate.0;

        // Resampling state — accumulator-based linear interpolation.
        // For every output sample we advance by `step` in the input domain.
        let step: f64 = native_rate as f64 / TARGET_SAMPLE_RATE as f64;
        let mut resample_pos: f64 = 0.0;     // fractional position in the input
        let mut prev_sample: i16 = 0;         // last input sample (for lerp)
        let mut input_index: usize = 0;       // monotonic count of input samples seen
        let mut output_buf: Vec<i16> = Vec::with_capacity(TARGET_CHUNK_SAMPLES * 2);

        let err_fn = |err| eprintln!("[cpal] stream error: {}", err);

        let stream = device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    // Extract mono (channel 0) from each frame.
                    for frame in data.chunks(channels) {
                        let sample: i16 = cpal::Sample::from_sample(frame[0]);

                        // Emit resampled output samples while our position is
                        // at or before this input sample.
                        while resample_pos < (input_index + 1) as f64 {
                            if native_rate == TARGET_SAMPLE_RATE {
                                // No resampling needed — pass through.
                                output_buf.push(sample);
                            } else {
                                // Linear interpolation between prev_sample and sample.
                                let frac = resample_pos - resample_pos.floor();
                                let interp =
                                    prev_sample as f64 * (1.0 - frac) + sample as f64 * frac;
                                output_buf.push(interp as i16);
                            }
                            resample_pos += step;

                            // Flush a full chunk.
                            if output_buf.len() >= TARGET_CHUNK_SAMPLES {
                                let chunk_data: Vec<i16> =
                                    output_buf.drain(..TARGET_CHUNK_SAMPLES).collect();
                                let timestamp = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_millis() as u64;
                                let _ = sender.send(AudioChunk::new(chunk_data, timestamp));
                            }
                        }

                        prev_sample = sample;
                        input_index += 1;
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| format!("Error building stream: {}", e))?;

        Ok(stream)
    }
}

#[cfg(feature = "desktop-audio")]
impl MicrophoneCapture for CpalCapture {
    fn start(&self) -> Result<Receiver<AudioChunk>, AsrError> {
        let mut active = self.active_flag.lock().map_err(|e| {
            AsrError::CaptureError(format!("lock error: {}", e))
        })?;
        if *active {
            return Err(AsrError::AlreadyListening);
        }
        *active = true;
        let flag_clone = Arc::clone(&self.active_flag);

        let (sender, receiver) = unbounded();

        std::thread::spawn(move || {
            let host = cpal::default_host();
            let device = match host.default_input_device() {
                Some(d) => d,
                None => {
                    eprintln!("[cpal] no default input device found");
                    if let Ok(mut f) = flag_clone.lock() { *f = false; }
                    return;
                }
            };

            // Use the device's default input config instead of forcing 16 kHz.
            let default_config = match device.default_input_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[cpal] failed to get default input config: {}", e);
                    if let Ok(mut f) = flag_clone.lock() { *f = false; }
                    return;
                }
            };

            let sample_format = default_config.sample_format();
            let config: cpal::StreamConfig = default_config.into();

            eprintln!(
                "[cpal] capturing at {} Hz, {} ch, format {:?} — resampling to {} Hz",
                config.sample_rate.0, config.channels, sample_format, TARGET_SAMPLE_RATE
            );

            let stream_result = match sample_format {
                cpal::SampleFormat::F32 => {
                    CpalCapture::build_stream::<f32>(&device, &config, sender.clone())
                }
                cpal::SampleFormat::I16 => {
                    CpalCapture::build_stream::<i16>(&device, &config, sender.clone())
                }
                cpal::SampleFormat::U16 => {
                    CpalCapture::build_stream::<u16>(&device, &config, sender.clone())
                }
                _ => Err(format!("Unsupported sample format: {:?}", sample_format)),
            };

            match stream_result {
                Ok(stream) => {
                    if let Err(e) = stream.play() {
                        eprintln!("[cpal] failed to play stream: {}", e);
                        if let Ok(mut f) = flag_clone.lock() { *f = false; }
                        return;
                    }
                    // Keep thread alive until stopped.
                    loop {
                        match flag_clone.lock() {
                            Ok(f) if !*f => break,
                            Err(_) => break,
                            _ => {}
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
                Err(e) => {
                    eprintln!("[cpal] stream build failed: {}", e);
                    if let Ok(mut f) = flag_clone.lock() { *f = false; }
                }
            }
        });

        Ok(receiver)
    }

    fn stop(&self) {
        if let Ok(mut active) = self.active_flag.lock() {
            *active = false;
        }
    }
}
