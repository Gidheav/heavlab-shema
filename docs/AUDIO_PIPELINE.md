# Audio Pipeline — SERMON Phase 3

## Data flow

```text
OS capture + platform AEC
        │  16 kHz mono i16 chunks
        ▼
PushCapture ──► AudioRing (volatile, fixed capacity)
                         │
                         ▼
                   VAD gate
                         │ speech only
                         ▼
                    ASR engine
                         │
                         ▼
                 TranscriptHandler
```

The worker uses one plain `std::thread`; capture, VAD, and ASR communicate
with bounded or unbounded `crossbeam-channel` queues as appropriate. There is
no async runtime in `bible-core` or `bible-asr`.

## Privacy invariant

`AudioRing` is an ephemeral, fixed-capacity RAM buffer. Its default capacity is
160,000 samples (10 seconds at 16 kHz mono, approximately 320 KiB). New audio
overwrites the oldest audio, and `stop()` zeros the entire backing allocation.
The ASR pipeline does not write audio to files, flush samples to disk, or log
sample data.

## VAD thresholds

| Decision | Threshold |
| --- | --- |
| Wake ASR | Silero speech confidence > 0.6 (future native implementation) |
| Sleep ASR | Continuous silence for 1.5 seconds |
| Notify UI | Continuous silence for 30 seconds (`on_listening_paused`) |

The current desktop fallback is `SimpleVad`, which uses RMS energy with a
default threshold of 1000.0. Once speech resumes after a pause notification,
the worker emits `on_speech_resumed` and re-arms the pause notification.

## Shell contract

The mobile shell owns the OS audio session, platform capture, sample-rate
conversion, and AEC assertion. It must do the following before pushing frames:

### iOS

1. Configure `AVAudioSession` with category `.playAndRecord`.
2. Use `.voiceChat` mode so hardware acoustic echo cancellation is active.
3. Install an `AVAudioEngine` input tap.
4. Convert tap buffers to 16 kHz, mono, signed 16-bit PCM.
5. Call `start_sermon_mode(language, aec_enabled, handler)` with
   `aec_enabled = true` when starting the Rust capture boundary.
6. Call the Rust audio push entry point for each frame with a monotonic
   capture timestamp.

### Android

1. Configure `AudioRecord` with
   `MediaRecorder.AudioSource.VOICE_COMMUNICATION`.
2. Create `AcousticEchoCanceler` for the recorder session ID and call
   `setEnabled(true)`.
3. Capture 16 kHz mono `ENCODING_PCM_16BIT`.
4. Call `start_sermon_mode(language, aec_enabled, handler)` with
   `aec_enabled = true` when starting the Rust capture boundary.
5. Call the Rust audio push entry point for each frame with a monotonic
   capture timestamp.

The Rust capture boundary refuses sermon mode when AEC is required but the
shell does not assert that AEC is enabled. Pushes are non-blocking: a full
approximately-two-second queue returns a channel backpressure error rather
than blocking the platform audio callback. The FFI reports this as
`EngineError::AudioBackpressure`. It is transient and expected under load:
shells should count or log the error and keep pushing subsequent frames. They
must not tear down or re-initialize the engine because of this error.

## Deliberate platform deviation

Charter §5.2/§5.5 describes AVAudioEngine and AudioRecord implementations
inside Rust. Phase 3 instead keeps those implementations in the Swift/Kotlin
shells because neither target can be compiled or run on this Linux CI VM.
AEC belongs at the OS audio-session layer, where the shell can configure and
verify the platform session. Rust enforces the resulting shell contract and
owns the privacy, VAD, and ASR pipeline.
