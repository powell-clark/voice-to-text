# TASK-VTT086: Fix portable tray model submenu — legacy names don't match catalogue

## Context

Found during TASK-VTT082 (Windows on-hardware bring-up, 2026-06-25). The portable
tray (`src/tray/portable.rs`, used on Windows + macOS) hardcodes a model submenu
list that no longer matches the real model catalogue in `src/models.rs`.

Tray menu offers (portable.rs:81-90):
`"W base", "W small", "W medium", "W large", "CT2 base", "CT2 small",
"CT2 distil-large-v3.5", "CT2 large-v3-turbo"`

Actual catalogue (`models::MODELS`):
`small.en, small, medium.en, medium, large-v3-turbo, large-v3`

The "W"/"CT2" prefixes are pre-v2.0 backend labels (whisper.cpp vs CTranslate2)
that were removed when v2.0 went single-backend in-process whisper-rs. Selecting
any of these menu entries sets `settings.selected_model` to a string that
`models::find()` cannot resolve, so the worker silently falls back to the default
model. The user believes they switched models; nothing changed.

The Linux GTK tray (`src/tray/linux.rs`) builds its model list correctly — this
defect is portable-tray-only (Windows + macOS).

## Acceptance criteria

- [ ] Portable tray model submenu is generated from `models::MODELS`, not a
      hardcoded list — adding/removing a catalogue model updates the menu
- [ ] Selecting any model in the submenu sets a name that `models::find()` resolves
- [ ] The currently-selected model shows checked on menu build
- [ ] A unit/integration test asserts every portable-tray model label resolves via
      `models::find()` (guards against future drift) — see TASK-VTT087

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT004
- Found-by: TASK-VTT082
