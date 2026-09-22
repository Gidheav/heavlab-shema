//! CPAL desktop capture with linear resampling to 16 kHz mono.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::sync::{Arc, Mutex};

use crate::capture::{AudioChunk, AudioDevice, CaptureError, MicrophoneCapture};

const TARGET_SAMPLE_RATE: u32 = 16_000;
const TARGET_CHUNK_SAMPLES: usize = 1600;

#[derive(Default)]
pub struct CpalCapture {
    active_flag: Arc<Mutex<bool>>,
    preferred_device: Option<String>,
}

impl CpalCapture {
    pub fn new() -> Self {
        Self {
            active_flag: Arc::new(Mutex::new(false)),
            preferred_device: None,
        }
    }

    pub fn with_device(name: impl Into<String>) -> Self {
        Self {
            active_flag: Arc::new(Mutex::new(false)),
            preferred_device: Some(name.into()),
        }
    }

    /// List host input devices for the UI dropdown.
    pub fn list_devices() -> Result<Vec<AudioDevice>, CaptureError> {
        let host = cpal::default_host();
        let default_name = host
            .default_input_device()
            .and_then(|device| device.name().ok());

        let mut devices = Vec::new();
        let iter = host
            .input_devices()
            .map_err(|error| CaptureError::Capture(format!("input device list: {error}")))?;
        for device in iter {
            let name = match device.name() {
                Ok(name) => name,
                Err(_) => continue,
            };
            let (sample_rate, channels) = match device.default_input_config() {
                Ok(config) => (config.sample_rate().0, config.channels()),
                Err(_) => (0, 0),
            };
            let is_default = default_name.as_deref() == Some(name.as_str());
            devices.push(AudioDevice {
                name,
                is_default,
                sample_rate,
                channels,
            });
        }
        Ok(devices)
    }

    fn resolve_device(preferred: Option<&str>) -> Result<cpal::Device, CaptureError> {
        let host = cpal::default_host();
        if let Some(want) = preferred {
            if let Ok(iter) = host.input_devices() {
                for device in iter {
                    if device.name().ok().as_deref() == Some(want) {
                        return Ok(device);
                    }
                }
            }
            return Err(CaptureError::Capture(format!(
                "audio device not found: {want}"
            )));
        }
        host.default_input_device()
            .ok_or_else(|| CaptureError::Capture("no default input device found".to_string()))
    }

    fn build_stream<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        sender: Sender<AudioChunk>,
    ) -> Result<cpal::Stream, String>
    where
        T: cpal::Sample + cpal::SizedSample,
        i16: cpal::FromSample<T>,
    {
        let channels = config.channels.max(1) as usize;
        let native_rate = config.sample_rate.0;
        let step: f64 = f64::from(native_rate) / f64::from(TARGET_SAMPLE_RATE);
        let mut resample_pos: f64 = 0.0;
        let mut prev_sample: i16 = 0;
        let mut input_index: usize = 0;
        let mut output_buf: Vec<i16> = Vec::with_capacity(TARGET_CHUNK_SAMPLES * 2);

        let err_fn = |err| tracing_cpal_error(err);
        let stream = device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    for frame in data.chunks(channels) {
                        let Some(first) = frame.first() else {
                            continue;
                        };
                        let sample: i16 = cpal::Sample::from_sample(*first);

                        while resample_pos < (input_index + 1) as f64 {
                            if native_rate == TARGET_SAMPLE_RATE {
                                output_buf.push(sample);
                            } else {
                                let frac = resample_pos - resample_pos.floor();
                                let interp =
                                    f64::from(prev_sample) * (1.0 - frac) + f64::from(sample) * frac;
                                output_buf.push(interp as i16);
                            }
                            resample_pos += step;

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
            .map_err(|error| format!("Error building stream: {error}"))?;

        Ok(stream)
    }
}

fn tracing_cpal_error(err: cpal::StreamError) {
    eprintln!("[cpal] stream error: {err}");
}

impl MicrophoneCapture for CpalCapture {
    fn start(&self) -> Result<Receiver<AudioChunk>, CaptureError> {
        let mut active = self
            .active_flag
            .lock()
            .map_err(|error| CaptureError::Capture(format!("lock error: {error}")))?;
        if *active {
            return Err(CaptureError::AlreadyListening);
        }
        *active = true;
        let flag_clone = Arc::clone(&self.active_flag);
        let preferred = self.preferred_device.clone();
        let (sender, receiver) = unbounded();

        std::thread::Builder::new()
            .name("hv-audio-cpal".to_string())
            .spawn(move || {
                let device = match CpalCapture::resolve_device(preferred.as_deref()) {
                    Ok(device) => device,
                    Err(error) => {
                        eprintln!("[cpal] {error}");
                        if let Ok(mut flag) = flag_clone.lock() {
                            *flag = false;
                        }
                        return;
                    }
                };

                let default_config = match device.default_input_config() {
                    Ok(config) => config,
                    Err(error) => {
                        eprintln!("[cpal] failed to get default input config: {error}");
                        if let Ok(mut flag) = flag_clone.lock() {
                            *flag = false;
                        }
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
                    other => Err(format!("Unsupported sample format: {other:?}")),
                };

                match stream_result {
                    Ok(stream) => {
                        if let Err(error) = stream.play() {
                            eprintln!("[cpal] failed to play stream: {error}");
                            if let Ok(mut flag) = flag_clone.lock() {
                                *flag = false;
                            }
                            return;
                        }
                        loop {
                            match flag_clone.lock() {
                                Ok(flag) if !*flag => break,
                                Err(_) => break,
                                _ => {}
                            }
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                    }
                    Err(error) => {
                        eprintln!("[cpal] stream build failed: {error}");
                        if let Ok(mut flag) = flag_clone.lock() {
                            *flag = false;
                        }
                    }
                }
            })
            .map_err(|error| {
                *active = false;
                CaptureError::Capture(format!("capture thread: {error}"))
            })?;

        Ok(receiver)
    }

    fn stop(&self) {
        if let Ok(mut active) = self.active_flag.lock() {
            *active = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CpalCapture;

    #[test]
    fn list_devices_does_not_panic() {
        match CpalCapture::list_devices() {
            Ok(devices) => {
                for device in &devices {
                    assert!(!device.name.is_empty() || device.sample_rate == 0);
                }
            }
            Err(_) => {
                // Headless CI with no audio host is acceptable.
            }
        }
    }
}
