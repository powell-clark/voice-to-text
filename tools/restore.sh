#!/bin/bash
# Restore tool for Voice to Text
# Restores settings, recordings, and models from backup archive

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

BACKUP_FILE="$1"

echo -e "${BOLD}========================================"
echo "Voice to Text - Restore Tool"
echo "========================================${NC}"
echo ""

# Check if backup file specified
if [ -z "$BACKUP_FILE" ]; then
    echo -e "${RED}Error: No backup file specified${NC}"
    echo ""
    echo "Usage: $0 <backup_file>"
    echo ""
    echo "Example:"
    echo "  $0 ~/voice-to-text-backups/vtt_backup_20250115_120000.tar.gz"
    echo ""
    echo "Available backups:"
    BACKUP_DIR="$HOME/voice-to-text-backups"
    if [ -d "$BACKUP_DIR" ]; then
        ls -lh "$BACKUP_DIR"/*.tar.gz 2>/dev/null || echo "  No backups found"
    else
        echo "  No backup directory found"
    fi
    exit 1
fi

# Check if backup file exists
if [ ! -f "$BACKUP_FILE" ]; then
    echo -e "${RED}Error: Backup file not found: $BACKUP_FILE${NC}"
    exit 1
fi

# Display backup info
BACKUP_SIZE=$(du -sh "$BACKUP_FILE" 2>/dev/null | cut -f1)
BACKUP_DATE=$(stat -c %y "$BACKUP_FILE" 2>/dev/null | cut -d' ' -f1)

echo "Backup file: $BACKUP_FILE"
echo "Size: $BACKUP_SIZE"
echo "Date: $BACKUP_DATE"
echo ""

# Extract to temporary directory to inspect
TEMP_DIR=$(mktemp -d)
echo "Extracting backup..."
tar -xzf "$BACKUP_FILE" -C "$TEMP_DIR"

# Show backup info if available
if [ -f "$TEMP_DIR/backup_info.txt" ]; then
    echo ""
    echo -e "${BOLD}Backup Information:${NC}"
    cat "$TEMP_DIR/backup_info.txt"
    echo ""
fi

# Check what's in the backup
HAS_USER_DATA=false
HAS_MODELS=false

if [ -d "$TEMP_DIR/user_data" ]; then
    HAS_USER_DATA=true
    USER_DATA_SIZE=$(du -sh "$TEMP_DIR/user_data" 2>/dev/null | cut -f1)
fi

if [ -d "$TEMP_DIR/models" ]; then
    HAS_MODELS=true
    MODEL_COUNT=$(find "$TEMP_DIR/models" -name "*.bin" -type f | wc -l)
    MODELS_SIZE=$(du -sh "$TEMP_DIR/models" 2>/dev/null | cut -f1)
fi

# Show what will be restored
echo -e "${BOLD}Restore Plan:${NC}"
echo ""

if [ "$HAS_USER_DATA" = true ]; then
    echo -e "${GREEN}✓ User data${NC} ($USER_DATA_SIZE)"
    echo "  → $USER_DATA_DIR"
else
    echo -e "${YELLOW}⊘ No user data in backup${NC}"
fi

if [ "$HAS_MODELS" = true ]; then
    echo -e "${GREEN}✓ Models${NC} ($MODEL_COUNT models, $MODELS_SIZE)"
    echo "  → $MODEL_DIR"
else
    echo -e "${YELLOW}⊘ No models in backup${NC}"
fi

echo ""

# Warn about overwriting
WILL_OVERWRITE=false
if [ "$HAS_USER_DATA" = true ] && [ -d "$USER_DATA_DIR" ]; then
    echo -e "${YELLOW}⚠ Warning: Existing user data will be backed up and replaced${NC}"
    WILL_OVERWRITE=true
fi

if [ "$HAS_MODELS" = true ] && [ -d "$MODEL_DIR" ]; then
    echo -e "${YELLOW}⚠ Warning: Existing models will be backed up and replaced${NC}"
    WILL_OVERWRITE=true
fi

if [ "$WILL_OVERWRITE" = true ]; then
    echo ""
    echo "Your current data will be saved to:"
    echo "  ~/voice-to-text-backups/pre_restore_$(date +%Y%m%d_%H%M%S).tar.gz"
fi

echo ""
read -p "Continue with restore? (y/n) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    rm -rf "$TEMP_DIR"
    echo "Restore cancelled"
    exit 0
fi

echo ""
echo -e "${BOLD}Restoring backup...${NC}"
echo ""

# Backup current data if it exists
if [ "$WILL_OVERWRITE" = true ]; then
    echo "Creating backup of current data..."
    PRE_RESTORE_BACKUP="$HOME/voice-to-text-backups/pre_restore_$(date +%Y%m%d_%H%M%S)"
    mkdir -p "$(dirname "$PRE_RESTORE_BACKUP")"

    TEMP_BACKUP=$(mktemp -d)

    if [ -d "$USER_DATA_DIR" ]; then
        mkdir -p "$TEMP_BACKUP/user_data"
        cp -r "$USER_DATA_DIR"/* "$TEMP_BACKUP/user_data/" 2>/dev/null || true
    fi

    if [ -d "$MODEL_DIR" ]; then
        mkdir -p "$TEMP_BACKUP/models"
        cp -r "$MODEL_DIR"/* "$TEMP_BACKUP/models/" 2>/dev/null || true
    fi

    tar -czf "$PRE_RESTORE_BACKUP.tar.gz" -C "$TEMP_BACKUP" .
    rm -rf "$TEMP_BACKUP"

    echo -e "${GREEN}✓ Current data backed up to: $PRE_RESTORE_BACKUP.tar.gz${NC}"
fi

# Restore user data
if [ "$HAS_USER_DATA" = true ]; then
    echo "Restoring user data..."
    mkdir -p "$USER_DATA_DIR"
    cp -r "$TEMP_DIR/user_data"/* "$USER_DATA_DIR/" 2>/dev/null || true
    echo -e "${GREEN}✓ User data restored${NC}"
fi

# Restore models
if [ "$HAS_MODELS" = true ]; then
    echo "Restoring models (this may take a while)..."
    mkdir -p "$MODEL_DIR"
    cp -r "$TEMP_DIR/models"/* "$MODEL_DIR/" 2>/dev/null || true
    echo -e "${GREEN}✓ Models restored${NC}"
fi

# Cleanup temp directory
rm -rf "$TEMP_DIR"

echo ""
echo -e "${BOLD}========================================${NC}"
echo -e "${GREEN}✓ Restore complete!${NC}"
echo -e "${BOLD}========================================${NC}"
echo ""
echo "Restored data:"
if [ "$HAS_USER_DATA" = true ]; then
    echo "  - User data → $USER_DATA_DIR"
fi
if [ "$HAS_MODELS" = true ]; then
    echo "  - Models → $MODEL_DIR"
fi
echo ""
echo "You may need to restart Voice to Text for changes to take effect:"
echo "  systemctl --user restart vtt"
echo ""
