# TASK-VTT101: Windows installer pre-provisions the default model (parity FEAT-VTT028)

Linux postinst downloads ggml-small.en at install so first run is offline-ready. The
Windows MSI installs without the model, so first launch downloads ~465 MB. Add
installer-time or first-run provisioning with the same offline-first guarantee.

- [ ] Default model present after MSI install OR clearly fetched first-run with progress
- [ ] First launch transcribes without a surprise blocking download
- Story: STORY-VTT013 · Directive: DIRECT-VTT004 · Parity §6
