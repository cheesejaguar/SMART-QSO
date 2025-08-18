from __future__ import annotations

import json
import os
import sys
from typing import Optional

from software.payload.formatter import (
    Health,
    build_system_prompt,
    build_user_prompt,
    compose_info_field,
    fallback_text,
    now_iso_z,
)
from software.payload.llama_runner import LlamaRunner, LlamaConfig


def parse_health_line(line: str) -> Optional[Health]:
    try:
        o = json.loads(line)
        return Health(
            call=str(o.get("call", "SMARTQ-1")),
            time_iso_z=str(o.get("time", now_iso_z())),
            mode=str(o.get("mode", "IDLE")).upper(),
            soc=int(o.get("soc", 50)),
            sun=int(1 if o.get("sun", True) else 0),
            rf=int(1 if o.get("rf", True) else 0),
            pwr=str(o.get("pwr", "FAIR")).upper(),
            qso=int(o.get("qso", 0)),
        )
    except Exception:
        return None


def run_stdio_loop():
    model = os.environ.get(
        "LLAMA_MODEL", "/models/Llama-3.2-3B-Instruct-Q4_K_M.gguf"
    )
    n_ctx = int(os.environ.get("LLAMA_CTX", "1024"))
    gpu_layers = int(os.environ.get("LLAMA_GPU_LAYERS", "0"))
    cache_k = os.environ.get("LLAMA_CACHE_K")
    cache_v = os.environ.get("LLAMA_CACHE_V")
    runner = LlamaRunner(
        LlamaConfig(
            model_path=model,
            n_ctx=n_ctx,
            n_gpu_layers=gpu_layers,
            cache_type_k=cache_k,
            cache_type_v=cache_v,
        )
    )

    sys_prompt = build_system_prompt()

    for line in sys.stdin:
        health = parse_health_line(line)
        if not health:
            continue
        # Policy gates are implemented by OBC; here we just format
        user = build_user_prompt(health)
        try:
            text = runner.generate_one_line(sys_prompt, user, max_tokens=32)
        except Exception:
            text = fallback_text(health)
        info = compose_info_field(health, text)
        print(info, flush=True)


if __name__ == "__main__":
    run_stdio_loop()
