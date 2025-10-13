# PPA Setup Guide for Voice to Text

This guide explains how to set up and maintain a PPA (Personal Package Archive) for distributing Voice to Text on Ubuntu/Debian.

---

## 📦 What is a PPA?

A **PPA** (Personal Package Archive) is a repository hosted on Launchpad that allows users to easily install and update software:

```bash
# Users can install with:
sudo add-apt-repository ppa:powell-clark/voice-to-text
sudo apt update
sudo apt install voice-to-text

# And get automatic updates with:
sudo apt upgrade
```

---

## 🚀 Initial Setup (One-Time, ~30-60 minutes)

### Step 1: Create Launchpad Account

1. Go to https://launchpad.net/
2. Sign up with Ubuntu SSO
3. Complete your profile
4. Set up your GPG key (for signing packages)

### Step 2: Generate GPG Key (if you don't have one)

```bash
# Generate a new GPG key
gpg --full-generate-key

# Select:
# - Key type: RSA and RSA
# - Key size: 4096 bits
# - Expiration: 0 (never expires) or 2y (2 years)
# - Name: Emmanuel Powell-Clark
# - Email: emmanuel@powell-clark.com

# List your keys
gpg --list-secret-keys --keyid-format=long

# Example output:
# sec   rsa4096/ABCD1234EFGH5678 2025-10-13 [SC]
#       Full-Key-ID-Here
# uid   Emmanuel Powell-Clark <emmanuel@powell-clark.com>

# Export public key
gpg --armor --export ABCD1234EFGH5678 > ~/gpg-public-key.asc

# Upload to Ubuntu keyserver
gpg --keyserver keyserver.ubuntu.com --send-keys ABCD1234EFGH5678
```

### Step 3: Upload GPG Key to Launchpad

1. Go to https://launchpad.net/~your-username/+editpgpkeys
2. Paste contents of `~/gpg-public-key.asc`
3. Click "Import Key"
4. Check your email for confirmation link
5. Click confirmation link

### Step 4: Create PPA on Launchpad

1. Go to https://launchpad.net/~your-username/+activate-ppa
2. PPA name: `voice-to-text`
3. Display name: `Voice to Text - Offline Speech Transcription`
4. Description:
   ```
   Offline voice-to-text transcription with 99+ language support using OpenAI Whisper AI.

   Features:
   - Push-to-talk recording
   - 99+ languages with automatic detection
   - GPU acceleration (CUDA support)
   - System tray integration
   - 100% offline operation
   ```
5. Click "Activate"

---

## 📝 Building and Uploading Packages

### Install Required Tools

```bash
sudo apt install -y \
    devscripts \
    debhelper \
    dh-make \
    build-essential \
    lintian \
    dput \
    ubuntu-dev-tools
```

### Build Source Package

```bash
cd ~/projects/voice-to-text

# Clean any previous builds
make -f Makefile.linux clean
rm -rf debian/.debhelper debian/voice-to-text* debian/*.substvars

# Build source package (doesn't require compilation)
debuild -S -sa

# This creates:
# - voice-to-text_1.0.0-1.dsc (package description)
# - voice-to-text_1.0.0-1.tar.xz (source archive)
# - voice-to-text_1.0.0-1_source.changes (upload control file)
# - voice-to-text_1.0.0-1_source.buildinfo (build information)

# All files are created in parent directory:
ls -la ~/projects/*.dsc ~/projects/*.tar.xz ~/projects/*.changes
```

### Upload to PPA

```bash
# Configure dput for your PPA (one-time)
cat > ~/.dput.cf << 'EOF'
[voice-to-text-ppa]
fqdn = ppa.launchpad.net
method = ftp
incoming = ~powell-clark/ubuntu/voice-to-text/
login = anonymous
allow_unsigned_uploads = 0
EOF

# Upload source package
cd ~/projects
dput voice-to-text-ppa voice-to-text_1.0.0-1_source.changes

# Launchpad will:
# 1. Verify GPG signature
# 2. Build package for multiple Ubuntu versions (noble, jammy, focal)
# 3. Build for amd64, arm64 (if architecture: any)
# 4. Run quality checks
# 5. Publish to PPA (10-20 minutes)
```

### Monitor Build Status

1. Go to https://launchpad.net/~powell-clark/+archive/ubuntu/voice-to-text
2. Check "Builds" tab
3. Wait for all builds to complete (green checkmarks)
4. If build fails (red X), click to see build log

---

## 🔄 Updating the Package (Very Easy! ~5-10 minutes)

When you make changes and want to release an update:

### Step 1: Update Version Number

```bash
cd ~/projects/voice-to-text

# Add new changelog entry (this updates version)
dch -i

# This opens an editor with template:
voice-to-text (1.0.1-1) noble; urgency=medium

  * [Add your change description here]
  * Example: Fix model download progress indicator
  * Example: Add support for custom hotkeys

 -- Emmanuel Powell-Clark <emmanuel@powell-clark.com>  Mon, 14 Oct 2025 10:00:00 +0000

# Save and exit
```

### Step 2: Build and Upload

```bash
# Build source package (same as before)
debuild -S -sa

# Upload to PPA
cd ~/projects
dput voice-to-text-ppa voice-to-text_1.0.1-1_source.changes

# Done! Users will get update within 10-20 minutes
```

