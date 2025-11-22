#!/bin/bash
# Backup tool for Voice to Text
# Creates timestamped archives of settings, recordings, and models

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

USER_DATA_DIR="$HOME/.local/share/voice-to-text"
MODEL_DIR="$HOME/.cache/whisper"
BACKUP_DIR="${BACKUP_DIR:-$HOME/voice-to-text-backups}"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_NAME="vtt_backup_$TIMESTAMP"
BACKUP_PATH="$BACKUP_DIR/$BACKUP_NAME"

echo -e "${BOLD}========================================"
echo "Voice to Text - Backup Tool"
echo "========================================${NC}"
echo ""

# Check if data exists
if [ ! -d "$USER_DATA_DIR" ] && [ ! -d "$MODEL_DIR" ]; then
    echo -e "${RED}Error: No data found to backup${NC}"
    echo "  User data: $USER_DATA_DIR (not found)"
    echo "  Models: $MODEL_DIR (not found)"
    exit 1
fi

# Calculate sizes
if [ -d "$USER_DATA_DIR" ]; then
    DATA_SIZE=$(du -sh "$USER_DATA_DIR" 2>/dev/null | cut -f1)
else
    DATA_SIZE="0B"
fi

if [ -d "$MODEL_DIR" ]; then
    MODEL_SIZE=$(du -sh "$MODEL_DIR" 2>/dev/null | cut -f1)
else
    MODEL_SIZE="0B"
fi

echo "Data to backup:"
echo "  User data: $DATA_SIZE ($USER_DATA_DIR)"
echo "  Models: $MODEL_SIZE ($MODEL_DIR)"
echo ""
echo "Backup destination: $BACKUP_PATH.tar.gz"
echo ""

# Ask what to include
INCLUDE_MODELS=false
echo "Whisper models are large (${MODEL_SIZE})"
read -p "Include models in backup? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    INCLUDE_MODELS=true
fi

echo ""
read -p "Continue with backup? (y/n) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Backup cancelled"
    exit 0
fi

# Create backup directory
mkdir -p "$BACKUP_DIR"
TEMP_DIR=$(mktemp -d)

echo ""
echo -e "${BOLD}Creating backup...${NC}"
echo ""

# Copy user data
if [ -d "$USER_DATA_DIR" ]; then
    echo "Backing up user data..."
    mkdir -p "$TEMP_DIR/user_data"
    cp -r "$USER_DATA_DIR"/* "$TEMP_DIR/user_data/" 2>/dev/null || true
    echo -e "${GREEN}✓ User data backed up${NC}"
fi

# Copy models if requested
if [ "$INCLUDE_MODELS" = true ] && [ -d "$MODEL_DIR" ]; then
    echo "Backing up models (this may take a while)..."
    mkdir -p "$TEMP_DIR/models"
    cp -r "$MODEL_DIR"/* "$TEMP_DIR/models/" 2>/dev/null || true
    echo -e "${GREEN}✓ Models backed up${NC}"
fi

# Create metadata
echo "Creating metadata..."
cat > "$TEMP_DIR/backup_info.txt" <<EOF
Voice to Text Backup
====================

Created: $(date)
Hostname: $(hostname)
User: $USER

Backup Contents:
- User data: $([ -d "$USER_DATA_DIR" ] && echo "yes" || echo "no")
- Models: $([ "$INCLUDE_MODELS" = true ] && echo "yes" || echo "no")

Paths:
- User data: $USER_DATA_DIR
- Models: $MODEL_DIR

Restore with:
  ./tools/restore.sh $BACKUP_NAME.tar.gz
EOF

echo -e "${GREEN}✓ Metadata created${NC}"

# Create archive
echo "Creating compressed archive..."
cd "$TEMP_DIR"
tar -czf "$BACKUP_PATH.tar.gz" .
cd - >/dev/null

# Cleanup temp directory
rm -rf "$TEMP_DIR"

# Calculate backup size
BACKUP_SIZE=$(du -sh "$BACKUP_PATH.tar.gz" 2>/dev/null | cut -f1)

echo -e "${GREEN}✓ Archive created${NC}"
echo ""

echo -e "${BOLD}========================================${NC}"
echo -e "${GREEN}✓ Backup complete!${NC}"
echo -e "${BOLD}========================================${NC}"
echo ""
echo "Backup file: $BACKUP_PATH.tar.gz"
echo "Size: $BACKUP_SIZE"
echo ""
echo "To restore this backup:"
echo "  ./tools/restore.sh $BACKUP_PATH.tar.gz"
echo ""
echo "To list all backups:"
echo "  ls -lh $BACKUP_DIR"
echo ""
