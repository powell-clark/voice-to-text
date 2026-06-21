# TASK-VTT048: GitHub Actions matrix workflow — all platforms

## Acceptance Criteria
1. A single workflow file builds on `ubuntu-latest`, `macos-latest`, `macos-14` (ARM), and `windows-latest`
2. Each platform produces a release binary in the correct format (.bin, .app, .exe)
3. All four jobs run in parallel on every push to main and on every tag
4. The matrix is gated on the existing Linux CI checks (fmt, clippy, test) passing first
