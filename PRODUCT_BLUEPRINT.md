
# HV-Bible Desktop
## Professional Broadcast-Grade Bible Verse Detection System

> **Document Type:** Product Vision & Architecture Blueprint
> **Status:** Foundational — Pre-Implementation
> **Date:** 2026-09-06
> **Target Platforms:** Windows, macOS, Linux
> **Philosophy:** No compromises. This is a professional audio production tool that happens to display Bible verses.

---

## 1. Product Vision

### 1.1 What This Is

HV-Bible Desktop is a **professional-grade, standalone desktop application** that listens to live speech in real-time, detects Bible verse references, and displays them instantly with zero-latency visual feedback.

This is **not** a "Bible app with voice." This is an **audio production engine**—a tool designed for the same environments that use professional audio interfaces, broadcast mixing consoles, and stadium-grade sound systems. The Bible display is the *output*. The audio engine is the *product.

### 1.2 What This Is NOT

- **Not** a mobile app ported to desktop.
- **Not** a web app wrapped in Electron.
- **Not** a consumer-grade tool with "good enough" accuracy.
- **Not** connected to the cloud—zero network dependency.
- **Not** dependent on the mobile SKU for anything.

### 1.3 The Core Promise

> **"When a pastor speaks a verse reference, the verse appears on screen before they finish the second syllable of the chapter number. Every time. Without exception. In any acoustic environment."**

---

## 2. The User & The Environment

### 2.1 Primary User Persona: The Broadcast Media Director

- **Location:** Soundproof broadcast control room, separate from the auditorium.
- **Hardware:** High-end PC or Mac workstation with dedicated GPU.
- **Audio Source:** Direct XLR feed from the soundboard's auxiliary output—**clean, uncompressed, zero room echo.**
- **Responsibilities:** Ensuring the verse appears on the Jumbotron and broadcast feed instantly. Manual override when necessary. Maintaining service logs.

### 2.2 Secondary User Persona: The Pastor

- **Location:** At the pulpit.
- **Hardware:** Tablet or phone running the **separate** Mobile SKU.
- **Audio Source:** Device microphone.
- **Responsibilities:** Preaching. The desktop app is their safety net—the media team handles the display.

### 2.3 The Environment

- **Auditorium Size:** 1,000 to 10,000+ seats.
- **Acoustics:** Reverberant. PA system delay. Crowd noise (10,000+ people shouting "Amen!").
- **Duration:** Continuous operation for 2-3 hour services.
- **Criticality:** Failure is not an option. Millions of viewers may be watching via broadcast.

---

## 3. Core Architecture Philosophy

### 3.1 The "Audio-First" Principle

The application is structured around the audio pipeline. Everything else—the UI, the Bible display, the LED system, the logging—is a *consumer* of the audio pipeline's output. The audio pipeline runs on a dedicated, real-time priority thread. The UI runs on a separate thread. They communicate via lock-free channels.

**This is non-negotiable.** If the UI freezes, the audio pipeline continues. If the audio pipeline experiences a micro-stutter, the UI is unaffected.

### 3.2 The "Zero-Compromise" Principle

| Area | Compromise | Our Stance |
|------|------------|------------|
| **ASR Model Size** | "Use a small model to save disk space" | **Rejected.** We download the best model. Disk space is cheap. Accuracy is not. |
| **GPU Requirement** | "Make it work on CPU only" | **Rejected.** We leverage GPU acceleration. We recommend minimum RTX 3060 / Apple M1+ for optimal performance. |
| **Model Download** | "Ship a tiny model with the app" | **Rejected.** We download the 1.7GB+ model on first launch. The user gets the best experience, not the smallest download. |
| **Latency** | "300ms is fast enough" | **Rejected.** Target is <200ms. We optimize for speed at every layer. |
| **Audio Quality** | "Use the built-in mic" | **Rejected.** The desktop app requires a professional audio input—XLR from the soundboard or a high-quality USB interface. |

### 3.3 The "Independence" Principle

**The desktop app is completely independent of the mobile app.** They do not connect. They do not sync. They do not share state.

- **Why:** In a 10,000-person auditorium, a network drop or Bluetooth failure is unacceptable. The media team has their own audio feed, their own GPU, their own display output. They are the authoritative source for the broadcast display.
- **The mobile app is a backup.** If the pastor's tablet crashes, the desktop app continues running. If the desktop app crashes, the mobile app continues running. They are redundant, not dependent.

