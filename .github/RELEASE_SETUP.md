# Automated Release Setup

## One-time setup

1. **Create GitHub Personal Access Token:**
   - Go to: https://github.com/settings/tokens/new
   - Token name: `homebrew-tap-updater`
   - Expiration: No expiration (or 1 year)
   - Scopes: Check `repo` (Full control of private repositories)
   - Click "Generate token"
   - Copy the token

2. **Add token to repository secrets:**
   - Go to: https://github.com/powell-clark/voice-to-text/settings/secrets/actions
   - Click "New repository secret"
   - Name: `TAP_GITHUB_TOKEN`
   - Value: Paste the token you copied
   - Click "Add secret"

## How to release

Once setup is complete, releasing is automatic:

```bash
# 1. Make your changes and commit
git add .
git commit -m "feat: your changes"

# 2. Create and push a version tag
git tag v0.3.0
git push origin v0.3.0
```

That's it! GitHub Actions will:
- Build VTT.app
- Create VTT.app.tar.gz
- Calculate SHA256
- Create GitHub release
- Update homebrew-voice-to-text/Casks/voice-to-text.rb
- Push the cask update

Users can then install with:
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
