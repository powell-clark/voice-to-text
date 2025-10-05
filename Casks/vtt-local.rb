cask "vtt-local" do
  version "0.2.2"
  sha256 :no_check

  # Local development URL
  url "file:///Users/powell-clark/projects/voice-to-text/VTT.app.tar.gz"
  name "VTT"
  desc "Voice to Text - macOS menu bar app (Local Dev Build)"
  homepage "https://github.com/powell-clark/voice-to-text"

  depends_on macos: ">= :monterey"

  app "VTT.app"

  postflight do
    # Reset permissions to trigger fresh prompts on every install
    system_command "/usr/bin/tccutil",
                   args: ["reset", "Microphone", "com.local.vtt"],
                   sudo: false
    system_command "/usr/bin/tccutil",
                   args: ["reset", "Accessibility", "com.local.vtt"],
                   sudo: false
    system_command "/usr/bin/tccutil",
                   args: ["reset", "ListenEvent", "com.local.vtt"],
                   sudo: false
  end

  uninstall quit: "com.local.vtt"

  zap trash: [
    "~/Library/Preferences/com.local.vtt.plist",
    "~/Library/Preferences/VTT.plist",
    "/tmp/VTT",
  ]

  caveats <<~EOS
    🔐 Local Dev Build - Permissions Auto-Reset

    On every install, permissions are reset. You'll need to grant:
      • Microphone - For audio capture
      • Accessibility - For pasting text
      • Input Monitoring - For global hotkey

    🎤 Usage: Hold Right Option key and speak. Release to transcribe and paste.

    To reinstall: brew reinstall --cask vtt-local
  EOS
end