---

## 4. The Audio Engine

### 4.1 The Pipeline (End-to-End)

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                           AUDIO ENGINE                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────────────┐ │
│  │   Audio Input   │───▶│   Pre-Process   │───▶│    VAD (Silero)        │ │
│  │                 │    │                 │    │                         │ │
│  │ • CPAL (ASIO/   │    │ • Sample Rate   │    │ • Speech Confidence    │ │
│  │   CoreAudio)    │    │   Conversion    │    │ • Silence Detection    │ │
│  │ • Device Select │    │ • Channel Mix   │    │ • Wake/Sleep Gating    │ │
│  │ • Buffer Mgmt   │    │ • Gain Staging  │    │                         │ │
│  └─────────────────┘    └─────────────────┘    └─────────────────────────┘ │
│                                                                             │
│                              ▼                                              │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                      ASR ENGINE (Qwen3-ASR / WhisperForge)             ││
│  │                                                                         ││
│  │  • Streaming inference (real-time, chunked)                             ││
│  │  • GPU acceleration (Metal / CUDA / WGPU)                              ││
│  │  • Multilingual support (English, Chinese, code-switched)              ││
│  │  • Confidence scoring per utterance                                    ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                                                             │
│                              ▼                                              │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                      TRANSCRIPT PROCESSOR                              ││
│  │                                                                         ││
│  │  • Rolling window (last 15-20 words)                                   ││
│  │  • Verse reference parser (state machine)                              ││
│  │  • Confidence threshold enforcement                                    ││
│  │  • Deduplication (prevent double-trigger)                              ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                                                             │
│                              ▼                                              │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                      OUTPUT DISPATCHER                                 ││
│  │                                                                         ││
│  │  • UI Update (Verse Display)                                           ││
│  │  • LED Control (Hardware + Software)                                   ││
│  │  • Logging (Timestamped transcript)                                    ││
│  │  • NDI / HDMI Program Out                                              ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Audio Input — Professional-Grade Capture

| Requirement | Implementation |
|-------------|----------------|
| **Audio API** | **ASIO** on Windows (sub-5ms latency, bypasses Windows kernel mixer). **CoreAudio** on macOS. |
| **Library** | **CPAL** (Cross-Platform Audio Library) with ASIO feature flag enabled. |
| **Sample Rate** | 48kHz or 44.1kHz input → resampled to 16kHz for ASR. |
| **Bit Depth** | 24-bit or 16-bit → converted to 16-bit PCM. |
| **Channels** | Stereo → mixed to mono. |
| **Buffer Size** | Configurable (default: 256 samples = ~5.3ms at 48kHz). |
| **Device Selection** | User-selectable from all available audio inputs. Visual level meters for each. |

### 4.3 Pre-Processing

| Stage | Purpose | Implementation |
|-------|---------|----------------|
| **Sample Rate Conversion** | Convert input (44.1/48kHz) to ASR-required 16kHz | CPAL's built-in resampler or `rubato` crate |
| **Channel Mixing** | Stereo → mono | Simple average of L+R channels |
| **Gain Staging** | Normalize input level | Automatic gain control (AGC) or manual slider |
| **Noise Gate** | Remove low-level background noise | Configurable threshold |

### 4.4 Voice Activity Detection (VAD)

**Engine:** Silero VAD (pure Rust implementation)

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| **Speech Threshold** | 0.6 | Conservative—avoids false positives from crowd noise |
| **Silence Duration (Sleep)** | 1.5 seconds | Puts ASR to sleep during pauses, saving CPU/GPU |
| **Silence Duration (Notify)** | 30 seconds | Notifies UI that listening is paused |
| **Min Speech Duration** | 250ms | Ignores short bursts of noise |

**Why Silero VAD?**
- Proven in production environments.
- Small footprint (~5MB).
- Runs on CPU with negligible latency (~2-5ms).
- Configurable thresholds for different acoustic environments.

---

## 5. ASR Engine — The Heart of the System

### 5.1 The Decision: Qwen3-ASR is the Primary Engine

Based on extensive evaluation, **Qwen3-ASR** is the recommended primary ASR engine for HV-Bible Desktop.

**Benchmark Data (QwenASR on Apple M5 Pro):**

| Metric | Value |
|--------|-------|
| **Inference Latency** (28.2s audio clip) | **613 ms** |
| **Realtime Factor** | **46× faster than realtime** |
| **Performance vs. MLX (GPU)** | **1.12× faster** than GPU implementation |
| **Performance vs. Pure C** | **2.71× faster** than upstream C implementation |

