# PPA Setup and Upload Guide

> **⚠️ PARTIALLY STALE — use `scripts/release-ppa.sh` instead.** This guide
> documents the manual `debuild` + `dput` flow. The actual release process
> is now `scripts/release-ppa.sh` which does all of this plus:
> - Pre-flight offline cargo build check
> - pbuilder hard-gate against noble + jammy chroots before any dput
> - Bytemark mirror pinning (archive.ubuntu.com round-robin serves
>   devel/resolute under pinned release names)
> - apt-based satisfydepends (works around dpkg-deb temp-file bug)
> - TMPDIR=/tmp hygiene (systemd private tmp paths leak into chroot)
> - Auto-commit of vtt-linux.prebuilt after rebuild
>
> The one-time Launchpad account + GPG setup below is still accurate.
> But for any actual release, run `./scripts/release-ppa.sh` not the
> manual steps here.

## Prerequisites

### 1. Install Build Tools

```bash
sudo apt install devscripts build-essential debhelper lintian
```

### 2. Install Build Dependencies

```bash
sudo apt install build-essential pkg-config \
    portaudio19-dev libx11-dev libxtst-dev libxext-dev \
    libgtk-3-dev libayatana-appindicator3-dev libnotify-dev
```

### 3. Set Up GPG Key (for signing packages)

If you don't have a GPG key yet:

```bash
# Generate a new GPG key
gpg --full-generate-key
# Choose: (1) RSA and RSA, 4096 bits, key does not expire
# Enter your name and email (must match debian/changelog)

# List your keys
gpg --list-keys

# Upload your public key to Ubuntu keyserver
gpg --keyserver keyserver.ubuntu.com --send-keys YOUR_KEY_ID
```

### 4. Create Launchpad Account and PPA

1. Go to https://launchpad.net and create an account
2. Import your GPG key: https://launchpad.net/~/+editpgpkeys
3. Create a PPA: https://launchpad.net/~/+activate-ppa
   - Name: `voice-to-text`
   - Display name: `Voice to Text - Offline Voice Transcription`
   - Description: See below

```
Offline voice-to-text transcription with 99+ language support

Push-to-talk voice transcription using OpenAI Whisper AI. Supports 99+ languages
with automatic detection. 100% offline - no internet required.

Features:
- Push-to-talk recording (Scroll Lock key by default)
- 99+ language support with automatic detection
- Multiple Whisper model sizes (tiny to large-v3)
- GPU acceleration with NVIDIA CUDA (optional)
- System tray integration
```

## Building and Testing Locally

### 1. Test Build

```bash
cd /home/powell-clark/projects/voice-to-text

# Build binary package (unsigned, for local testing)
dpkg-buildpackage -us -uc -b

# This creates ../voice-to-text_1.0.1-1_amd64.deb
```

### 2. Test Install

```bash
# Install the built package
sudo dpkg -i ../voice-to-text_1.0.1-1_amd64.deb

# Fix any dependency issues
sudo apt install -f

# Test the application
systemctl --user daemon-reload
systemctl --user start vtt
systemctl --user status vtt
```

### 3. Uninstall Test Package

```bash
sudo apt remove voice-to-text
```

## Uploading to PPA

### 1. Build Source Package

```bash
cd /home/powell-clark/projects/voice-to-text

# Build signed source package for PPA
debuild -S -sa

# This creates:
# - ../voice-to-text_1.0.1-1.dsc
# - ../voice-to-text_1.0.1-1.tar.xz
# - ../voice-to-text_1.0.1-1_source.changes
# - ../voice-to-text_1.0.1-1_source.build (build log)
```

If you get signing errors, ensure your GPG key email matches debian/changelog.

### 2. Upload to PPA

```bash
# Upload to your PPA (replace YOURUSERNAME)
dput ppa:powellclark/voice-to-text ../voice-to-text_1.0.1-1_source.changes
```

First time setup for dput:

