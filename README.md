# HV-Bible Desktop — Complete Feature & Settings Specification

> **Document type:** Product Feature Tree  
> **Version:** 1.0 — Greenfield Specification  
> **Target ship date:** 2027  
> **Target users:** Professional broadcast teams, sound engineers, church media operators, stadium AV crews  
> **Platforms:** Windows 10/11, macOS 12+, Linux (Ubuntu 22.04+, Fedora 38+)  
> **Architecture:** Offline-first, on-device, zero cloud dependency on hot path

---

# 1. APPLICATION

## 1.1 General

### 1.1.1 Application Name
- Display name: "HV-Bible — Broadcast Engine"
- Internal identifier: `hv-bible-desktop`
- Scope: global
- Persistence: permanent (compiled in)

### 1.1.2 UI Language
- Values: English (US), English (UK), Spanish, Portuguese (BR), Portuguese (PT), French, German, Korean, Mandarin (Simplified), Mandarin (Traditional), Japanese, Arabic, Russian, Swahili, Hindi, Tagalog, Indonesian, Dutch, Italian, Polish, Romanian, Ukrainian, Afrikaans, Zulu, Yoruba, Amharic, Twi
- Default: English (US)
- Scope: global
- Persistence: permanent
- Behavior: UI strings reload without restart. Right-to-left layout auto-activates for Arabic.

### 1.1.3 Startup Behavior
- **Launch on system startup**
  - Values: on / off
  - Default: off
  - Scope: global
  - Persistence: permanent
- **Start minimized to tray**
  - Values: on / off
  - Default: off
  - Scope: global
  - Persistence: permanent
- **Auto-start last session**
  - Values: on / off
  - Default: off
  - Scope: global
  - Persistence: permanent
- **Restore window position**
  - Values: on / off
  - Default: on
  - Scope: global
  - Persistence: permanent
- **Restore window size**
  - Values: on / off
  - Default: on
  - Scope: global
  - Persistence: permanent
- **Restore workspace layout**
  - Values: on / off
  - Default: on
  - Scope: global
  - Persistence: permanent
- **Startup splash screen**
  - Values: show / hide
  - Default: show
  - Scope: global
  - Persistence: permanent
- **Startup self-test**
  - Values: full / quick / skip
  - Default: quick
  - Scope: global
  - Persistence: permanent
  - Note: full runs audio loopback test, model validation, translation checksum. Quick checks model presence only.
- **Pre-warm ASR model on launch**
  - Values: on / off
  - Default: on
  - Scope: global
  - Persistence: permanent
  - Note: loads model into memory during splash so first recognition is instant

### 1.1.4 Shutdown Behavior
- **Confirm on close**
  - Values: always / if session active / never
  - Default: if session active
  - Scope: global
  - Persistence: permanent
- **Auto-save session on close**
  - Values: on / off
  - Default: on
  - Scope: global
  - Persistence: permanent
- **Auto-export log on close**
  - Values: on / off
  - Default: off
  - Scope: global
  - Persistence: permanent
- **Close to tray instead of quit**
  - Values: on / off
  - Default: off
  - Scope: global
  - Persistence: permanent
- **Stop recording on close**
  - Values: on / off
  - Default: on
  - Scope: global
  - Persistence: permanent

### 1.1.5 System Tray
- **Show tray icon**
  - Values: on / off
  - Default: on
  - Scope: global
  - Persistence: permanent
- **Tray icon style**
  - Values: color / monochrome / adaptive
  - Default: adaptive
  - Scope: global
  - Persistence: permanent
- **Tray click behavior**
  - Values: show/hide window / open quick menu / do nothing
  - Default: show/hide window
  - Scope: global
  - Persistence: permanent
- **Tray notifications**
  - Values: all / errors only / none
  - Default: errors only
  - Scope: global
  - Persistence: permanent

### 1.1.6 Data Directories
- **Application data root**
  - Default (Windows): `%APPDATA%\HV-Bible`
  - Default (macOS): `~/Library/Application Support/HV-Bible`
  - Default (Linux): `~/.local/share/hv-bible`
  - Configurable: yes (via settings or `--data-dir` CLI flag)
  - Scope: global
  - Persistence: permanent
- **Session storage directory**
  - Default: `<data_root>/sessions`
  - Configurable: yes
  - Scope: global
  - Persistence: permanent
- **Recording storage directory**
  - Default: `<data_root>/recordings`
  - Configurable: yes
  - Scope: global
  - Persistence: permanent
- **Model storage directory**
  - Default: `<data_root>/models`
  - Configurable: yes
  - Scope: global
  - Persistence: permanent
- **Translation pack directory**
  - Default: `<data_root>/packs`
  - Configurable: yes
  - Scope: global
  - Persistence: permanent
- **Plugin directory**
  - Default: `<data_root>/plugins`
  - Configurable: yes
  - Scope: global
  - Persistence: permanent
- **Temporary / cache directory**
  - Default: `<data_root>/cache`
  - Max size: configurable, 100 MB – 10 GB
  - Default max: 1 GB
  - Auto-cleanup: on / off (default: on)
  - Scope: global
  - Persistence: permanent

### 1.1.7 Undo / Redo
- **Undo depth**
  - Values: 1 – 100
  - Default: 20
  - Scope: per-session
  - Persistence: ephemeral (cleared on session close)
- **Undo scope**
  - Covers: verse approvals, verse rejections, manual input, translation changes, log edits
  - Does NOT cover: audio settings, pipeline start/stop, broadcast state

---

## 1.2 Window & Appearance

### 1.2.1 Window Chrome
- **Title bar style**
  - Values: system native / custom (frameless) / hybrid
  - Default: custom (frameless)
  - Scope: global
  - Persistence: permanent
- **Window opacity**
  - Values: 50% – 100%
  - Default: 100%
  - Scope: global
  - Persistence: permanent
- **Always on top**
  - Values: on / off / when broadcasting
  - Default: off
  - Scope: global
  - Persistence: permanent
- **Multiple windows**
  - Program Out is a separate window
  - Confidence Monitor is a separate window
  - Detachable panels can become their own windows

### 1.2.2 Theme System
- **Active theme**
  - Values: Purple Graphite (Dark), Studio Light, AMOLED Black, High Contrast, Broadcast Red, Custom
  - Default: Purple Graphite (Dark)
  - Scope: global
  - Persistence: permanent
- **Dark mode**
  - Values: dark / light / follow system
  - Default: dark
  - Scope: global
  - Persistence: permanent
- **Accent color**
  - Values: preset palette (purple, blue, red, green, orange, teal, pink, gold) or custom hex
  - Default: #A778F2 (purple)
  - Scope: global
  - Persistence: permanent
- **Background color (base)**
  - Customizable: yes
  - Default: theme-dependent
  - Scope: global
  - Persistence: permanent
- **Background color (surface)**
  - Customizable: yes
  - Default: theme-dependent
- **Background color (raised)**
  - Customizable: yes
  - Default: theme-dependent
- **Background color (sunken)**
  - Customizable: yes
  - Default: theme-dependent
- **Border color (subtle)**
  - Customizable: yes
  - Default: theme-dependent
- **Border color (strong)**
  - Customizable: yes
  - Default: theme-dependent
- **Text color (primary)**
  - Customizable: yes
  - Default: theme-dependent
- **Text color (secondary)**
  - Customizable: yes
  - Default: theme-dependent
- **Text color (tertiary)**
  - Customizable: yes
  - Default: theme-dependent
- **Status color (success)**
  - Default: #78B178
- **Status color (warning)**
  - Default: #C89B5D
- **Status color (error)**
  - Default: #BE6372
- **Status color (info)**
  - Default: #7C92D6
- **Custom theme import/export**
  - Format: JSON theme file
  - Shareable: yes
- **Theme preview**
  - Live preview before applying
  - Revert if not confirmed within 15 seconds

### 1.2.3 UI Scaling
- **Scale factor**
  - Values: 75% / 100% / 125% / 150% / 175% / 200% / custom
  - Default: 100% (or system DPI)
  - Scope: global
  - Persistence: permanent
- **Follow system DPI**
  - Values: on / off
  - Default: on
  - Scope: global
  - Persistence: permanent
- **Per-monitor DPI awareness**
  - Values: on / off
  - Default: on
  - Scope: global
  - Persistence: permanent (Windows only)

### 1.2.4 Animations & Transitions
- **Enable animations**
  - Values: on / off
  - Default: on
  - Scope: global
  - Persistence: permanent
- **Animation speed**
  - Values: slow (0.5x) / normal (1x) / fast (2x) / instant
  - Default: normal
  - Scope: global
  - Persistence: permanent
- **Verse fade-in duration**
  - Values: 0 ms – 2000 ms
  - Default: 400 ms
  - Scope: global
  - Persistence: permanent
- **Panel transition style**
  - Values: slide / fade / instant
  - Default: fade
  - Scope: global
  - Persistence: permanent
- **Reduce motion (accessibility)**
  - Values: on / off
  - Default: off (follows system preference if available)
  - Scope: global
  - Persistence: permanent

---

# 2. AUDIO

## 2.1 Input Source

### 2.1.1 Device Selection
- **Driver type**
  - Windows: WASAPI (exclusive / shared) / ASIO / DirectSound (legacy)
  - macOS: CoreAudio
  - Linux: ALSA / PulseAudio / PipeWire / JACK
  - Default: WASAPI Shared (Windows), CoreAudio (macOS), PipeWire (Linux)
  - Scope: global
  - Persistence: permanent
- **Device list**
  - Auto-populated from OS audio subsystem
  - Refreshable manually or auto-refresh on device change
  - Per-device metadata displayed:
    - Device name (string)
    - Driver version (string)
    - Input channel count (integer)
    - Output channel count (integer)
    - Supported sample rates (list of integers)
    - Supported bit depths (list: 16, 24, 32f, 32i)
    - Current latency (ms, read-only)
    - Min achievable latency (ms, read-only)
    - Max latency (ms, read-only)
    - Connection status: connected / disconnected / busy / error
    - Exclusive mode available: yes / no
    - Device type: USB / built-in / virtual / network / Dante / AVB / Thunderbolt
- **Primary device**
  - Values: any listed device
  - Default: system default input
  - Scope: global
  - Persistence: permanent
- **Fallback device**
  - Values: any listed device / none / system default
  - Default: system default
  - Scope: global
  - Persistence: permanent
  - Behavior: if primary device becomes unavailable, automatically switch to fallback
- **Auto-select preferred device on startup**
  - Values: on / off
  - Default: on
  - Scope: global
  - Persistence: permanent
- **Device hot-plug behavior**
  - **Auto-reconnect on device return**
    - Values: on / off
    - Default: on
    - Scope: global
    - Persistence: permanent
  - **Reconnect timeout**
    - Values: 1 s – 60 s
    - Default: 5 s
    - Scope: global
    - Persistence: permanent
  - **Max reconnect attempts**
    - Values: 1 – 10 / unlimited
    - Default: 3
    - Scope: global
    - Persistence: permanent
  - **Notify operator on disconnect**
    - Values: visual alert / audible alert / both / none
    - Default: visual alert
    - Scope: global
    - Persistence: permanent
  - **Behavior during disconnect**
    - Values: pause session / continue with silence / stop session / switch to fallback
    - Default: switch to fallback
    - Scope: global
    - Persistence: permanent
- **Device profiles**
  - Save current device config as named profile
  - Load profile on device connect (auto-match by device name)
  - Export profile as JSON
  - Import profile from JSON
  - Scope: per-device
  - Persistence: permanent

### 2.1.2 Format
- **Sample rate**
  - Values: 8000 / 16000 / 22050 / 44100 / 48000 / 88200 / 96000 / 176400 / 192000 / custom
  - Default: 48000
  - Scope: per-device
  - Persistence: permanent
  - Note: internal ASR processing always resamples to 16000 Hz. Higher input rates preserve monitoring fidelity.
- **Bit depth**
  - Values: 16-bit integer / 24-bit integer / 32-bit float / 32-bit integer
  - Default: 24-bit integer
  - Scope: per-device
  - Persistence: permanent
- **Buffer size**
  - Values: 32 / 64 / 128 / 256 / 512 / 1024 / 2048 / 4096 / custom
  - Default: 256
  - Unit: samples
  - Scope: per-device
  - Persistence: permanent
  - Display: show equivalent latency in ms (e.g., "256 samples = 5.3 ms @ 48kHz")
- **Exclusive mode (WASAPI / CoreAudio)**
  - Values: on / off
  - Default: off
  - Scope: per-device
  - Persistence: permanent
  - Note: exclusive mode gives lower latency but locks the device from other applications

### 2.1.3 Channel Routing
- **Input channel selection**
  - For stereo devices: left / right / both (mono mix)
  - For multichannel devices: per-channel select (1–64)
  - Default: both (mono mix)
  - Scope: per-device
  - Persistence: permanent
- **Mono downmix mode**
  - Values: L+R average / L only / R only / custom weighted mix
  - Default: L+R average
  - Scope: per-device
  - Persistence: permanent
- **Custom mix weights**
  - Per-channel weight: -1.0 to +1.0
  - Default: all 1.0
  - Scope: per-device
  - Persistence: permanent
- **Phase per channel**
  - Values: normal / inverted
  - Default: normal
  - Scope: per-channel, per-device
  - Persistence: permanent
- **Channel swap**
  - Values: on / off
  - Default: off
  - Scope: per-device
  - Persistence: permanent
- **Channel delay (per-channel)**
  - Values: 0 ms – 100 ms, 0.01 ms steps
  - Default: 0 ms
  - Scope: per-channel, per-device
  - Persistence: permanent
  - Use case: compensate for physical routing delays in large venues

### 2.1.4 Gain Staging
- **Input gain**
  - Range: -60 dB to +24 dB
  - Step: 0.1 dB
  - Default: 0.0 dB
  - Scope: per-device
  - Persistence: permanent
- **Digital trim**
  - Range: -24 dB to +24 dB
  - Step: 0.1 dB
  - Default: 0.0 dB
  - Scope: per-device
  - Persistence: permanent
- **Pad (analog simulation)**
  - Values: off / -10 dB / -20 dB / -30 dB
  - Default: off
  - Scope: per-device
  - Persistence: permanent
- **DC offset removal**
  - Values: on / off
  - Default: on
  - Scope: per-device
  - Persistence: permanent

### 2.1.5 Auto Gain Control (AGC)
- **Enable AGC**
  - Values: on / off
  - Default: off
  - Scope: per-session
  - Persistence: per-session (saved with session)
- **Target level**
  - Range: -30 dBFS to -6 dBFS
  - Step: 1 dB
  - Default: -18 dBFS
  - Scope: per-session
  - Persistence: per-session
- **Attack time**
  - Range: 1 ms – 500 ms
  - Default: 10 ms
  - Scope: per-session
  - Persistence: per-session
- **Release time**
  - Range: 10 ms – 5000 ms
  - Default: 200 ms
  - Scope: per-session
  - Persistence: per-session
- **Max gain limit**
  - Range: 0 dB – 40 dB
  - Default: 20 dB
  - Scope: per-session
  - Persistence: per-session
- **Lookahead**
  - Range: 0 ms – 20 ms
  - Default: 5 ms
  - Scope: per-session
  - Persistence: per-session

### 2.1.6 High-Pass Filter
- **Enable HPF**
  - Values: on / off
  - Default: on
  - Scope: per-device
  - Persistence: permanent
- **Frequency**
  - Values: 20 / 40 / 60 / 80 / 100 / 120 / 150 / 200 / custom Hz
  - Default: 80 Hz
  - Scope: per-device
  - Persistence: permanent
- **Slope**
  - Values: 6 / 12 / 18 / 24 / 48 dB/octave
  - Default: 12 dB/octave
  - Scope: per-device
  - Persistence: permanent
- **Filter type**
  - Values: Butterworth / Bessel / Linkwitz-Riley / Chebyshev
  - Default: Butterworth
  - Scope: per-device
  - Persistence: permanent

### 2.1.7 Low-Pass Filter (Anti-aliasing)
- **Enable LPF**
  - Values: on / off
  - Default: off
  - Scope: per-device
  - Persistence: permanent
- **Frequency**
  - Values: 4000 / 6000 / 8000 / 10000 / 12000 / 16000 / 20000 / custom Hz
  - Default: 8000 Hz (speech band)
  - Scope: per-device
  - Persistence: permanent
- **Slope**
  - Values: 6 / 12 / 24 / 48 dB/octave
  - Default: 24 dB/octave

### 2.1.8 Noise Gate
- **Enable noise gate**
  - Values: on / off
  - Default: off
  - Scope: per-session
  - Persistence: per-session
- **Threshold**
  - Range: -80 dBFS to -10 dBFS
  - Default: -45 dBFS
  - Scope: per-session
  - Persistence: per-session
- **Attack**
  - Range: 0.1 ms – 50 ms
  - Default: 1 ms
- **Hold**
  - Range: 0 ms – 500 ms
  - Default: 50 ms
- **Release**
  - Range: 5 ms – 2000 ms
  - Default: 100 ms
- **Range (depth)**
  - Range: -80 dB to 0 dB
  - Default: -80 dB (full gate)

