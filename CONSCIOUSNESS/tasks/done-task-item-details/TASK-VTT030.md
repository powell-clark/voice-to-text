# TASK-VTT030: Simplify model menu — flat list, status display

## Context
The current tray model menu at `src/tray/linux.rs:297-383` has two separate groups labelled "Whisper.cpp" (prefix `W`) and "CTranslate2" (prefix `CT2`) with a separator between them. After the ADR-0003 migration, there is only one backend (whisper.cpp via whisper-rs), so the split is meaningless. The W/CT2 prefixes must go, and the model list must reflect the 2.0.0 offering.

## Acceptance Criteria
1. The tray model submenu shows a single flat list of radio items: `Small`, `Medium`, `Large-v3-turbo`, `Large-v3` — no prefixes, no separator, no disabled entries
2. For `Small` and `Medium`, the selected item reflects the active model family regardless of whether the underlying file is `small.en` or `small`; language mode decides the variant automatically
3. When the user selects a model that is not yet downloaded, the tray emits `UiMessage::SetStatus("Downloading <name>...")` and triggers `ensure_model` in a non-blocking thread; progress updates flow to the tray
4. When the user selects a model that is downloaded, the worker receives `WorkItem::SwitchModel(name)` and reloads within 5 seconds; tray status reflects `Loading model...` → `Ready`
5. `settings.conf` after the migration stores model names in the new format (`small`, `medium`, `large-v3-turbo`, `large-v3`) — old values `CT2 large-v3-turbo`, `W medium`, etc. are migrated to the closest new equivalent on settings load; unrecognised values fall back to `small`
6. The `is_tiny_or_base` disabled check at linux.rs:376-377 is removed — neither tiny nor base is in the new menu
7. The portable tray (`src/tray/portable.rs`) mirrors the same flat menu structure for macOS and Windows
8. Tray status text transitions cleanly: `Ready` → `Recording...` (on keypress) → `Transcribing...` (on release) → `Ready` (on completion), or `Loading model...` during a switch

## Technical Approach
Replace the two arrays:
```rust
let w_models = ["W base", "W small", "W medium", "W large"];
let ct2_models = ["CT2 base", "CT2 small", "CT2 distil-large-v3.5", "CT2 large-v3-turbo"];
```
With a single flat list:
```rust
let models = ["Small", "Medium", "Large-v3-turbo", "Large-v3"];
```

`on_model_selected` now passes the raw name (no prefix stripping); the worker handles `.en` suffixing via the language setting before calling `ensure_model` / `WhisperEngine::new`.

The settings migration in `src/settings.rs::Settings::load` inspects the loaded `selected_model` and rewrites legacy values:
```rust
settings.selected_model = match settings.selected_model.as_str() {
    s if s.starts_with("CT2 ") => migrate_legacy(s.trim_start_matches("CT2 ")),
    s if s.starts_with("W ") => migrate_legacy(s.trim_start_matches("W ")),
    _ => settings.selected_model,
};
```

## Test Strategy
Start with a `settings.conf` containing `model=CT2 large-v3-turbo`, launch VTT, verify the menu shows `Large-v3-turbo` radio-selected and the settings are saved back with the new bare name. Switch to `Small` — verify `Loading model...` appears briefly, new model loads, next transcription succeeds with the new model.

## Files
- `src/tray/linux.rs` (modify — `rebuild_model_menu`, `on_model_selected`)
- `src/tray/portable.rs` (modify — same menu structure)
- `src/settings.rs` (modify — add legacy value migration in `load`)
