#!/bin/bash
# Model downloader for Voice to Text
# Downloads Whisper models from HuggingFace

set -e

MODEL_DIR="${HOME}/.cache/whisper"
WHISPER_CPP_MODELS="https://huggingface.co/ggerganov/whisper.cpp/resolve/main"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "=========================================="
echo "Voice to Text - Model Downloader"
echo "=========================================="
echo ""

# Create model directory
mkdir -p "$MODEL_DIR"

# Available models
declare -A MODELS=(
    ["tiny.en"]="ggml-tiny.en.bin"
    ["tiny"]="ggml-tiny.bin"
    ["base.en"]="ggml-base.en.bin"
    ["base"]="ggml-base.bin"
    ["small.en"]="ggml-small.en.bin"
    ["small"]="ggml-small.bin"
    ["medium.en"]="ggml-medium.en.bin"
    ["medium"]="ggml-medium.bin"
    ["large-v3"]="ggml-large-v3.bin"
)

declare -A SIZES=(
    ["tiny.en"]="39 MB"
    ["tiny"]="39 MB"
    ["base.en"]="74 MB"
    ["base"]="74 MB"
    ["small.en"]="244 MB"
    ["small"]="244 MB"
    ["medium.en"]="769 MB"
    ["medium"]="769 MB"
    ["large-v3"]="1.5 GB"
)

list_models() {
    echo "Available models:"
    echo ""
    echo "Model        | Size    | Languages | Speed     | Use Case"
    echo "-------------|---------|-----------|-----------|---------------------------"
    echo "tiny.en      | 39 MB   | English   | ⚡⚡⚡⚡⚡ | Testing"
    echo "tiny         | 39 MB   | 99+       | ⚡⚡⚡⚡⚡ | Testing multilingual"
    echo "base.en      | 74 MB   | English   | ⚡⚡⚡⚡   | Simple dictation"
    echo "base         | 74 MB   | 99+       | ⚡⚡⚡⚡   | Simple multilingual"
    echo "small.en     | 244 MB  | English   | ⚡⚡⚡     | Recommended"
    echo "small        | 244 MB  | 99+       | ⚡⚡⚡     | Recommended multilingual"
    echo "medium.en    | 769 MB  | English   | ⚡⚡       | Higher accuracy"
    echo "medium       | 769 MB  | 99+       | ⚡⚡       | Higher accuracy multilingual"
    echo "large-v3     | 1.5 GB  | 99+       | ⚡         | Maximum accuracy"
    echo ""
    echo "Downloaded models:"
    for model_key in "${!MODELS[@]}"; do
        model_file="${MODELS[$model_key]}"
        if [ -f "$MODEL_DIR/$model_file" ]; then
            size=$(du -h "$MODEL_DIR/$model_file" | cut -f1)
            echo -e "  ${GREEN}✓${NC} $model_key ($size)"
        fi
    done
}

download_model() {
    local model_name="$1"

    if [ -z "${MODELS[$model_name]}" ]; then
        echo -e "${RED}Error: Unknown model '$model_name'${NC}"
        echo ""
        list_models
        exit 1
    fi

    local model_file="${MODELS[$model_name]}"
    local model_path="$MODEL_DIR/$model_file"

    if [ -f "$model_path" ]; then
        echo -e "${YELLOW}Model already exists: $model_path${NC}"
        read -p "Re-download? (y/n) " -n 1 -r
        echo ""
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Skipping download"
            exit 0
        fi
        rm -f "$model_path"
    fi

    echo "Downloading $model_name (${SIZES[$model_name]})..."
    echo "URL: $WHISPER_CPP_MODELS/$model_file"
    echo ""

    # Download with progress
    if command -v wget &> /dev/null; then
        wget --show-progress -O "$model_path" "$WHISPER_CPP_MODELS/$model_file"
    elif command -v curl &> /dev/null; then
        curl -L --progress-bar -o "$model_path" "$WHISPER_CPP_MODELS/$model_file"
    else
        echo -e "${RED}Error: wget or curl not found${NC}"
        exit 1
    fi

    if [ -f "$model_path" ]; then
        size=$(du -h "$model_path" | cut -f1)
        echo ""
        echo -e "${GREEN}✓ Download complete: $model_path ($size)${NC}"
    else
        echo -e "${RED}✗ Download failed${NC}"
        exit 1
    fi
}

# Main
if [ $# -eq 0 ]; then
    list_models
    echo ""
    echo "Usage: $0 <model>"
    echo "Example: $0 small.en"
    exit 0
fi

if [ "$1" == "list" ]; then
    list_models
    exit 0
fi

download_model "$1"

echo ""
echo "Model ready for use with Voice to Text!"
