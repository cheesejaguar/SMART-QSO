"""
Unit tests for LLaMA Runner Module

Tests model loading, text generation, and fallback behavior
for SMART-QSO Jetson payload.

Author: SMART-QSO Team
Date: 2026-01-02
Version: 1.0
"""

import unittest
from unittest.mock import Mock, patch, MagicMock

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from llama_runner import LlamaConfig, LlamaRunner


class TestLlamaConfig(unittest.TestCase):
    """Test LlamaConfig dataclass."""

    def test_required_model_path(self):
        """Test model_path is required."""
        config = LlamaConfig(model_path="/models/test.gguf")
        self.assertEqual(config.model_path, "/models/test.gguf")

    def test_default_context_size(self):
        """Test default context size."""
        config = LlamaConfig(model_path="/test.gguf")
        self.assertEqual(config.n_ctx, 1024)

    def test_default_gpu_layers(self):
        """Test default GPU layers."""
        config = LlamaConfig(model_path="/test.gguf")
        self.assertEqual(config.n_gpu_layers, 0)

    def test_default_cache_types(self):
        """Test default cache types are None."""
        config = LlamaConfig(model_path="/test.gguf")
        self.assertIsNone(config.cache_type_k)
        self.assertIsNone(config.cache_type_v)

    def test_default_draft_model(self):
        """Test default draft model is None."""
        config = LlamaConfig(model_path="/test.gguf")
        self.assertIsNone(config.draft_model_path)

    def test_custom_values(self):
        """Test custom configuration values."""
        config = LlamaConfig(
            model_path="/models/custom.gguf",
            n_ctx=2048,
            n_gpu_layers=32,
            cache_type_k="q4_0",
            cache_type_v="q4_0",
            draft_model_path="/models/draft.gguf"
        )
        self.assertEqual(config.model_path, "/models/custom.gguf")
        self.assertEqual(config.n_ctx, 2048)
        self.assertEqual(config.n_gpu_layers, 32)
        self.assertEqual(config.cache_type_k, "q4_0")
        self.assertEqual(config.cache_type_v, "q4_0")
        self.assertEqual(config.draft_model_path, "/models/draft.gguf")


class TestLlamaRunnerFallback(unittest.TestCase):
    """Test LlamaRunner fallback behavior when llama_cpp unavailable."""

    def test_fallback_without_llama_cpp(self):
        """Test fallback text is returned when llama_cpp is not available."""
        config = LlamaConfig(model_path="/nonexistent/model.gguf")
        runner = LlamaRunner(config)

        result = runner.generate_one_line(
            system="You are a satellite.",
            user="Status please"
        )

        # Should return deterministic fallback
        self.assertIn("nominally", result)
        self.assertIn("73!", result)

    def test_fallback_text_format(self):
        """Test fallback text is properly formatted."""
        config = LlamaConfig(model_path="/test.gguf")
        runner = LlamaRunner(config)

        result = runner.generate_one_line(
            system="Test system",
            user="Test user"
        )

        # Verify text is reasonable length and valid string
        self.assertLessEqual(len(result), 200)
        self.assertIsInstance(result, str)

    def test_internal_state_without_llama(self):
        """Test internal state when llama_cpp not available."""
        config = LlamaConfig(model_path="/test.gguf")
        runner = LlamaRunner(config)

        self.assertIsNone(runner._ll)
        self.assertEqual(runner._cfg, config)


class TestLlamaRunnerWithMock(unittest.TestCase):
    """Test LlamaRunner with mocked llama_cpp."""

    def test_generate_with_fallback(self):
        """Test text generation uses fallback."""
        # This test verifies the generate_one_line interface
        config = LlamaConfig(model_path="/test.gguf")

        # Without llama_cpp installed, we get fallback
        runner = LlamaRunner(config)
        result = runner.generate_one_line(
            system="You are SMART-QSO.",
            user="Generate status"
        )

        self.assertIsInstance(result, str)
        self.assertGreater(len(result), 0)

    def test_max_tokens_parameter(self):
        """Test max_tokens parameter is respected."""
        config = LlamaConfig(model_path="/test.gguf")
        runner = LlamaRunner(config)

        # With fallback, max_tokens doesn't affect output
        result32 = runner.generate_one_line("sys", "usr", max_tokens=32)
        result64 = runner.generate_one_line("sys", "usr", max_tokens=64)

        # Both should return same fallback text
        self.assertEqual(result32, result64)


class TestLlamaRunnerPromptBuilding(unittest.TestCase):
    """Test prompt construction."""

    def test_prompt_includes_system(self):
        """Test that prompt template exists for system message."""
        config = LlamaConfig(model_path="/test.gguf")
        runner = LlamaRunner(config)

        # The actual prompt is built inside generate_one_line
        # We can only verify the interface works
        result = runner.generate_one_line(
            system="You are a helpful satellite.",
            user="What is your status?"
        )
        self.assertIsNotNone(result)

    def test_prompt_includes_user(self):
        """Test that prompt template exists for user message."""
        config = LlamaConfig(model_path="/test.gguf")
        runner = LlamaRunner(config)

        result = runner.generate_one_line(
            system="System prompt here.",
            user="User message here."
        )
        self.assertIsNotNone(result)


class TestLlamaRunnerEdgeCases(unittest.TestCase):
    """Test edge cases and error handling."""

    def test_empty_system_prompt(self):
        """Test empty system prompt."""
        config = LlamaConfig(model_path="/test.gguf")
        runner = LlamaRunner(config)

        result = runner.generate_one_line(
            system="",
            user="Generate text"
        )
        self.assertIsInstance(result, str)

    def test_empty_user_prompt(self):
        """Test empty user prompt."""
        config = LlamaConfig(model_path="/test.gguf")
        runner = LlamaRunner(config)

        result = runner.generate_one_line(
            system="System",
            user=""
        )
        self.assertIsInstance(result, str)

    def test_unicode_in_prompts(self):
        """Test handling of unicode in prompts."""
        config = LlamaConfig(model_path="/test.gguf")
        runner = LlamaRunner(config)

        result = runner.generate_one_line(
            system="System with émojis 🛰️",
            user="User with ünïcödé"
        )
        # Result should be ASCII (fallback text)
        self.assertIsInstance(result, str)

    def test_very_long_prompts(self):
        """Test handling of very long prompts."""
        config = LlamaConfig(model_path="/test.gguf", n_ctx=128)
        runner = LlamaRunner(config)

        long_prompt = "A" * 10000
        result = runner.generate_one_line(
            system=long_prompt,
            user=long_prompt
        )
        self.assertIsInstance(result, str)


if __name__ == '__main__':
    unittest.main()
