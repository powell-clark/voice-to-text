//! Rolling segments for a held key (TASK-VTT195).
//!
//! A hold used to stop growing at five minutes, and everything said after
//! that was lost. Now the capture is cut into segments while the key stays
//! down: once a segment passes the threshold it ends at the next pause, and a
//! segment still without a pause a minute later is cut at the quietest
//! stretch in that minute. Every captured sample lands in exactly one
//! segment, so nothing is lost or repeated across a cut.
//!
//! Pure: no audio device and no engine, so the cut rules are unit-testable.
//! The live capture and `--file` both feed audio a second at a time, so they
//! cut the same audio in the same places.

/// A segment ends at the first pause after this much audio.
pub const THRESHOLD_SECS: f32 = 240.0;
/// How long past the threshold a segment waits for a pause. With none by
/// then, it is cut at the quietest stretch in that wait.
const PAUSE_WAIT_SECS: f32 = 60.0;
/// Environment override for the threshold, in seconds, so a test can cut a
/// short capture into many segments. Unset in normal use.
const THRESHOLD_ENV: &str = "VTT_SEGMENT_SECS";
/// Shortest silence taken as a pause. The silent closure inside a word lasts
/// well under this, so a cut never lands mid-word.
const PAUSE_MS: usize = 300;
/// Energy is measured over frames of this length.
const FRAME_MS: usize = 20;
/// A frame is quiet under this many times the segment's noise floor (6 dB)...
const FLOOR_RATIO: f64 = 4.0;
/// ...and under this fraction of its loudest speech (-10 dB), so a noisy room
/// does not raise the bar into speech.
const SPEECH_RATIO: f64 = 0.1;
/// Mean-square energy every frame of true digital silence is under.
const MIN_QUIET: f64 = 1e-8;

/// Where cuts may fall, in frames of `frame` samples.
#[derive(Debug, Clone, Copy)]
pub struct Policy {
    frame: usize,
    threshold: usize,
    cap: usize,
    pause: usize,
}

impl Policy {
    /// Cut audio at `rate` Hz at the first pause after `threshold_secs`, and
    /// `PAUSE_WAIT_SECS` later at the latest.
    pub fn new(rate: u32, threshold_secs: f32) -> Self {
        let frames = |secs: f32| ((secs.max(0.0) * 1000.0) as usize / FRAME_MS).max(1);
        let threshold = frames(threshold_secs);
        Self {
            frame: (rate as usize * FRAME_MS / 1000).max(1),
            threshold,
            cap: frames(threshold_secs + PAUSE_WAIT_SECS).max(threshold + 1),
            pause: PAUSE_MS / FRAME_MS,
        }
    }

    /// The threshold in seconds, for log lines.
    pub fn threshold_secs(&self) -> f32 {
        (self.threshold * FRAME_MS) as f32 / 1000.0
    }
}

/// The threshold holds and `--file` run on: `THRESHOLD_SECS`, or the
/// override in `THRESHOLD_ENV` when it is set to a usable number of seconds.
pub fn live_threshold_secs() -> f32 {
    threshold_from(std::env::var(THRESHOLD_ENV).ok().as_deref())
}

/// Pure: the threshold `THRESHOLD_ENV` asks for, or `THRESHOLD_SECS` when it
/// is unset, unparsable or under one second.
fn threshold_from(value: Option<&str>) -> f32 {
    value
        .and_then(|v| v.trim().parse::<f32>().ok())
        .filter(|secs| secs.is_finite() && *secs >= 1.0)
        .unwrap_or(THRESHOLD_SECS)
}

/// The audio of one hold that has not yet been cut off as a segment.
pub struct Segmenter {
    policy: Policy,
    pending: Vec<f32>,
    /// Mean-square energy of each complete frame of `pending`.
    energy: Vec<f64>,
}

impl Segmenter {
    pub fn new(policy: Policy) -> Self {
        Self {
            policy,
            pending: Vec::new(),
            energy: Vec::new(),
        }
    }

