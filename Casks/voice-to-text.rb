cask "voice-to-text" do
  version "0.3.15"
  sha256 "fa6ad651ee3eb47c96f8586c1601c348278d3f706c8b0e7a47365ec83cdea076"

  # For local development tap - uses local built file
  url "file://#{ENV.fetch('HOME')}/projects/voice-to-text/VTT.app.tar.gz"

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
