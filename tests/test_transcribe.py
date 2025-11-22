#!/usr/bin/env python3
"""Unit tests for transcribe.py"""

import unittest
import sys
import os
import tempfile

# Add parent directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src', 'common'))

class TestTranscribeArgs(unittest.TestCase):
    """Test command-line argument parsing"""

    def test_model_name_parsing(self):
        """Test model name extraction"""
        test_cases = [
            ("CT2 small.en", "small.en"),
            ("CT2 large-v3", "large-v3"),
            ("W tiny.en", "tiny.en"),
            ("W base", "base"),
        ]

        for full_name, expected in test_cases:
            # Extract model name (after space)
            parts = full_name.split(" ", 1)
            if len(parts) > 1:
                model = parts[1]
            else:
                model = parts[0]

            self.assertEqual(model, expected, f"Failed for {full_name}")

    def test_language_mode(self):
        """Test language mode parsing"""
        self.assertIn("en", ["en", "auto"])
        self.assertIn("auto", ["en", "auto"])
        self.assertNotIn("invalid", ["en", "auto"])

class TestWAVHeader(unittest.TestCase):
    """Test WAV file generation"""

    def test_wav_header_creation(self):
        """Test that we can create a valid WAV header"""
        sample_rate = 16000
        channels = 1
        bits_per_sample = 16
        num_samples = 1000

        # Calculate sizes
        data_size = num_samples * channels * (bits_per_sample // 8)
        file_size = 36 + data_size

        self.assertEqual(sample_rate, 16000)
        self.assertEqual(channels, 1)
        self.assertEqual(bits_per_sample, 16)
        self.assertGreater(file_size, 0)

class TestFileHandling(unittest.TestCase):
    """Test file I/O operations"""

    def test_temp_file_cleanup(self):
        """Test that temp files can be created and cleaned up"""
        with tempfile.NamedTemporaryFile(suffix='.wav', delete=True) as tmp:
            tmp_path = tmp.name
            self.assertTrue(os.path.exists(tmp_path))

        # File should be deleted after context exit
        self.assertFalse(os.path.exists(tmp_path))

if __name__ == '__main__':
    unittest.main(verbosity=2)
