# TASK-VTT101: Windows installer pre-provisions the default model (parity FEAT-VTT028) — OPERATOR-DECISION-PENDING (needs a Windows machine to verify)

Linux postinst downloads ggml-small.en at install so first run is offline-ready. The
Windows MSI installs without the model, so first launch downloads ~465 MB. Add
installer-time or first-run provisioning with the same offline-first guarantee.

## Investigation (2026-09-05, vtt-c52f564e)

The provisioning already exists as shared, cross-platform code — nothing
Windows-specific needs adding:

- `models::ensure()` (`src/models.rs:149`) downloads the missing model into
  `dirs::cache_dir()` (`dirs = "5"`, unconditional dependency — resolves to
  `%LOCALAPPDATA%` on Windows) with a `(done, total)` progress callback,
  SHA-256 verified before the file is promoted.
- The live path, `load_engine` (`src/main.rs:1001`), wires that callback to
  `tray::UiMessage::SetStatus("Downloading {name}... {pct}%")`.
- `src/tray/portable.rs:413` (the shared Windows+macOS tray) handles
  `SetStatus` by writing the tray menu's Status line AND the tray icon
  tooltip, and turns the icon amber while loading — so the download is
  visible, not silent, satisfying the "OR clearly fetched first-run with
  progress" branch of criterion 1 and the "not a surprise" framing of
  criterion 2.
- The download stack is Windows-safe: `reqwest` uses `rustls-tls` (no OpenSSL
  dependency to provision on the build or target machine).

So this is not an implementation gap — re-adding a second download path would
duplicate `models::ensure()` for no reason. What is NOT yet proven is the
runtime behaviour on an actual Windows install, which per STORY-VTT013's own
standard ("designed-in is theory, a green Windows build is actual proof") is
exactly the kind of claim this project does not accept from code-reading
alone. Confirmed today: `build (windows-latest, x86_64-msvc, Vulkan whisper)`
and `build (windows-11-arm, aarch64-msvc, CPU whisper)` are both green on the
latest `main` CI run (`gh run view 33952807287`), so TASK-VTT063/064's compile
gate is real — but CI does not install the MSI or run a live first launch, so
it does not touch this task's criteria.

## Acceptance criteria

- [ ] On a real Windows machine: install the current MSI (or run the exe with
      no model cached), delete/rename any cached model, launch, hold the
      hotkey once — confirm the tray tooltip/menu shows
      "Downloading small.en... N%" during the fetch (not a frozen/blank UI)
- [ ] Confirm the resulting transcription succeeds once the download
      completes, with no separate error path taken
- [ ] If either observation differs from the Linux/mac behaviour, file the
      gap as a new task citing the exact tray/log output — do not patch this
      card in place once it is closed

## Dependencies
- Story: STORY-VTT013 · Directive: DIRECT-VTT004 · Parity §6
