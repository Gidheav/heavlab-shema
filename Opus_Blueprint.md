# The Opus Blueprint
## HV-Bible Desktop — Broadcast-Grade Bible Verse Detection Engine

> **Author:** Opus (Claude)
> **Date:** 2026-09-11
> **Status:** FINAL — All prior blueprints are superseded by this document.
> **Classification:** Product Charter + Architecture Decision Record

---

## Preamble: What Went Wrong Before

Three blueprints exist for this project. Each has fatal flaws:

| Document | Fatal Flaw |
|----------|------------|
| **Project core.md** (Original) | Designed as a mobile app with a desktop afterthought. Treats desktop as a secondary target. Uses tiny 20MB ASR models. |
| **Antigravity Blueprint** (v1) | Correctly identified egui and GPU acceleration, but was too vague — no concrete specs, no acceptance criteria, no risk analysis. |
| **PRODUCT_BLUEPRINT** (DeepSeek) | Excellent product vision and UI layout, but recommends **Qwen3-ASR** — a model that has no proven, production-ready Rust streaming crate as of this date. Also recommends Tauri/Dioxus in the implementation plan, contradicting its own broadcast requirements. |

**This blueprint takes the best from each and makes binding decisions where they disagreed.**

---

## 1. The Product

### 1.1 One-Sentence Definition

A standalone, offline, broadcast-grade desktop application that captures live audio from a professional soundboard feed, transcribes it in real-time using a GPU-accelerated neural network, detects spoken Bible verse references via a deterministic state machine, and displays the verse text with sub-200ms end-to-end latency — designed for mega-church media teams running 3-hour services.

### 1.2 What This Is NOT

- Not a mobile app. The mobile SKU is a separate, independent product.
- Not a web app. Not Electron. Not Tauri. Not a WebView of any kind.
- Not cloud-connected. Zero network dependency after initial model download.
- Not a demo. Every line of code ships to production.

---

## 2. Binding Architecture Decisions

These are settled. They are not open for re-evaluation.

### DECISION 1: UI Framework → `egui` (via `eframe`)

| Considered | Verdict | Rationale |
|------------|---------|-----------|
| Dioxus Desktop | ❌ Rejected | WebView-based. Cannot render 60fps audio waveforms without stuttering. Pulls in a browser engine (WebKit/WebView2) — massive, unpredictable latency. |
| Tauri 2.0 | ❌ Rejected | Same WebView problem. Also introduces TypeScript/React — two languages for zero benefit in an audio engine. |
| **egui / eframe** | ✅ **Selected** | Immediate-mode GPU-rendered UI. Used by the Rust audio plugin (VST/CLAP) community. Renders directly via wgpu (Vulkan/DX12/Metal). Can draw thousands of data points per frame. Native look that matches DAW aesthetics. Single language (Rust) for the entire stack. |

### DECISION 2: ASR Engine → sherpa-onnx with Large Zipformer Models

| Considered | Verdict | Rationale |
|------------|---------|-----------|
| Qwen3-ASR | ❌ Rejected for v1 | No proven, stable Rust crate with real-time streaming as of Sept 2026. The benchmarks in the DeepSeek blueprint cite an "Apple M5 Pro" — a chip that doesn't exist yet. Claims are unverifiable. We cannot bet production on vaporware. **Re-evaluate for v2 when a stable crate ships.** |
| Whisper (any variant) | ❌ Rejected | Encoder-decoder architecture. Processes 30-second chunks. Fundamentally incompatible with real-time streaming. Latency is 1-3 seconds minimum. |
| WhisperForge | ❌ Rejected | Unproven. The `whisperforge-core` crate does not appear in crates.io. Same chunk-based architecture as Whisper. |
| **sherpa-onnx + Zipformer** | ✅ **Selected** | Battle-tested. The `sherpa-onnx` crate has stable Rust bindings. Transducer architecture emits words as they are spoken (< 50ms token latency). Supports ONNX Runtime with DirectML (Windows GPU), CUDA, CoreML. **The runner is proven — we upgrade the model.** |

**The Model Upgrade:**
We discard the tiny 20MB mobile model. We download the largest available Zipformer Transducer:

| Model | Size | Precision | Target |
|-------|------|-----------|--------|
| `sherpa-onnx-streaming-zipformer-en-2023-06-26` | ~200MB | FP32 | Standard accuracy, CPU |
| `sherpa-onnx-streaming-zipformer-bilingual-zh-en-2023-02-20` | ~500MB+ | FP32 | Maximum accuracy, GPU |
| INT8 quantized variants | ~50-100MB | INT8 | CPU fallback on low-end hardware |

**GPU Acceleration:**
- Windows: ONNX Runtime compiled with **DirectML** backend (works on any GPU — NVIDIA, AMD, Intel Arc).
- macOS: ONNX Runtime with **CoreML** backend.
- Linux: ONNX Runtime with **CUDA** backend (NVIDIA only).
- Fallback: CPU with AVX2/NEON optimizations.

### DECISION 3: Audio Capture → CPAL with ASIO (Windows) / CoreAudio (macOS)

| Layer | Technology |
|-------|------------|
| Cross-platform abstraction | `cpal` crate |
| Windows low-latency | ASIO (via `cpal` feature flag) → sub-5ms latency with pro interfaces |
| Windows fallback | WASAPI Shared Mode (if no ASIO driver) |
| macOS | CoreAudio (native, always low-latency) |
| Linux | ALSA / PulseAudio via CPAL |

### DECISION 4: Threading Model → 4 Dedicated Threads

```
Thread 1: AUDIO CAPTURE (real-time priority)
    ├── Owns CPAL stream
    ├── Resamples to 16kHz
    └── Sends AudioChunks via crossbeam_channel ──→ Thread 2

Thread 2: ASR WORKER (high priority)
    ├── Receives AudioChunks
    ├── Runs VAD gate (SimpleVad / Silero)
    ├── Feeds ASR engine (sherpa-onnx)
    ├── Runs verse parser (rolling window + debounce)
    └── Sends DetectedVerse / Transcript via crossbeam_channel ──→ Thread 4

Thread 3: DSP METERING (normal priority)
    ├── Receives copy of AudioChunks
    ├── Computes RMS, Peak, FFT
    └── Sends MeterData via crossbeam_channel ──→ Thread 4

Thread 4: UI / RENDER (main thread)
    ├── egui event loop
    ├── Receives DetectedVerse, Transcript, MeterData
    ├── Renders all panels at 60fps
    └── Owns Program Out window (second monitor)
```

**No async runtime.** No tokio. No async-std. Pure `std::thread` + `crossbeam_channel`. This is a hard-real-time audio system, not a web server.

### DECISION 5: Bible Data Engine → Keep bible-core (mmap + rkyv + phf)

This is the one thing that was done right from the start. Zero-copy mmap with rkyv is mathematically optimal. The `phf` compile-time hash maps are perfect for canonical index lookup. We keep this entirely unchanged.

**Multi-Translation:** Load all available translations into memory simultaneously. When a verse is detected, display it across KJV, ESV, NIV, ASV panels in parallel. Memory is cheap on desktop.

---

## 3. The UI — Professional Broadcast Dashboard