**Key Advantages:**

| Advantage | Details |
|-----------|---------|
| **Pure Rust Implementation** | No Python, no PyTorch, no runtime dependencies |
| **CPU-Optimized** | Hand-tuned NEON, Accelerate, and AMX-aware kernels |
| **Multiple Model Sizes** | 0.6B (1.7GB) and 1.7B (4.5GB) options |
| **Streaming Support** | Native real-time streaming with `feed_audio()` / `finish_streaming()` |
| **Multilingual** | English, Chinese, and code-switched audio |
| **Cross-Platform** | macOS (Metal), Linux/Windows (CUDA), or CPU fallback |
| **GPU Acceleration** | Metal (Apple Silicon) and CUDA (NVIDIA) support |

**Model Selection:**

| Model | Size | Use Case |
|-------|------|----------|
| **Qwen3-ASR-0.6B** | 1.7GB | Primary recommendation. Best balance of speed and accuracy. |
| **Qwen3-ASR-1.7B** | 4.5GB | Maximum accuracy. For high-end GPUs with 8GB+ VRAM. |

### 5.2 Secondary Engine: WhisperForge (Fallback)

**Why WhisperForge?** It provides a complementary capability—**speaker diarization** (identifying *who* is speaking).

| Metric | Value |
|--------|-------|
| **Framework** | Rust + Burn 0.20 (GPU-accelerated) |
| **GPU Backend** | WGPU (Vulkan/DX12/Metal) |
| **Quantization** | INT8 (~4× compression) |
| **Streaming** | Native streaming with KV-cached decoding |
| **Model Sizes** | All Whisper sizes (tiny.en through large-v3) |

**Fallback Strategy:**
1. If Qwen3-ASR fails to load or produces low-confidence results, the engine automatically falls back to WhisperForge.
2. This is transparent to the user. The UI shows "ASR: Qwen3-ASR (Primary) | Fallback: Ready".

### 5.3 ASR Engine Architecture

```rust
// The ASR engine trait — allows swapping engines without changing the pipeline
pub trait AsrEngine: Send + Sync {
    fn initialize(&mut self, config: AsrConfig) -> Result<(), AsrError>;
    fn process_chunk(&mut self, chunk: AudioChunk) -> Result<Option<Transcript>, AsrError>;
    fn reset(&mut self) -> Result<(), AsrError>;
    fn is_streaming(&self) -> bool;
    fn confidence_threshold(&self) -> f32;
}

// Qwen3-ASR implementation
pub struct QwenAsrEngine {
    // Uses qwen3-asr crate
}

// WhisperForge implementation
pub struct WhisperForgeEngine {
    // Uses whisperforge-core crate
}
```

### 5.4 ASR Confidence & Rejection

| Scenario | Confidence | Action |
|----------|------------|--------|
| Clear speech, clear reference | > 0.85 | Auto-display instantly |
| Clear speech, ambiguous reference | 0.60 - 0.84 | Display in UI with yellow "Verify" indicator |
| Noisy audio, weak match | < 0.60 | **Reject.** Do not display. |
| Crowd shouting verse | < 0.60 | **Reject.** The VAD detects speech but ASR confidence is low. |

**Rule:** No result is infinitely better than a wrong result.

---

## 6. The Audio Monitoring & LED System

### 6.1 Visual Feedback — The "Studio Monitor" Philosophy

In a broadcast control room, the media team needs **instant, at-a-glance** awareness of the audio pipeline's state. The LED system provides this.

### 6.2 LED Status Indicators

| LED Color | State | Meaning |
|-----------|-------|---------|
| 🔴 **Red (Solid)** | Error | Audio device failed. ASR model failed to load. Critical error. |
| 🔴 **Red (Blinking)** | Not Listening | App is running but audio capture is paused or disabled. |
| 🟡 **Yellow** | Idle / Silence | Audio is being captured but no speech is detected (VAD sleeping). |
| 🟢 **Green** | Active Listening | VAD has detected speech. ASR is processing. |
| 🔵 **Blue (Flash)** | Verse Detected | A verse reference has been detected and displayed. |
| ⚪ **White** | Manual Override | Media team has manually pushed a verse to the display. |

### 6.3 LED Implementation