### 2.1.9 Compressor
- **Enable compressor**
  - Values: on / off
  - Default: off
  - Scope: per-session
  - Persistence: per-session
- **Threshold**
  - Range: -60 dBFS to 0 dBFS
  - Default: -20 dBFS
- **Ratio**
  - Values: 1:1 / 2:1 / 3:1 / 4:1 / 6:1 / 8:1 / 10:1 / 20:1 / ∞:1
  - Default: 3:1
- **Attack**
  - Range: 0.1 ms – 200 ms
  - Default: 5 ms
- **Release**
  - Range: 10 ms – 2000 ms
  - Default: 100 ms
- **Knee**
  - Values: hard / soft (0–20 dB)
  - Default: soft 6 dB
- **Makeup gain**
  - Range: 0 dB – 24 dB
  - Default: 0 dB (auto-makeup available)
- **Auto-makeup gain**
  - Values: on / off
  - Default: off
- **Sidechain filter**
  - Enable: on / off
  - HPF frequency: 60 – 600 Hz
  - Default: 150 Hz
  - Purpose: prevent low rumble from triggering compression

### 2.1.10 De-esser
- **Enable de-esser**
  - Values: on / off
  - Default: off
- **Frequency range**
  - Range: 2000 Hz – 12000 Hz
  - Default: 5000 – 8000 Hz
- **Threshold**
  - Range: -40 dBFS to 0 dBFS
  - Default: -20 dBFS
- **Reduction amount**
  - Range: 0 dB – 20 dB
  - Default: 6 dB

### 2.1.11 Acoustic Echo Cancellation (AEC)
- **Enable AEC**
  - Values: on / off
  - Default: on
  - Scope: per-session
  - Persistence: per-session
- **Strength**
  - Values: off / low / medium / high / aggressive
  - Default: medium
- **Reference signal source**
  - Values: none / loopback / specific output device
  - Default: none
- **Tail length**
  - Range: 50 ms – 500 ms
  - Default: 150 ms

---

## 2.2 Metering

### 2.2.1 VU Meter
- **Display style**
  - Values: horizontal bar / vertical bar / analog needle / numeric only
  - Default: horizontal bar
  - Scope: global
  - Persistence: permanent
- **Meter ballistics**
  - Values: VU (300 ms integration) / PPM (5 ms attack, 1.7 s release) / True Peak / K-weighted
  - Default: PPM
  - Scope: global
  - Persistence: permanent
- **Scale**
  - Values: -48 to 0 dBFS / -60 to 0 dBFS / -96 to 0 dBFS / custom
  - Default: -48 to 0 dBFS
  - Scope: global
  - Persistence: permanent
- **Color gradient**
  - Green zone: configurable threshold (default: below -12 dBFS)
  - Yellow zone: configurable threshold (default: -12 to -3 dBFS)
  - Red zone: configurable threshold (default: above -3 dBFS)
  - Scope: global
  - Persistence: permanent

### 2.2.2 Peak Hold
- **Enable peak hold**
  - Values: on / off
  - Default: on
- **Hold duration**
  - Values: 1 s / 2 s / 3 s / 5 s / 10 s / infinite (manual reset)
  - Default: 3 s
- **Peak indicator style**
  - Values: line / dot / segment
  - Default: line

### 2.2.3 Clip Indicator
- **Enable clip indicator**
  - Values: on / off
  - Default: on
- **Clip threshold**
  - Values: 0 dBFS / -0.1 dBFS / -0.3 dBFS / -1 dBFS / custom
  - Default: -0.1 dBFS
- **Clip hold time**
  - Values: 1 s / 3 s / 5 s / 10 s / until manual clear
  - Default: 3 s
- **Clip count display**
  - Values: show / hide
  - Default: show

### 2.2.4 Loudness Metering (EBU R128 / ATSC A/85)
- **Enable LUFS meter**
  - Values: on / off
  - Default: off
  - Note: useful for broadcast compliance
- **Standard**
  - Values: EBU R128 / ATSC A/85 / custom
  - Default: EBU R128
- **Target loudness**
  - Range: -30 LUFS to -10 LUFS
  - Default: -23 LUFS (EBU) or -24 LKFS (ATSC)
- **Integration time**
  - Values: momentary (400 ms) / short-term (3 s) / integrated (full session)
  - Default: short-term
- **Loudness range display**
  - Values: show / hide
  - Default: show

### 2.2.5 SNR Display
- **Show signal-to-noise ratio**
  - Values: on / off
  - Default: on
- **SNR calculation method**
  - Values: RMS-based / A-weighted / ITU-R 468
  - Default: RMS-based
- **Noise floor reference**
  - Values: measured at session start / fixed -60 dBFS / auto-tracking
  - Default: auto-tracking

### 2.2.6 Spectrum Analyzer (optional)
- **Enable spectrum display**
  - Values: on / off
  - Default: off
- **FFT size**
  - Values: 256 / 512 / 1024 / 2048 / 4096 / 8192
  - Default: 2048
- **Window function**
  - Values: Hann / Hamming / Blackman / Flat-top
  - Default: Hann
- **Display range**
  - Frequency: 20 Hz – 20 kHz
  - Amplitude: -96 dBFS to 0 dBFS
- **Averaging**
  - Values: off / 2x / 4x / 8x
  - Default: 4x

---

## 2.3 Monitoring

### 2.3.1 Monitor Output
- **Monitor output device**
  - Values: any available output device / system default / same as input device
  - Default: system default
  - Scope: global
  - Persistence: permanent
- **Monitor volume**
  - Range: -∞ (mute) to +12 dB
  - Default: 0 dB
  - Scope: global
  - Persistence: permanent
- **Monitor mute**
  - Values: on / off
  - Default: off
  - Scope: per-session
  - Persistence: ephemeral
- **Monitor solo**
  - Values: on / off
  - Default: off
  - Scope: per-session
  - Persistence: ephemeral
  - Note: solo isolates the raw input signal, bypassing all processing

### 2.3.2 Monitor Mix
- **Source selection**
  - Values: raw input / post-processing / post-gain / ASR-filtered
  - Default: post-processing
- **Test tone generator**
  - Frequencies: 100 / 440 / 1000 / 5000 / 10000 Hz / sweep
  - Level: -20 dBFS
  - Duration: continuous until stopped
- **Talkback**
  - Values: on / off
  - Default: off
  - Note: routes the operator's microphone to the monitor mix for communication

### 2.3.3 Headphone Output
- **Headphone device**
  - Values: any output device / same as monitor
  - Default: same as monitor
- **Headphone volume (independent)**
  - Range: -∞ to +12 dB
  - Default: 0 dB
- **Headphone mix**
  - Values: same as monitor / independent
  - Default: same as monitor

---

## 2.4 Recording

### 2.4.1 Audio Recording
- **Enable recording**
  - Values: on / off
  - Default: off
  - Scope: per-session
  - Persistence: per-session
- **Auto-record on session start**
  - Values: on / off
  - Default: off
  - Scope: global
  - Persistence: permanent
- **Recording format**
  - Values: WAV 16-bit / WAV 24-bit / WAV 32-bit float / FLAC / OGG Vorbis / MP3 (if licensed)
  - Default: WAV 24-bit
  - Scope: global
  - Persistence: permanent
- **Recording sample rate**
  - Values: same as input / 16000 / 44100 / 48000 / 96000
  - Default: same as input
- **Recording channels**
  - Values: mono (processed) / stereo (raw) / multi-track
  - Default: mono (processed)
- **Recording source**
  - Values: raw input / post-processing / post-ASR (with markers) / all (multi-track)
  - Default: post-processing
- **File naming convention**
  - Values: date-time / session name + date / custom pattern
  - Pattern variables: `{date}`, `{time}`, `{session}`, `{device}`, `{counter}`
  - Default: `{date}_{time}_{session}`
- **Max file size**
  - Values: unlimited / 650 MB / 2 GB / 4 GB / custom
  - Default: 2 GB
  - Behavior on limit: auto-split to new file
- **Max recording duration**
  - Values: unlimited / 1 hr / 2 hr / 4 hr / 8 hr / custom
  - Default: unlimited
- **Disk space warning threshold**
  - Values: 500 MB / 1 GB / 5 GB / 10 GB / custom
  - Default: 1 GB
  - Behavior: yellow warning at threshold, red warning at 500 MB, auto-stop at 100 MB

### 2.4.2 Marker System (within recordings)
- **Auto-mark verse detections**
  - Values: on / off
  - Default: on
- **Auto-mark session events**
  - Values: on / off (marks start, stop, pause, errors)
  - Default: on
- **Manual marker hotkey**
  - Default: M
  - Configurable: yes
- **Marker export format**
  - Values: CSV / JSON / cue sheet / Adobe Audition markers / Reaper markers
  - Default: CSV

---

# 3. ASR (AUTOMATIC SPEECH RECOGNITION)

## 3.1 Engine Selection

### 3.1.1 ASR Backend
- **Engine**
  - Values: Sherpa-ONNX (Zipformer) / Whisper.cpp / Vosk / Custom ONNX model
  - Default: Sherpa-ONNX (Zipformer)
  - Scope: global
  - Persistence: permanent
  - Note: all engines run 100% on-device, no cloud API

### 3.1.2 Model Selection
- **Model size**
  - Values: tiny (20 MB) / small (100 MB) / medium (300 MB) / large (500 MB) / extra-large (1 GB+)
  - Default: large
  - Scope: global
  - Persistence: permanent
- **Model variant**
  - Values: streaming (lower latency) / batch (higher accuracy)
  - Default: streaming
- **Model language**
  - Values: English / Spanish / Portuguese / French / German / Korean / Mandarin / Arabic / Multi-lingual
  - Default: English
  - Scope: global
  - Persistence: permanent
- **Model download manager**
  - List available models from registry
  - Download progress indicator
  - Checksum verification after download (SHA-256)
  - Resume interrupted downloads
  - Delete unused models
  - Model file integrity check on startup: on / off (default: on)
- **Custom model import**
  - Accept: ONNX encoder, decoder, joiner, tokens.txt
  - Validate model format before import
  - Name and version custom models

### 3.1.3 Hardware Acceleration
- **Compute device**
  - Values: CPU / GPU (auto-detect) / specific GPU
  - Default: CPU
  - Scope: global
  - Persistence: permanent
- **GPU provider**
  - Windows: DirectML / CUDA (if NVIDIA GPU)
  - macOS: CoreML / Metal
  - Linux: CUDA / ROCm / Vulkan
  - Default: auto-detect best available
- **Number of CPU threads**
  - Range: 1 – (system thread count)
  - Default: 4
  - Scope: global
  - Persistence: permanent
- **Thread priority**
  - Values: normal / above normal / high / real-time
  - Default: above normal
  - Scope: global
  - Persistence: permanent

## 3.2 Recognition Settings

### 3.2.1 Streaming Parameters
- **Chunk duration**
  - Range: 10 ms – 100 ms
  - Default: 30 ms
  - Note: smaller = lower latency but higher CPU
- **Endpoint detection**
  - **Enable endpoint detection**
    - Values: on / off
    - Default: on
  - **Silence duration for endpoint**
    - Range: 200 ms – 5000 ms
    - Default: 1500 ms
  - **Trailing silence rule**
    - Values: strict (long pause = end) / relaxed (allow natural pauses)
    - Default: relaxed
- **Max utterance length**
  - Range: 10 s – 300 s
  - Default: 60 s
  - Behavior on limit: force endpoint and reset

### 3.2.2 Language Model Bias
- **Bible vocabulary boost**
  - Values: off / low / medium / high / aggressive
  - Default: high
  - Note: boosts recognition probability for Bible book names, numbers, and verse-related words
- **Custom vocabulary**
  - Add custom words/phrases the ASR should recognize
  - Example: pastor's name, church name, recurring phrases
  - Scope: per-session or global
- **Number recognition mode**
  - Values: cardinal only / ordinal only / both / contextual
  - Default: contextual
  - Note: "three sixteen" vs "316" vs "third"

### 3.2.3 Text Normalization
- **Capitalization**
  - Values: as-recognized / sentence case / lowercase / uppercase
  - Default: lowercase (for parser compatibility)
- **Punctuation insertion**
  - Values: on / off
  - Default: off
  - Note: punctuation is noise for verse detection, but useful for transcript display
- **Number format**
  - Values: words ("three sixteen") / digits ("3:16") / both
  - Default: words (raw ASR output)
- **Profanity filter**
  - Values: on / off
  - Default: off
  - Note: a broadcast environment should not filter; the operator decides

## 3.3 Confidence & Rejection

### 3.3.1 Global Confidence Threshold
- **Minimum confidence to display**
  - Range: 0.0 – 1.0
  - Default: 0.50
  - Scope: global
  - Persistence: permanent

### 3.3.2 Per-Domain Thresholds
- **Clean audio threshold**
  - Range: 0.0 – 1.0
  - Default: 0.50
- **Noisy environment threshold**
  - Range: 0.0 – 1.0
  - Default: 0.65
- **Crowd noise threshold**
  - Range: 0.0 – 1.0
  - Default: 0.75
- **Reverberant room threshold**
  - Range: 0.0 – 1.0
  - Default: 0.70
- **Domain detection**
  - Values: auto (analyze noise floor) / manual select
  - Default: auto

### 3.3.3 Per-Book Thresholds
- Per Bible book confidence adjustment: -0.2 to +0.2
- Default: 0.0 for all books
- Use case: some books (e.g., Philemon vs Philippians) are harder to distinguish
- Scope: global
- Persistence: permanent

### 3.3.4 Confidence Display
- **Show confidence score to operator**
  - Values: on / off
  - Default: on
- **Confidence format**
  - Values: percentage / decimal / bar / color band
  - Default: percentage
- **Color-code confidence bands**
  - High (>0.90): green
  - Medium (0.70–0.90): yellow
  - Low (<0.70): red
  - Thresholds configurable
- **Show alternative interpretations**
  - Values: off / top-2 / top-3 / top-5
  - Default: top-3
  - Note: shows alternate possible verses the ASR might have meant

### 3.3.5 Rejection Behavior
- **Below-threshold action**
  - Values: silent reject / show as low-confidence / flag for manual review / log only
  - Default: show as low-confidence
- **Rejected transcript visibility**
  - Values: show in transcript (dimmed) / hide from transcript / separate rejected log
  - Default: show in transcript (dimmed)

### 3.3.6 Ambiguity Handling
- **When two verses match with similar confidence**
  - Values: ask operator / prefer higher confidence / prefer more common book / defer to context / show both
  - Default: ask operator
- **Context window influence**
  - Values: on / off
  - Default: on
  - Note: if the pastor was in John, prefer John references over Joel

### 3.3.7 Negative Learning
- **Operator false-positive marking**
  - Values: on / off
  - Default: on
- **System remembers marked patterns**
  - Values: per-session / permanent
  - Default: per-session
- **Threshold adapts per session**
  - Values: on / off
  - Default: on
- **Export/import learned patterns**
  - Format: JSON
  - Scope: global
  - Persistence: permanent

---

## 3.4 Voice Activity Detection (VAD)

### 3.4.1 VAD Engine
- **VAD type**
  - Values: Energy-based / Silero VAD (neural) / WebRTC VAD
  - Default: Energy-based
  - Scope: global
  - Persistence: permanent

### 3.4.2 VAD Parameters
- **Energy threshold**
  - Range: 100 – 10000
  - Default: 1000
  - Scope: per-session
  - Persistence: per-session
- **Silence timeout**
  - Range: 200 ms – 5000 ms
  - Default: 1500 ms
- **Minimum speech duration**
  - Range: 50 ms – 1000 ms
  - Default: 250 ms
- **Speech padding**
  - Range: 0 ms – 500 ms
  - Default: 100 ms
  - Note: how much audio before/after detected speech to include

### 3.4.3 VAD Display
- **Show VAD state in UI**
  - Values: on / off
  - Default: on
- **VAD indicator style**
  - Values: LED / waveform highlight / text label
  - Default: LED

---

# 4. BIBLE

## 4.1 Translations

### 4.1.1 Translation Management
- **Installed translations**
  - Pre-bundled: KJV (public domain)
  - Downloadable: ASV, WEB, YLT, Darby, BBE, AKJV (all public domain)
  - Licensed (requires activation): ESV, NIV, NASB, NKJV, NLT, CSB, MSG, AMP, CEV, GNT, HCSB, RSV, NRSV
  - Format: `.bible.bin` + `.offsets.bin` binary packs
- **Active translation**
  - Values: any installed translation
  - Default: KJV
  - Scope: per-session
  - Persistence: per-session (saved with session, global default in settings)
- **Secondary translation (parallel display)**
  - Values: none / any installed translation
  - Default: none
  - Scope: per-session
  - Persistence: per-session
- **Tertiary translation**
  - Values: none / any installed translation
  - Default: none
- **Translation download manager**
  - Download from pack repository
  - Verify checksum (BLAKE3)
  - Show download progress
  - Resume interrupted downloads
  - Scope: global

### 4.1.2 Per-Translation Settings
- **Display name override**
  - Example: rename "KJV" to "King James Version" in UI
  - Scope: per-translation
  - Persistence: permanent
