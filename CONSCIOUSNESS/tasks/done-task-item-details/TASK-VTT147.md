# TASK-VTT147: Warn when the hotkey is an ordinary typing key

## Context

This is the trigger behind the 2026-08-18 incident, as distinct from the code
defects it exposed (TASK-VTT146, TASK-VTT148).

The hotkey capture dialog accepted any keycode in 8..=255 and bound it on
release. Nothing checked whether the key was one the user types with. The
operator's settings.conf ended up carrying `hotkey=65`, which is space.

Push-to-talk grabs its key globally with XGrabKey, so the grab swallows every
press before the focused window sees it. Binding space therefore did two
things at once: every space keypress opened a recording, and no space
character reached any application. The operator's own bug report arrived as

    Itshouldn'tbeSpacebar,itshouldbeSkrolok

with every space missing, repeated four times by the wedge. Space and Return
are also exactly the keys that activate a focused GTK widget, so capturing one
by accident in that dialog is easy.

The code defects were latent from the Rust rewrite and could not fire while
the hotkey was Scroll Lock, which never coincides with typing. Binding space
raised press-during-typing from rare to constant, which is what finally tripped
them. Fixing the defects without fixing the binding would leave the trigger
armed.

## Acceptance criteria

- [x] `hotkey::is_typing_key` classifies a keysym as typing or bindable, as a
      pure function unit-testable without an X server
- [x] Latin-1 printable keysyms (0x20..=0xFF) are rejected — space, letters,
      digits, punctuation
- [x] The editing keys are rejected too, since they sit outside that range but
      are just as destructive to grab: Return, KP_Enter, Tab, BackSpace,
      Escape, Delete
- [x] Scroll Lock, Pause, Num Lock, Caps Lock, Insert, Home, Print and the
      function keys stay bindable
- [x] `hotkey::keycode_is_typing` resolves a hardware keycode against the live
      keymap, so the answer follows the operator's actual layout; an
      unresolvable keycode is treated as safe rather than blocking the bind
- [x] The capture dialog refuses a typing key and says why, naming the key and
      suggesting Scroll Lock, Pause or a function key
- [x] Startup warns when an existing settings.conf already carries a typing
      key, since the dialog check cannot help a config written before it
- [x] Verified live: launching against the operator's saved `hotkey=65` config
      logs "WARNING: hotkey keycode 65 is 'space', a key you type with"
- [x] Full suite green — 132 passed, 0 failed

## Notes

Refusal rather than a confirmation prompt, because the failure is silent and
self-concealing: once the key is bound, the user has lost the character they
would need to type their way out, and the only recovery is hand-editing
settings.conf.

## Dependencies

- Story: STORY-VTT015
- Directive: DIRECT-VTT002
