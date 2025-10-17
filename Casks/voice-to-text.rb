cask "voice-to-text" do
  version "0.3.17"
  sha256 "3ca9ecfae1e06543414b176c25e3e7f457c047fb43c3c36a505fc00ad80f3411"

  url "https://github.com/powell-clark/voice-to-text/releases/download/v#{version}/VTT.app.tar.gz"

  name "VTT"
  desc "Voice to Text - macOS menu bar app for real-time voice transcription"
  homepage "https://github.com/powell-clark/voice-to-text"

  app "VTT.app"

  zap trash: [
    "~/Library/Preferences/com.powellclark.voice-to-text.plist",
    "/tmp/VTT",
  ]

  caveats <<~EOS
    On first run, grant these permissions in System Settings → Privacy & Security:
      • Microphone - For audio capture
      • Accessibility - For pasting text
      • Input Monitoring - For global hotkey

    VTT will automatically download Whisper models when you select a size.

    Usage: Hold Right Option key and speak. Release to transcribe and paste.
  EOS
end
