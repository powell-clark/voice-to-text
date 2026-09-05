# Platform path conventions

Moved from `docs/PLATFORM-PARITY.md` §0 (TASK-VTT172) — this table is
cross-cutting, not tied to any single feature card, so it needs a home that
survives `PLATFORM-PARITY.md`'s retirement.
`scripts/generate-platform-spec.sh` prepends this file verbatim.

The Linux cards assume XDG paths; Windows needs a defined equivalent. Canonical:

| Data | Linux | Windows | Status |
|------|-------|---------|--------|
| Settings | `~/.config/voice-to-text/settings.conf` | `%APPDATA%\voice-to-text\settings.conf` (via `dirs::config_dir`) | 🟡 verify |
| Model cache (user) | `~/.cache/voice-to-text/models/` | `%LOCALAPPDATA%\voice-to-text\models\` (via `dirs::cache_dir`) | ✅ |
| Model cache (shared) | `/usr/share/voice-to-text/models/` | n/a (per-user only) | n/a |
| Logs | `~/.local/share/voice-to-text/` | `%APPDATA%\voice-to-text\logs\` | ✅ |
| Recordings archive | `~/.local/share/voice-to-text/recordings/` | `%LOCALAPPDATA%\voice-to-text\recordings\` | ✅ |

`models.rs::system_cache()` is `#[cfg(target_os = "linux")]`-gated (returns
`None` on Windows/macOS, falling through to the user cache) — corrected
2026-09-05, TASK-VTT172; TASK-VTT097 (which tracked this exact gap) is done.