    /// Append captured audio.
    pub fn push(&mut self, samples: &[f32]) {
        self.pending.extend_from_slice(samples);
        let f = self.policy.frame;
        let complete = self.pending.len() / f;
        for i in self.energy.len()..complete {
            let frame = &self.pending[i * f..(i + 1) * f];
            let sum: f64 = frame.iter().map(|&s| f64::from(s) * f64::from(s)).sum();
            self.energy.push(sum / f as f64);
        }
    }

    /// The next segment, when one is due, leaving the audio after the cut
    /// pending. None while the pending audio is under the threshold, or past
    /// it and waiting for a pause.
    pub fn take_due(&mut self) -> Option<Vec<f32>> {
        let cut = find_cut(&self.energy, &self.policy)?;
        let rest = self.pending.split_off(cut * self.policy.frame);
        self.energy.drain(..cut);
        Some(std::mem::replace(&mut self.pending, rest))
    }

    /// Everything not yet cut off: the hold's last segment.
    pub fn finish(self) -> Vec<f32> {
        self.pending
    }
}

/// Pure: the frame to cut at, if a cut is due. That is the middle of the
/// first pause reaching past the threshold, or, with none by the cap, the
/// middle of the quietest pause-length stretch between threshold and cap.
fn find_cut(energy: &[f64], p: &Policy) -> Option<usize> {
    if energy.len() < p.threshold {
        return None;
    }
    let quiet = quiet_level(energy);
    let end = energy.len().min(p.cap);
    let mut run_start = None;
    for (i, &e) in energy.iter().enumerate().take(end) {
        if e >= quiet {
            run_start = None;
            continue;
        }
        let start = *run_start.get_or_insert(i);
        if i + 1 - start >= p.pause && i >= p.threshold {
            // A pause long enough, reaching past the threshold. A pause that
            // began well before the threshold is cut near the threshold, not
            // at its own start; one still going is cut a pause-length in.
            let run_end = (i..energy.len())
                .find(|&j| energy[j] >= quiet)
                .unwrap_or(energy.len());
            let from = start.max(p.threshold.saturating_sub(p.pause));
            let cut = from + (run_end - from).min(2 * p.pause) / 2;
            return Some(cut.clamp(1, p.cap));
        }
    }
    if energy.len() < p.cap {
        return None;
    }
    Some(quietest_stretch(energy, p.threshold, p.cap, p.pause))
}

/// Pure: the energy under which a frame counts as silence in this segment,
/// set from its own noise floor and loudest speech.
fn quiet_level(energy: &[f64]) -> f64 {
    let mut sorted = energy.to_vec();
    sorted.sort_by(f64::total_cmp);
    let floor = sorted[sorted.len() / 10];
    let loud = sorted[sorted.len() * 99 / 100];
    (floor * FLOOR_RATIO)
        .min(loud * SPEECH_RATIO)
        .max(MIN_QUIET)
}

