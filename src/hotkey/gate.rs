//! Push-to-talk state machine, lifted out of the hotkey callback so the
//! press/release decisions are unit-testable without an X server.
//!
//! The callback runs on the single X11 event thread (see `linux.rs`), which is
//! also the thread that delivers KeyRelease. Anything that blocks or discards
//! there costs the user an open microphone, so the decisions live here where
//! they can be exercised directly.

use std::time::Duration;

/// Longest a single hold may last before the watchdog force-stops it. A hold
/// past this is a lost KeyRelease, not someone still talking — `audio.rs` caps
/// the buffer at the same 300s, so nothing is captured by staying open and an
/// open mic is the worst failure this tool has (TASK-VTT146).
pub const MAX_HOLD: Duration = Duration::from_secs(300);

/// What the caller should do with the capture stream in response to an event.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Action {
    /// Open the microphone.
    Start,
    /// Close the microphone and hand the audio downstream.
    Stop,
    /// The event carries no state change.
    Ignore,
}

/// Tracks whether a push-to-talk hold is in progress.
#[derive(Debug, Default)]
pub struct PushToTalk {
    recording: bool,
}

impl PushToTalk {
    pub fn new() -> Self {
        Self { recording: false }
    }

    pub fn is_recording(&self) -> bool {
        self.recording
    }

    /// A press starts a recording unless one is already running. Auto-repeat
    /// re-presses land here while `recording` is true and are ignored.
    pub fn press(&mut self) -> Action {
        if self.recording {
            return Action::Ignore;
        }
        self.recording = true;
        Action::Start
    }

    /// A release always ends the recording.
    ///
    /// `held_for` is accepted so the caller can log it, but it deliberately
    /// does not gate the decision. The previous implementation dropped any
    /// release arriving within 150ms of the press as an auto-repeat artefact;
    /// when the press handler blocked waiting on typing, the genuine release
    /// queued behind it and arrived at ~0ms, so it was discarded and the flag
    /// stayed true with no path back (TASK-VTT146). Auto-repeat is already
    /// suppressed at the X11 layer by XkbSetDetectableAutoRepeat plus the
    /// paired-event peek in `linux.rs`, and a genuinely brief tap is discarded
    /// downstream by `audio.rs`'s MIN_DURATION_SECS check — which closes the
    /// microphone instead of leaving it open.
    pub fn release(&mut self, _held_for: Duration) -> Action {
        if !self.recording {
            return Action::Ignore;
        }
        self.recording = false;
        Action::Stop
    }

    /// Watchdog tick. Force-stops a hold that has outlived [`MAX_HOLD`], so a
    /// release lost anywhere below this layer still cannot strand the mic.
    pub fn poll(&mut self, held_for: Duration) -> Action {
        if self.recording && held_for >= MAX_HOLD {
            self.recording = false;
            return Action::Stop;
        }
        Action::Ignore
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn press_starts_a_recording() {
        let mut gate = PushToTalk::new();
        assert_eq!(gate.press(), Action::Start);
        assert!(gate.is_recording());
    }

    #[test]
    fn repeat_press_while_recording_is_ignored() {
        let mut gate = PushToTalk::new();
        gate.press();
        assert_eq!(gate.press(), Action::Ignore);
        assert!(gate.is_recording());
    }

    #[test]
    fn release_stops_the_recording() {
        let mut gate = PushToTalk::new();
        gate.press();
        assert_eq!(gate.release(Duration::from_secs(2)), Action::Stop);
        assert!(!gate.is_recording());
    }

    /// The TASK-VTT146 regression. The press handler used to block up to 30s
    /// waiting for the previous transcription to finish typing, all on the X11
    /// event thread, so the user's real release sat in the queue and was
    /// delivered the instant the wait ended — measured 0ms after the recording
    /// start. The old guard read that as auto-repeat and dropped it, wedging
    /// the mic open with no key able to close it.
    #[test]
    fn release_arriving_instantly_still_stops_the_recording() {
        let mut gate = PushToTalk::new();
        gate.press();
        assert_eq!(gate.release(Duration::ZERO), Action::Stop);
        assert!(
            !gate.is_recording(),
            "a lost release strands the microphone"
        );
    }

    #[test]
    fn release_without_a_recording_is_ignored() {
        let mut gate = PushToTalk::new();
        assert_eq!(gate.release(Duration::from_secs(1)), Action::Ignore);
    }

    #[test]
    fn a_wedged_recording_can_never_block_the_next_one() {
        let mut gate = PushToTalk::new();
        gate.press();
        gate.release(Duration::ZERO);
        assert_eq!(gate.press(), Action::Start);
    }

    #[test]
    fn watchdog_force_stops_a_hold_past_the_cap() {
        let mut gate = PushToTalk::new();
        gate.press();
        assert_eq!(gate.poll(MAX_HOLD), Action::Stop);
        assert!(!gate.is_recording());
    }

    #[test]
    fn watchdog_leaves_a_hold_under_the_cap_alone() {
        let mut gate = PushToTalk::new();
        gate.press();
        assert_eq!(gate.poll(MAX_HOLD - Duration::from_secs(1)), Action::Ignore);
        assert!(gate.is_recording());
    }

    #[test]
    fn watchdog_is_inert_when_idle() {
        let mut gate = PushToTalk::new();
        assert_eq!(gate.poll(MAX_HOLD * 2), Action::Ignore);
    }
}