**Hardware Support:**
- USB LED controllers (e.g., Corsair iCUE, NZXT Hue, or generic USB-RGB controllers).
- Support for standard LED strips via serial/USB HID.

**Software Emulation:**
- On-screen LED panel that mimics the hardware LEDs.
- Useful for testing and for users without hardware LEDs.

**Audio-Reactive Mode (Optional):**
- LED brightness pulses with audio input level.
- Useful for visual confirmation that audio is being captured.

### 6.4 Audio Level Metering

The UI displays real-time audio levels:

| Meter Element | Purpose |
|---------------|---------|
| **Input Level (VU Meter)** | Visual confirmation that audio is reaching the app |
| **Signal-to-Noise Ratio** | Indicates audio quality |
| **Clipping Indicator** | Warns if input is too hot (distortion) |
| **VAD Activity** | Shows when VAD detects speech |

---

## 7. The Bible Engine

### 7.1 Core Capabilities

| Capability | Implementation |
|------------|----------------|
| **Verse Lookup** | O(1) using `phf` perfect hash map (compile-time generated) |
| **Zero-Copy Access** | `mmap` + `rkyv` for instant verse retrieval |
| **Multiple Translations** | KJV, WEB, ASV (public domain) shipped. ESV/NIV via licensed download. |
| **Parsing Grammar** | Deterministic state machine — no LLM, no NLP |
| **Book Aliases** | Per-language alias tables (e.g., "John", "Jn", "Juan") |

### 7.2 Verse Reference Parser

The parser is a **deterministic finite state machine**. It does not use AI or machine learning.

**Supported Patterns:**

| Pattern | Example |
|---------|---------|
| `<Book> <Chapter>:<Verse>` | "John 3:16" |
| `<Book> <Chapter> <Verse>` | "Genesis 1 1" |
| `<Ordinal> <Book> <Chapter>:<Verse>` | "First Corinthians 13:4" |
| `<Book> <Chapter> verse <Verse>` | "Romans 8 verse 28" |

**Rejection Logic:**
- If the parser sees "John" but no number follows → **ignore**.
- If the parser sees "John 3" but the next word is "apples" → **ignore**.
- If the parser sees "John 3:16" → **display**.

### 7.3 Rolling Window

The parser operates on a **rolling window** of the last 15-20 words from the ASR output.

- Every 200ms, the window is updated with the latest ASR output.
- The parser runs against the entire window.
- If a match is found, the window is **flushed** to prevent double-triggering.
- If no match is found, the window continues to roll.

---

## 8. User Interface — Professional Dashboard

### 8.1 Layout Philosophy

The UI is designed for the **broadcast control room**. It is a dashboard, not a consumer app. Information density is high. Keyboard shortcuts are essential.

### 8.2 Main Window Layout

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│  HV-Bible Desktop — Broadcast Edition                              [─][□][×]      │
├─────────────────────────────────────────────────────────────────────────────────────┤
│  File  View  Audio  Bible  Tools  Help                                              │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                     │
│  ┌─────────────────────┐  ┌──────────────────────────────────┐  ┌─────────────────┐│
│  │   AUDIO CONTROL     │  │        VERSE DISPLAY            │  │   HISTORY       ││
│  │                     │  │                                  │  │                 ││
│  │  🎤 Input: [Line 1] │  │     ┌────────────────────┐      │  │  Recent:        ││
│  │  📊 Level: ██████░  │  │     │                    │      │  │  • Rom 8:28 ✓  ││
│  │  🔇 Gain: [====o=]  │  │     │    John 3:16       │      │  │  • Gen 1:1  ✓  ││
│  │                     │  │     │                    │      │  │  • John 3:16 ✓ ││
│  │  [▶ Start] [⏸ Pause]│  │     │  For God so loved │      │  │                 ││
│  │                     │  │     │  the world that he │      │  │  Search:        ││
│  │  LED Status: 🟢     │  │     │  gave his only Son,│      │  │  [__________]   ││
│  │  [Hardware] [Screen]│  │     │  that whoever      │      │  │                 ││
│  │                     │  │     │  believes in him   │      │  │  Export:        ││
│  │  VAD: Active        │  │     │  should not perish │      │  │  [PDF] [TXT]    ││
│  │  SNR: 24 dB         │  │     │  but have eternal  │      │  │                 ││
│  │                     │  │     │  life.             │      │  │                 ││
│  │  ASR: Qwen3-0.6B    │  │     └────────────────────┘      │  │                 ││
│  │  Conf: 94%          │  │                                  │  │                 ││
│  └─────────────────────┘  └──────────────────────────────────┘  └─────────────────┘│
│                                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────────────────┐│
│  │  RAW TRANSCRIPT (rolling window)                                               ││
│  │  "... and we turn to Romans chapter 8 verse 28 ..."                           ││
│  └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                     │
│  Status: Active | Audio: 48kHz/16-bit | ASR: Qwen3-ASR-0.6B (GPU) | Memory: 1.2GB │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