```bash
# Create ~/.dput.cf if it doesn't exist
cat > ~/.dput.cf << 'EOF'
[powell-clark-voice-to-text]
fqdn = ppa.launchpad.net
method = ftp
incoming = ~powell-clark/ubuntu/voice-to-text/
login = anonymous
allow_unsigned_uploads = 0
EOF
```

Then upload:

```bash
dput powell-clark-voice-to-text ../voice-to-text_1.0.1-1_source.changes
```

### 3. Monitor Build Status

After upload:
1. Check https://launchpad.net/~powell-clark/+archive/ubuntu/voice-to-text
2. Launchpad will build packages for supported Ubuntu releases (noble, etc.)
3. Build logs available if there are errors
4. Usually takes 5-30 minutes depending on build queue

## Releasing New Versions

### 1. Update Version Number

```bash
# For a new upstream release (e.g., 1.0.2):
dch -v 1.0.2-1

# For a packaging-only update (e.g., 1.0.1-2):
dch -i

# This opens an editor to update debian/changelog
# Add your changes, save, and exit
```

### 2. Commit Changes

```bash
git add debian/changelog
git commit -m "chore: bump version to 1.0.2-1"
git push
```

### 3. Build and Upload

```bash
# Build source package
debuild -S -sa

# Upload to PPA
dput ppa:powellclark/voice-to-text ../voice-to-text_1.0.2-1_source.changes
```

## Supporting Multiple Ubuntu Releases

To support multiple Ubuntu releases (e.g., noble, jammy, focal):

### Option 1: Backport with dch

```bash
# After uploading for noble, create backports:

# For jammy (22.04)
dch -v 1.0.1-1~jammy1
# Change the distribution line in debian/changelog from "noble" to "jammy"
debuild -S -sa
dput ppa:powellclark/voice-to-text ../voice-to-text_1.0.1-1~jammy1_source.changes

# For focal (20.04)
dch -v 1.0.1-1~focal1
# Change distribution to "focal"
debuild -S -sa
dput ppa:powellclark/voice-to-text ../voice-to-text_1.0.1-1~focal1_source.changes
```

### Option 2: PPA Copy Packages

Use Launchpad's "Copy packages" feature to copy from noble to other releases.

## User Installation

Once published, users can install with:

```bash
sudo add-apt-repository ppa:powellclark/voice-to-text
sudo apt update
sudo apt install voice-to-text
```

## Troubleshooting

### Build Fails on PPA

- Check build logs on Launchpad
- Ensure all Build-Depends are available in Ubuntu repos
- Test locally with `pbuilder` or `sbuild` for clean environment

### GPG Signing Errors

```bash
# List your keys
gpg --list-keys

# Export and re-import if needed
gpg --armor --export YOUR_KEY_ID > mykey.asc
gpg --import mykey.asc
```

### Package Rejected

- Email in changelog must match GPG key
- GPG key must be registered on Launchpad
- Source format must be acceptable (we use 3.0 quilt)

### Dependencies Not Available

If dependencies like `libayatana-appindicator3-dev` aren't in older Ubuntu:
- Use alternative packages
- Add fallback dependencies with `|` operator
- Consider separate packaging for older releases

## Useful Commands

```bash
# Check package for common issues
lintian -i ../voice-to-text_1.0.1-1_amd64.deb

# List package contents
dpkg -c ../voice-to-text_1.0.1-1_amd64.deb

# Check package info
dpkg -I ../voice-to-text_1.0.1-1_amd64.deb

# Clean build artifacts
debclean

# Or manually:
rm -f ../voice-to-text_*
```

## References

- [Debian New Maintainers' Guide](https://www.debian.org/doc/manuals/maint-guide/)
- [Ubuntu Packaging Guide](https://packaging.ubuntu.com/html/)
- [Launchpad PPA Documentation](https://help.launchpad.net/Packaging/PPA)
- [Debian Policy Manual](https://www.debian.org/doc/debian-policy/)
