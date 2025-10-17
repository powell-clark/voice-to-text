cask "voice-to-text" do
  version "0.3.18"
  sha256 "153843d5199f1e28a142e952560749224711896a992166a12c511153676ece37"

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
