cask "voice-to-text" do
  version "0.3.0"
  sha256 "3d8be6361b5d9aade42bae83aaefa763ef5b760cf5764111144c163ca769afc6"

  # Production URL - points to GitHub releases
  url "https://github.com/powell-clark/voice-to-text/releases/download/v#{version}/VTT.app.tar.gz"
  name "VTT"
  desc "Voice to Text - macOS menu bar app for real-time voice transcription"
  homepage "https://github.com/powell-clark/voice-to-text"

  depends_on macos: ">= :monterey"

  app "VTT.app"

  postflight do
    # Reset permissions to trigger fresh prompts
    system_command "/usr/bin/tccutil",
                   args: ["reset", "Microphone", "com.powellclark.voice-to-text"],
                   sudo: false
    system_command "/usr/bin/tccutil",
                   args: ["reset", "Accessibility", "com.powellclark.voice-to-text"],
                   sudo: false
    system_command "/usr/bin/tccutil",
                   args: ["reset", "ListenEvent", "com.powellclark.voice-to-text"],
                   sudo: false
  end

  zap trash: [
    "~/Library/Preferences/com.powellclark.voice-to-text.plist",
    "~/Library/Preferences/VTT.plist",
    "/tmp/VTT",
  ]

  caveats <<~EOS
    🔐 Permission Setup Required:

    When you first launch VTT, grant these permissions in System Settings → Privacy & Security:
      • Microphone - For audio capture
      • Accessibility - For pasting text
      • Input Monitoring - For global hotkey

    📝 Note: VTT will automatically download Whisper models when you select a size.

    🎤 Usage: Hold Right Option key and speak. Release to transcribe and paste.

    To reinstall and reset permissions:
      brew reinstall --cask voice-to-text
  EOS
end