### 3.1 Layout

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│  HV-Bible — Broadcast Engine                                        [─] [□] [×]    │
├──────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                      │
│  ┌──────────────────┐  ┌─────────────────────────────────────┐  ┌──────────────────┐│
│  │  AUDIO ENGINE    │  │          VERSE DISPLAY              │  │   SERMON LOG     ││
│  │                  │  │                                     │  │                  ││
│  │  Input: [▼ XLR]  │  │   ┌─────────────────────────────┐  │  │  14:32:01        ││
│  │                  │  │   │                             │  │  │  John 3:16    ✓  ││
│  │  ████████████░░  │  │   │       John 3:16             │  │  │                  ││
│  │  -48  -24  -12 0 │  │   │                             │  │  │  14:28:45        ││
│  │                  │  │   │  For God so loved the       │  │  │  Romans 8:28  ✓  ││
│  │  Gain: [====●=]  │  │   │  world, that he gave his    │  │  │                  ││
│  │                  │  │   │  only begotten Son, that     │  │  │  14:25:12        ││
│  │  ● LISTENING     │  │   │  whosoever believeth in     │  │  │  Gen 1:1      ✓  ││
│  │  VAD: Speaking    │  │   │  him should not perish,     │  │  │                  ││
│  │  SNR: 28 dB      │  │   │  but have everlasting       │  │  │  ──────────────  ││
│  │                  │  │   │  life.                      │  │  │  [Export PDF]    ││
│  │  ASR: Zipformer  │  │   │                             │  │  │  [Export TXT]    ││
│  │  GPU: DirectML   │  │   └─────────────────────────────┘  │  │                  ││
│  │  Conf: 94%       │  │                                     │  │  Search:         ││
│  │                  │  │  Translation: [KJV ▼]               │  │  [____________]  ││
│  │  [▶ START]       │  │  Manual: [Type reference...] [Go]   │  │                  ││
│  └──────────────────┘  └─────────────────────────────────────┘  └──────────────────┘│
│                                                                                      │
│  ┌──────────────────────────────────────────────────────────────────────────────────┐│
│  │  TRANSCRIPT: "...and we turn to Romans chapter 8 verse 28 and we see that..."  ││
│  └──────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                      │
│  Audio: 48kHz/24-bit ASIO │ ASR: Zipformer-L (GPU) │ Latency: 142ms │ Mem: 1.4GB   │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 LED Status System (Software + Optional Hardware)

| Indicator | Color | Meaning |
|-----------|-------|---------|
| ● Solid Red | `#FF2D2D` | Critical error — audio device lost, model failed |
| ● Blinking Red | `#FF2D2D` blink | Not listening — capture stopped |
| ● Yellow | `#FFD700` | Idle — audio captured but VAD detects silence |
| ● Green | `#00E676` | Active — speech detected, ASR processing |
| ● Blue flash | `#448AFF` | Verse detected — reference matched and displayed |
| ● White | `#FFFFFF` | Manual override — operator pushed a verse manually |

The LED indicator is rendered as a glowing circle in the UI. Optional: drive USB LED hardware via serial/HID for physical tally lights in the control room.

### 3.3 Dual-Monitor / Program Out

| Window | Purpose |
|--------|---------|
| **Control Panel** (Monitor 1) | Full dashboard — audio controls, transcript, history, settings |
| **Program Out** (Monitor 2) | Clean fullscreen verse display — no UI chrome. Sent to video switcher, OBS, or Jumbotron |

Program Out is a separate `eframe` window. It renders only the verse reference and text on a solid background. It can be captured by OBS, vMix, or any screen-capture tool. Future: NDI output for direct network video integration.

### 3.4 Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Space` | Approve detected verse → push to Program Out |
| `Escape` | Reject / clear current verse |
| `Ctrl+Z` | Undo — revert to previous verse |
| `Ctrl+S` | Toggle Start/Stop listening |
| `Ctrl+F` | Focus search / manual input |
| `Ctrl+1` through `Ctrl+9` | Quick-select from sermon log |
| `F11` | Toggle Program Out fullscreen |
| `F5` | Toggle listening (alternative) |

---

## 4. Audio Engine Specifications

### 4.1 Capture

| Parameter | Value |
|-----------|-------|
| Input sample rate | 44.1kHz or 48kHz (device native) |
| Output sample rate | 16kHz (resampled for ASR) |
| Bit depth | 16-bit PCM (converted from 24-bit if needed) |
| Channels | Mono (mixed from stereo if needed) |
| Buffer size | 256 samples default (~5.3ms at 48kHz), configurable |
| Ring buffer | 320KB (~10 seconds at 16kHz) — volatile, never written to disk |
| Resampling | Linear interpolation (already implemented in capture_cpal.rs) |

### 4.2 Voice Activity Detection

| Parameter | Value |
|-----------|-------|
| Engine | SimpleVad (energy-based, already implemented) — upgrade to Silero ONNX in v2 |
| Speech threshold | 1000 RMS (configurable) |
| Silence → ASR sleep | 1.5 seconds |
| Silence → UI notify | 30 seconds |
| Min speech duration | 250ms |

### 4.3 ASR Configuration