### 8.3 Dual-Monitor Support

| Monitor | Content | Purpose |
|---------|---------|---------|
| **Monitor 1 (Control Panel)** | Full UI with controls, raw transcript, history, LED status | Media team's workspace |
| **Monitor 2 (Program Out)** | Fullscreen verse display. No UI. No overlays. | Sent to video switcher / broadcast |

The Program Out monitor is rendered in a **separate window** that can be positioned on the second display. It can also be output via NDI or captured directly by the video switcher.

### 8.4 Keyboard Shortcuts (Non-Negotiable)

| Shortcut | Action |
|----------|--------|
| `Space` | Approve detected verse (push to Program Out) |
| `Esc` | Reject detected verse |
| `Ctrl+Z` | Undo — revert to previous verse |
| `Ctrl+Shift+Z` | Redo |
| `Ctrl+F` | Search for verse |
| `Ctrl+1-9` | Quick-select from queue |
| `Enter` | Manual verse entry |
| `Ctrl+S` | Start/Stop listening |
| `Ctrl+P` | Toggle pause |
| `F11` | Toggle fullscreen (Program Out) |

---

## 9. Implementation Plan (High-Level)

### 9.1 Phase 0: Foundation (Weeks 1-3)

| Task | Deliverable |
|------|-------------|
| Set up Dioxus Desktop project with Rust backend | Working application shell |
| Implement CPAL audio capture with ASIO/CoreAudio | Audio input working |
| Implement basic UI framework | Empty but functional UI |
| Set up cross-platform build pipeline | Windows/macOS/Linux builds |

### 9.2 Phase 1: Audio Engine (Weeks 4-6)

| Task | Deliverable |
|------|-------------|
| Implement full audio pipeline (capture → VAD → ASR) | Audio pipeline working end-to-end |
| Integrate Qwen3-ASR (download model, inference) | ASR producing transcripts |
| Implement audio level metering and LED status | Visual feedback working |
| Add audio settings panel | User-configurable audio |

### 9.3 Phase 2: Bible Engine (Weeks 7-9)

| Task | Deliverable |
|------|-------------|
| Implement verse parser (state machine) | Parser passing all test cases |
| Integrate `bible-core` for lookup | Verse display working |
| Add translation management | Multiple translations selectable |
| Implement rolling window and deduplication | No double-triggering |

### 9.4 Phase 3: UI & Broadcast Features (Weeks 10-13)

| Task | Deliverable |
|------|-------------|
| Complete Control Panel UI | All UI components functional |
| Implement Program Out window | Clean output for broadcast |
| Add keyboard shortcuts | Full keyboard control |
| Implement logging and export | Timestamped service logs |

### 9.5 Phase 4: Polish & Production (Weeks 14-16)

| Task | Deliverable |
|------|-------------|
| Performance optimization | <200ms latency verified |
| Comprehensive testing | All edge cases covered |
| Installer packaging | .msi, .dmg, .deb/.rpm |
| Documentation | User manual + quick start |

---

## 10. System Requirements

### 10.1 Minimum Requirements

| Component | Specification |
|-----------|---------------|
| **OS** | Windows 10+, macOS 11+, Linux (Ubuntu 20.04+) |
| **CPU** | 4 cores, 2.5GHz+ |
| **RAM** | 8GB |
| **Storage** | 5GB available (for models + translations) |
| **GPU** | Integrated graphics (CPU fallback) |
| **Audio** | Professional audio interface or high-quality USB mic |

### 10.2 Recommended Requirements

| Component | Specification |
|-----------|---------------|
| **OS** | Windows 11, macOS 13+, Linux (Ubuntu 22.04+) |
| **CPU** | 8 cores, 3.0GHz+ (Apple M1+/AMD Ryzen 7+/Intel i7+) |
| **RAM** | 16GB |
| **Storage** | 10GB available (SSD) |
| **GPU** | NVIDIA RTX 3060+ (6GB+ VRAM) or Apple M-series (Metal) |
| **Audio** | Professional audio interface with ASIO (Windows) / CoreAudio (macOS) |