- **Copyright notice**
  - Displayed in footer when translation is shown on broadcast output
  - Configurable text per translation
  - Auto-populated from translation metadata
- **Red letter edition**
  - Values: on / off (highlight words of Jesus in red)
  - Default: off
  - Scope: per-translation
  - Persistence: permanent
- **Verse numbering style**
  - Values: superscript / inline / hidden
  - Default: superscript
  - Scope: per-translation

## 4.2 Versification

### 4.2.1 Canon
- **Canon system**
  - Values: Protestant 66-book / Catholic 73-book / Orthodox 78-book / Ethiopian 81-book
  - Default: Protestant 66-book
  - Scope: global
  - Persistence: permanent
- **Apocrypha / Deuterocanonical support**
  - Values: include / exclude
  - Default: exclude
  - Scope: global
  - Persistence: permanent

### 4.2.2 Book Name Aliases
- **Built-in aliases**
  - Full names: "Genesis", "Exodus", etc.
  - Abbreviations: "Gen", "Exo", "Lev", etc.
  - Spoken forms: "First Corinthians", "Second Kings", etc.
  - Informal: "Psalms" / "Psalm", "Song of Solomon" / "Song of Songs"
- **Custom aliases**
  - Add per-book aliases
  - Example: add "SOS" for "Song of Solomon"
  - Scope: global
  - Persistence: permanent
- **Language-specific aliases**
  - Spanish: "Génesis", "Éxodo", "Juan", etc.
  - Portuguese: "Gênesis", "Êxodo", "João", etc.
  - Loaded based on ASR language setting

### 4.2.3 Number Word Recognition
- **Cardinal numbers**: one through one hundred
- **Ordinal numbers**: first through one hundredth
- **Compound numbers**: twenty-three, thirty-one, etc.
- **Spoken separators**: "chapter", "verse", "and", "through"
- **Range detection**: "John 3:16-18" → display verses 16, 17, 18 or just 16
  - Values: first verse only / all verses in range / ask operator
  - Default: first verse only

## 4.3 Verse Detection (Parser)

### 4.3.1 Parser Settings
- **Rolling word window size**
  - Range: 5 – 50 words
  - Default: 20 words
  - Scope: global
  - Persistence: permanent
- **Debounce after match**
  - Range: 500 ms – 10000 ms
  - Default: 3000 ms
  - Scope: global
  - Persistence: permanent
  - Note: prevents the same verse from being detected repeatedly
- **Cooldown per verse**
  - Range: 0 s – 60 s
  - Default: 10 s
  - Note: once "John 3:16" is detected, don't detect it again for N seconds
- **Verse range detection**
  - Values: on / off
  - Default: off
  - Note: "John 3:16 through 18" detected as a range
- **Multi-verse detection**
  - Values: on / off
  - Default: on
  - Note: "Romans 8:28 and John 3:16" detected as two separate verses

### 4.3.2 Context Intelligence
- **Context tracking**
  - Values: on / off
  - Default: on
  - Note: if the speaker is in "John chapter 3", a subsequent "verse 17" should resolve to John 3:17
- **Context decay time**
  - Range: 10 s – 300 s
  - Default: 60 s
  - Note: how long to remember the last book/chapter context
- **Book preference weighting**
  - Values: equal / prefer common books / prefer current context
  - Default: prefer current context

## 4.4 Verse Display

### 4.4.1 Display Formatting
- **Reference format**
  - Values: "John 3:16" / "Jn 3:16" / "John 3.16" / "John III:16" / custom pattern
  - Default: "John 3:16"
  - Scope: global
  - Persistence: permanent
- **Text wrapping**
  - Values: word wrap / character wrap / no wrap (scroll)
  - Default: word wrap
- **Verse text alignment**
  - Values: left / center / right / justified
  - Default: center (broadcast output), left (operator view)
- **Show translation label**
  - Values: always / on hover / never
  - Default: always
- **Show copyright notice**
  - Values: always / on licensed translations only / never
  - Default: on licensed translations only

### 4.4.2 Manual Verse Input
- **Manual input field**
  - Accepts: typed reference (e.g., "John 3:16")
  - Parser: same as ASR parser
  - Autocomplete: on / off (default: on)
  - Autocomplete source: all books → chapters → verses
  - Hotkey to focus: Ctrl+F
- **Search by text**
  - Full-text search across active translation
  - Results ranked by relevance
  - Scope: active translation(s)

### 4.4.3 Verse History
- **History depth**
  - Range: 10 – 500
  - Default: 100
  - Scope: per-session
  - Persistence: per-session
- **History display**
  - Values: sidebar list / dropdown / timeline
  - Default: sidebar list
- **Navigate history**
  - Previous: Ctrl+Z or ←
  - Next: Ctrl+Y or →
- **Pin verse in history**
  - Values: on / off
  - Default: off
  - Note: pinned verses remain at the top of the history list

---

# 5. BROADCAST

## 5.1 Output Modes

### 5.1.1 Program Output Window
- **Enable program output**
  - Values: on / off
  - Default: off
  - Scope: per-session
  - Persistence: per-session
- **Output display**
  - Values: second monitor (fullscreen) / windowed / borderless
  - Default: second monitor (fullscreen)
- **Output monitor selection**
  - Values: any connected monitor
  - Default: secondary monitor (if available)
- **Output resolution**
  - Values: match monitor / 1920×1080 / 3840×2160 / custom
  - Default: match monitor
- **Background**
  - Values: solid color / gradient / image / transparent / video loop
  - Default: solid black
  - Custom: any hex color
- **Transition style (verse in)**
  - Values: cut / fade / slide up / slide down / slide left / slide right / dissolve / zoom / typewriter
  - Default: fade
  - Duration: 0 ms – 2000 ms (default: 400 ms)
- **Transition style (verse out)**
  - Same options as verse in
  - Default: fade
  - Duration: 0 ms – 2000 ms (default: 400 ms)
- **Blank output hotkey**
  - Default: B
  - Note: immediately clears the output to background

### 5.1.2 Confidence Monitor
- **Enable confidence monitor**
  - Values: on / off
  - Default: off
- **Confidence monitor content**
  - Shows: next verse (preview), current verse, confidence score, ASR transcript, operator notes
  - Layout: configurable

### 5.1.3 NDI Output
- **Enable NDI output**
  - Values: on / off
  - Default: off
- **NDI stream name**
  - Default: "HV-Bible"
  - Configurable: yes
- **NDI resolution**
  - Values: 1920×1080 / 3840×2160 / 1280×720 / custom
  - Default: 1920×1080
- **NDI frame rate**
  - Values: 24 / 25 / 29.97 / 30 / 50 / 59.94 / 60
  - Default: 30
- **NDI alpha channel**
  - Values: on / off
  - Default: on
  - Note: enables overlay compositing in vMix/OBS
- **NDI groups**
  - Values: public / custom group name
  - Default: public
- **NDI failover source**
  - Values: none / black / last frame / specific source
  - Default: black

### 5.1.4 Spout / Syphon Output
- **Enable Spout (Windows) / Syphon (macOS)**
  - Values: on / off
  - Default: off
- **Texture name**
  - Default: "HV-Bible Output"
- **Resolution**
  - Same options as NDI

### 5.1.5 Virtual Camera Output
- **Enable virtual camera**
  - Values: on / off
  - Default: off
- **Virtual camera driver**
  - Windows: OBS Virtual Camera protocol
  - macOS: CoreMediaIO
  - Linux: v4l2loopback
- **Resolution**
  - Values: 1920×1080 / 1280×720 / 640×480
  - Default: 1920×1080

### 5.1.6 HTML/CSS Overlay Output (Browser Source)
- **Enable local HTTP overlay server**
  - Values: on / off
  - Default: off
- **Server port**
  - Range: 1024 – 65535
  - Default: 8765
- **Overlay URL**
  - Auto-generated: `http://localhost:8765/overlay`
- **Overlay template**
  - Values: default / custom HTML/CSS/JS
  - Custom template directory
- **WebSocket push**
  - Real-time verse updates via WebSocket to connected browser sources

## 5.2 Output Typography

### 5.2.1 Verse Reference Font
- **Font family**
  - Values: system fonts + bundled fonts (Inter, Roboto, Outfit, Georgia, Garamond, Playfair Display, Merriweather, Lora)
  - Default: Inter
  - Scope: per-output
  - Persistence: permanent
- **Font size**
  - Range: 12 pt – 200 pt
  - Default: 48 pt (program output), 32 pt (operator view)
- **Font weight**
  - Values: light / regular / medium / semi-bold / bold / extra-bold
  - Default: bold
- **Font color**
  - Default: white (dark background) / black (light background)
  - Custom: any hex color
- **Text shadow**
  - Enable: on / off
  - Default: on
  - Color: black 50% opacity
  - Offset: 2px 2px
  - Blur: 4px
- **Text outline (stroke)**
  - Enable: on / off
  - Default: off
  - Width: 0.5 – 5 px
  - Color: black

### 5.2.2 Verse Text Font
- Same options as reference font, independently configurable
- **Font family default**: Georgia (serif for readability)
- **Font size default**: 28 pt (program output), 22 pt (operator view)
- **Font weight default**: regular
- **Line height**
  - Range: 1.0 – 3.0
  - Default: 1.6
- **Letter spacing**
  - Range: -2 px to +5 px
  - Default: 0 px
- **Max lines before scroll**
  - Range: 1 – 20
  - Default: 8

### 5.2.3 Translation Label Font
- Same options, smaller defaults
- **Font size default**: 14 pt
- **Font color default**: 60% opacity white

### 5.2.4 Copyright Notice Font
- **Font size**: 10 pt – 16 pt (default: 11 pt)
- **Position**: bottom-left / bottom-center / bottom-right
- **Default**: bottom-right
- **Visibility**: always / 3 seconds after verse appears / never

## 5.3 Broadcast Delay & Safety

### 5.3.1 Broadcast Delay
- **Output delay**
  - Range: 0 ms – 10000 ms
  - Default: 0 ms
  - Scope: per-output
  - Persistence: permanent
  - Use case: match video delay in broadcast chain

### 5.3.2 Confidence Gate
- **Minimum confidence to push to broadcast**
  - Range: 0.0 – 1.0
  - Default: 0.85
  - Note: verses below this threshold are shown to operator but NOT pushed to program output
- **Operator approval required**
  - Values: always / only for low confidence / never
  - Default: never
  - Note: "always" means every verse must be approved with Space before it goes to program output

### 5.3.3 Fallback Content
- **When no verse is detected**
  - Values: blank / last verse / static image / custom text / logo
  - Default: blank
- **Auto-clear verse after timeout**
  - Values: on / off
  - Default: on
  - Timeout: 5 s – 300 s (default: 30 s)
- **Blank trigger**
  - Values: manual only / auto after timeout / on VAD silence
  - Default: auto after timeout

---

# 6. LED & VISUAL FEEDBACK

## 6.1 Software LED

### 6.1.1 LED Display
- **Show LED indicator**
  - Values: on / off
  - Default: on
- **LED size**
  - Range: 12 px – 48 px
  - Default: 28 px
- **LED style**
  - Values: circle (solid) / circle (glow) / square / diamond / bar
  - Default: circle (glow)

### 6.1.2 LED Color Mapping
- **Stopped / Standby**: red (blinking)
- **Listening / Armed**: yellow (steady)
- **Silence detected**: yellow (dim)
- **Speaking detected**: green (steady)
- **Verse detected**: blue (steady, 3 seconds)
- **Verse approved**: white (steady, 1 second)
- **Error**: red (steady)
- **Recording**: red dot overlay
- Each color customizable
- Scope: global
- Persistence: permanent

### 6.1.3 LED Blink Rate
- Range: 100 ms – 2000 ms
- Default: 400 ms
- Scope: global
- Persistence: permanent

### 6.1.4 LED Brightness
- Range: 10% – 100%
- Default: 80%
- Scope: global
- Persistence: permanent

## 6.2 Hardware LED (External)

### 6.2.1 USB LED Device Support
- **Protocol**: HID, serial (COM port), or custom USB
- **Supported devices**: BlinkStick, Luxafor, Busylight, custom Arduino
- **Device selection**: dropdown of detected devices
- **LED sync**: mirror software LED state to hardware
- **Independent patterns**: on / off
- Default: mirror software LED

### 6.2.2 DMX LED Output
- **Enable DMX control**
  - Values: on / off
  - Default: off
- **DMX universe**: 1 – 32767
- **DMX start channel**: 1 – 512
- **Channel mapping**: R/G/B or RGBW or single intensity
- **DMX interface**: Art-Net / sACN / USB-DMX adapter

## 6.3 Screen Flash
- **Flash screen on verse detection**
  - Values: on / off
  - Default: off
- **Flash color**: green (configurable)
- **Flash duration**: 100 ms – 500 ms (default: 200 ms)
- **Flash border only (non-intrusive)**
  - Values: on / off
  - Default: on

---

# 7. SESSION & LOGGING

## 7.1 Session Lifecycle

### 7.1.1 Session Creation
- **Auto-create session on startup**
  - Values: on / off
  - Default: off
- **Session name**
  - Values: auto-generated (date/time) / custom
  - Default: auto-generated
  - Pattern: `{YYYY-MM-DD}_{HH-mm}_{optional-name}`
- **Session template**
  - Values: blank / last session / named template
  - Default: blank

### 7.1.2 Session State
- **States**: new → active → paused → completed → archived
- **Auto-pause on silence**
  - Values: on / off
  - Default: off
  - Timeout: 30 s – 600 s (default: 120 s)
- **Auto-complete on close**
  - Values: on / off
  - Default: on

### 7.1.3 Session Metadata
- **Service type**: Sunday AM / Sunday PM / Wednesday / Special / Conference / Other / custom
- **Speaker name**: free text
- **Topic / Title**: free text
- **Notes**: multiline free text
- **Tags**: comma-separated
- **Venue**: free text
- Scope: per-session
- Persistence: per-session (saved with session file)

## 7.2 Sermon Log

### 7.2.1 Log Entries
- Each entry contains:
  - Timestamp (HH:MM:SS from session start)
  - Wall clock time
  - Verse reference
  - Verse text
  - Translation used
  - Confidence score
  - Detection method: ASR / manual / imported
  - Approved: yes / no
  - Operator notes (optional)

### 7.2.2 Log Operations
- **Click to restore verse**
- **Right-click context menu**: approve, reject, delete, edit reference, add note
- **Search/filter**: by reference, text, confidence, approval status
- **Sort**: by time (default), by reference, by confidence
- **Bulk operations**: approve all, reject all, export selected

### 7.2.3 Run Sheet
- **Pre-loaded verse list** (planned verses for the service)
- **Import from**: CSV, Planning Center, ProPresenter, plain text
- **Auto-match detected verses against run sheet**
- **Show progress**: X of Y planned verses detected
- **Reorder**: drag and drop
- **Add/remove entries live**

## 7.3 Transcript Log

### 7.3.1 Transcript Display
- **Show raw ASR transcript**
  - Values: on / off
  - Default: on
- **Transcript font**
  - Default: monospace
- **Transcript color**
  - Default: dim text (60% opacity)
- **Auto-scroll**
  - Values: on / off
  - Default: on
- **Max transcript length in UI**
  - Range: 100 – 5000 characters
  - Default: 400 characters
  - Note: full transcript always saved to session file regardless of display limit

### 7.3.2 Transcript Editing
- **Edit transcript post-session**
  - Values: on / off
  - Default: on
- **Timestamp per word**
  - Values: on / off
  - Default: off
  - Note: if on, each word is timestamped for precise alignment with recording

## 7.4 Export

### 7.4.1 Export Formats
- **Sermon log export**
  - Formats: PDF / CSV / JSON / DOCX / plain text / HTML / Markdown
  - Default: PDF
- **Transcript export**
  - Formats: plain text / SRT (subtitle) / VTT (WebVTT) / JSON
  - Default: plain text
- **Session export (full)**
  - Format: `.hvb-session` (proprietary, includes all data)
  - Importable: yes
- **Recording + markers export**
  - Format: WAV + CSV markers / WAV + cue points / AAF (Advanced Authoring Format)

### 7.4.2 Export Settings
- **Auto-export on session complete**
  - Values: on / off
  - Default: off
- **Export directory**
  - Default: `<data_root>/exports`
  - Configurable: yes
- **Include verse text in PDF**
  - Values: reference only / reference + text / reference + text + translation
  - Default: reference + text + translation
- **PDF layout**
  - Values: simple list / formatted cards / bulletin insert / projection script
  - Default: simple list

---

# 8. WORKSPACE & LAYOUT

## 8.1 Workspace Architecture

### 8.1.1 Panel System
- All panels are:
  - **Dockable**: can be placed in left, right, center, top, bottom docks
  - **Detachable**: can become floating windows
  - **Collapsible**: can be collapsed to header only
  - **Closable**: can be hidden entirely
  - **Resizable**: drag borders to resize
  - **Pinnable**: prevent accidental close/move

