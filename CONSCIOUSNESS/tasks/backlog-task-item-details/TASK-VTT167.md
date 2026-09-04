# TASK-VTT167: Package ct2-daemon into the shipped install (deb/msi)

## Context

Filed alongside TASK-VTT054. ct2_client::resolve_daemon_script() only finds transcribe_daemon.py in a self-built checkout (CARGO_MANIFEST_DIR-relative) or a sibling of the running executable -- there is no .deb/.msi install step yet that places ct2-daemon/ (plus its faster-whisper/ctranslate2 Python deps) alongside the installed binary. A real end-user install with backend=ct2 selected today would fail to locate the daemon and silently fall back to native. Needs: debian/control or postinst copying ct2-daemon/, a Python dependency story (bundle a venv? require system faster-whisper? matches the project's existing 'no Python needed by default' promise -- this stays true since backend defaults to native), and a Windows/macOS equivalent.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT017
- Features: FEAT-VTT034

## Pre-mortem

### Failure modes

- _(to be filled in)_

### Weak assumptions

- _(to be filled in)_
