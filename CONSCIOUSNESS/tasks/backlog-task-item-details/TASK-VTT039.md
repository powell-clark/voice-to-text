# TASK-VTT039: dput to PPA and verify install

## Context
After the local binary validates via TASK-VTT034 and the Debian packaging builds cleanly locally, the package must be pushed to Launchpad PPA `powellclark/voice-to-text` and installed from there on a test system to confirm the full round-trip works.

## Acceptance Criteria
1. `debuild -S -sa` produces a signed source package in the parent directory
2. `dput powellclark-voice-to-text ../voice-to-text_2.0.0_source.changes` succeeds and Launchpad accepts the upload
3. Launchpad build succeeds for Noble (24.04) and any other targeted distributions (Jammy 22.04 if supported); build logs show no errors
4. On a fresh Ubuntu Noble VM with the PPA added (`sudo add-apt-repository ppa:powellclark/voice-to-text && sudo apt update`), `sudo apt install voice-to-text` installs version 2.0.0 and only version 2.0.0 (no older version offered)
5. The installed binary at `/usr/bin/vtt-linux` is the Rust binary (verifiable via `file /usr/bin/vtt-linux` showing reasonable size ~10 MB, not the 2-3 MB the old C binary had)
6. The postinst downloads the default model successfully
7. Launching VTT from the desktop environment produces the tray icon; push-to-talk records and transcribes
8. `git tag v2.0.0` is pushed to origin after successful PPA build

## Technical Approach
1. Ensure Launchpad GPG key is correctly configured: `dput --print --config-file debian/dput.cf powellclark-voice-to-text ../*.changes`
2. Sign the source package: `debuild -S -sa` uses the default GPG key
3. Upload: `dput powellclark-voice-to-text ../voice-to-text_2.0.0_source.changes`
4. Monitor https://launchpad.net/~powellclark/+archive/ubuntu/voice-to-text for build status
5. Once build succeeds for all targeted distributions, add the PPA on a test VM and install
6. Tag: `git tag -a v2.0.0 -m "v2.0.0 Rust rewrite shipped to PPA" && git push --tags`

## Test Strategy
Full install test on a fresh Multipass VM:
```
multipass launch -n vtt-test 24.04
multipass shell vtt-test
sudo add-apt-repository -y ppa:powellclark/voice-to-text
sudo apt update
sudo apt install -y voice-to-text
# Confirm install
which vtt-linux
dpkg -s voice-to-text | grep Version
ls /usr/share/voice-to-text/models/
# (Cannot test GUI in headless VM; manual test on user's machines)
```

## Files
- No source file changes — operational task
- `git tag v2.0.0` (new git ref)