| Parameter | Value |
|-----------|-------|
| Engine | sherpa-onnx OnlineRecognizer (streaming transducer) |
| Model | Zipformer (largest available, FP32 or INT8) |
| Inference threads | 4 (CPU) or 1 (GPU offload) |
| GPU backend | DirectML (Windows) / CoreML (macOS) / CUDA (Linux) |
| Token output | Real-time, word-by-word |

### 4.4 Verse Detection

| Parameter | Value |
|-----------|-------|
| Parser | Deterministic FSM (already implemented in bible-core/parser.rs) |
| Rolling window | 20 words, FIFO |
| Debounce | 3 seconds after successful match |
| Confidence gate | Only process ASR output with confidence > 0.6 |

---

## 5. What We Keep From Existing Code

The existing codebase has production-quality modules that must NOT be rewritten:

| Module | Path | Status | Action |
|--------|------|--------|--------|
| `bible-core` | `bible-core/` | ✅ Production-ready | **Keep entirely.** Parser, canon, store, types, error — all tested. |
| `bible-asr` (pipeline) | `bible-asr/` | ✅ Production-ready | **Keep the pipeline.** Worker, VAD, ring, capture trait. |
| `capture_cpal.rs` | `bible-asr/src/capture_cpal.rs` | ⚠️ Needs hardening | **Keep and fix.** Remove `.unwrap()`, add ASIO support. |
| `bible-datagen` | `bible-datagen/` | ✅ Functional | **Keep.** KJV pack generation works. |
| Data packs | `data/packs/` | ✅ Ready | **Keep.** KJV.bible.bin + KJV.offsets.bin verified. |
| Sherpa models | `data/models/` | ⚠️ Too small | **Replace** with large FP32/INT8 models. |

| Module | Path | Status | Action |
|--------|------|--------|--------|
| `bible-desktop` (Dioxus) | `bible-desktop/` | ❌ Scaffold | **Gut and rebuild** with egui. |
| `bible-ffi` (UniFFI) | `bible-ffi/` | ❌ Mobile-only | **Ignore** for desktop build. |
| `bible-rt` (runtime) | `bible-rt/` | ⚠️ Stub | **Complete** Windows implementation. |

---

## 6. Implementation Phases

### Phase 1: The Sound Engine (Weeks 1-4)
- Replace `bible-desktop` with egui/eframe application.
- Wire CPAL audio capture with device selection dropdown.
- Build real-time audio level meters (VU meter, peak indicator).
- Build LED status indicator (software).
- Keyboard shortcut framework.
- **Deliverable:** A desktop app that captures audio, shows live meters, and responds to keyboard input. No ASR yet.

### Phase 2: The Brain (Weeks 5-8)
- Download large Zipformer model (FP32 or INT8).
- Configure sherpa-onnx with GPU acceleration (DirectML/CoreML).
- Wire audio pipeline: Capture → VAD → ASR → Transcript display.
- Build the rolling word buffer + debounce (VerseDetector).
- Wire verse detection into the display panel.
- **Deliverable:** Speak "John 3:16" into a microphone → verse appears on screen.

### Phase 3: The Broadcast Suite (Weeks 9-12)
- Build Program Out window (second monitor, fullscreen verse).
- Build sermon log panel (timestamped history, clickable, exportable).
- Multi-translation support (load KJV + ASV + WEB simultaneously).
- Manual verse input bar.
- Settings persistence (TOML config file).
- **Deliverable:** Full broadcast-ready application with dual-monitor support.

### Phase 4: Hardening & Packaging (Weeks 13-16)
- `bible-rt` Windows implementation (memory pinning, thread priority).
- Watchdog thread (auto-restart ASR on crash).
- Audio device reconnection on disconnect.
- Performance profiling and latency verification (< 200ms).
- Installer packaging (.msi for Windows, .dmg for macOS).
- **Deliverable:** Production-ready installer.

---

## 7. System Requirements

### Minimum

