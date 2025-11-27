#!/bin/bash
# Run this script ON YOUR MAC to set up the GitHub Actions self-hosted runner
#
# Prerequisites:
# 1. Go to https://github.com/powell-clark/voice-to-text/settings/actions/runners
# 2. Click "New self-hosted runner" → macOS
# 3. Copy the token from the configure command
#
# Usage: ./setup-runner.sh YOUR_TOKEN_HERE

set -e

TOKEN="$1"

if [ -z "$TOKEN" ]; then
    echo "Usage: ./setup-runner.sh YOUR_RUNNER_TOKEN"
    echo ""
    echo "Get your token from:"
    echo "https://github.com/powell-clark/voice-to-text/settings/actions/runners/new"
    exit 1
fi

# Detect architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    RUNNER_ARCH="osx-arm64"
else
    RUNNER_ARCH="osx-x64"
fi

echo "Setting up GitHub Actions runner for $RUNNER_ARCH..."

# Create runner directory
mkdir -p ~/actions-runner && cd ~/actions-runner

# Download latest runner
RUNNER_VERSION="2.321.0"
curl -o actions-runner.tar.gz -L "https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-${RUNNER_ARCH}-${RUNNER_VERSION}.tar.gz"

# Extract
tar xzf actions-runner.tar.gz
rm actions-runner.tar.gz

# Configure
./config.sh --url https://github.com/powell-clark/voice-to-text --token "$TOKEN" --name "mac-local" --labels "self-hosted,macOS,$(uname -m)" --unattended

echo ""
echo "✅ Runner configured!"
echo ""
echo "To start the runner:"
echo "  cd ~/actions-runner && ./run.sh"
echo ""
echo "To install as a service (runs on startup):"
echo "  cd ~/actions-runner && ./svc.sh install && ./svc.sh start"
echo ""