### 8.1.2 Available Panels
- Audio Control
- Live Verse Stage (center operator view)
- Sermon Log
- Run Sheet
- Queue (upcoming verses)
- Transcript
- Metrics / Diagnostics
- Events Log
- Bible Search
- Settings (inline)
- Confidence Monitor Preview
- Program Output Preview
- Recording Controls
- Integration Status

### 8.1.3 Workspace Presets
- **Default Broadcast**: large center stage, audio left, log/queue right, transcript bottom
- **Compact**: single-column, everything stacked
- **Dual Monitor**: operator on primary, program out on secondary
- **Sound Engineer**: audio panel dominant, minimal verse display
- **Confidence Monitor**: verse + confidence + transcript, no controls
- **Custom**: save / load / export / import custom layouts
- **Quick-switch hotkey**: Ctrl+1 through Ctrl+9 for presets

### 8.1.4 Panel Weights
- **Left panel weight**: 0.0 – 0.5 (default: 0.22)
- **Center panel weight**: 0.0 – 1.0 (default: 0.48)
- **Right panel weight**: 0.0 – 0.5 (default: 0.30)
- **Bottom panel height**: 40 px – 400 px (default: 120 px)

## 8.2 Tab System

### 8.2.1 Right Dock Tabs
- Default tabs: Run Sheet, Log, Queue, Detected
- Tab order: drag to reorder
- Tab visibility: right-click to show/hide tabs

### 8.2.2 Bottom Dock Tabs
- Default tabs: Transcript, Metrics, Events
- Same reorder/visibility behavior

---

# 9. INPUT & SHORTCUTS

## 9.1 Keyboard Shortcuts

### 9.1.1 Default Shortcuts
| Action | Default | Configurable |
|--------|---------|-------------|
| Start/Stop listening | F5 or Ctrl+S | Yes |
| Approve current verse | Space | Yes |
| Reject current verse | Escape | Yes |
| Undo (restore previous verse) | Ctrl+Z | Yes |
| Redo | Ctrl+Y | Yes |
| Focus manual input | Ctrl+F | Yes |
| Toggle program output | F11 | Yes |
| Blank output | B | Yes |
| Toggle recording | Ctrl+R | Yes |
| Add marker | M | Yes |
| Next verse in history | → | Yes |
| Previous verse in history | ← | Yes |
| Quick-select log entry 1-9 | Ctrl+1 – Ctrl+9 | Yes |
| Toggle mute monitor | Ctrl+M | Yes |
| Increase gain | Ctrl+↑ | Yes |
| Decrease gain | Ctrl+↓ | Yes |
| Open settings | Ctrl+, | Yes |
| Toggle dark/light mode | Ctrl+Shift+D | Yes |
| Toggle ribbon | Ctrl+Shift+R | Yes |
| Full screen | F11 (when not broadcasting) | Yes |
| Quit | Ctrl+Q / Alt+F4 | Yes |

### 9.1.2 Shortcut Profiles
- **Default**: as above
- **ProPresenter-like**: mapped to match ProPresenter shortcuts
- **vMix-like**: mapped to match vMix shortcuts
- **Custom**: fully user-defined
- **Import/Export**: JSON format

## 9.2 MIDI Input

### 9.2.1 MIDI Device
- **Enable MIDI input**
  - Values: on / off
  - Default: off
- **MIDI device selection**: dropdown of connected MIDI devices
- **MIDI channel**: 1–16 or omni
  - Default: omni

### 9.2.2 MIDI Mapping
- Map any MIDI note or CC to any application action
- **Note On**: trigger action (e.g., approve verse, start listening)
- **CC (continuous controller)**: map to slider (e.g., gain, monitor volume)
- **Program Change**: switch workspace preset
- **Learn mode**: press MIDI button → select action to map
- Scope: global
- Persistence: permanent

## 9.3 Foot Pedals
- **Enable foot pedal input**
  - Values: on / off
  - Default: off
- **Detection**: USB HID or MIDI
- **Pedal assignments**:
  - Pedal 1: approve verse (default)
  - Pedal 2: reject verse (default)
  - Pedal 3: next verse (default)
  - All configurable

## 9.4 Stream Deck (Elgato)
- **Enable Stream Deck integration**
  - Values: on / off
  - Default: off
- **Stream Deck plugin**: installable plugin for Elgato software
- **Available actions**: all keyboard-shortcut actions + custom icons
- **Dynamic buttons**: show current verse reference on button, show LED color, show confidence
- **Multi-action support**: chain actions together
- **Profile auto-switch**: switch Stream Deck profile when HV-Bible gains focus

## 9.5 Loupedeck
- Same integration model as Stream Deck
- Rotary encoders map to: gain, monitor volume, confidence threshold

## 9.6 Touch Screen
- **Touch-optimized mode**
  - Values: on / off
  - Default: off (auto-detected on touch devices)
- **Minimum touch target size**: 44 px × 44 px
- **Touch gestures**:
  - Swipe left: reject verse
  - Swipe right: approve verse
  - Swipe up: next verse in queue
  - Pinch: zoom verse text
  - Long press: context menu

---

# 10. INTEGRATIONS

## 10.1 Presentation Software

### 10.1.1 ProPresenter
- **Enable integration**: on / off (default: off)
- **Connection**: REST API
- **Host**: IP address or hostname (default: localhost)
- **Port**: default 1025
- **Protocol**: HTTP / HTTPS
- **Authentication**: none / API token
- **Actions**:
  - Push detected verse to ProPresenter as a slide
  - Auto-trigger slide on verse detection
  - Match verse to existing slide in library
  - Create new slide if no match found
- **Slide library mapping**: manual or auto-match by reference
- **Auto-advance**: on / off
- **Connection health check**: ping every N seconds (default: 5 s)
- **Failure mode**: notify operator, continue without ProPresenter
- **Reconnect**: automatic on connection loss (1/5/10/30 s intervals)

### 10.1.2 EasyWorship
- Same connection model
- Protocol: COM automation (Windows) or HTTP API
- Actions: push verse, trigger display

### 10.1.3 MediaShout
- Protocol: COM automation (Windows)
- Same actions as ProPresenter

### 10.1.4 OpenLP
- Protocol: HTTP REST API
- Same actions

### 10.1.5 FreeShow
- Protocol: WebSocket
- Same actions

### 10.1.6 Generic Presentation (custom)
- HTTP webhook: POST verse data to custom URL
- WebSocket: push verse data to custom WebSocket endpoint
- File output: write current verse to a text file (for OBS text source, etc.)
  - File path: configurable
  - Format: reference only / reference + text / JSON
  - Write interval: on change / every N seconds

## 10.2 Broadcast Switchers

### 10.2.1 OBS Studio
- **Protocol**: WebSocket (obs-websocket v5)
- **Host/Port**: configurable (default: localhost:4455)
- **Password**: configurable
- **Actions**:
  - Switch scene on verse detection
  - Toggle source visibility
  - Update text source with verse content
  - Trigger recording
  - Trigger streaming
- **Failure mode**: notify, continue without OBS

### 10.2.2 vMix
- **Protocol**: HTTP API / WebSocket
- **Host/Port**: configurable
- **Actions**:
  - Update title input with verse text
  - Trigger overlay on/off
  - Switch input
  - Execute vMix script
- **Failure mode**: notify, continue

### 10.2.3 Blackmagic ATEM
- **Protocol**: ATEM SDK
- **Actions**: trigger macro, switch input, enable downstream key
- **Failure mode**: notify

### 10.2.4 NewTek TriCaster
- **Protocol**: SDK / NDI
- **Actions**: trigger macro, update title page

### 10.2.5 Wirecast
- **Protocol**: custom API
- **Actions**: switch layer, update title

## 10.3 Streaming Platforms

### 10.3.1 YouTube Live
- **Push metadata**: update live chat with current verse
- **Push captions**: real-time caption stream from ASR transcript
- **Authentication**: OAuth 2.0
- **Failure mode**: notify, continue without YouTube

### 10.3.2 Facebook Live
- Same model as YouTube

### 10.3.3 Twitch
- **Chat bot**: post current verse reference in chat
- **Authentication**: OAuth
- **Failure mode**: notify

### 10.3.4 Restream
- Push to multiple platforms simultaneously
- API integration for metadata push

### 10.3.5 Custom RTMP
- Metadata injection into RTMP stream

## 10.4 Church Management Systems

### 10.4.1 Planning Center
- **Import service plan**: load expected verses from Planning Center service plan
- **Export sermon log**: push detected verses back to Planning Center
- **Authentication**: OAuth 2.0
- **Failure mode**: offline mode, sync when reconnected

### 10.4.2 Church Community Builder
- Import/export support

### 10.4.3 Rock RMS
- Import/export support

### 10.4.4 Custom REST Endpoint
- POST detected verses to any REST API
- Configurable: URL, headers, body template, auth

## 10.5 Audio Consoles

### 10.5.1 Behringer X32 / Midas M32
- **Protocol**: OSC over UDP
- **Actions**: mute/unmute channel on verse detection, adjust gain
- **Host/Port**: configurable
- **Failure mode**: notify

### 10.5.2 Allen & Heath (dLive, SQ, Avantis)
- **Protocol**: MIDI / TCP
- **Actions**: scene recall, mute control

### 10.5.3 Yamaha (CL, TF, QL)
- **Protocol**: MIDI
- **Actions**: scene recall, channel control

### 10.5.4 DiGiCo
- **Protocol**: integration SDK
- **Actions**: snapshot recall

## 10.6 Stage Lighting

### 10.6.1 DMX Control
- **Trigger DMX scene on verse detection**
- **Protocol**: Art-Net / sACN
- **Universe/Channel**: configurable
- **Per-verse lighting cues**: map specific verse references to specific DMX scenes

### 10.6.2 Philips Hue (for smaller venues)
- **Protocol**: Hue Bridge API
- **Actions**: change color on verse detection
- **Failure mode**: notify

## 10.7 Timing & Show Control

### 10.7.1 MIDI Show Control (MSC)
- **Send MSC cues on verse detection**
- **MSC device ID**: configurable
- **MSC command**: GO, STOP, RESUME

### 10.7.2 Timecode
- **LTC output**: embed verse timestamps in LTC stream
- **MTC output**: MIDI timecode
- **Art-Net timecode**: network timecode

### 10.7.3 OSC Output
- **Send OSC messages on verse detection**
- **Host/Port**: configurable
- **Message path**: configurable (e.g., `/hv-bible/verse`)
- **Data format**: reference, text, confidence, timestamp

## 10.8 External Triggering

### 10.8.1 HTTP Webhooks
- **On verse detected**: POST to configurable URL
- **On session start/stop**: POST
- **On error**: POST
- **Retry**: 1/3/5 attempts with exponential backoff
- **Timeout**: 1–30 s (default: 5 s)
- **Authentication**: none / bearer token / basic auth / custom headers

### 10.8.2 WebSocket Server
- **Enable WebSocket server**
  - Values: on / off
  - Default: off
- **Port**: configurable (default: 9876)
- **Broadcasts**: verse events, state changes, meter data, transcript updates
- **Client authentication**: none / token

---

# 11. AUTOMATION

## 11.1 Triggers

### 11.1.1 Event Triggers
- On verse detected
- On verse approved
- On verse rejected
- On session start
- On session stop
- On recording start/stop
- On ASR error
- On device disconnect
- On confidence threshold crossed (high → low or low → high)
- On silence detected (after N seconds)
- On speaking detected
- On specific verse detected (e.g., always highlight John 3:16)

### 11.1.2 Trigger Actions
- Run macro
- Send OSC message
- Send MIDI message
- Send HTTP webhook
- Execute system command
- Change lighting scene
- Switch ProPresenter slide
- Push to broadcast output
- Play sound effect
- Flash LED
- Log event

## 11.2 Macros
- **Named macros**: sequence of actions
- **Macro editor**: add/remove/reorder steps
- **Macro triggers**: keyboard shortcut, MIDI, event, schedule
- **Macro variables**: `{verse_ref}`, `{verse_text}`, `{confidence}`, `{timestamp}`
- **Import/Export**: JSON format

## 11.3 Schedules
- **Scheduled actions**: run macro at specific time
- **Recurring schedules**: daily, weekly
- Example: auto-start session at 9:00 AM every Sunday

---

# 12. SECURITY & PRIVACY

## 12.1 Permissions
- **No internet required for core operation**
- **Network access only for**: model downloads, translation downloads, integrations, update checks
- **Network access can be fully disabled**

## 12.2 Data Privacy
- **All audio processed on-device**: no audio leaves the machine
- **No telemetry**: zero data sent to any server unless explicitly configured
- **Opt-in analytics**: anonymous usage statistics (disabled by default)

## 12.3 Data Retention
- **Session data retention**: forever / 30 days / 90 days / 1 year / custom
- **Recording retention**: forever / 7 days / 30 days / custom
- **Auto-cleanup**: on / off (default: off)
- **Secure delete**: overwrite deleted files (on / off, default: off)

## 12.4 Encryption
- **Encrypt session files**: on / off (default: off)
- **Encryption algorithm**: AES-256
- **Password protection**: per-session or global master password

## 12.5 Audit Log
- **Enable audit log**: on / off (default: off)
- **Log contents**: all settings changes, session operations, export events
- **Log retention**: same as data retention policy

---

# 13. ACCESSIBILITY

## 13.1 Vision
- **High contrast mode**: on / off (follows system preference)
- **Color blind safe palette**: deuteranopia / protanopia / tritanopia
- **Minimum text size enforcement**: 12 pt minimum
- **Zoom**: 75% – 300%
- **Focus indicators**: high-visibility focus rings on all interactive elements

## 13.2 Screen Readers
- **ARIA / UI Automation support**: all panels, buttons, sliders, dropdowns
- **Announce verse detections**: speak verse reference via screen reader
- **Announce state changes**: speak "listening started", "verse detected", etc.

## 13.3 Motor
- **Keyboard-only operation**: all features accessible via keyboard
- **Sticky keys support**: on / off
- **Adjustable click/hold durations**
- **Large click targets**: 44 px minimum in touch mode

## 13.4 Audio (for operator)
- **Audio cue on verse detection**: beep / chime / custom sound / off
  - Default: off
- **Audio cue on error**: beep / chime / custom / off
  - Default: off
- **Audio cue volume**: 0% – 100% (default: 50%)

---

# 14. PERFORMANCE & DIAGNOSTICS

## 14.1 Real-Time Metrics
- **CPU usage**: total and per-thread (audio, ASR, UI)
- **Memory usage**: RSS, heap, model memory
- **GPU usage** (if GPU acceleration enabled)
- **Audio thread jitter**: max deviation from expected callback interval
- **Audio buffer underruns**: count per session
- **ASR latency**: time from audio chunk to transcript output
- **End-to-end latency**: time from spoken word to verse displayed
- **ASR queue depth**: chunks waiting to be processed
- **Dropped frames**: audio chunks that were discarded
- **Uptime**: session duration

## 14.2 Health Monitoring
- **Audio health**: green/yellow/red based on jitter and underruns
- **ASR health**: green/yellow/red based on latency and queue depth
- **Disk health**: free space monitoring
- **Integration health**: per-integration connection status

## 14.3 Watchdog
- **Audio thread watchdog**: if audio thread is unresponsive for >500 ms, restart it
- **ASR thread watchdog**: if ASR thread is unresponsive for >2 s, restart it
- **Auto-recovery**: attempt to restart failed subsystem without operator intervention

## 14.4 Performance Logging
- **Log performance metrics to file**
  - Values: on / off
  - Default: off
- **Log interval**: 1 s / 5 s / 10 s / 30 s
- **Log format**: CSV / JSON
- **Log directory**: configurable

---

# 15. NETWORK & CONNECTIVITY

## 15.1 Offline Mode
- **Core features (audio, ASR, Bible, display) work 100% offline**
- **Integrations degrade gracefully when offline**
- **Offline indicator**: show "OFFLINE" badge when no network

## 15.2 Download Manager
- **Model downloads**: with progress, pause, resume, checksum verification
- **Translation downloads**: same
- **Update downloads**: same
- **Proxy support**: HTTP / SOCKS5 / system proxy
  - Host, port, username, password
- **Bandwidth limit**: 0 (unlimited) – 10 Mbps
  - Default: unlimited

## 15.3 Network Discovery
- **NDI discovery**: auto-discover NDI sources on local network
- **Integration discovery**: auto-discover ProPresenter, OBS, vMix on local network (mDNS/Bonjour)

---

# 16. UPDATES & LICENSING

## 16.1 Updates
- **Auto-check for updates**: on / off (default: on)
- **Check frequency**: daily / weekly / monthly / manual only
  - Default: weekly
- **Update channel**: stable / beta / nightly
  - Default: stable
- **Auto-download updates**: on / off (default: off)
- **Auto-install updates**: on / off (default: off)
- **Rollback**: keep previous version for rollback
- **Offline updates**: import update package from USB / file

## 16.2 Licensing
- **License type**: free (community) / professional / broadcast / enterprise
- **Activation**: online activation / offline activation (USB dongle or license file)
- **License features**:
  - Community: basic ASR, KJV only, no NDI, no integrations
  - Professional: all ASR models, all public-domain translations, NDI, basic integrations
  - Broadcast: all features, all translations (with licensing), all integrations, priority support
  - Enterprise: multi-seat, central management, custom branding
