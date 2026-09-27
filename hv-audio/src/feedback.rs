//! Level control and acoustic feedback suppression.
//!
//! Two processors that did not fit the original `dsp` module and need their own
//! tests: [`Agc`] (envelope leveler) and [`Aec`] (acoustic feedback suppressor).
//! Both operate on the pipeline's 16 kHz mono `i16` stream.

use crate::AudioChunk;

/// Every processor here runs on the pipeline's fixed 16 kHz mono stream.
const SAMPLE_RATE: f32 = 16_000.0;

/// One-pole smoothing coefficient for a time constant in milliseconds.
fn coefficient(time_ms: f32) -> f32 {
    let ms = time_ms.max(0.1);
    (-1.0 / (ms * SAMPLE_RATE / 1000.0)).exp()
}

/// Asymmetric envelope follower: fast into transients, slow out of them.
fn follower(previous: f32, magnitude: f32, attack: f32, release: f32) -> f32 {
    if magnitude > previous {
        attack * previous + (1.0 - attack) * magnitude
    } else {
        release * previous + (1.0 - release) * magnitude
    }
}

/// Envelope-following leveler (AGC).
///
/// Tracks a slow envelope of the input and rides a smoothed gain multiplier so
/// the envelope sits at `target_db`, bounded by `max_gain_db`. Gain is clamped
/// to unity or above: it only ever *lifts* quiet material, so a hot take is
/// never ducked mid-sentence.
pub struct Agc {
    pub target_db: f32,
    pub max_gain_db: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    envelope: f32,
    current_gain: f32,
}

impl Default for Agc {
    fn default() -> Self {
        Self {
            target_db: -18.0,
            max_gain_db: 20.0,
            attack_ms: 10.0,
            release_ms: 200.0,
            envelope: 0.0,
            current_gain: 1.0,
        }
    }
}