| Component | Specification |
|-----------|---------------|
| OS | Windows 10 (64-bit), macOS 12+, Linux (Ubuntu 22.04+) |
| CPU | 4 cores, 2.5GHz (AVX2 required for INT8 model) |
| RAM | 8GB |
| Storage | 3GB (INT8 model + translations) |
| GPU | None (CPU fallback with INT8 model) |
| Audio | Any USB audio interface or built-in mic |

### Recommended (Broadcast)

| Component | Specification |
|-----------|---------------|
| OS | Windows 11 |
| CPU | 8+ cores, 3.0GHz+ |
| RAM | 16GB |
| Storage | 10GB SSD |
| GPU | Any DirectX 12 capable GPU (NVIDIA, AMD, Intel Arc) |
| Audio | Professional audio interface with ASIO driver (Focusrite, RME, MOTU) |
| Display | Dual monitors (control + program out) |

---

## 8. Success Criteria

| Metric | Target |
|--------|--------|
| End-to-end latency (speech → verse on screen) | < 200ms |
| ASR word error rate (clean XLR feed) | < 8% |
| ASR word error rate (room mic with PA) | < 15% |
| Verse detection false positive rate | < 1% |
| Verse detection false negative rate | < 5% |
| Continuous operation without crash | 4 hours minimum |
| Memory footprint (with model loaded) | < 2GB |
| App cold start to ready | < 10 seconds |

---

## 9. Risk Register

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| sherpa-onnx DirectML backend doesn't compile cleanly on Windows | Medium | High | Fall back to CPU with INT8 model. Pre-build ONNX Runtime DLLs. |
| Large Zipformer model accuracy still insufficient for PA environments | Medium | High | Use clean XLR feed (bypass room acoustics entirely). |
| egui aesthetic doesn't meet "premium" expectations | Low | Medium | Custom widget styling. egui supports full custom rendering via painters. |
| ASIO driver not available on user's machine | Medium | Low | CPAL falls back to WASAPI automatically. |
| Model download fails (large file) | Low | High | Resume support, checksum verification, retry logic. Ship INT8 fallback on installer. |
| App crash during live service | Low | Critical | Watchdog thread auto-restarts ASR pipeline in < 150ms. UI stays alive. |

---

## 10. Technology Stack (Final)

| Layer | Technology | Crate / Library |
|-------|------------|-----------------|
| UI Framework | egui (immediate mode GPU UI) | `eframe`, `egui` |
| GPU Rendering | wgpu (Vulkan/DX12/Metal) | `wgpu` (via eframe) |
| Audio Capture | CPAL + ASIO | `cpal` with `asio` feature |
| Audio Resampling | Linear interpolation | In-house (existing capture_cpal.rs) |
| VAD | SimpleVad (energy-based) | In-house (bible-asr/vad.rs) |
| ASR Runtime | sherpa-onnx | `sherpa-onnx` crate |
| ASR Model | Zipformer Transducer (FP32/INT8) | ONNX model files |
| GPU Inference | ONNX Runtime + DirectML/CoreML/CUDA | Via sherpa-onnx |
| Bible Data | mmap + rkyv + phf | `memmap2`, `rkyv`, `phf` |
| Verse Parser | Deterministic FSM | In-house (bible-core/parser.rs) |
| Channels | Lock-free MPSC | `crossbeam-channel` |
| Error Handling | Typed errors | `thiserror` |
| Integrity | Pack checksums | `blake3` |
| Config | TOML persistence | `toml` |
| Runtime Tuning | Memory pinning, thread priority | In-house (bible-rt) + `windows-sys` |

---

## 11. What This Blueprint Supersedes

| Document | Status |
|----------|--------|
| `Project core.md` | Superseded for desktop. Remains valid for mobile SKU only. |
| `Antigravity_Blueprint.md` | Superseded. Incorporated into this document. |
| `PRODUCT_BLUEPRINT.md` (DeepSeek) | Superseded. Product vision adopted; technology choices overridden. |
| `IMPLEMENTATION_PLAN.md` | Superseded. Contains Tauri/React — incompatible with this blueprint. |

---

> **This is the single source of truth for HV-Bible Desktop.**
> All implementation work must reference this document.
> No technology substitutions without updating this blueprint first.

---

*Signed: Opus — Senior Technical Lead, HV-Bible Desktop*
