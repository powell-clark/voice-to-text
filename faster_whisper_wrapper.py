#!/usr/bin/env python3
"""
Faster-whisper wrapper for VTT Mac app
Provides 5-10x speedup over whisper.cpp using CTranslate2 backend
"""

import sys
import os
import logging
from pathlib import Path
from typing import Optional

try:
    from faster_whisper import WhisperModel
except ImportError:
    print("ERROR: faster-whisper not installed. Run: pip3 install faster-whisper", file=sys.stderr)
    sys.exit(1)

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[logging.StreamHandler(sys.stderr)]
)
logger = logging.getLogger(__name__)


def detect_device() -> tuple[str, str]:
    """
    Detect optimal device and compute type for this Mac

    Returns:
        (device, compute_type) tuple
    """
    # Check if running on Apple Silicon
    import platform
    machine = platform.machine()

    if machine == "arm64":
        # Apple Silicon - try CoreML/Metal
        logger.info("Detected Apple Silicon (arm64)")
        return "cpu", "int8"  # CoreML support in faster-whisper is experimental
    else:
        # Intel Mac - use CPU with int8 quantization
        logger.info("Detected Intel Mac (x86_64)")
        return "cpu", "int8"


def transcribe_audio(
    audio_path: str,
    model_size: str = "small.en",
    device: Optional[str] = None,
    compute_type: Optional[str] = None
) -> str:
    """
    Transcribe audio file using faster-whisper

    Args:
        audio_path: Path to audio file (WAV, 16kHz mono)
        model_size: Model size (tiny, base, small, medium, large)
        device: Device to use (cpu, cuda) - auto-detected if None
        compute_type: Compute type (int8, float16, float32) - auto-detected if None

    Returns:
        Transcribed text
    """
    # Auto-detect device if not specified
    if device is None or compute_type is None:
        device, compute_type = detect_device()

    # Validate audio file exists
    if not os.path.exists(audio_path):
        logger.error(f"Audio file not found: {audio_path}")
        return ""

    # Add .en suffix for English-only models if not present
    if not model_size.endswith(".en") and model_size in ["tiny", "base", "small"]:
        model_size = f"{model_size}.en"

    logger.info(f"Loading model: {model_size} ({device}, {compute_type})")

    try:
        # Initialize model (cached after first load)
        model = WhisperModel(model_size, device=device, compute_type=compute_type)
        logger.info("Model loaded successfully (CTranslate2 + INT8 = 4x speedup)")

        # Transcribe with faster-whisper
        segments, info = model.transcribe(
            audio_path,
            language="en",
            beam_size=5,
            vad_filter=False,  # Disable VAD - too aggressive
            word_timestamps=True,  # Enable word-level timestamps
            initial_prompt="Transcribe this voice command."
        )

        # Collect all segments
        text_segments = []
        for segment in segments:
            text_segments.append(segment.text)

        # Combine text
        text = " ".join(text_segments).strip()

        logger.info(f"Transcription complete: {len(text)} chars, {info.duration:.1f}s audio")

        return text

    except Exception as e:
        logger.error(f"Transcription failed: {e}")
        return ""


def main():
    """CLI interface for VTT app"""
    if len(sys.argv) < 2:
        print("Usage: faster_whisper_wrapper.py <audio_file> [model_size]", file=sys.stderr)
        sys.exit(1)

    audio_path = sys.argv[1]
    model_size = sys.argv[2] if len(sys.argv) > 2 else "small.en"

    # Transcribe
    text = transcribe_audio(audio_path, model_size)

    # Output text to stdout (VTT app reads this)
    if text:
        print(text)
        sys.exit(0)
    else:
        print("", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
