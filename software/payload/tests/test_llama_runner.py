"""
Unit tests for LLaMA Runner Module

Tests model loading, text generation, and fallback behavior
for SMART-QSO Jetson payload.

Author: SMART-QSO Team
Date: 2026-01-02
Version: 1.0
"""

import unittest

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
    """Without a model the runner says nothing (SW-12): the caller falls back
    to health-derived text instead of broadcasting an invented status."""

    def test_no_model_no_text(self):
        """No llama_cpp or no model: empty text, not a canned status."""
        runner = LlamaRunner(LlamaConfig(model_path="/nonexistent/model.gguf"))

        self.assertFalse(runner.loaded)
        self.assertEqual(runner.generate_one_line("You are a satellite.", "Status please"), "")

    def test_internal_state_without_llama(self):
        """Test internal state when llama_cpp not available."""
        config = LlamaConfig(model_path="/test.gguf")
        runner = LlamaRunner(config)

        self.assertIsNone(runner._ll)
        self.assertEqual(runner._cfg, config)


class _FakeLlama:
    def __init__(self, text):
        self.text = text
        self.calls = []

    def create_completion(self, prompt, **kwargs):
        self.calls.append((prompt, kwargs))
        return {"choices": [{"text": self.text}]}


class TestLlamaRunnerGeneration(unittest.TestCase):
    """Generation settings that keep the output constrained (SW-12)."""

    def _runner(self, text="  Hello from orbit  "):
        runner = LlamaRunner(LlamaConfig(model_path="/test.gguf"))
        runner._ll = _FakeLlama(text)
        runner._grammar = "GRAMMAR"
        return runner

    def test_low_temperature_and_grammar(self):
        runner = self._runner()
        self.assertEqual(runner.generate_one_line("sys", "usr", max_tokens=40), "Hello from orbit")
        prompt, kwargs = runner._ll.calls[0]
        self.assertIn("sys", prompt)
        self.assertIn("usr", prompt)
        self.assertLessEqual(kwargs["temperature"], 0.3)
        self.assertEqual(kwargs["grammar"], "GRAMMAR")
        self.assertEqual(kwargs["max_tokens"], 40)

    def test_no_grammar_when_unavailable(self):
        runner = self._runner()
        runner._grammar = None
        runner.generate_one_line("sys", "usr")
        self.assertNotIn("grammar", runner._ll.calls[0][1])

    def test_grammar_admits_no_digits_or_field_separator(self):
        from llama_runner import BEACON_GBNF
        char_class = BEACON_GBNF[BEACON_GBNF.index("[") + 1:BEACON_GBNF.rindex("]")]
        self.assertNotIn("0-9", char_class)
        for ch in "0123456789|$@=*":
            self.assertNotIn(ch, char_class)
        self.assertIn("A-Za-z", char_class)


class TestLlamaRunnerPromptBuilding(unittest.TestCase):
    """The chat prompt sent to the model."""

    def _runner(self, text="Hello"):
        runner = LlamaRunner(LlamaConfig(model_path="/test.gguf"))
        runner._ll = _FakeLlama(text)
        return runner

    def test_prompt_includes_system_and_user(self):
        """System and user messages go into the chat template, in that order."""
        runner = self._runner()
        runner.generate_one_line(system="You are a helpful satellite.",
                                 user="What is your status?")
        prompt = runner._ll.calls[0][0]
        self.assertEqual(prompt, "<|system|>\nYou are a helpful satellite.\n"
                                 "<|user|>\nWhat is your status?\n<|assistant|>\n")

    def test_default_sampling(self):
        runner = self._runner()
        runner.generate_one_line("sys", "usr")
        kwargs = runner._ll.calls[0][1]
        self.assertEqual(kwargs["max_tokens"], 32)
        self.assertEqual(kwargs["top_k"], 3)


class TestLlamaRunnerEdgeCases(unittest.TestCase):
    """Edge cases in the prompt and in the model's reply."""

    def _runner(self, reply):
        runner = LlamaRunner(LlamaConfig(model_path="/test.gguf"))
        runner._ll = _FakeLlama("")
        runner._ll.create_completion = lambda prompt, **kw: reply
        return runner

    def test_empty_system_prompt(self):
        runner = LlamaRunner(LlamaConfig(model_path="/test.gguf"))
        runner._ll = _FakeLlama("ok")
        self.assertEqual(runner.generate_one_line(system="", user="Generate text"), "ok")
        self.assertEqual(runner._ll.calls[0][0],
                         "<|system|>\n\n<|user|>\nGenerate text\n<|assistant|>\n")

    def test_unicode_in_prompts_passed_through(self):
        runner = LlamaRunner(LlamaConfig(model_path="/test.gguf"))
        runner._ll = _FakeLlama("ok")
        runner.generate_one_line(system="System with émojis 🛰️", user="User with ünïcödé")
        prompt = runner._ll.calls[0][0]
        self.assertIn("émojis 🛰️", prompt)
        self.assertIn("ünïcödé", prompt)

    def test_whitespace_only_reply_is_empty(self):
        self.assertEqual(self._runner({"choices": [{"text": " \n "}]}).generate_one_line("s", "u"), "")

    def test_reply_without_choices_is_empty(self):
        self.assertEqual(self._runner({}).generate_one_line("s", "u"), "")

    def test_very_long_prompts_without_model(self):
        """No model: even an oversized prompt gives empty text, never an exception."""
        runner = LlamaRunner(LlamaConfig(model_path="/test.gguf", n_ctx=128))
        long_prompt = "A" * 10000
        self.assertEqual(runner.generate_one_line(system=long_prompt, user=long_prompt), "")


if __name__ == '__main__':
    unittest.main()
