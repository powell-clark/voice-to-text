# Release Process

## Manual Release (Current - saves $5-6 per release)

Auto-release via GitHub Actions is **disabled** to avoid macOS CI costs ($0.08/min).

### How to release

On your Mac:

```bash
# 1. Make your changes and commit
git add .
git commit -m "feat: your changes"
git push

# 2. Build the app
make clean && make package

# 3. Create GitHub release with the artifact
git tag v0.4.0
git push origin v0.4.0
gh release create v0.4.0 VTT.app.tar.gz --generate-notes

# 4. Update the Homebrew cask manually
SHA256=$(shasum -a 256 VTT.app.tar.gz | awk '{print $1}')
echo "SHA256: $SHA256"
# Edit homebrew-voice-to-text/Casks/voice-to-text.rb with new version and SHA
```

Users can then install/upgrade with:
```bash
brew upgrade voice-to-text
```

## Development workflow

For local development (no release):
```bash
make && ./install-dev.sh
```

For testing the build before releasing:
```bash
make package
# This creates VTT.app.tar.gz locally
```

---

## Optional: Re-enable Automated Releases

To restore automated releases without CI costs, set up a self-hosted runner on your Mac:

### 1. Get runner token
- Go to: https://github.com/powell-clark/voice-to-text/settings/actions/runners
- Click "New self-hosted runner" → macOS
- Copy the token

### 2. Set up runner on your Mac
```bash
./setup-runner.sh YOUR_TOKEN_HERE
```

### 3. Start the runner
```bash
# Run manually:
cd ~/actions-runner && ./run.sh

# Or install as service (runs on startup):
cd ~/actions-runner && ./svc.sh install && ./svc.sh start
```

### 4. Re-enable the workflow
Edit `.github/workflows/release.yml` and uncomment the tag trigger:
```yaml
on:
  push:
    tags:
      - 'v*'
```

Then releasing is automatic again - just push a tag.
