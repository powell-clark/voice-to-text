# Wayland Setup Guide

Voice to Text has **experimental** support for Wayland. This guide will help you set up the necessary components.

## ⚠️ Current Status

- **Keyboard Hotkeys**: Experimental (GNOME/KDE only)
- **Text Typing**: Supported via ydotool or clipboard
- **Audio Recording**: Fully supported
- **Recommendation**: Use X11 session for best experience

## Prerequisites

### Check Your Session Type

```bash
echo $XDG_SESSION_TYPE
# Output should be "wayland" or "x11"

echo $WAYLAND_DISPLAY
# Should show something like "wayland-0" if running Wayland
```

### Check Your Compositor

```bash
echo $XDG_CURRENT_DESKTOP
# Shows: GNOME, KDE, sway, etc.

ps aux | grep -E 'gnome-shell|kwin_wayland|sway'
# Shows running compositor process
```

## Required Packages

### Ubuntu/Debian

```bash
# Essential for typing (choose one or both)
sudo apt install ydotool wl-clipboard

# For GNOME keyboard shortcuts
sudo apt install dbus libdbus-1-dev

# Optional: alternative typing tool
sudo apt install dotool  # May need to build from source
```

### Fedora

```bash
sudo dnf install ydotool wl-clipboard dbus-devel
```

### Arch Linux

```bash
sudo pacman -S ydotool wl-clipboard dbus
```

## Setup Steps

### 1. Set Up ydotool (Recommended)

ydotool is the most reliable method for text input on Wayland.

#### Start the daemon:

```bash
# One-time start
sudo ydotoold

# Or enable systemd service (recommended)
sudo systemctl enable ydotool
sudo systemctl start ydotool
```

#### Add your user to input group:

```bash
sudo usermod -aG input $USER
# Log out and log back in for group changes to take effect
```

#### Test ydotool:

```bash
# This should type "hello" in the focused window after 2 seconds
sleep 2 && ydotool type "hello"
```

### 2. Configure Keyboard Shortcuts

#### GNOME (gnome-shell / mutter)

1. Open **Settings** → **Keyboard** → **Keyboard Shortcuts**
2. Scroll to bottom and click **"+"** to add custom shortcut
3. Configure:
   - **Name**: Voice to Text Record
   - **Command**: `dbus-send --session --type=signal /org/gnome/Shell org.gnome.Shell.AcceleratorActivated string:'voice-to-text-record'`
   - **Shortcut**: Press `Scroll Lock` (or your preferred key)

**Note**: D-Bus integration is experimental. The application will log instructions on first run.

#### KDE Plasma (kwin_wayland)

1. Open **System Settings** → **Shortcuts**
2. Add custom shortcut with command:
   ```bash
   qdbus org.kde.kglobalaccel /component/voice-to-text invokeShortcut voice-to-text-record
   ```
3. Assign `Scroll Lock` key

#### Sway / wlroots compositors

Edit `~/.config/sway/config`:

```
# Voice to Text hotkey
bindsym --release Scroll_Lock exec pkill -SIGUSR1 vtt-linux
```

Restart Sway: `swaymsg reload`

### 3. Verify Setup

Run Voice to Text with verbose logging:

```bash
./vtt-linux 2>&1 | tee vtt.log
```

Check the log for:
- ✅ "Wayland typing initialized: method=ydotool"
- ✅ "ydotoold daemon detected"
- ❌ "No Wayland typing method available" → Install ydotool

## Troubleshooting

### Text Not Typing

**Problem**: Transcription completes but text doesn't appear

**Solutions**:
1. Verify ydotool daemon is running:
   ```bash
   pgrep -a ydotoold
   ```

2. Check permissions:
   ```bash
   ls -l /dev/uinput
   # Should show: crw-rw---- 1 root input

   groups $USER | grep input
   # Should show "input" in your groups
   ```

3. Try clipboard fallback:
   ```bash
   # Test clipboard method
   echo "test" | wl-copy
   wl-paste  # Should output "test"
   ```

### Keyboard Shortcuts Not Working

**Problem**: Hotkey doesn't trigger recording

**Solutions**:

1. **GNOME**: Check D-Bus connection
   ```bash
   dbus-monitor --session "interface='org.gnome.Shell'"
   # Press your hotkey, should show signal
   ```

2. **Fallback to X11**: Run Voice to Text under XWayland
   ```bash
   # Force X11 mode
   GDK_BACKEND=x11 ./vtt-linux
   ```

3. **Ultimate fallback**: Switch to X11 session
   - Log out
   - At login screen, click gear icon
   - Select "GNOME on Xorg" or "Plasma (X11)"
   - Log in

### Permission Denied Errors

```bash
# Fix ydotool permissions
sudo chmod 666 /dev/uinput  # Temporary (lost on reboot)

# Permanent fix: add udev rule
echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' | sudo tee /etc/udev/rules.d/80-uinput.rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### Performance Issues

If typing is slow or laggy:

1. **Reduce typing delay** in GUI settings
2. **Use ydotool instead of clipboard method**
3. **Switch to X11 session** for better performance

## Method Comparison

| Method | Speed | Reliability | Setup Difficulty | Notes |
|--------|-------|-------------|------------------|-------|
| **ydotool** | ⚡⚡⚡ | ✅ Excellent | Medium | Requires daemon, best option |
| **dotool** | ⚡⚡⚡ | ✅ Good | Medium | Alternative to ydotool |
| **clipboard** | ⚡⚡ | ⚠️ OK | Easy | Fallback, may interfere with clipboard |
| **XWayland** | ⚡⚡⚡⚡ | ✅ Excellent | Easy | Recommended if available |
| **X11 session** | ⚡⚡⚡⚡⚡ | ✅ Perfect | Easy | **Most reliable option** |

## Known Limitations

1. **No native Wayland protocol**: Text injection isn't standardized in Wayland
2. **Compositor-specific**: Each compositor (GNOME, KDE, Sway) has different APIs
3. **Security restrictions**: Wayland blocks many input injection methods for security
4. **Experimental features**: Wayland support is still under development

## Recommended Configuration

For best experience on Wayland:

1. ✅ Install ydotool + enable daemon
2. ✅ Configure manual keyboard shortcut in DE settings
3. ✅ Test with `tools/first_run_wizard.sh`
4. ⚠️ If issues persist, use X11 session instead

## Still Having Issues?

1. **Check logs**: `~/.local/share/voice-to-text/vtt.log`
2. **Run first-run wizard**: `./tools/first_run_wizard.sh`
3. **Report bug**: https://github.com/powell-clark/voice-to-text/issues
4. **Fall back to X11**: Most reliable option until native Wayland support matures

## Future Improvements

Planned enhancements for Wayland support:

- [ ] Native virtual-keyboard-v1 protocol support
- [ ] Automatic keyboard shortcut registration
- [ ] IBus/fcitx input method integration
- [ ] Per-compositor optimizations
- [ ] Wayland session auto-detection and setup wizard
