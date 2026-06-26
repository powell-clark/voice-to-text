# Feature parity matrix — Linux · macOS · Windows

Tracks which capabilities are present, partial, or missing per platform, so
parity gaps are explicit and specced rather than discovered by surprise in use.
Created 2026-06-26 from Emmanuel's Windows bring-up testing.

Legend: ✅ done · 🟡 partial / unverified · ❌ missing · n/a not applicable

| # | Capability | Linux | macOS | Windows | Tracking |
|---|------------|:-----:|:-----:|:-------:|----------|
| 1 | Push-to-talk hotkey (Scroll Lock) | ✅ | 🟡 | ✅ | — |
| 2 | In-process Whisper transcription | ✅ | ✅ | ✅ | — |
| 3 | GPU acceleration | ✅ Vulkan | ✅ Metal | ✅ Vulkan | FEAT-VTT024 (Windows done) |
| 4 | Text injection (typing) | ✅ xdotool | 🟡 enigo, unverified | ✅ enigo.text | TASK-VTT092 |
| 5 | System-tray icon visible | ✅ GTK | 🟡 | ✅ | — |
| 6 | Tray context menu opens | ✅ | 🟡 unverified | ✅ (msg pump) | TASK-VTT091 |
| 7 | Tray icon reflects state (idle/recording/processing) | 🟡 | ❌ | ❌ | TASK-VTT093 |
| 8 | Model selection from tray | ✅ | ❌ | ❌ legacy names | TASK-VTT086 |
| 9 | Language selection from tray | ✅ | 🟡 | 🟡 (needs menu) | TASK-VTT091 |
| 10 | Settings persistence (settings.conf) | ✅ | ✅ | ✅ | — |
| 11 | Model auto-download on first use | ✅ | ✅ | ✅ | — |
| 12 | Desktop notifications | ✅ notify-send | 🟡 | ❌ | TASK-VTT096 (file) |
| 13 | Start on login / autostart | 🟡 | ❌ | ❌ | TASK-VTT094 |
| 14 | Installer / packaging | ✅ .deb + PPA | ❌ .app planned | ✅ .msi | TASK-VTT040 (macOS) |
| 15 | Update mechanism | ✅ apt/PPA | ❌ | 🟡 MSI in-place upgrade, manual | TASK-VTT095 |
| 16 | No stray console window | ✅ | ✅ | ✅ (v2.3.1) | TASK-VTT089 |

## Notes on the Windows gaps surfaced 2026-06-26

- **Tray menu** (#6) was inert — fixed by pumping the Win32 message queue (TASK-VTT091).
- **Icon state** (#7): the portable tray logs `[UI] Icon: recording` but never calls
  `set_icon()`, so the icon never changes colour. Wire the UiMessage through to the
  tray on the main thread (TASK-VTT093).
- **Model menu** (#8): portable tray lists legacy `W base` / `CT2 …` names that don't
  resolve against the catalogue; Emmanuel wants `large-v3-turbo` / `medium`
  selectable (TASK-VTT086).
- **Autostart** (#13) and **update** (#15) are unaddressed on Windows — see the
  dedicated cards.

## Principle

New capability lands on one platform first (gensho), but a parity row must be opened
the same day so the other platforms are tracked, not forgotten. This matrix is the
checklist; each ❌/🟡 should map to a task.
