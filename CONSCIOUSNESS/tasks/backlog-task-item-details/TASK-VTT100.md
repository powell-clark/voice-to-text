# TASK-VTT100: Windows — suppress hotkey auto-repeat (rdev)

Parity with Linux FEAT-VTT013. rdev fired Down on every OS-repeated KeyPress; an
F-key/letter hotkey would re-fire while held. Now tracks pressed state (AtomicBool
swap) so only the first press fires Down and only a real release fires Up.

- [x] Holding the hotkey yields exactly one Down/Up pair
- [ ] Verified on a non-toggle hotkey (e.g. F4) on Windows
- Story: STORY-VTT013 · Directive: DIRECT-VTT004 · Parity §4