/// Pure: the middle of the `width`-frame stretch with the least energy that
/// lies in `from..to`; the earliest wins a tie.
fn quietest_stretch(energy: &[f64], from: usize, to: usize, width: usize) -> usize {
    let width = width.min(to - from).max(1);
    let mut best = (from, f64::INFINITY);
    for start in from..=to - width {
        let sum: f64 = energy[start..start + width].iter().sum();
        if sum < best.1 {
            best = (start, sum);
        }
    }
    best.0 + width / 2
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: u32 = 16_000;
    /// The shrunk threshold every test cuts at.
    const TEST_SECS: f32 = 4.0;

    fn secs(s: f32) -> usize {
        (s * RATE as f32) as usize
    }

    /// Deterministic pseudo-random numbers, so a failing case replays.
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> f32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((self.0 >> 33) as f32) / (1u64 << 31) as f32
        }
        fn between(&mut self, lo: f32, hi: f32) -> f32 {
            lo + (hi - lo) * self.next()
        }
    }

    /// A faint room-noise sample.
    fn floor(rng: &mut Lcg) -> f32 {
        rng.between(-0.002, 0.002)
    }

    /// Speech-like audio: words of noisy tone separated by short gaps, and a
    /// longer pause after every sentence of a few words, over a faint noise
    /// floor. Returns the audio and the sample span of every word.
    fn dictation(total_secs: f32, seed: u64) -> (Vec<f32>, Vec<(usize, usize)>) {
        let mut rng = Lcg(seed);
        let total = secs(total_secs);
        let mut audio = Vec::with_capacity(total);
        let mut words = Vec::new();
        while audio.len() < total {
            let in_sentence = 3 + (rng.next() * 9.0) as usize;
            for w in 0..in_sentence {
                let start = audio.len();
                let len = secs(rng.between(0.15, 0.6));
                let pitch = rng.between(120.0, 300.0);
                let loudness = rng.between(0.08, 0.5);
                for i in 0..len {
                    let t = i as f32 / RATE as f32;
                    let tone = (t * pitch * std::f32::consts::TAU).sin();
                    audio.push(loudness * (0.8 * tone + 0.2 * rng.between(-1.0, 1.0)));
                }
                words.push((start, audio.len()));
                // The gap after a word: a closure inside a sentence, never as
                // long as a pause; a pause after the sentence's last word.
                let gap = if w + 1 == in_sentence {
                    secs(rng.between(0.45, 1.2))
                } else {
                    secs(rng.between(0.02, 0.15))
                };
                for _ in 0..gap {
                    audio.push(floor(&mut rng));
                }
            }
        }
        audio.truncate(total);
        (audio, words)
    }

    /// Feed `audio` the way the live capture does, a second at a time, taking
    /// every segment as it falls due; the remainder is the last segment.
    fn segment(audio: &[f32], policy: Policy) -> Vec<Vec<f32>> {
        let mut seg = Segmenter::new(policy);
        let mut out = Vec::new();
        for chunk in audio.chunks(RATE as usize) {
            seg.push(chunk);
            while let Some(s) = seg.take_due() {
                out.push(s);
            }
        }
        out.push(seg.finish());
        out
    }

    fn cut_points(segments: &[Vec<f32>]) -> Vec<usize> {
        let mut at = 0;
        segments[..segments.len() - 1]
            .iter()
            .map(|s| {
                at += s.len();
                at
            })
            .collect()
    }

    fn tone(len: usize, level: f32) -> Vec<f32> {
        (0..len).map(|i| level * (i as f32 * 0.07).sin()).collect()
    }

    #[test]
    fn nothing_is_cut_under_the_threshold() {
        let (audio, _) = dictation(TEST_SECS - 0.1, 1);
        let segments = segment(&audio, Policy::new(RATE, TEST_SECS));
        assert_eq!(segments.len(), 1);
        assert!(segments[0] == audio);
    }

    #[test]
    fn production_threshold_is_four_minutes_with_a_five_minute_cap() {
        let p = Policy::new(RATE, THRESHOLD_SECS);
        assert_eq!(p.threshold_secs(), 240.0);
        assert_eq!(p.cap * FRAME_MS, 300_000);
    }

    #[test]
    fn a_ten_minute_hold_is_cut_without_losing_or_repeating_a_sample() {
        // The 10-minute criterion at the shrunk threshold: over a hundred
        // cuts, and the segments put back together are the capture, bit for bit.
        let (audio, _) = dictation(600.0, 7);
        let policy = Policy::new(RATE, TEST_SECS);
        let segments = segment(&audio, policy);
        assert!(segments.len() > 100, "{} segments", segments.len());
        let joined: Vec<f32> = segments.concat();
        assert_eq!(joined.len(), audio.len(), "samples lost or added");
        assert!(joined == audio, "a sample changed or moved across a cut");
        for s in &segments[..segments.len() - 1] {
            assert!(s.len() >= secs(TEST_SECS - 0.3), "cut too early");
            assert!(s.len() <= policy.cap * policy.frame, "segment past the cap");
        }
    }

    #[test]
    fn no_word_is_split_across_two_segments() {
        // Speech running up to and past the threshold, on many voices: every
        // cut must fall between words, never inside one.
        for seed in 1..=40 {
            let (audio, words) = dictation(60.0, seed);
            let segments = segment(&audio, Policy::new(RATE, TEST_SECS));
            assert!(
                segments.len() > 5,
                "seed {seed}: {} segments",
                segments.len()
            );
            for cut in cut_points(&segments) {
                if let Some(&(s, e)) = words.iter().find(|&&(s, e)| s < cut && cut < e) {
                    panic!("seed {seed}: cut at {cut} splits the word {s}..{e}");
                }
            }
        }
    }

    #[test]
    fn the_cut_is_the_first_pause_after_the_threshold() {
        // Unbroken speech with two pauses: one before the threshold, which
        // must be passed over, and one after it, which must be taken.
        let mut audio = tone(secs(2.0), 0.3);
        audio.extend(vec![0.0005; secs(0.6)]); // 2.0..2.6 s
        audio.extend(tone(secs(2.0), 0.3));
        let pause = (audio.len(), audio.len() + secs(0.5)); // 4.6..5.1 s
        audio.extend(vec![0.0005; secs(0.5)]);
        audio.extend(tone(secs(2.0), 0.3));
        let segments = segment(&audio, Policy::new(RATE, TEST_SECS));
        let cuts = cut_points(&segments);
        assert_eq!(cuts.len(), 1, "cuts at {cuts:?}");
        assert!(
            pause.0 < cuts[0] && cuts[0] < pause.1,
            "cut at {} outside the pause {pause:?}",
            cuts[0]
        );
    }

    #[test]
    fn with_no_pause_the_cut_is_the_quietest_stretch_by_the_cap() {
        // Unbroken speech for over a minute: no pause anywhere, only a softer
        // stretch after the threshold. The hard cut must land in it, and no
        // later than the cap.
        let policy = Policy::new(RATE, TEST_SECS);
        let soft = (secs(30.0), secs(30.5));
        let audio: Vec<f32> = (0..secs(70.0))
            .map(|i| {
                let level = if (soft.0..soft.1).contains(&i) {
                    0.2
                } else {
                    0.4
                };
                level * (i as f32 * 0.05).sin().signum()
            })
            .collect();
        let segments = segment(&audio, policy);
        let cuts = cut_points(&segments);
        assert_eq!(cuts.len(), 1, "cuts at {cuts:?}");
        assert!(
            soft.0 <= cuts[0] && cuts[0] <= soft.1,
            "hard cut at {} outside the soft stretch {soft:?}",
            cuts[0]
        );
        assert!(cuts[0] <= policy.cap * policy.frame);
    }

    #[test]
    fn a_silent_hold_is_cut_at_the_threshold() {
        let segments = segment(&vec![0.0; secs(11.0)], Policy::new(RATE, TEST_SECS));
        assert_eq!(segments.len(), 3, "{} segments", segments.len());
        assert_eq!(segments[0].len(), secs(TEST_SECS));
        assert_eq!(segments.concat().len(), secs(11.0));
    }

    #[test]
    fn the_threshold_override_needs_a_number_of_seconds() {
        assert_eq!(threshold_from(None), THRESHOLD_SECS);
        assert_eq!(threshold_from(Some("4")), 4.0);
        assert_eq!(threshold_from(Some(" 12.5 ")), 12.5);
        assert_eq!(threshold_from(Some("0.5")), THRESHOLD_SECS);
        assert_eq!(threshold_from(Some("-3")), THRESHOLD_SECS);
        assert_eq!(threshold_from(Some("four")), THRESHOLD_SECS);
        assert_eq!(threshold_from(Some("inf")), THRESHOLD_SECS);
    }
}
