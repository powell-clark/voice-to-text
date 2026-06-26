# TASK-VTT102: Rename binary — drop the misleading "vtt-linux" name

The cargo `[[bin]]` is named `vtt-linux` on every platform (historical), so the
Windows/macOS build output is `vtt-linux.exe` / `vtt-linux` — confusing on a
Windows machine ("why is it called linux?"). The Windows MSI already renames it to
`vtt.exe` on install, but the raw build artifact and the release asset still read
`vtt-linux`.

Proposal: rename the bin to a platform-neutral `vtt` (or `voice-to-text`). Cross-
cutting — touches `Cargo.toml [[bin]]`, `debian/` install paths + `vtt.service`,
`wix/main.wxs` source path, `ci.yml`/`release.yml` verify+package steps that grep
`vtt-linux`, `gen-release-notes.sh`/`build-windows.ps1`, and CLAUDE.md. Do as one
atomic rename with all consumers updated; verify each platform still builds/packages.

- [ ] Single bin name, platform-neutral, no "linux" on Windows/macOS
- [ ] deb, msi, and release assets all use the new name; CI green on all three
- Story: STORY-VTT013 · Directive: DIRECT-VTT002
