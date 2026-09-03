//! Low-frequency noise suppression on the transcription path (TASK-VTT145).
//!
//! Measured on twelve of Emmanuel's own recordings, 79.5% of the noise energy
//! sits below 100 Hz and only 7.6% of the speech energy does. The interference
//! is desk and fan rumble, not broadband hiss, so a high-pass removes most of it
//! for a cost that lands almost entirely outside the 300-3400 Hz band carrying
//! intelligibility.
//!
//! Deliberately NOT spectral subtraction. That needs an FFT and introduces
//! musical noise — isolated time-frequency artefacts that sound like tones.
//! Whisper is trained on noisy speech and shrugs off steady rumble; it is far
//! less forgiving of artefacts that resemble speech. Subtraction is filed as a
//! follow-up to be justified by measurement rather than assumed.
//!
//! This runs on the samples bound for Whisper only. Archived audio stays
//! unfiltered: it is voice-clone training data, and baking a filter into a
//! corpus cannot be undone later.

/// One biquad section in direct form I, with its own state.
#[derive(Debug, Clone, Copy)]
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
    /// RBJ cookbook high-pass, normalised by a0.
    fn high_pass(cutoff_hz: f32, sample_rate: f32, q: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * cutoff_hz / sample_rate;
        let (sin_w0, cos_w0) = w0.sin_cos();
        let alpha = sin_w0 / (2.0 * q);
        let a0 = 1.0 + alpha;
        Self {
            b0: ((1.0 + cos_w0) / 2.0) / a0,
            b1: (-(1.0 + cos_w0)) / a0,
            b2: ((1.0 + cos_w0) / 2.0) / a0,
            a1: (-2.0 * cos_w0) / a0,
            a2: (1.0 - alpha) / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// Default corner frequency. Chosen against the measured profile: high enough to
/// cut the band holding four fifths of the noise, low enough to sit under a male
/// fundamental (roughly 85-180 Hz) rather than through it.
pub const DEFAULT_CUTOFF_HZ: f32 = 90.0;

/// Fourth-order Butterworth high-pass — two cascaded biquads at the Butterworth
/// section Q values. Second order alone reaches only about 12 dB at 50 Hz;
/// doubling it puts deep rumble more than 20 dB down while leaving 300 Hz and
/// above untouched.
pub struct HighPass {
    /// `None` when the requested corner cannot be realised at this sample rate,
    /// in which case the filter passes audio through untouched. Silence is the
    /// wrong failure here: `2*PI*cutoff/0` is infinity, `sin_cos` of infinity is
    /// NaN, and NaN coefficients turn every sample into NaN — Whisper would
    /// receive NaN audio rather than a clean error (TASK-VTT159).
    sections: Option<[Biquad; 2]>,
}

impl HighPass {
    /// Build a filter for one recording. Coefficients derive from `sample_rate`,
    /// so the same code is correct whether it runs on a 16 kHz or a 48 kHz path.
    ///
    /// Construct one per recording and drop it afterwards: biquad state carried
    /// across recordings would leak the tail of one into the head of the next.
    pub fn new(cutoff_hz: f32, sample_rate: u32) -> Self {
        let fs = sample_rate as f32;
        // A corner at or above Nyquist is not a high-pass — the coefficients
        // come out with gain above 1 and the filter amplifies instead of
        // attenuating (measured b0 = 1.98 at 100 Hz). Rate 0 is worse still.
        // Neither can arise from the direct capture path, but `capture_rate` is
        // read from the device on the native-format fallback and nothing
        // guarantees it is sane.
        let realisable =
            sample_rate > 0 && cutoff_hz > 0.0 && cutoff_hz * 2.0 < fs && cutoff_hz.is_finite();
        Self {
            sections: realisable.then(|| {
                [
                    Biquad::high_pass(cutoff_hz, fs, 0.541_196_1),
                    Biquad::high_pass(cutoff_hz, fs, 1.306_562_9),
                ]
            }),
        }
    }

    fn sample(&mut self, x: f32) -> f32 {
        let Some(sections) = self.sections.as_mut() else {
            return x;
        };
        let mut y = x;
        for s in sections {
            y = s.process(y);
        }
        y
    }

    /// Filter a whole recording. Returns a new buffer; the input is left alone
    /// because the caller may still need the unfiltered audio for the archive.
    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        input.iter().map(|&x| self.sample(x)).collect()
    }
}

/// Convenience for the common case: a fresh filter at the default corner over
/// one recording.
pub fn suppress_rumble(input: &[f32], sample_rate: u32) -> Vec<f32> {
    HighPass::new(DEFAULT_CUTOFF_HZ, sample_rate).process(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RMS of a signal, ignoring the settling transient at the head.
    fn rms(xs: &[f32]) -> f32 {
        let skip = xs.len() / 4;
        let tail = &xs[skip..];
        (tail.iter().map(|x| x * x).sum::<f32>() / tail.len() as f32).sqrt()
    }

    fn tone(freq: f32, sample_rate: u32, secs: f32) -> Vec<f32> {
        let n = (sample_rate as f32 * secs) as usize;
        (0..n)
            .map(|i| {
                (2.0 * std::f32::consts::PI * freq * i as f32 / sample_rate as f32).sin() * 0.5
            })
            .collect()
    }

    /// Attenuation in dB the filter applies to a pure tone.
    fn attenuation_db(freq: f32, sample_rate: u32) -> f32 {
        let input = tone(freq, sample_rate, 1.0);
        let output = suppress_rumble(&input, sample_rate);
        20.0 * (rms(&output).max(1e-9) / rms(&input)).log10()
    }

    #[test]
    fn deep_rumble_is_cut_hard() {
        // 50 Hz is where desk and fan rumble concentrates. The card's gate is
        // 12 dB; a fourth-order corner at 90 Hz clears that comfortably.
        let att = attenuation_db(50.0, 16_000);
        assert!(
            att <= -12.0,
            "50 Hz should be at least 12 dB down, got {att:.1} dB"
        );
    }

    #[test]
    fn the_intelligibility_band_passes_untouched() {
        // 300-3400 Hz carries the formants Whisper actually reads. Anything
        // more than 1 dB here is the filter eating speech.
        for freq in [300.0, 1000.0, 3000.0] {
            let att = attenuation_db(freq, 16_000);
            assert!(
                att.abs() <= 1.0,
                "{freq} Hz should pass within 1 dB, got {att:.1} dB"
            );
        }
    }

    #[test]
    fn the_corner_sits_below_a_male_fundamental() {
        // Emmanuel's fundamental runs roughly 85-180 Hz. 150 Hz must survive
        // nearly intact or the filter thins his voice rather than the fan.
        let att = attenuation_db(150.0, 16_000);
        assert!(
            att >= -3.0,
            "150 Hz should lose under 3 dB, got {att:.1} dB"
        );
    }

    #[test]
    fn coefficients_track_the_sample_rate() {
        // The same corner must behave the same way at 48 kHz as at 16 kHz —
        // a filter that hardcoded one rate would attenuate 50 Hz differently.
        let at_16k = attenuation_db(50.0, 16_000);
        let at_48k = attenuation_db(50.0, 48_000);
        assert!(
            (at_16k - at_48k).abs() < 2.0,
            "50 Hz attenuation should match across rates: 16k {at_16k:.1} dB vs 48k {at_48k:.1} dB"
        );
    }

    #[test]
    fn dc_offset_is_removed() {
        // A constant offset is the limiting case of rumble, and some interfaces
        // deliver one. It must not survive.
        let input = vec![0.4f32; 16_000];
        let out = suppress_rumble(&input, 16_000);
        assert!(
            rms(&out) < 0.01,
            "DC should be flattened, got rms {}",
            rms(&out)
        );
    }

    #[test]
    fn a_zero_sample_rate_passes_audio_through_rather_than_nan() {
        // 2*PI*cutoff/0 is infinity and sin_cos of infinity is NaN, so an
        // unguarded filter turns every sample into NaN and hands Whisper NaN
        // audio instead of failing cleanly.
        let input: Vec<f32> = (0..64).map(|i| (i as f32 / 64.0) - 0.5).collect();
        let out = suppress_rumble(&input, 0);
        assert!(
            out.iter().all(|s| s.is_finite()),
            "no NaN may reach Whisper"
        );
        assert_eq!(out, input, "an unrealisable filter must pass audio through");
    }

    #[test]
    fn a_corner_above_nyquist_passes_through_rather_than_amplifying() {
        // At 100 Hz the 90 Hz corner sits above Nyquist and the coefficients
        // come out with gain above 1 — the filter would amplify what it is
        // meant to remove.
        let input: Vec<f32> = (0..64).map(|i| ((i % 8) as f32 / 8.0) - 0.5).collect();
        for fs in [100, 180] {
            let out = suppress_rumble(&input, fs);
            assert_eq!(out, input, "fs={fs} cannot realise a 90 Hz corner");
        }
    }

    #[test]
    fn the_lowest_realisable_rate_still_filters() {
        // Just above 2x the corner must still build a real filter, so the guard
        // is not quietly disabling the feature at ordinary rates.
        let input: Vec<f32> = (0..2000).map(|i| (i as f32 / 50.0).sin() * 0.4).collect();
        let out = suppress_rumble(&input, 8_000);
        assert!(out.iter().all(|s| s.is_finite()));
        assert_ne!(out, input, "8 kHz is realisable and must actually filter");
    }

    #[test]
    fn empty_input_is_empty_output() {
        assert!(suppress_rumble(&[], 16_000).is_empty());
    }

    #[test]
    fn output_stays_finite_and_bounded() {
        // A filter that rings or blows up would hand Whisper NaNs and turn a
        // noisy transcription into no transcription at all.
        let mut input = tone(200.0, 16_000, 0.5);
        input.extend(tone(60.0, 16_000, 0.5));
        let out = suppress_rumble(&input, 16_000);
        assert!(out.iter().all(|s| s.is_finite()), "no NaN or inf");
        assert!(
            out.iter().all(|s| s.abs() < 2.0),
            "no runaway gain from the cascade"
        );
    }

    #[test]
    fn each_recording_starts_from_clean_state() {
        // State carried across recordings would leak the tail of one into the
        // head of the next. Two identical inputs must filter identically.
        let input = tone(500.0, 16_000, 0.2);
        let a = suppress_rumble(&input, 16_000);
        let b = suppress_rumble(&input, 16_000);
        assert_eq!(a, b);
    }
}