- **License validation**: on startup (if online) / periodic (every 30 days) / offline (license file)
- **Grace period**: 7 days if license server unreachable

---

# 17. ADVANCED & DEVELOPER

## 17.1 Developer Mode
- **Enable developer mode**: hidden setting (Ctrl+Shift+D+E+V)
- **Debug overlay**: show FPS, frame time, event count, ASR queue depth
- **Verbose logging**: enable trace-level logging
- **Log level**: error / warn / info / debug / trace
  - Default: info
- **Log output**: file / console / both
- **Log file rotation**: daily / by size (10 MB) / none
- **Log file max count**: 5 / 10 / 30 / unlimited

## 17.2 Experimental Flags
- **Enable experimental features**: on / off (default: off)
- **Feature flags** (examples):
  - `exp_silero_vad`: use Silero neural VAD instead of energy-based
  - `exp_whisper_backend`: use Whisper.cpp instead of Sherpa-ONNX
  - `exp_gpu_decode`: GPU-accelerated ASR decoding
  - `exp_multi_speaker`: track multiple speakers
  - `exp_sentiment`: sentiment analysis on transcript
  - `exp_topic_detection`: auto-detect sermon topic
  - `exp_verse_prediction`: predict next likely verse based on context

## 17.3 Plugin API
- **Enable plugin system**: on / off (default: off)
- **Plugin format**: dynamic library (.dll / .so / .dylib) or script (Lua / WASM)
- **Plugin sandboxing**: on / off
- **Plugin permissions**: audio access / UI access / file access / network access
- **Plugin marketplace**: future (community plugin repository)

## 17.4 CLI Mode
- **Run as headless service**: `hv-bible --headless`
- **CLI commands**: start session, stop session, query status, export log
- **REST API** (when running): localhost:8080
- **Config file**: `config.toml` (all settings configurable via file)

## 17.5 Debug Tools
- **Audio test signal generator**: sine / white noise / pink noise / silence
- **ASR test mode**: feed pre-recorded audio file through pipeline
- **Parser test mode**: type text and see parse result in real-time
- **Event inspector**: show all pipeline events in real-time
- **Memory profiler**: show heap usage over time
- **Thread inspector**: show all threads, their state, CPU usage

---

# 18. LOCALIZATION & INTERNATIONALIZATION

## 18.1 UI Localization
- All UI strings externalized to `.ftl` (Fluent) or `.json` language files
- Right-to-left layout for Arabic, Hebrew
- Date format follows locale (DD/MM/YYYY vs MM/DD/YYYY)
- Time format follows locale (12h vs 24h)
- Number format follows locale (1,000.00 vs 1.000,00)

## 18.2 ASR Localization
- Model per language (separate downloads)
- Book name aliases per language
- Number word recognition per language
- Parser supports localized verse reference formats

## 18.3 Bible Localization
- Translation packs per language
- Versification varies by canon (Protestant vs Catholic chapter/verse numbering)
- Display fonts support: Latin, Cyrillic, CJK, Arabic, Hebrew, Devanagari, Ethiopic

---

# 19. HELP, ONBOARDING & SUPPORT

