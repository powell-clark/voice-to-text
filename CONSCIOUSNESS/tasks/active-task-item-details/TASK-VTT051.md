# TASK-VTT051: GTK settings dialog

## Acceptance Criteria
1. Tray menu has a "Settings…" item that opens a GTK dialog
2. Dialog exposes: hotkey, language, model, initial_prompt, VAD toggle, input device
3. Changes are saved to `settings.conf` on "Apply" and take effect without restart
4. Dialog is navigable by keyboard alone (accessibility)
