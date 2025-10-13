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
    Detect optimal device and compute type for this system

    Returns:
        (device, compute_type) tuple
    """
    import platform
    import subprocess

    system = platform.system()
    machine = platform.machine()

    # Linux - Check for NVIDIA CUDA
    if system == "Linux":
        try:
            # Check if nvidia-smi exists and works
            result = subprocess.run(['nvidia-smi'],
                                    capture_output=True,
                                    timeout=2,
                                    check=False)
            if result.returncode == 0:
                # CUDA GPU detected - now check for cuDNN
                import ctypes.util
                cudnn_lib = ctypes.util.find_library('cudnn')

                if cudnn_lib:
                    # Both CUDA and cuDNN available - use GPU
                    logger.info("✓ CUDA GPU detected with cuDNN - using GPU acceleration")

                    try:
                        gpu_info = subprocess.run(['nvidia-smi', '--query-gpu=name', '--format=csv,noheader'],
                                                 capture_output=True, text=True, timeout=2, check=False)
                        if gpu_info.returncode == 0:
                            gpu_name = gpu_info.stdout.strip().split('\n')[0]
                            logger.info(f"✓ GPU: {gpu_name}")
                    except:
                        pass

                    return "cuda", "float16"
                else:
                    # CUDA available but cuDNN missing - use CPU
                    logger.warning("⚠ CUDA GPU detected but cuDNN not found - using CPU")
                    logger.info("  Install cuDNN for GPU acceleration: sudo apt install libcudnn8")
                    return "cpu", "int8"
        except (FileNotFoundError, subprocess.TimeoutExpired):
            pass

        # No CUDA - fall back to CPU
        logger.info("CUDA not available - using CPU with INT8 quantization")
        return "cpu", "int8"

    # macOS - Check architecture
    elif system == "Darwin":
        if machine == "arm64":
            # Apple Silicon - try CoreML/Metal
            logger.info("Detected Apple Silicon (arm64)")
            return "cpu", "int8"  # CoreML support in faster-whisper is experimental
        else:
            # Intel Mac - use CPU with int8 quantization
            logger.info("Detected Intel Mac (x86_64)")
            return "cpu", "int8"

    # Other platforms - default to CPU
    else:
        logger.info(f"Unknown platform: {system} - using CPU")
        return "cpu", "int8"


def transcribe_audio(
    audio_path: str,
    model_size: str = "small.en",
    language: str = "en",
    device: Optional[str] = None,
    compute_type: Optional[str] = None
) -> str:
    """
    Transcribe audio file using faster-whisper

    Args:
        audio_path: Path to audio file (WAV, 16kHz mono)
        model_size: Model size (tiny, base, small, medium, large)
        language: Language code ("en" for English, "auto" for auto-detect)
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

    # Detect if model is English-only (.en suffix) or multilingual
    is_english_only = model_size.endswith(".en")

    # Determine language setting for transcription
    # If language="en", use "en" for faster transcription
    # If language="auto", use None for auto-detection (multilingual models only)
    lang_setting = "en" if language == "en" else None

    logger.info(f"Loading model: {model_size} ({'English-only' if is_english_only else 'Multilingual'}) ({device}, {compute_type})")
    logger.info(f"Language: {'English (faster)' if language == 'en' else 'Auto-detect (99 languages)'}")

    try:
        # Initialize model (cached after first load)
        model = WhisperModel(model_size, device=device, compute_type=compute_type)
        logger.info(f"✓ Model loaded: {model_size} on {device.upper()} with {compute_type}")

        # Transcribe with faster-whisper
        segments, info = model.transcribe(
            audio_path,
            language=lang_setting,
            beam_size=5,
            vad_filter=False,  # Disable VAD - too aggressive
            word_timestamps=True,  # Enable word-level timestamps
            initial_prompt="Transcribe this voice command."
        )

        # Collect all segments
        text_segments = []
        for segment in segments:
            # Strip newlines and extra whitespace from each segment
            segment_text = segment.text.replace('\n', ' ').replace('\r', ' ').strip()
            text_segments.append(segment_text)

        # Combine text - single line output
        text = " ".join(text_segments).strip()

        logger.info(f"Transcription complete: {len(text)} chars, {info.duration:.1f}s audio")

        return text

    except Exception as e:
        # If CUDA fails (e.g., cuDNN not installed), fall back to CPU
        if device == "cuda" and ("cudnn" in str(e).lower() or "cuda" in str(e).lower()):
            logger.warning(f"CUDA failed ({e}), falling back to CPU...")
            try:
                model = WhisperModel(model_size, device="cpu", compute_type="int8")
                logger.info("✓ Fallback successful - using CPU with INT8")

                segments, info = model.transcribe(
                    audio_path,
                    language=lang_setting,
                    beam_size=5,
                    vad_filter=False,
                    word_timestamps=True,
                    initial_prompt="Transcribe this voice command."
                )

                text_segments = []
                for segment in segments:
                    segment_text = segment.text.replace('\n', ' ').replace('\r', ' ').strip()
                    text_segments.append(segment_text)

                text = " ".join(text_segments).strip()
                logger.info(f"✓ CPU transcription: {len(text)} chars")
                return text
            except Exception as fallback_error:
                logger.error(f"CPU fallback also failed: {fallback_error}")
                return ""

        logger.error(f"Transcription failed: {e}")
        return ""


def main():
    """CLI interface for VTT app"""
    if len(sys.argv) < 2:
        print("Usage: faster_whisper_wrapper.py <audio_file> [model_size] [language]", file=sys.stderr)
        sys.exit(1)

    audio_path = sys.argv[1]
    model_size = sys.argv[2] if len(sys.argv) > 2 else "small.en"
    language = sys.argv[3] if len(sys.argv) > 3 else "en"

    # Transcribe
    text = transcribe_audio(audio_path, model_size, language)

    # Output text to stdout (VTT app reads this)
    if text:
        print(text)
        sys.exit(0)
    else:
        print("", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
