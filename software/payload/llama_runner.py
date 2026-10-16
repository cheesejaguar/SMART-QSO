from __future__ import annotations
from dataclasses import dataclass
from typing import Optional

#: Output grammar (llama.cpp GBNF): one line of letters, spaces and plain
#: punctuation. No digits, so the model cannot state telemetry values; the
#: OBC appends the measured ones. A subset of what content_filter accepts.
BEACON_GBNF = "root ::= [A-Za-z .,!?':;()/&-]{1,120}"

#: Sampling temperature: low, so the output stays close to the prompt
TEMPERATURE = 0.3


@dataclass
class LlamaConfig:
    model_path: str
    n_ctx: int = 1024
    n_gpu_layers: int = 0
    cache_type_k: Optional[str] = None
    cache_type_v: Optional[str] = None
    draft_model_path: Optional[str] = None


class LlamaRunner:
    """Thin wrapper for llama.cpp Python bindings.

    The dependency is optional so this can run in dev environments without
    GPU/VRAM. Without a model, generate_one_line() returns "" and the caller
    falls back to health-derived text (formatter.beacon_text); the runner
    never invents a status of its own.
    """

    def __init__(self, cfg: LlamaConfig):
        self._cfg = cfg
        self._ll = None
        self._grammar = None
        try:
            from llama_cpp import Llama  # type: ignore
            self._ll = Llama(
                model_path=cfg.model_path,
                n_ctx=cfg.n_ctx,
                n_gpu_layers=cfg.n_gpu_layers,
                type_k=cfg.cache_type_k,
                type_v=cfg.cache_type_v,
            )
        except Exception:
            self._ll = None
            return
        try:
            from llama_cpp import LlamaGrammar  # type: ignore
            self._grammar = LlamaGrammar.from_string(BEACON_GBNF)
        except Exception:
            self._grammar = None  # the content filter still applies

    @property
    def loaded(self) -> bool:
        return self._ll is not None

    def generate_one_line(
        self, system: str, user: str, max_tokens: int = 32
    ) -> str:
        if self._ll is None:
            return ""
        prompt = f"<|system|>\n{system}\n<|user|>\n{user}\n<|assistant|>\n"
        kwargs = dict(max_tokens=max_tokens, temperature=TEMPERATURE, top_k=3)
        if self._grammar is not None:
            kwargs["grammar"] = self._grammar
        out = self._ll.create_completion(prompt, **kwargs)
        txt = out.get("choices", [{}])[0].get("text", "").strip()
        return txt