### 10.3 ASR Model Storage

| Model | Size | Download |
|-------|------|----------|
| Qwen3-ASR-0.6B | 1.7GB | First launch (auto-download) |
| Qwen3-ASR-1.7B | 4.5GB | Optional (user selects) |

---

## 11. Success Metrics

### 11.1 Technical Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **End-to-End Latency** | <200ms | High-resolution timer |
| **ASR Accuracy (Clean Audio)** | >95% | WER on test set |
| **ASR Accuracy (Noisy Audio)** | >85% | WER on test set |
| **False Positive Rate** | <1% | Verse detected when none spoken |
| **False Negative Rate** | <5% | Verse missed when spoken |
| **Memory Footprint** | <2GB | With ASR model loaded |
| **CPU Usage** | <30% average | During active listening |
| **GPU Usage** | <50% | During ASR inference |

### 11.2 User Metrics

| Metric | Target |
|--------|--------|
| **User Satisfaction** | >4.5/5 |
| **Time to First Verse** | <5 minutes |
| **Setup Time** | <10 minutes |
| **Service Reliability** | 99.9% uptime |

---

## 12. Risk Register

### 12.1 Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **Qwen3-ASR accuracy in reverberant environments** | Medium | High | Use clean XLR feed. Deploy WhisperForge as fallback. |
| **GPU compatibility** | Medium | Medium | CPU fallback mode. Model selection (0.6B works on CPU). |
| **ASIO driver issues on Windows** | Medium | High | CPAL abstraction. WASAPI fallback. |
| **Model download failure** | Low | High | Retry logic. Resume support. Checksum verification. |

### 12.2 Operational Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **Media team mistakes verse** | Low | High | Manual override. Undo (Ctrl+Z). Visual verification. |
| **App crash during service** | Low | Critical | Watchdog thread. Auto-restart <150ms. Fallback graphic. |
| **Audio device disconnection** | Medium | High | Auto-reconnect. Visual alert. |

---

## 13. Technology Stack — Finalized

| Layer | Technology | Language |
|-------|------------|----------|
| **Audio Capture** | CPAL + ASIO (Win) / CoreAudio (macOS) | Rust |
| **VAD** | Silero VAD (pure Rust) | Rust |
| **ASR (Primary)** | Qwen3-ASR (via `qwen-asr` crate) | Rust |
| **ASR (Fallback)** | WhisperForge (via `whisperforge-core` crate) | Rust |
| **Bible Engine** | `bible-core` (phf + rkyv + mmap) | Rust |
| **Desktop Framework** | **Dioxus Desktop (WebView mode — NOT Blitz)** | Rust |
| **UI Frontend** | `rsx!` macros + HTML/CSS | Rust |
| **State Management** | Dioxus `use_state` + `crossbeam_channel` | Rust |
| **LED Control** | USB HID / Serial (custom Rust driver) | Rust |
| **Build System** | Cargo + Dioxus CLI | - |

---

## 14. What This Blueprint Delivers

| Requirement | Status |
|-------------|--------|
| **Professional audio pipeline** | ✅ CPAL + ASIO/CoreAudio |
| **Best-in-class ASR** | ✅ Qwen3-ASR (46× realtime, pure Rust) |
| **GPU acceleration** | ✅ Metal / CUDA / WGPU |
| **Zero-compromise model size** | ✅ 1.7GB+ model downloaded on first launch |
| **LED status system** | ✅ Hardware + software emulation |
| **Dual-monitor output** | ✅ Control Panel + Program Out |
| **Manual override** | ✅ Keyboard shortcuts + UI controls |
| **Service logging** | ✅ Timestamped transcript export |
| **Sub-200ms latency** | ✅ End-to-end optimization |
| **Cross-platform** | ✅ Windows, macOS, Linux |
| **Independent deployment** | ✅ No mobile dependencies |

---

## 15. Next Steps

1. **Review and approve** this blueprint.
2. **Allocate resources** — 2 Rust engineers, 1 QA engineer.
3. **Set up repository** — Dioxus project with CPAL + Qwen3-ASR integration.
4. **Begin Phase 0** — Foundation (weeks 1-3).

---

**This is the product we are building. No compromises. No shortcuts. Professional-grade, broadcast-ready, production-tested.**