### Step 3: Users Get Automatic Updates

Users who have the PPA installed will automatically see the update:

```bash
# User runs normal system update:
sudo apt update
sudo apt upgrade

# Output shows:
# The following packages will be upgraded:
#   voice-to-text (1.0.0-1 → 1.0.1-1)
```

**That's it!** No manual download, no reinstall, just automatic updates like any other Ubuntu package.

---

## 📊 PPA vs Manual Installation Comparison

| Aspect | Manual Install | PPA Install |
|--------|---------------|-------------|
| **Initial Install** | Clone repo, install deps, make | `apt install voice-to-text` |
| **Updates** | Git pull, rebuild | `apt upgrade` (automatic) |
| **Dependencies** | Manual install | Automatic resolution |
| **Uninstall** | Manual cleanup | `apt remove voice-to-text` |
| **System Integration** | Manual .desktop file | Automatic menu entry |
| **Systemd Service** | Manual enable | Automatic setup |
| **User Experience** | Technical | User-friendly |

---

## 🐛 Troubleshooting

### Build Fails on Launchpad

**Problem:** Build log shows missing dependencies

**Solution:** Update `debian/control` Build-Depends section

```bash
# Check build log at:
https://launchpad.net/~powell-clark/+archive/ubuntu/voice-to-text/+builds

# Common issues:
# - Missing build dependency → Add to Build-Depends
# - Compilation error → Fix in source code
# - Test failure → Fix tests or skip with dh_auto_test override
```

### GPG Signature Verification Failed

**Problem:** `dput` shows "GPG signature verification failed"

**Solution:**
```bash
# Re-sign the changes file
debsign voice-to-text_1.0.0-1_source.changes

# Upload again
dput voice-to-text-ppa voice-to-text_1.0.0-1_source.changes
```

### Package Not Appearing in PPA

**Problem:** Upload succeeded but package not visible

**Solution:**
- Wait 10-20 minutes for build to complete
- Check https://launchpad.net/~powell-clark/+archive/ubuntu/voice-to-text/+builds
- If build failed, check build log for errors

---

## 📚 Advanced Topics

### Building for Multiple Ubuntu Versions

The PPA can build for multiple Ubuntu releases:

```bash
# Edit debian/changelog to target different releases:
dch -r -D noble   # Ubuntu 24.04 (current)
debuild -S -sa
dput voice-to-text-ppa voice-to-text_1.0.0-1_source.changes

dch -r -D jammy   # Ubuntu 22.04 LTS
debuild -S -sa
dput voice-to-text-ppa voice-to-text_1.0.0-2_source.changes

dch -r -D focal   # Ubuntu 20.04 LTS
debuild -S -sa
dput voice-to-text-ppa voice-to-text_1.0.0-3_source.changes
```

### Building for Multiple Architectures

Currently set to `Architecture: amd64` (Intel/AMD 64-bit only).

To support ARM (Raspberry Pi, Apple M1 under Rosetta):

```bash
# Edit debian/control:
Architecture: any  # Build for all architectures

# Or specific architectures:
Architecture: amd64 arm64 armhf
```

### Creating Backports

For older Ubuntu versions not in the PPA:

```bash
# Use backportpackage tool
backportpackage -d focal -u ppa:powell-clark/voice-to-text voice-to-text
```

---

## 📋 Checklist: Releasing a New Version

- [ ] Make code changes
- [ ] Test locally (`make -f Makefile.linux && ./vtt-linux`)
- [ ] Update `debian/changelog` (`dch -i`)
- [ ] Commit changes to git
- [ ] Push to GitHub
- [ ] Build source package (`debuild -S -sa`)
- [ ] Upload to PPA (`dput voice-to-text-ppa *.changes`)
- [ ] Monitor build status on Launchpad
- [ ] Test installation on clean Ubuntu VM
- [ ] Announce update (GitHub release notes, blog, etc.)

---

## 🔗 Useful Links

- **Launchpad PPA Docs**: https://help.launchpad.net/Packaging/PPA
- **Debian Packaging Guide**: https://www.debian.org/doc/manuals/maint-guide/
- **Ubuntu Packaging Guide**: https://packaging.ubuntu.com/html/
- **Debhelper Manual**: https://manpages.ubuntu.com/manpages/noble/man7/debhelper.7.html
- **GPG Tutorial**: https://help.ubuntu.com/community/GnuPrivacyGuardHowto

---

## 📝 Summary

**Initial Setup (30-60 min):**
1. Create Launchpad account
2. Generate and upload GPG key
3. Create PPA
4. Build and upload first version

**Future Updates (5-10 min):**
1. Make changes
2. Update changelog (`dch -i`)
3. Build (`debuild -S -sa`)
4. Upload (`dput`)
5. Wait 10-20 minutes
6. **Done!** Users get automatic updates

**Result:**
- Users install with: `apt install voice-to-text`
- Users update with: `apt upgrade` (automatic)
- Much better UX than manual installation
- Professional distribution method

---

*Last Updated: 2025-10-13*
*Maintainer: Emmanuel Powell-Clark <emmanuel@powell-clark.com>*
