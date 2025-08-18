from __future__ import annotations
from dataclasses import dataclass
from typing import Optional


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

    The actual dependency is optional so this can run in dev environments
    without GPU/VRAM. On flight, bind to the real llama_cpp APIs.
    """

    def __init__(self, cfg: LlamaConfig):
        self._cfg = cfg
        self._ll = None
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

    def generate_one_line(
        self, system: str, user: str, max_tokens: int = 32
    ) -> str:
        if self._ll is None:
            # Deterministic fallback in dev
            return (
                "I am operating nominally—sunlit ops with healthy power. 73!"
            )
        prompt = f"<|system|>\n{system}\n<|user|>\n{user}\n<|assistant|>\n"
        out = self._ll.create_completion(
            prompt,
            max_tokens=max_tokens,
            temperature=0.7,
            top_k=3,
        )
        txt = out.get("choices", [{}])[0].get("text", "").strip()
        return txt