impl Agc {
    pub fn process(&mut self, chunk: &mut AudioChunk) {
        let attack_coef = coefficient(self.attack_ms);
        let release_coef = coefficient(self.release_ms);
        let target_linear = 10f32.powf(self.target_db / 20.0);
        let max_gain_linear = 10f32.powf(self.max_gain_db / 20.0);

        for sample in &mut chunk.samples {
            let value = f32::from(*sample) / 32_768.0;
            self.envelope = follower(self.envelope, value.abs(), attack_coef, release_coef);

            let wanted = if self.envelope > 1e-5 {
                (target_linear / self.envelope).clamp(1.0, max_gain_linear)
            } else {
                1.0
            };

            // Reducing gain is an attack; handing it back is a release.
            self.current_gain = if wanted < self.current_gain {
                attack_coef * self.current_gain + (1.0 - attack_coef) * wanted
            } else {
                release_coef * self.current_gain + (1.0 - release_coef) * wanted
            };

            *sample = (value * self.current_gain * 32_767.0)
                .clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Acoustic feedback / echo suppression
// ─────────────────────────────────────────────────────────────────────────────

/// Watch frequencies, log-spaced across the speech band.
const AEC_BANDS_HZ: [f32; 6] = [250.0, 500.0, 1_000.0, 2_000.0, 3_500.0, 5_000.0];
/// A band holding this much of the broadband envelope counts as feeding back.
const FEEDBACK_RATIO: f32 = 0.62;
/// Below this the input is too quiet to judge; no notch depth is accumulated.
const FLOOR: f32 = 0.002;
/// Per-chunk change in notch depth, 0.0 (open) .. 1.0 (fully closed).
const ENGAGE_STEP: f32 = 0.30;
const RELEASE_STEP: f32 = 0.06;
/// Chunks the condition must stay clear before the notch starts reopening.
/// Without this, a tone that is already being notched drops below the detector
/// floor, the notch releases, the tone returns, and the whole thing pumps.
const RELEASE_HOLD_CHUNKS: u8 = 8;

/// Acoustic echo suppressor (AEC).
///
/// Real-time feedback control, not a stub. Six narrow band followers watch the
/// input; a band that holds most of the broadband energy is being re-captured
/// from the PA, so a cascaded notch at that band deepens. Speech is broadband
/// and never trips the ratio, so a voice passes through untouched.
pub struct Aec {
    /// Detection: one band-pass per band, fed the raw sample.
    detectors: [Biquad; AEC_BANDS_HZ.len()],
    detector_env: [f32; AEC_BANDS_HZ.len()],
    /// Suppression: one notch per band, cascaded in series.
    notches: [Notch; AEC_BANDS_HZ.len()],
    depth: [f32; AEC_BANDS_HZ.len()],
    clear_for: [u8; AEC_BANDS_HZ.len()],
    input_env: f32,
}

impl Default for Aec {
    fn default() -> Self {
        let mut detectors = [Biquad::unit(); AEC_BANDS_HZ.len()];
        let mut notches = [Notch::unit(); AEC_BANDS_HZ.len()];
        for (index, frequency) in AEC_BANDS_HZ.iter().enumerate() {
            detectors[index] = Biquad::bandpass(SAMPLE_RATE, *frequency, 10.0);
            notches[index] = Notch::new(SAMPLE_RATE, *frequency, 8.0);
        }
        Self {
            detectors,
            detector_env: [0.0; AEC_BANDS_HZ.len()],
            notches,
            depth: [0.0; AEC_BANDS_HZ.len()],
            clear_for: [0; AEC_BANDS_HZ.len()],
            input_env: 0.0,
        }
    }
}

impl Aec {
    pub fn process(&mut self, chunk: &mut AudioChunk) {
        let env_attack = coefficient(10.0);
        let env_release = coefficient(120.0);

        for sample in &mut chunk.samples {
            let value = f32::from(*sample) / 32_768.0;
            self.input_env = follower(self.input_env, value.abs(), env_attack, env_release);

            for band in 0..AEC_BANDS_HZ.len() {
                let detected = self.detectors[band].process(value).abs();
                self.detector_env[band] =
                    follower(self.detector_env[band], detected, env_attack, env_release);
            }
        }

        // Decide once per chunk, not per sample: the envelope needs time to
        // settle, and a notch that chatters is worse than no notch at all.
        for band in 0..AEC_BANDS_HZ.len() {
            let ratio = self.detector_env[band] / self.input_env.max(1e-6);
            if ratio > FEEDBACK_RATIO && self.input_env > FLOOR {
                self.clear_for[band] = 0;
                self.depth[band] = (self.depth[band] + ENGAGE_STEP).min(1.0);
            } else {
                self.clear_for[band] = self.clear_for[band].saturating_add(1);
                if self.clear_for[band] > RELEASE_HOLD_CHUNKS {
                    self.depth[band] = (self.depth[band] - RELEASE_STEP).max(0.0);
                }
            }
            self.notches[band].set_depth(self.depth[band]);
        }

        if self.depth.iter().all(|depth| *depth <= 0.0) {
            return; // Nothing is feeding back — pass the buffer straight through.
        }

        for sample in &mut chunk.samples {
            let mut value = f32::from(*sample) / 32_768.0;
            for notch in &mut self.notches {
                value = notch.process(value);
            }
            *sample =
                (value * 32_767.0).clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16;
        }
    }

    /// Current notch depth per watched band, 0.0 .. 1.0. For diagnostics.
    pub fn band_depths(&self) -> &[f32] {
        &self.depth
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Biquads (RBJ audio-EQ-cookbook coefficients, direct form I)
// ─────────────────────────────────────────────────────────────────────────────

fn w0_cos(sample_rate: f32, frequency: f32) -> (f32, f32) {
    let w0 = 2.0 * std::f32::consts::PI * frequency / sample_rate;
    (w0, w0.cos())
}

fn alpha(w0: f32, q: f32) -> f32 {
    (w0 / 2.0 / q.max(0.5)).sin()
}

/// Fixed-coefficient biquad. Used for the narrow detection band-passes.
#[derive(Clone, Copy)]
struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Biquad {
    const fn unit() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    /// Constant peak-gain band-pass, for measuring one narrow band.
    fn bandpass(sample_rate: f32, frequency: f32, q: f32) -> Self {
        let (w0, cos_w0) = w0_cos(sample_rate, frequency);
        let alpha = alpha(w0, q);
        let a0 = 1.0 + alpha;
        Self {
            b0: alpha / a0,
            b1: 0.0,
            b2: -alpha / a0,
            a1: -2.0 * cos_w0 / a0,
            a2: (1.0 - alpha) / a0,
            ..Self::unit()
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input
            + self.b1 * self.x1
            + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }
}

/// A notch whose depth can be reopened at runtime.
#[derive(Clone, Copy)]
struct Notch {
    sample_rate: f32,
    frequency: f32,
    base_q: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Notch {
    const fn unit() -> Self {
        Self {
            sample_rate: 16_000.0,
            frequency: 0.0,
            base_q: 8.0,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    fn new(sample_rate: f32, frequency: f32, base_q: f32) -> Self {
        let mut notch = Self {
            sample_rate,
            frequency,
            base_q,
            ..Self::unit()
        };
        notch.set_depth(0.0);
        notch
    }

    /// Widen the notch as `depth` runs 0.0 → 1.0. A wider notch removes more of
    /// a sustained tone, at the cost of reaching further into the band.
    fn set_depth(&mut self, depth: f32) {
        let depth = depth.clamp(0.0, 1.0);
        let (w0, cos_w0) = w0_cos(self.sample_rate, self.frequency);
        let alpha = alpha(w0, self.base_q * (1.0 + depth * 2.0));
        let a0 = 1.0 + alpha;
        self.b0 = 1.0 / a0;
        self.b1 = -2.0 * cos_w0 / a0;
        self.b2 = 1.0 / a0;
        self.a1 = -2.0 * cos_w0 / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input
            + self.b1 * self.x1
            + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }
}



#[cfg(test)]
mod tests {
    use super::{Aec, Agc, SAMPLE_RATE};
    use crate::AudioChunk;

    /// One capture chunk, the size the device actually delivers.
    const CHUNK: f32 = 0.1;

    /// A steady sine — the shape of PA feedback arriving at a mic.
    fn tone(frequency: f32, amplitude: f32, seed: u32) -> AudioChunk {
        sine_chunk(frequency, amplitude, CHUNK, seed)
    }

    /// Broadband noise — the shape of a voice, which must never be notched.
    fn noise(amplitude: f32, seed: u32) -> AudioChunk {
        sine_chunk(0.0, amplitude, CHUNK, seed)
    }

    /// `frequency == 0.0` yields deterministic pseudo-random noise instead.
    fn sine_chunk(frequency: f32, amplitude: f32, seconds: f32, seed: u32) -> AudioChunk {
        let count = (SAMPLE_RATE * seconds) as usize;
        let mut state = seed | 1;
        let samples = (0..count)
            .map(|i| {
                let value = if frequency > 0.0 {
                    let t = i as f32 / SAMPLE_RATE;
                    amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin()
                } else {
                    state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    amplitude * ((state >> 8) as f32 / 8_388_608.0 - 1.0)
                };
                (value * 32_767.0) as i16
            })
            .collect();
        AudioChunk::new(samples, 0)
    }

    fn rms(chunk: &AudioChunk) -> f32 {
        let sum: f64 = chunk
            .samples
            .iter()
            .map(|s| f64::from(*s) * f64::from(*s))
            .sum();
        (sum / chunk.samples.len() as f64).sqrt() as f32
    }

    fn moved_db(before: f32, after: f32) -> f32 {
        20.0 * (after / before).max(1e-9).log10()
    }

    /// One second of a single tone, for the level tests.
    fn one_second(frequency: f32, amplitude: f32) -> AudioChunk {
        sine_chunk(frequency, amplitude, 1.0, 1)
    }

    #[test]
    fn agc_lifts_a_quiet_take_inside_its_ceiling() {
        // -34 dBFS in a quiet room; the -18 dB target wants it lifted.
        let mut audio = one_second(440.0, 0.02);
        let before = rms(&audio);
        Agc::default().process(&mut audio);
        let moved = moved_db(before, rms(&audio));
        assert!(
            moved > 6.0 && moved <= 21.0,
            "AGC moved {moved:.1} dB, expected a large lift inside the 20 dB ceiling"
        );
    }

    #[test]
    fn agc_stops_at_a_lower_ceiling() {
        let mut audio = one_second(440.0, 0.02);
        let before = rms(&audio);
        Agc {
            max_gain_db: 6.0,
            ..Agc::default()
        }
        .process(&mut audio);
        let moved = moved_db(before, rms(&audio));
        assert!(
            moved > 1.0 && moved < 9.0,
            "capped AGC moved {moved:.1} dB, expected roughly the 6 dB ceiling"
        );
    }

    #[test]
    fn agc_never_ducks_a_hot_take() {
        // -4 dBFS sits well above the -18 dB target; AGC only lifts.
        let mut audio = one_second(440.0, 0.63);
        let before = rms(&audio);
        Agc::default().process(&mut audio);
        assert!(
            moved_db(before, rms(&audio)) < 1.0,
            "AGC must not attenuate a hot take"
        );
    }

    #[test]
    fn aec_notches_a_sustained_feedback_tone() {
        let mut aec = Aec::default();
        let mut moved = 0.0;
        for index in 0..40 {
            let mut chunk = tone(1_000.0, 0.2, index);
            let before = rms(&chunk);
            aec.process(&mut chunk);
            moved = moved_db(before, rms(&chunk));
        }
        assert!(
            aec.band_depths().iter().any(|depth| *depth > 0.0),
            "a 1 kHz tone should engage a notch"
        );
        assert!(moved < -6.0, "feedback tone should be notched, moved {moved:.1} dB");
    }

    #[test]
    fn aec_leaves_broadband_material_alone() {
        let mut aec = Aec::default();
        let mut moved = 0.0;
        for index in 0..40 {
            let mut chunk = noise(0.15, 12_345 + index);
            let before = rms(&chunk);
            aec.process(&mut chunk);
            moved = moved_db(before, rms(&chunk));
        }
        assert!(
            aec.band_depths().iter().all(|depth| *depth <= 0.0),
            "broadband material must not open any notch"
        );
        assert!(
            moved.abs() < 1.0,
            "broadband material must pass through untouched, moved {moved:.2} dB"
        );
    }

    #[test]
    fn aec_releases_its_notches_when_the_tone_stops() {
        let mut aec = Aec::default();
        for index in 0..40 {
            aec.process(&mut tone(1_000.0, 0.2, index));
        }
        assert!(aec.band_depths().iter().any(|depth| *depth > 0.0));

        for index in 0..80 {
            aec.process(&mut noise(0.15, 777 + index));
        }
        assert!(
            aec.band_depths().iter().all(|depth| *depth <= 0.0),
            "notches should release once the feedback stops"
        );
    }
}


