# TASK-VTT097: models::system_cache() returns a Linux path on Windows

`models::system_cache()` returns `/usr/share/voice-to-text/models` unconditionally.
On Windows it never exists (harmless — falls through to the user cache) but should be
cfg-gated for cleanliness.

- [ ] system_cache() is cfg(linux)-only or returns None on Windows
- Story: STORY-VTT013 · Directive: DIRECT-VTT004 · Parity §0