## 19.1 First-Run Wizard
- **Step 1**: Select UI language
- **Step 2**: Select audio device
- **Step 3**: Run audio test (speak a verse, see if it's recognized)
- **Step 4**: Select default translation
- **Step 5**: Choose workspace preset
- **Step 6**: Optional: connect integrations
- **Skip wizard**: on / off

## 19.2 Tooltips
- **Enable tooltips**: on / off (default: on)
- **Tooltip delay**: 0 ms – 2000 ms (default: 500 ms)
- **Tooltip detail level**: basic / detailed / off
- **Show keyboard shortcut in tooltip**: on / off (default: on)

## 19.3 In-App Help
- **Help panel**: searchable help content
- **Context-sensitive help**: F1 on any panel opens relevant help
- **Video tutorials**: embedded or linked (if online)

## 19.4 Diagnostics Report
- **Generate diagnostics report**: button in Help menu
- **Report contents**: system info, audio device info, model info, recent errors, performance metrics
- **Report format**: text file, anonymized
- **Report destination**: save to file / copy to clipboard

## 19.5 Bug Reporting
- **Built-in bug report form**: description, steps to reproduce, attach diagnostics
- **Submit**: via email or HTTP (if online)
- **Offline**: save report to file for manual submission

---

# 20. PROCESS TUNING & OS INTEGRATION

## 20.1 Process Priority
- **Process priority class**
  - Values: normal / above normal / high / real-time
  - Default: above normal
  - Scope: global
  - Persistence: permanent

## 20.2 Thread Priority
- **Audio thread priority**: real-time (default)
- **ASR thread priority**: above normal (default)
- **UI thread priority**: normal (default)
- **Worker thread priority**: normal (default)

## 20.3 Memory Management
- **Memory pinning (Windows)**
  - Values: on / off
  - Default: on
  - Note: prevents audio buffers from being paged to disk
- **Pre-allocate memory for model**
  - Values: on / off
  - Default: on
- **Max memory usage limit**
  - Range: 512 MB – 16 GB
  - Default: no limit

## 20.4 Power Management
- **Prevent system sleep while session active**
  - Values: on / off
  - Default: on
- **Prevent display sleep while broadcasting**
  - Values: on / off
  - Default: on

## 20.5 File Associations
- **`.hvb-session`**: HV-Bible session file
- **`.bible.bin`**: HV-Bible translation pack
- **Register file associations on install**: on / off (default: on)

---

# 21. MULTI-LANGUAGE ASR & TRANSLATION SYNCHRONIZATION

## 21.1 Simultaneous Multi-Language Recognition
- **Enable multi-language mode**
  - Values: on / off
  - Default: off
- **Primary language**: drives verse detection
- **Secondary language**: transcript display only
- **Language switching**: auto-detect / manual / hotkey

## 21.2 Translation Synchronization
- **When verse detected in Language A, auto-display in Language B**
  - Use case: pastor speaks in Spanish, display in English
- **Parallel display**: show verse in 2 or 3 translations simultaneously
- **Translation priority order**: configurable list

---

# 22. QUEUE & PREVIEW SYSTEM

## 22.1 Verse Queue
- **Queue panel**: list of upcoming verses
- **Queue sources**: manual add / run sheet / AI prediction
- **Queue operations**: add, remove, reorder, clear
- **Queue to broadcast**: push next queued verse to program output
- **Hotkey**: N (next from queue)

## 22.2 Preview
- **Preview pane**: see how the verse will look on broadcast output before going live
- **Preview vs. Program**: standard broadcast preview/program workflow
- **Take (cut to program)**: Space or dedicated button
- **Preview monitors**: separate preview and program feeds

---

# 23. BACKUP & RECOVERY

## 23.1 Auto-Backup
- **Enable auto-backup**: on / off (default: on)
- **Backup interval**: 1 min / 5 min / 15 min / 30 min
  - Default: 5 min
- **Backup scope**: session data / settings / both
- **Backup location**: `<data_root>/backups`
- **Max backups**: 5 / 10 / 20 / unlimited (default: 10)

## 23.2 Crash Recovery
- **Auto-save session state**: on / off (default: on)
- **On crash recovery**: offer to restore last auto-save
- **Crash report**: generate and save crash dump

## 23.3 Settings Backup
- **Export all settings**: to `.hvb-config` file
- **Import settings**: from `.hvb-config` file
- **Reset to factory defaults**: available in settings

---

# 24. HARDWARE PROFILE SYSTEM

## 24.1 Venue Profiles
- **Named venue profiles**: combine audio device, gain, processing, broadcast, integration settings
- **Example profiles**: "Main Sanctuary", "Fellowship Hall", "Outdoor Stage", "Studio A"
- **Quick-switch**: dropdown in toolbar or hotkey
- **Import/Export**: JSON format
- **Auto-detect venue**: match by audio device name (if same device always used at same venue)

## 24.2 Operator Profiles
- **Named operator profiles**: combine workspace layout, shortcut map, theme, preferences
- **Login/switch operator**: dropdown in title bar
- **Per-operator settings**: separate from venue settings

---

# 25. COMPLIANCE & BROADCAST STANDARDS

## 25.1 Copyright Compliance
- **Per-translation copyright display**: required text shown when translation is displayed
- **Auto-insert copyright**: on broadcast output
- **Copyright position**: configurable (default: bottom-right)
- **Copyright display duration**: always / 3 s after verse appears / configurable

## 25.2 Broadcast Standards
- **EBU R128 loudness compliance**: monitoring and warning
- **ATSC A/85 compliance**: for North American broadcast
- **Safe area guides**: on broadcast output (on / off, default: off)
- **Color space**: sRGB / Rec. 709 / Rec. 2020 (for NDI output)

## 25.3 Accessibility Standards
- **WCAG 2.1 AA compliance**: for operator interface
- **FCC closed captioning**: ASR transcript can feed closed caption stream

---

> [!IMPORTANT]
> This document defines the **complete product vision** for HV-Bible Desktop. Not all features need to ship in v1.0. Features should be prioritized into release tiers:
> - **v1.0 (MVP)**: Audio, ASR, Bible core, basic broadcast output, session logging
> - **v1.5**: NDI, ProPresenter, OBS integration, advanced metering
> - **v2.0**: Full integration suite, automation, plugin API, multi-language
> - **v3.0**: Hardware LED, DMX, MIDI Show Control, enterprise licensing



# 26. AUDIO CLOCK, SYNC & MULTI-DEVICE OPERATION

## 26.1 Clock Source

### 26.1.1 Clock Source Selection
- **Clock master**
  - Values: internal / external word clock / specific audio device / network (PTP/IEEE 1588) / video reference (blackburst/tri-level sync)
  - Default: internal
  - Scope: global
  - Persistence: permanent
- **Word clock input**
  - Values: BNC (via compatible interface) / AES11 / S/PDIF / ADAT
  - Default: none
  - Scope: per-device
  - Persistence: permanent
- **Word clock output**
  - Values: on / off
  - Default: off
  - Note: distribute clock to downstream devices
- **Clock frequency**
  - Values: 44.1 kHz / 48 kHz / 88.2 kHz / 96 kHz / 176.4 kHz / 192 kHz
  - Default: 48 kHz
  - Note: all devices in the chain must agree
- **Superclock**
  - Values: 1x / 256x
  - Default: 1x

### 26.1.2 Clock Status Display
- **Current clock source**: displayed in status bar
- **Lock status**: locked / unlocked / searching
- **Clock drift**: measured in PPM (parts per million)
  - Green: <1 PPM
  - Yellow: 1–10 PPM
  - Red: >10 PPM
- **Clock error count**: cumulative since session start
- **Clock health indicator**: green / yellow / red

### 26.1.3 Drift Compensation
- **Auto-drift compensation**
  - Values: on / off
  - Default: on
  - Scope: global
  - Persistence: permanent
- **Compensation algorithm**
  - Values: sample-drop/insert / async SRC / PLL-based
  - Default: async SRC
- **Max drift before warning**
  - Range: 0.5 PPM – 50 PPM
  - Default: 5 PPM
- **Max drift before failover**
  - Range: 10 PPM – 500 PPM
  - Default: 100 PPM
  - Behavior: switch to internal clock or failover device

## 26.2 Multi-Device Aggregation

### 26.2.1 Device Aggregation
- **Enable multi-device mode**
  - Values: on / off
  - Default: off
  - Scope: global
  - Persistence: permanent
- **Aggregated devices**
  - Add: select any connected audio device
  - Max: 8 simultaneous devices
  - Per-device role: primary / backup / aux / monitor
- **Aggregate device name**
  - Configurable: yes
  - Default: "HV-Bible Aggregate"

### 26.2.2 Per-Device Configuration (within aggregate)
- **Channel assignment**: which physical channels map to which virtual channels
- **Gain offset**: per-device gain trim (-24 to +24 dB)
- **Delay offset**: per-device delay compensation (0–100 ms, 0.01 ms steps)
- **Phase**: normal / inverted per device
- **Priority**: 1 (highest) to N (lowest) — determines failover order
- **Health monitoring**: per-device connection status, error count, latency

### 26.2.3 Failover Logic
- **Primary device failure behavior**
  - Values: switch to next priority device / switch to specific backup / stop session / continue with silence
  - Default: switch to next priority device
- **Failover latency budget**
  - Range: 0 ms – 1000 ms
  - Default: 50 ms
  - Note: how much silence is acceptable during failover
- **Failback behavior**
  - Values: auto-return to primary when available / stay on backup until manual switch
  - Default: auto-return to primary
- **Failback delay**
  - Range: 0 s – 60 s
  - Default: 5 s
  - Note: wait N seconds after primary returns before switching back (prevents flapping)
- **Failover event logging**
  - Values: always / never
  - Default: always

### 26.2.4 Audio Hot-Swap
- **Change device mid-session**
  - Behavior: seamless crossfade between old and new device
  - Crossfade duration: 0 ms – 500 ms (default: 50 ms)
  - Pre-buffer: maintain a 200 ms ring buffer across device transitions
  - Pipeline state: maintained (no restart required)
- **Hot-swap notification**
  - Values: visual / audible / both / none
  - Default: visual

---

# 27. SAMPLE RATE CONVERSION & DITHERING

## 27.1 Sample Rate Conversion

### 27.1.1 SRC Algorithm
- **Algorithm selection**
  - Values: 
    - Linear interpolation (fastest, lowest quality)
    - Cubic interpolation (fast, acceptable quality)
    - Sinc (short filter, 64 taps) — low-latency, good quality
    - Sinc (medium filter, 128 taps) — balanced
    - Sinc (long filter, 256 taps) — highest quality, highest latency
    - SoX Very High Quality — reference quality
    - Speex resampler — optimized for speech
  - Default: Sinc (medium, 128 taps)
  - Scope: global
  - Persistence: permanent
- **Latency mode**
  - Values: minimum-phase (lower latency, non-linear phase) / linear-phase (symmetric, higher latency)
  - Default: minimum-phase
  - Scope: global
  - Persistence: permanent
- **Anti-aliasing filter bandwidth**
  - Range: 85% – 99% of Nyquist
  - Default: 95%
  - Note: higher = more aliasing risk, lower = more high-frequency rolloff
- **Pre-ringing suppression**
  - Values: on / off
  - Default: on
  - Note: minimum-phase filters eliminate pre-ringing but alter group delay

### 27.1.2 SRC Paths
- **Input → ASR path**: always 48k/96k → 16k
  - Algorithm: configurable (default: Sinc medium)
  - Quality priority: accuracy over latency
- **Input → Monitor path**: native rate passthrough (no SRC if rates match)
- **Input → Recording path**: configurable (default: same as input)
- **ASR output → Display path**: no SRC needed (text only)

### 27.1.3 SRC Quality Metrics
- **Display SRC quality indicator**: on / off (default: off)
- **SRC latency display**: show in ms
- **SRC distortion measurement**: THD+N (Total Harmonic Distortion + Noise)

## 27.2 Dithering

### 27.2.1 Dither Settings
- **Enable dithering on bit-depth reduction**
  - Values: on / off
  - Default: on
  - Note: applies when converting from higher to lower bit depth (e.g., 24-bit → 16-bit for recording)
  - Scope: global
  - Persistence: permanent
- **Dither type**
  - Values:
    - None (truncation only)
    - Rectangular (RPDF) — flat noise floor, simplest
    - Triangular (TPDF) — most common, eliminates distortion harmonics
    - Noise-shaped (Lipshitz/Wannamaker) — pushes noise to inaudible frequencies
    - Modified E-weighted — optimized for human hearing curve
    - F-weighted — alternative weighting for speech
  - Default: Triangular (TPDF)
  - Scope: global
  - Persistence: permanent
- **Dither bit depth**
  - Values: 1-bit / 2-bit
  - Default: 1-bit
  - Note: 2-bit dither is slightly quieter but uses more noise floor
- **Auto-black (mute dither in silence)**
  - Values: on / off
  - Default: on
  - Note: stops dithering when signal is digital silence, preventing audible noise

### 27.2.2 Bit-Depth Preservation
- **Never reduce bit depth internally**
  - Values: on / off
  - Default: on
  - Note: internal processing always in 32-bit float; dithering only applied at output stage
- **Output bit depth**
  - Values: 16 / 24 / 32 float
  - Default: 24
  - Note: different outputs can have different bit depths

---

# 28. ADVANCED SPEECH PROCESSING

## 28.1 AI Noise Suppression

### 28.1.1 Neural Noise Suppression
- **Enable AI noise suppression**
  - Values: on / off
  - Default: on
  - Scope: per-session
  - Persistence: per-session
- **Model**
  - Values: RNNoise (lightweight, CPU) / DeepFilterNet (heavy, GPU) / Custom ONNX model
  - Default: RNNoise
  - Scope: global
  - Persistence: permanent
- **Suppression strength**
  - Range: 0% (bypass) – 100% (maximum suppression)
  - Default: 70%
  - Scope: per-session
  - Persistence: per-session
- **Artifact reduction**
  - Values: off / low / medium / high
  - Default: medium
  - Note: higher reduces musical artifacts at the cost of slight speech degradation
- **Processing position in chain**
  - Values: before AGC / after AGC / before HPF / after compressor / custom
  - Default: after HPF, before AGC
- **Latency added**
  - Display only: shows the additional latency (typically 10–40 ms)
- **Dry/wet mix**
  - Range: 0% (fully dry/bypassed) – 100% (fully processed)
  - Default: 100%
  - Note: allows blending original signal for more natural sound

### 28.1.2 Noise Profile
- **Auto-learn noise profile**
  - Values: on / off
  - Default: on
  - Note: learns the room's noise signature during first 5 seconds of session
- **Manual noise capture**
  - Button: "Capture Noise" — records 3 seconds of room tone for calibration
- **Noise profile storage**
  - Values: per-session / per-venue / per-device
  - Default: per-venue
  - Persistence: permanent
- **Noise profile export/import**
  - Format: `.hvb-noise` file
  - Shareable between installations

## 28.2 Dereverberation

### 28.2.1 Dereverb Settings
- **Enable dereverberation**
  - Values: on / off
  - Default: off
  - Scope: per-session
  - Persistence: per-session
- **Dereverb strength**
  - Range: 0% – 100%
  - Default: 50%
- **Reverb tail estimation**
  - Values: auto / manual (50 ms – 2000 ms)
  - Default: auto
  - Note: auto-detects RT60 (reverberation time) from the signal
- **Early reflections suppression**
  - Values: on / off
  - Default: on
- **Late reverberation suppression**
  - Values: on / off
  - Default: on
- **Dereverb model**
  - Values: spectral subtraction / Weighted Prediction Error (WPE) / neural (ONNX)
  - Default: WPE
- **Latency added**
  - Display only (typically 20–60 ms)

### 28.2.2 Room Calibration
- **Room calibration wizard**
  - Step 1: Play test signal (impulse or sweep)
  - Step 2: Record room response
  - Step 3: Compute room impulse response (RIR)
  - Step 4: Generate inverse filter
  - Step 5: Apply to dereverb engine
- **Calibration storage**: per-venue profile
- **Re-calibration reminder**: every N sessions (default: 10)

## 28.3 Source Separation

### 28.3.1 Speech/Music Separation
- **Enable source separation**
  - Values: on / off
  - Default: off
  - Scope: per-session
  - Persistence: per-session
- **Separation model**
  - Values: Demucs (heavy, GPU recommended) / SpeechBrain / Conv-TasNet / Custom ONNX
  - Default: Conv-TasNet (lighter)
- **Output selection**
  - Values: speech only / music only / both (separate channels)
  - Default: speech only
- **Separation quality**
  - Values: fast (lower quality) / balanced / best (highest latency)
  - Default: balanced
- **Pass speech to ASR, pass music to monitor**
  - Values: on / off
  - Default: on (when source separation is enabled)

### 28.3.2 Multi-Source Extraction
- **Number of sources**
  - Values: 2 (speech+background) / 3 (speech+music+noise) / 4 (speech+music+noise+crowd)
  - Default: 2
- **Per-source gain**
  - Range: -∞ to +12 dB
  - Default: 0 dB
- **Per-source routing**
  - Route each separated source to: ASR / monitor / recording / discard

## 28.4 Beamforming

### 28.4.1 Beamforming Engine
- **Enable beamforming**
  - Values: on / off
  - Default: off
  - Requires: stereo or multichannel input (2+ microphones)
- **Beamforming algorithm**
  - Values: Delay-and-Sum / MVDR (Minimum Variance Distortionless Response) / GSC (Generalized Sidelobe Canceller)
  - Default: MVDR
- **Steering direction**
  - Values: auto (track loudest source) / fixed angle / manual (adjust in real-time)
  - Default: auto
- **Beam width**
  - Range: narrow (15°) to wide (120°)
  - Default: 45°
- **Null steering**
  - Values: none / manual (steer nulls toward known noise sources)
  - Default: none

## 28.5 Automatic Mic Mixing (Dugan-style)

### 28.5.1 Auto-Mixer
- **Enable auto-mixer**
  - Values: on / off
  - Default: off
  - Requires: multi-channel input (2+ mics)
- **Algorithm**
  - Values: Dugan gain-sharing / Gate-based / NOM (Number of Open Mics) attenuation
  - Default: Dugan gain-sharing
- **Max open mics**
  - Range: 1 – all
  - Default: 1
  - Note: NOM attenuation reduces gain by 3 dB for every doubling of open mics
- **Per-mic priority**
  - Range: 1 (highest) to N (lowest)
  - Default: all equal
  - Note: higher priority mics get preferential gain allocation
- **Crossfade time**
  - Range: 1 ms – 100 ms
  - Default: 10 ms
- **Hold time**
  - Range: 0 ms – 500 ms
  - Default: 50 ms
  - Note: how long a mic stays "open" after speech stops

## 28.6 Feedback Suppression

### 28.6.1 Anti-Feedback
- **Enable feedback suppression**
  - Values: on / off
  - Default: off
- **Detection algorithm**
  - Values: frequency-domain (FFT-based) / phase-based / hybrid
  - Default: hybrid
- **Number of notch filters**
  - Range: 1 – 12
  - Default: 6
- **Notch bandwidth**
  - Values: narrow (1/20 octave) / medium (1/10 octave) / wide (1/5 octave)
  - Default: narrow
- **Attack speed**
  - Values: fast (10 ms) / medium (50 ms) / slow (200 ms)
  - Default: fast
- **Auto-release**
  - Values: on / off
  - Default: on
  - Note: release notch filter after feedback stops (prevents permanent frequency holes)
- **Release time**
  - Range: 1 s – 60 s
  - Default: 10 s

---

# 29. AUDIO ROUTING MATRIX

## 29.1 Patch Bay

### 29.1.1 Virtual Patch Bay
- **Enable routing matrix**
  - Values: on / off
  - Default: off (simple mode uses direct input→output)
  - Scope: global
  - Persistence: permanent
- **Input sources** (rows)
  - Physical device inputs (per-channel)
  - Virtual inputs (test tone, silence, file playback)
  - Network inputs (NDI audio, Dante, AVB)
  - Return from processing chain
- **Output destinations** (columns)
  - ASR engine
  - Monitor output (per-channel)
  - Recording (per-track)
  - Broadcast output (embedded audio)
  - Network outputs (NDI, Dante)
  - Processing chain input
  - Discard (null sink)
- **Matrix node**: each intersection has:
  - Enable: on / off
  - Gain: -∞ to +24 dB (default: 0 dB)
  - Polarity: normal / inverted
  - Delay: 0 – 100 ms
  - Mute: on / off

### 29.1.2 Buses
- **Named buses**: up to 16
- **Bus types**: mono / stereo / multi-channel
- **Default buses**:
  - ASR Bus (mono, goes to speech recognition)
  - Monitor Bus (stereo, goes to headphones/speakers)
  - Record Bus (mono or stereo, goes to recording)
  - Broadcast Bus (stereo, goes to NDI/output)
- **Bus processing**: each bus can have its own processing chain (EQ, compressor, limiter)
- **Bus soloing**: solo any bus for monitoring
- **Bus muting**: mute any bus independently

### 29.1.3 Sends
- **Pre-fader sends**: signal tapped before gain control
- **Post-fader sends**: signal tapped after gain control
- **Send level**: per-send, -∞ to +12 dB
- **Send mute**: per-send
- **Max sends per input**: 8

### 29.1.4 Multi-Track Recording
- **Enable multi-track**
  - Values: on / off
  - Default: off
- **Track count**: 1 – 16 (limited by CPU and disk speed)
- **Track assignment**: route any input/bus to any recording track
- **Per-track format**: independent sample rate and bit depth
- **Per-track file**: each track saved as separate file
- **Track naming**: auto (by source) or custom

## 29.2 Processing Chain Order

### 29.2.1 Custom Processing Chain
- **Reorderable processing modules**:
  1. DC offset removal
  2. High-pass filter
  3. Low-pass filter
  4. Noise gate
  5. AI noise suppression
  6. Dereverberation
  7. Source separation
  8. Feedback suppression
  9. De-esser
  10. Compressor
  11. AGC
  12. Limiter
  13. AEC
  14. Digital trim
- **Drag-and-drop reorder**: yes
- **Per-module bypass**: on / off toggle per module
- **Per-module solo**: hear only this module's output
- **A/B comparison**: toggle between processed and unprocessed

---

# 30. SPEAKER DIARIZATION & IDENTIFICATION

## 30.1 Speaker Diarization

### 30.1.1 Diarization Engine
- **Enable speaker diarization**
  - Values: on / off
  - Default: off
  - Scope: per-session
  - Persistence: per-session
- **Diarization model**
  - Values: pyannote-style (ONNX) / ECAPA-TDNN / x-vector / custom
  - Default: ECAPA-TDNN
- **Max speakers**
  - Range: 1 – 10
  - Default: 4
  - Scope: per-session
- **Min segment duration**
  - Range: 100 ms – 5000 ms
  - Default: 500 ms
- **Speaker change sensitivity**
  - Range: 0.0 – 1.0
  - Default: 0.5

### 30.1.2 Speaker Timeline
- **Display speaker timeline**
  - Values: on / off
  - Default: on (when diarization is enabled)
- **Timeline position**: bottom dock panel
- **Color-coding**: each speaker gets a unique color (auto-assigned or manual)
- **Speaker labels**: "Speaker 1", "Speaker 2" until named
- **Click-to-seek**: click on timeline to jump to that point in recording

## 30.2 Speaker Identification

### 30.2.1 Speaker Enrollment
- **Enroll speaker**
  - Process: record 10–30 seconds of speech
  - Storage: voiceprint stored locally as `.hvb-voiceprint` file
  - Per-voiceprint metadata:
    - Name (e.g., "Pastor Johnson")
    - Role (e.g., "Senior Pastor", "Worship Leader", "Guest Speaker")
    - Photo (optional)
    - Language (if different from primary)
    - Notes (free text)
- **Voiceprint library**
  - Max enrolled speakers: 50
  - Import/export: JSON + binary voiceprint data
  - Scope: global
  - Persistence: permanent
- **Quick-enroll from recording**
  - Select a segment of a past recording → extract voiceprint
- **Auto-enroll suggestion**
  - After 3 sessions with an unknown speaker, suggest enrollment

### 30.2.2 Speaker-Based Filtering
- **ASR filter by speaker**
  - Values: all speakers / only enrolled speakers / only specific speaker(s)
  - Default: all speakers
  - Scope: per-session
- **Verse detection filter by speaker**
  - Values: all / only pastor / only enrolled / blacklist specific speakers
  - Default: all
  - Note: prevents congregation "Amen! John 3:16!" from triggering verse detection
- **Per-speaker ASR confidence threshold**
  - Values: inherit global / custom per speaker
  - Default: inherit global
- **Per-speaker language**
  - Values: inherit session / custom per speaker
  - Default: inherit session
  - Use case: bilingual service where Pastor A speaks English and Pastor B speaks Spanish

### 30.2.3 Speaker Analytics
- **Speaking time per speaker**: total and percentage
- **Speaking rate per speaker**: words per minute
- **Verse density per speaker**: verses per minute of speaking time
- **Speaker transitions**: count and timestamps
- **Export speaker analytics**: CSV / JSON / PDF

---

# 31. MUSIC & SOUND CLASSIFICATION

## 31.1 Audio Scene Classification

### 31.1.1 Scene Classifier
- **Enable audio scene classification**
  - Values: on / off
  - Default: on
  - Scope: per-session
  - Persistence: per-session
- **Classification model**
  - Values: YAMNet (ONNX) / AudioSet-based / custom
  - Default: YAMNet
- **Classification categories**:
  - Speech (sermon, announcement, prayer)
  - Music (worship, instrumental, choir)
  - Applause
  - Laughter
  - Crowd noise (ambient chatter)
  - Silence (room tone)
  - Technical noise (feedback, hum, buzz)
  - Other
- **Classification update rate**
  - Range: 100 ms – 2000 ms
  - Default: 500 ms
- **Classification confidence threshold**
  - Range: 0.0 – 1.0
  - Default: 0.70

### 31.1.2 Scene Display
- **Show current scene in UI**
  - Values: on / off
  - Default: on
- **Scene indicator style**
  - Values: text label / icon / color band / LED
  - Default: text label + icon
- **Scene timeline**: visual timeline showing scene changes over session duration
- **Scene log**: list of all scene changes with timestamps

## 31.2 Auto-Pause Behaviors

### 31.2.1 Music Auto-Pause
- **Pause ASR during music**
  - Values: on / off
  - Default: on
  - Scope: per-session
- **Music → ASR pause delay**
  - Range: 0 ms – 2000 ms
  - Default: 500 ms
  - Note: wait N ms of confirmed music before pausing
- **Music → ASR resume delay**
  - Range: 0 ms – 5000 ms
  - Default: 1000 ms
  - Note: wait N ms after music stops before resuming ASR
- **Override: force ASR during music**
  - Hotkey: configurable
  - Note: operator can force ASR to continue during music (e.g., spoken word over music bed)

### 31.2.2 Applause Suppression
- **Ignore applause in ASR**
  - Values: on / off
  - Default: on
- **Applause gate threshold**
  - Range: 0.5 – 1.0
  - Default: 0.80
  - Note: how confident the classifier must be that it's applause before suppressing

### 31.2.3 Non-Speech Event Actions
- **On music detected**: pause ASR / log event / trigger lighting / all / none
- **On applause detected**: pause ASR / log event / none
- **On laughter detected**: log event / none
- **On silence detected (>N seconds)**: pause session / dim output / notify / none
- **On technical noise detected**: alert operator / auto-mute / log / none
- Each configurable independently

---

# 32. KEYWORD SPOTTING & PHRASE DETECTION

## 32.1 Keyword System

### 32.1.1 Keyword Lists
- **Built-in keyword lists**:
  - Liturgical: "Let us pray", "Amen", "Hallelujah", "The Lord be with you", "And also with you", "Lift up your hearts", "We lift them to the Lord", "Let us give thanks", "It is right to give thanks and praise"
  - Navigational: "Turn with me to", "Open your Bibles to", "If you have your Bible", "Our scripture reading", "The text today is", "Please stand for the reading"
  - Transitional: "In conclusion", "Finally", "Let me close with", "Let us bow our heads", "Please be seated"
  - Custom: user-defined
- **Custom keyword creation**
  - Add phrase: free text
  - Keyword alias: alternative phrasings that mean the same thing
  - Keyword category: custom tag
  - Keyword confidence threshold: 0.0–1.0 (default: 0.70)
  - Keyword action: configurable (see below)
- **Import/export keyword lists**: JSON format
- **Keyword list sharing**: exportable for other installations

### 32.1.2 Keyword Matching
- **Match mode**
  - Values: exact / fuzzy (Levenshtein distance) / phonetic (Soundex/Metaphone) / semantic (embedding similarity)
  - Default: fuzzy
- **Fuzzy threshold**
  - Range: 0.0 – 1.0
  - Default: 0.80
  - Note: how similar a spoken phrase must be to the keyword
- **Context requirement**
  - Values: any position in transcript / sentence start / after pause
  - Default: any position
- **Cooldown after match**
  - Range: 0 s – 60 s
  - Default: 5 s
  - Note: prevent the same keyword from firing repeatedly

### 32.1.3 Keyword Actions
- **Per-keyword action mapping**:
  - No action (log only)
  - Display text on screen
  - Trigger macro
  - Send OSC message
  - Send MIDI message
  - Change lighting scene (DMX)
  - Switch ProPresenter slide
  - Play sound effect
  - Flash LED
  - Push webhook
  - Start/stop recording
  - Mark timestamp
  - Custom script
- **Multiple actions per keyword**: yes (chain)
- **Conditional actions**: execute only if confidence > threshold AND speaker == specific person

### 32.1.4 Keyword Analytics
- **Keyword frequency**: how often each keyword is detected per session
- **Keyword timeline**: when keywords occur in the session
- **Keyword accuracy report**: operator-verified vs auto-detected
- **Export**: CSV / JSON

---

# 33. REAL-TIME TRANSLATION & MULTI-LANGUAGE DISPLAY

## 33.1 On-Device Translation

### 33.1.1 Translation Engine
- **Enable real-time translation**
  - Values: on / off
  - Default: off
  - Scope: per-session
  - Persistence: per-session
- **Translation model**
  - Values: OPUS-MT (ONNX) / M2M-100 / NLLB (No Language Left Behind) / Custom
  - Default: OPUS-MT
  - Note: all models run 100% on-device
- **Model download manager**: same as ASR model manager
- **Supported language pairs**:
  - English ↔ Spanish
  - English ↔ Portuguese
  - English ↔ French
  - English ↔ German
  - English ↔ Korean
  - English ↔ Mandarin
  - English ↔ Arabic
  - English ↔ Russian
  - English ↔ Swahili
  - English ↔ Hindi
  - Spanish ↔ Portuguese
  - (extensible via model download)
- **Hardware acceleration**: same options as ASR (CPU / GPU)

### 33.1.2 Translation Display
- **Display mode**
  - Values: replace original / side-by-side / stacked / interleaved
  - Default: stacked
- **Primary language position**: top / left
- **Translation position**: bottom / right
- **Translation font**: independently configurable (same options as verse text font)
- **Translation label**: show source→target language pair (e.g., "ES → EN")
- **Translation confidence display**: on / off (default: on)

### 33.1.3 Verse Translation Synchronization
- **When verse detected in Language A, show translation in Language B**
  - Values: on / off
  - Default: on (when translation is enabled)
- **Translation source**:
  - Values: ASR transcript → translate / Bible translation pack → direct lookup
  - Default: Bible translation pack (more accurate for known verses)
  - Fallback: ASR transcript translation (for unknown text)
- **Parallel Bible display**: show same verse in 2–4 translations simultaneously
  - Layout: columns / rows / tabs
  - Default: columns (2 translations), rows (3+ translations)

### 33.1.4 Translation Override
- **Operator can edit translation**: yes
- **Translation memory**: store operator corrections for future use
- **Translation glossary**: custom term mappings (e.g., "Jesús" always → "Jesus", never "Joshua")
- **Export translation memory**: TMX format (Translation Memory eXchange)

## 33.2 Bilingual Service Mode

### 33.2.1 Bilingual Configuration
- **Enable bilingual mode**
  - Values: on / off
  - Default: off
- **Language A**: primary language
- **Language B**: secondary language
- **Auto-detect active language**
  - Values: on / off
  - Default: on
- **Language switch behavior**:
  - Auto-switch output language based on detected language
  - Always show both languages
  - Show primary language only, subtitle in secondary
  - Operator manually switches
- **Default**: always show both

---

# 34. FULL SERMON TRANSCRIPTION & ANALYSIS

## 34.1 Full Transcription

### 34.1.1 Transcription Mode
- **Enable full sermon transcription**
  - Values: on / off
  - Default: on
  - Scope: per-session
  - Persistence: per-session
- **Transcription quality**
  - Values: real-time streaming / post-processing (higher accuracy) / both
  - Default: real-time streaming
- **Word-level timestamps**
  - Values: on / off
  - Default: on
  - Note: each word gets a start/end timestamp for precise alignment
- **Sentence boundary detection**
  - Values: on / off
  - Default: on
- **Paragraph detection**
  - Values: on / off
  - Default: on
  - Note: uses pause length and topic shifts to detect paragraph boundaries

### 34.1.2 Punctuation Restoration
- **Enable punctuation**
  - Values: on / off
  - Default: on (for transcript display)
- **Punctuation model**
  - Values: rule-based / neural (ONNX)
  - Default: rule-based
- **Capitalization restoration**
  - Values: on / off
  - Default: on

### 34.1.3 Transcript Export
- **Export formats**:
  - Plain text (`.txt`)
  - SRT subtitles (`.srt`)
  - WebVTT subtitles (`.vtt`)
  - JSON (with timestamps, speaker labels, confidence)
  - DOCX (formatted, with headings per speaker)
  - PDF (formatted)
  - HTML (interactive, with audio playback links)
  - Markdown
- **Export with speaker labels**: on / off (default: on, if diarization enabled)
- **Export with timestamps**: on / off (default: on)
- **Export with confidence scores**: on / off (default: off)

## 34.2 Sermon Analytics

### 34.2.1 Content Analytics
- **Scripture density**
  - Metric: verse references per minute of speaking time
  - Display: graph over session timeline
  - Average: displayed at session end
- **Topic detection**
  - Values: on / off
  - Default: off
  - Model: LDA (Latent Dirichlet Allocation) or neural topic model
  - Output: top 5 detected topics with confidence
  - Example: "Grace (0.85), Salvation (0.72), Love (0.68)"
- **Key phrase extraction**
  - Values: on / off
  - Default: off
  - Output: list of repeated/emphasized phrases
- **Sentiment analysis**
  - Values: on / off
  - Default: off
  - Model: VADER or neural sentiment model
  - Output: positive/negative/neutral sentiment over time
  - Display: color-coded timeline
- **Emotion detection**
  - Values: on / off
  - Default: off
  - Categories: joy, sadness, anger, fear, surprise, trust
  - Display: emoji timeline or colored segments

### 34.2.2 Delivery Analytics
- **Words per minute (WPM)**
  - Display: real-time and average
  - Target range: configurable (default: 120–160 WPM)
  - Alert: if outside target range
- **Pause analysis**
  - Average pause length
  - Pause frequency
  - Longest pause
  - Pause-to-speech ratio
- **Filler word detection**
  - Words: "um", "uh", "like", "you know", "basically", "actually"
  - Count: per filler word
  - Display: highlighted in transcript
  - Alert: if filler rate exceeds threshold
- **Volume variation**
  - RMS variance over time
  - Whisper-to-shout ratio
  - Dynamic range

### 34.2.3 Session Summary
- **Auto-generate session summary**
  - Values: on / off
  - Default: off
  - Model: summarization model (ONNX)
  - Output: 3–5 sentence summary
- **Session report**
  - Duration
  - Word count
  - Verse count (total, unique, by book)
  - Speaker breakdown (if diarization enabled)
  - Top topics
  - WPM average
  - Filler word count
  - Recording file(s) and sizes
  - Format: PDF / HTML / Markdown
  - Auto-generate: on close / manual

---

# 35. VIDEO INTEGRATION

## 35.1 Video Input

### 35.1.1 Video Capture
- **Enable video input**
  - Values: on / off
  - Default: off
  - Scope: per-session
- **Video source**
  - Values: webcam / capture card / NDI input / screen capture / virtual camera / file playback
  - Default: none
- **Video device selection**: dropdown of detected devices
- **Resolution**: match source / 1920×1080 / 3840×2160 / 1280×720 / custom
- **Frame rate**: match source / 24 / 25 / 29.97 / 30 / 50 / 59.94 / 60
- **Deinterlace**: on / off / auto
- **Color space**: auto / BT.601 / BT.709 / BT.2020

### 35.1.2 Video Preview
- **Show video preview in operator view**
  - Values: on / off
  - Default: off
- **Preview position**: floating window / docked panel / picture-in-picture
- **Preview size**: small (160×90) / medium (320×180) / large (640×360) / custom
- **Preview quality**: low / medium / high
  - Default: medium (to reduce CPU)

## 35.2 Video Output Composition

### 35.2.1 Overlay Modes
- **Overlay style**
  - Values: lower third / full screen / side panel / picture-in-picture / split screen / custom region
  - Default: lower third
- **Lower third configuration**:
  - Height: 10% – 50% of frame (default: 25%)
  - Position: bottom / top
  - Background: solid / gradient / transparent / semi-transparent / frosted glass
  - Background color: any hex (default: black 70% opacity)
  - Animation in: slide up / fade / wipe / reveal
  - Animation out: slide down / fade / wipe / conceal
  - Animation duration: 0 – 2000 ms (default: 400 ms)
  - Safe area margin: 0 – 10% (default: 5%)
- **Full screen configuration**:
  - Background: solid / gradient / image / video / transparent
  - Verse position: center / upper third / lower third / custom
  - Padding: 0 – 20% (default: 10%)

### 35.2.2 Multi-Layer Composition
- **Layer count**: 1 – 8
- **Layer types**: verse text / video / image / solid color / gradient / custom HTML
- **Layer ordering**: drag to reorder (top layer renders on top)
- **Per-layer opacity**: 0% – 100%
- **Per-layer blend mode**: normal / multiply / screen / overlay / add
- **Per-layer position**: anchor point + offset
- **Per-layer size**: percentage of frame or absolute pixels
- **Per-layer mask**: none / rectangle / rounded rectangle / circle / custom SVG

## 35.3 PTZ Camera Control

### 35.3.1 PTZ Integration
- **Enable PTZ control**
  - Values: on / off
  - Default: off
- **Protocol**
  - Values: VISCA / VISCA-over-IP / Pelco-D / Pelco-P / ONVIF / NDI PTZ
  - Default: VISCA-over-IP
- **Connection**: serial (COM port) / TCP/IP / NDI
- **Camera list**: up to 8 cameras

### 35.3.2 PTZ Presets
- **Named presets**: up to 64 per camera
  - Name: free text
  - Pan/Tilt/Zoom values: stored
  - Focus: stored
  - Speed: transition speed to this preset (1–24)
- **Preset triggers**:
  - On verse detected: move to preset N
  - On specific verse: move to preset N
  - On keyword: move to preset N
  - On speaker change: move to speaker's preset
  - Manual: via keyboard/MIDI/Stream Deck

---

# 36. SERVICE PLANNING & WORKFLOW

## 36.1 Pre-Service

### 36.1.1 Pre-Service Checklist
- **Auto-checklist on session start**
  - Values: on / off
  - Default: on
- **Checklist items** (each pass/fail/skip):
  - [ ] Audio device connected and receiving signal
  - [ ] Audio levels in acceptable range
  - [ ] ASR model loaded and responding
  - [ ] ASR test: say "John 3:16" and verify detection
  - [ ] Active translation loaded and verified
  - [ ] Run sheet imported (if applicable)
  - [ ] Broadcast output connected (if enabled)
  - [ ] NDI stream visible on network (if enabled)
  - [ ] Integration connections healthy (each integration)
  - [ ] Recording disk space sufficient (>1 GB)
  - [ ] Clock sync locked (if multi-device)
  - [ ] All operator stations connected (if multi-seat)
- **Custom checklist items**: add/remove/reorder
- **Checklist result**: all-green = ready, any-yellow = warning, any-red = not ready
- **Block session start on failure**: on / off (default: off)

### 36.1.2 Service Template
- **Create service template**
  - Name: free text
  - Service type: Sunday AM / Sunday PM / Wednesday / Special / Custom
  - Default audio preset: link to venue profile
  - Default run sheet: link to run sheet template
  - Default broadcast config: link to output preset
  - Default integrations: which integrations to enable
- **Template library**: named templates, import/export
- **Auto-apply template**: on session creation, based on day/time schedule

## 36.2 During Service

### 36.2.1 Service Flow Editor
- **Visual timeline**
  - Horizontal timeline showing service segments
  - Segments: worship / sermon / communion / offering / announcements / prayer / other
  - Color-coded by type
  - Drag to reorder
  - Drag edges to resize
  - Add/remove segments
- **Per-segment settings**:
  - Name
  - Expected duration
  - Actual duration (auto-tracked)
  - ASR mode: on / off / auto (based on audio scene)
  - Speaker: which enrolled speaker is expected
  - Verse list: expected verses for this segment
  - Notes: free text
  - Cues: trigger actions at segment start/end
- **Countdown timer**: shows time remaining in current segment
- **Overtime alert**: flash when segment exceeds expected duration
  - Alert threshold: 0 / 1 / 2 / 5 / 10 minutes over
  - Alert style: subtle / moderate / aggressive

### 36.2.2 Cue System
- **Cue types**:
  - Manual cue: operator presses GO
  - Auto cue: triggers at specific time or on event
  - Follow cue: triggers N seconds after previous cue
- **Per-cue actions**: any automation action (lighting, slide, verse push, recording, etc.)
- **Cue stack**: ordered list of cues
- **Cue execution**: sequential or concurrent
- **Cue preview**: see what the next cue will do before executing

## 36.3 Post-Service

### 36.3.1 Service Report
- **Auto-generate report on session close**
  - Values: on / off
  - Default: off
- **Report contents**:
  - Service metadata (date, time, type, speaker, venue)
  - Duration (total, per-segment)
  - Verse list (detected, with timestamps and confidence)
  - Run sheet completion (X of Y planned verses detected)
  - Accuracy summary (approved vs rejected vs auto-detected)
  - Transcript summary (word count, WPM, filler count)
  - Audio quality summary (average SNR, clip count, noise events)
  - Integration health summary
  - Incident log (errors, failovers, disconnects)
- **Report format**: PDF / HTML / Markdown / JSON
- **Report distribution**:
  - Save to file
  - Email to team (configurable recipient list)
  - Push to Planning Center
  - Push to custom webhook

### 36.3.2 Rehearsal Mode
- **Enable rehearsal mode**
  - Values: on / off
  - Default: off
  - Scope: per-session
- **Rehearsal behavior**:
  - All features active
  - Broadcast output shows "REHEARSAL" watermark
  - Recording tagged as "rehearsal"
  - Integrations in test mode (no live pushes)
  - Run sheet tracking active
- **Rehearsal comparison**: compare rehearsal results with live service

---

# 37. CONTENT LIBRARY & DATA MANAGEMENT

## 37.1 Verse Library

### 37.1.1 Saved Verses
- **Save verse to library**: right-click → "Save to Library"
- **Verse metadata**:
  - Reference
  - Text (per-translation)
  - Tags: free text, comma-separated
  - Category: custom dropdown (Salvation, Love, Faith, Hope, Healing, etc.)
  - Color: label color for visual grouping
  - Notes: free text
  - Usage count: how many times used in sessions
  - Last used: date
- **Search library**: by reference, text, tag, category, notes
- **Sort library**: by reference, date added, usage count, category
- **Import/export**: JSON / CSV

### 37.1.2 Verse Collections
- **Create collection**: named group of verses
  - Name: free text
  - Description: free text
  - Verses: ordered list
  - Sharing: export as `.hvb-collection` file
- **Built-in collections**: (empty by default, populated from community sharing)
- **Collection from session**: auto-create collection from all verses in a session
- **Collection as run sheet**: use a collection as a run sheet

## 37.2 Cross-References & Commentary

### 37.2.1 Cross-Reference Display
- **Show cross-references**
  - Values: on / off
  - Default: off
- **Cross-reference source**: Treasury of Scripture Knowledge (public domain)
- **Display position**: sidebar / tooltip / below verse
- **Max cross-references shown**: 3 / 5 / 10 / all
- **Click cross-reference**: loads that verse

### 37.2.2 Commentary Integration
- **Show commentary**
  - Values: on / off
  - Default: off
- **Commentary source**: Matthew Henry (public domain) / custom imported
- **Import custom commentary**: CSV format (reference → text)
- **Display position**: below verse / sidebar / tooltip

### 37.2.3 Original Language
- **Show Strong's numbers**
  - Values: on / off
  - Default: off
  - Requires: KJV with Strong's (separate pack)
- **Interlinear display**
  - Values: on / off
  - Default: off
  - Shows: Hebrew/Greek text with English gloss below each word
- **Original language font**: configurable (default: SBL Hebrew / SBL Greek)

## 37.3 Topical Search

### 37.3.1 Topic Index
- **Built-in topic index**: Nave's Topical Bible (public domain)
- **Search by topic**: type topic name, see all related verses
- **Topic categories**: ~5,000 topics
- **Custom topics**: add verses to custom topic labels
- **Topic → run sheet**: convert topic search results to a run sheet

---

# 38. MULTI-SEAT & REMOTE OPERATION

## 38.1 Multi-Seat Architecture

### 38.1.1 Network Mode
- **Enable network mode**
  - Values: standalone / server / client
  - Default: standalone
  - Scope: global
  - Persistence: permanent
- **Server mode**:
  - This machine is the master
  - Processes audio, runs ASR, manages state
  - Accepts client connections
  - Server port: configurable (default: 9877)
  - Max clients: 1 / 3 / 5 / 10 / unlimited
  - Discovery: mDNS/Bonjour
- **Client mode**:
  - Connects to a server
  - Receives state updates
  - Can send commands (if authorized)
  - Connection: IP:port or auto-discover

### 38.1.2 Role-Based Access
- **Roles**:
  - **Admin**: full control, all settings, all actions
  - **Operator**: approve/reject verses, manual input, recording, broadcast
  - **Monitor**: read-only view of all panels
  - **Broadcast**: control broadcast output only
- **Per-user role assignment**: username + role
- **Authentication**: password / PIN / none
- **Session handoff**: transfer control from one operator to another

## 38.2 Remote Monitoring

### 38.2.1 Web Dashboard
- **Enable web dashboard**
  - Values: on / off
  - Default: off
- **Dashboard port**: configurable (default: 8080)
- **Dashboard content**:
  - Current verse
  - Confidence
  - Transcript (live)
  - Metering (live)
  - Session status
  - Integration health
  - Error log
- **Dashboard access control**: password / token / none
- **Dashboard read-only**: always (no control from web)
- **Mobile-responsive**: yes

### 38.2.2 Remote Control (Tablet/Phone)
- **Enable remote control**
  - Values: on / off
  - Default: off
- **Control actions**: approve/reject verse, manual input, start/stop, blank output
- **Authentication**: PIN + role
- **Connection**: same local network (WebSocket)
- **Latency requirement**: <100 ms for control commands

---

# 39. ENTERPRISE DEPLOYMENT

## 39.1 Installation

### 39.1.1 Install Modes
- **Standard installer**: MSI (Windows) / DMG (macOS) / AppImage + .deb + .rpm (Linux)
- **Portable mode**: extract to USB, run without installation
  - All data stored relative to executable
  - No registry changes, no system-wide files
- **Silent install**: command-line installation with no user interaction
  - `hv-bible-setup.exe /S /D=C:\Program Files\HV-Bible`
  - All options configurable via command-line flags or answer file
- **Network install**: install from network share
- **Docker mode**: containerized version for server deployment (headless mode)

### 39.1.2 Enterprise Management
- **Group policy / MDM support** (Windows)
  - ADMX templates for all settings
  - Settings lockdown: prevent users from changing specific settings
  - Default configuration push
- **macOS MDM profiles**
  - Configuration profiles for all settings
  - Managed preferences
- **Central configuration server**
  - Push settings to all installations
  - Collect health reports from all installations
  - Centralized license management
  - Centralized update distribution

## 39.2 Branding

### 39.2.1 Custom Branding
- **Organization name**: displayed in title bar
- **Organization logo**: displayed in splash screen and about dialog
- **Custom accent color**: override theme accent
- **Custom splash screen**: replace with organization image
- **Custom icon**: replace app icon
- **Branding config**: `.hvb-brand` file (JSON + images)
- **Scope**: global (applied at install or via config)

## 39.3 Kiosk Mode

### 39.3.1 Kiosk Settings
- **Enable kiosk mode**
  - Values: on / off
  - Default: off
  - Scope: global
  - Persistence: permanent
  - Note: requires admin password to exit
- **Kiosk restrictions**:
  - Prevent window close
  - Prevent window minimize
  - Prevent alt-tab
  - Prevent system key capture (Win key, etc.)
  - Lock workspace layout
  - Lock audio device
  - Lock translation
  - Lock all settings
  - Hide menu bar
  - Hide settings panel
- **Kiosk admin password**: set on enable
- **Kiosk auto-start**: launch in kiosk mode on boot

---

# 40. SAFETY & EMERGENCY SYSTEMS

## 40.1 Emergency Controls

### 40.1.1 Panic Button
- **Panic hotkey**
  - Default: Ctrl+Shift+Escape
  - Configurable: yes
- **Panic button in UI**: always visible, red, large
- **Panic action**:
  - Immediately blank all broadcast outputs
  - Mute all audio outputs
  - Stop recording
  - Clear current verse
  - Display fallback content (if configured)
  - Log incident
  - Flash LED red
- **Panic recovery**: requires manual confirmation to resume

### 40.1.2 Emergency Stop
- **Physical emergency stop**: USB-connected red button (if hardware connected)
- **Same actions as panic button**
- **Auto-detected**: USB HID device with specific VID/PID

## 40.2 Redundancy

### 40.2.1 Output Redundancy
- **Backup output**
  - Values: none / specific output / mirror primary
  - Default: none
- **Failover trigger**: primary output fails / primary output disconnected / manual
- **Failover delay**: 0 ms – 5000 ms (default: 500 ms)
- **Failback**: auto / manual (default: manual)

### 40.2.2 Pipeline Redundancy
- **Dual-pipeline mode**
  - Values: on / off
  - Default: off
  - Note: runs two independent ASR pipelines; if one fails, the other takes over
- **Pipeline health monitoring**: per-pipeline latency, error count, last output
- **Auto-failover**: switch to healthy pipeline on error

## 40.3 Incident Management

### 40.3.1 Incident Log
- **Log all incidents automatically**
  - Values: on / off
  - Default: on
- **Incident types**:
  - Device disconnect
  - ASR error
  - Integration failure
  - Clock sync loss
  - Buffer underrun
  - Clip detection
  - Panic button pressed
  - Pipeline restart
  - Failover triggered
- **Per-incident data**:
  - Timestamp
  - Type
  - Severity: info / warning / error / critical
  - Description
  - Auto-resolution: what the system did
  - Manual resolution: what the operator did (can be added post-session)
- **Incident report**: exportable (PDF / JSON)

---

# 41. ADVANCED ANALYTICS & REPORTING

## 41.1 Historical Analytics

### 41.1.1 Verse Analytics (Across Sessions)
- **Most-referenced verses**: across all sessions, ranked by frequency
- **Verse frequency by book**: which books are preached most
- **Verse frequency by chapter**: heatmap
- **Verse trends over time**: which verses are referenced more/less over months
- **Unique verses per session**: average, min, max
- **New verses**: verses referenced for the first time

### 41.1.2 Speaker Analytics (Across Sessions)
- **Speaker frequency**: which speakers reference which verses
- **Speaker style**: WPM, pause frequency, filler count per speaker
- **Speaker comparison**: side-by-side metrics
- **Speaker trends**: changes over time

### 41.1.3 Accuracy Analytics
- **Detection accuracy rate**: auto-detected vs operator-approved
- **False positive rate**: rejected detections / total detections
- **Confidence distribution**: histogram of confidence scores
- **Accuracy by SNR**: correlation between signal quality and accuracy
- **Accuracy by book**: which books have highest/lowest accuracy
- **Accuracy trend**: improving or degrading over time

### 41.1.4 Operational Analytics
- **Average latency per session**: end-to-end
- **Buffer underrun frequency**: per session, per device
- **Integration uptime**: per integration, per session
- **Disk usage trend**: recording storage over time
- **Session duration trends**: average service length over time

## 41.2 Reports

### 41.2.1 Scheduled Reports
- **Enable scheduled reports**
  - Values: on / off
  - Default: off
- **Report frequency**: daily / weekly / monthly / quarterly
- **Report type**: summary / detailed / custom
- **Report delivery**: save to file / email / webhook
- **Report format**: PDF / HTML / CSV / JSON

### 41.2.2 Dashboard
- **Built-in analytics dashboard**
  - Charts: line, bar, pie, heatmap, timeline
  - Date range selector: last 7 days / 30 days / 90 days / year / custom
  - Filters: by speaker, by venue, by service type
  - Export: PDF / PNG / CSV

---

# 42. INTEGRATION RESILIENCE & ORCHESTRATION

## 42.1 Integration Health

### 42.1.1 Integration Dashboard
- **Integration health panel**: single view of all integrations
- **Per-integration status**: connected / disconnected / degraded / error / disabled
- **Per-integration latency**: round-trip time
- **Per-integration error count**: since session start
- **Per-integration last success**: timestamp
- **Per-integration uptime**: percentage

### 42.1.2 Health Checks
- **Heartbeat interval**: 1 s / 5 s / 10 s / 30 s / 60 s (default: 5 s)
- **Health check timeout**: 1 s / 3 s / 5 s / 10 s (default: 3 s)
- **Failed health check threshold**: 1 / 3 / 5 consecutive failures before marking unhealthy
- **Recovery check**: after failure, check every N seconds for recovery

## 42.2 Fallback Chains

### 42.2.1 Integration Fallback
- **Per-integration fallback**:
  - Primary: ProPresenter
  - Fallback 1: OBS Studio
  - Fallback 2: direct file output
  - Fallback 3: none (degrade silently)
- **Fallback trigger**: connection loss / timeout / error count exceeded
- **Fallback notification**: always notify operator
- **Fallback auto-return**: when primary recovers, switch back

### 42.2.2 Integration Chaining
- **Action chains**: one event triggers multiple integrations in sequence
  - Example: verse detected → ProPresenter slide + OBS overlay + DMX scene + webhook
- **Chain error handling**: 
  - Continue chain if one integration fails
  - Abort chain on first failure
  - Default: continue
- **Chain logging**: log each step's success/failure

## 42.3 Integration Testing

### 42.3.1 Test Mode
- **Integration test panel**: test each integration without a live service
- **Per-integration test**:
  - Connection test: verify connectivity
  - Capability test: verify API version / features
  - Push test: send a test verse and verify receipt
  - Round-trip test: measure latency
- **Test results**: displayed inline with pass/fail/skip
- **Test on session start**: optional (part of pre-service checklist)

---

# 43. CONTENT CREATION TOOLS

## 43.1 Output Theme Editor

### 43.1.1 Theme Editor
- **Open theme editor**: from Settings → Broadcast → Output Theme
- **Visual WYSIWYG editor**: see changes in real-time
- **Editable elements**:
  - Background (solid / gradient / image / video)
  - Verse reference block (position, size, font, color, shadow, outline)
  - Verse text block (position, size, font, color, shadow, outline, line height)
  - Translation label (position, size, font, color)
  - Copyright notice (position, size, font, color)
  - Logo (image, position, size, opacity)
  - Custom text blocks (unlimited)
  - Custom image blocks (unlimited)
  - Clock / timer overlay
  - QR code overlay
- **Layout presets**: centered / left-aligned / right-aligned / lower-third / side-panel
- **Safe area guides**: toggle on/off (title-safe, action-safe)
- **Grid / snap**: toggle on/off, configurable grid size
- **Undo/redo**: in theme editor
- **Save theme**: named theme file (`.hvb-theme`)
- **Export/import themes**: shareable files

### 43.1.2 Animation Editor
- **Keyframe-based animation**: per-element
- **Properties**: position, opacity, scale, rotation, color
- **Easing**: linear / ease-in / ease-out / ease-in-out / bounce / elastic / custom bezier
- **Duration**: per-animation, 0 – 5000 ms
- **Delay**: per-animation, 0 – 5000 ms
- **Preview**: play animation in editor
- **Animation presets**: fade / slide / zoom / typewriter / reveal / bounce

## 43.2 Font Manager

### 43.2.1 Font Management
- **Bundled fonts**: Inter, Roboto, Outfit, Georgia, Garamond, Merriweather, Lora, Playfair Display, PT Serif, Source Sans, Noto Sans, Noto Serif
- **System fonts**: access all installed system fonts
- **Import custom fonts**: TTF / OTF / WOFF2
- **Font preview**: preview any font with sample verse text
- **Font favorites**: mark frequently used fonts
- **Font categories**: serif / sans-serif / display / handwriting / monospace

## 43.3 Media Library

### 43.3.1 Background Images
- **Import images**: PNG / JPG / WebP / TIFF
- **Built-in backgrounds**: 20+ included (abstract, nature, church architecture, solid gradients)
- **Image scaling**: fit / fill / stretch / tile
- **Image filters**: brightness / contrast / saturation / blur / overlay color

### 43.3.2 Background Videos
- **Import videos**: MP4 / MOV / WebM
- **Video loops**: auto-loop short videos as background
- **Video playback**: speed, opacity
- **Built-in loops**: 10+ included (abstract motion, particles, light rays)

---

# 44. PRINT & PHYSICAL OUTPUT

## 44.1 Print System

### 44.1.1 Print Options
- **Print current verse**: File → Print → Current Verse
- **Print verse history**: File → Print → Session Verses
- **Print run sheet**: File → Print → Run Sheet
- **Print sermon transcript**: File → Print → Transcript
- **Print service report**: File → Print → Service Report

### 44.1.2 Print Layouts
- **Verse card**: single verse, large text, suitable for pulpit stand
  - Size: A5 / A4 / Letter / custom
  - Font: configurable
  - Include: reference, text, translation, QR code (optional)
- **Bulletin insert**: multiple verses, compact layout
  - Columns: 1 / 2 / 3
  - Include header: service name, date, speaker
  - Include footer: church name, copyright notices
- **Projection script**: ordered verse list with notes
  - Include: reference, text, operator notes, timing cues
- **Sermon notes**: formatted transcript with headings
  - Include: speaker name, timestamps, verse highlights

### 44.1.3 Print Preview
- **WYSIWYG print preview**: see exactly what will print
- **Page setup**: margins, orientation, paper size
- **Print to PDF**: always available, even without a printer

## 44.2 QR Code Output

### 44.2.1 QR Code Display
- **Show QR code on broadcast output**
  - Values: on / off
  - Default: off
- **QR code content**:
  - Values: Bible Gateway link / YouVersion link / custom URL / verse text
  - Default: Bible Gateway link
- **QR code position**: corner (configurable)
- **QR code size**: small / medium / large / custom
- **QR code foreground/background colors**: configurable
- **QR code display duration**: always / 5 s / 10 s / same as verse

---

# 45. VOICE COMMANDS

## 45.1 Voice Control

### 45.1.1 Voice Command Engine
- **Enable voice commands**
  - Values: on / off
  - Default: off
  - Scope: global
  - Persistence: permanent
- **Wake word**
  - Values: none (always listening) / "Hey Bible" / custom
  - Default: none
- **Voice command source**: separate microphone (operator's mic, not the soundboard feed)
- **Command language**: same as UI language

### 45.1.2 Available Voice Commands
- "Start listening" / "Stop listening"
- "Approve" / "Reject"
- "Next verse" / "Previous verse"
- "Go to [reference]" — e.g., "Go to John 3:16"
- "Blank output" / "Show output"
- "Start recording" / "Stop recording"
- "Switch to [translation]" — e.g., "Switch to ESV"
- "Mute" / "Unmute"
- Custom commands: map voice phrase to any action

### 45.1.3 Voice Command Feedback
- **Audio confirmation**: beep / chime / spoken confirmation / none
- **Visual confirmation**: flash icon / toast notification / none
- **Default**: visual confirmation

---

# 46. ADVANCED DISPLAY MODES

## 46.1 Chapter View

### 46.1.1 Full Chapter Display
- **Show full chapter**
  - Trigger: operator request or double-click on reference
  - Display: scrollable chapter text with detected verse highlighted
  - Current verse: highlighted with accent color
  - Context: show surrounding verses (configurable: ±2 / ±5 / full chapter)

### 46.1.2 Reading Mode
- **Continuous reading mode**
  - Values: on / off
  - Default: off
  - Note: displays full text scrolling at a configurable pace
- **Scroll speed**: slow / medium / fast / custom WPM
- **Auto-advance**: move to next chapter when current chapter ends

## 46.2 Comparison Mode

### 46.2.1 Multi-Translation Comparison
- **Compare view**: show same verse in 2–6 translations side by side
- **Layout**: columns (default) / rows / tabbed
- **Highlight differences**: on / off (default: on)
  - Diff algorithm: word-level comparison
  - Highlight color: configurable
- **Quick-compare hotkey**: Ctrl+Shift+C

## 46.3 Presenter Notes

### 46.3.1 Operator Notes
- **Per-verse notes**
  - Type: free text, visible only on operator screen
  - Persist: per-session or permanent (saved to verse library)
  - Display: below verse text in operator view
- **Per-session notes**
  - Type: free text notepad
  - Persist: per-session
  - Display: separate panel or tab

---

# 47. RELEASE PLANNING

## 47.1 Feature-to-Release Mapping

### v1.0 — Foundation (MVP)
| Feature | Priority | Effort |
|---------|----------|--------|
| Audio input, gain, metering | Critical | L |
| ASR (Sherpa-ONNX, streaming) | Critical | L |
| Bible parser, verse detection | Critical | L |
| Bible translations (KJV + public domain) | Critical | M |
| Operator UI (workspace, verse stage) | Critical | L |
| Session management | Critical | M |
| Sermon log | Critical | S |
| Manual verse input | Critical | S |
| Keyboard shortcuts | Critical | S |
| Basic broadcast output (window) | Critical | M |
| Audio recording | Important | M |
| Config persistence | Critical | S |
| Theme system (dark/light) | Important | S |

### v1.5 — Professional
| Feature | Priority | Effort |
|---------|----------|--------|
| NDI output | Critical | M |
| ProPresenter integration | Critical | M |
| OBS Studio integration | Important | M |
| Advanced metering (LUFS, spectrum) | Important | M |
| Run sheet | Important | M |
| HTML overlay output | Important | M |
| MIDI input | Important | M |
| Stream Deck integration | Important | M |
| Audio processing chain (HPF, gate, compressor) | Important | L |
| Music/applause classification | Important | L |
| Speaker diarization | Important | L |
| Full sermon transcription | Important | L |
| SRT/VTT subtitle export | Important | S |

### v2.0 — Broadcast Grade
| Feature | Priority | Effort |
|---------|----------|--------|
| Multi-device aggregation & failover | Critical | L |
| Clock sync | Critical | L |
| AI noise suppression | Critical | M |
| Dereverberation | Important | M |
| Source separation | Important | L |
| vMix integration | Important | M |
| ATEM integration | Important | M |
| DMX / lighting control | Important | M |
| Keyword spotting | Important | M |
| Video integration (preview, PTZ) | Important | XL |
| Output theme editor | Important | L |
| Multi-seat operation | Important | XL |
| Service planning workflow | Important | L |
| Analytics dashboard | Important | L |
| Content library (cross-ref, commentary) | Nice | L |
| Real-time translation | Nice | XL |

### v3.0 — Enterprise
| Feature | Priority | Effort |
|---------|----------|--------|
| Enterprise deployment (MDM, silent install) | Critical | L |
| Custom branding | Important | M |
| Kiosk mode | Important | M |
| Plugin API | Important | XL |
| Voice commands | Nice | M |
| Sentiment/topic analysis | Nice | L |
| Auto-mixer (Dugan) | Nice | L |
| Beamforming | Nice | L |
| Full routing matrix | Nice | XL |
| Print system | Nice | M |

### Effort Key
- **S** = Small (1–3 days)
- **M** = Medium (1–2 weeks)
- **L** = Large (2–6 weeks)
- **XL** = Extra Large (6+ weeks)

---





