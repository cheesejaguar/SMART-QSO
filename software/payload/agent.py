"""
Beacon agent: Jetson side of the OBC-Jetson line protocol.

Reads ICD lines (docs/interfaces/ICD_OBC_JETSON.md section 4) and answers the
OBC's BEACON_REQ with a BEACON_RSP carrying only the beacon text; the OBC adds
the call sign and telemetry and applies its own content checks. TELEMETRY
updates the health used for the prompt. STATUS_REQ and SHUTDOWN belong to the
supervisor and are ignored here. Lines that fail their CRC or format are
answered with NACK.

Runs as `python -m software.payload.agent` (stdin/stdout), or is called by
the supervisor through handle_line().
"""

from __future__ import annotations

import os
import sys
from dataclasses import dataclass, field
from typing import Optional

try:
    import obc_link
    from formatter import (Health, STATION_CALL, beacon_text, build_system_prompt,
                           build_user_prompt, health_from_telemetry, now_iso_z)
    from llama_runner import LlamaConfig, LlamaRunner
except ImportError:  # running as software.payload.* from the repository root
    from software.payload import obc_link
    from software.payload.formatter import (Health, STATION_CALL, beacon_text,
                                            build_system_prompt, build_user_prompt,
                                            health_from_telemetry, now_iso_z)
    from software.payload.llama_runner import LlamaConfig, LlamaRunner


def _default_health() -> Health:
    return Health(call=STATION_CALL, time_iso_z=now_iso_z(), mode="IDLE", soc=50,
                  sun=1, rf=1, pwr="FAIR", qso=0)


@dataclass
class AgentState:
    """Latest health from TELEMETRY and the optional language-model runner."""
    health: Health = field(default_factory=_default_health)
    runner: Optional[object] = None


def _generate(state: AgentState) -> Optional[str]:
    if state.runner is None:
        return None
    try:
        return state.runner.generate_one_line(build_system_prompt(),
                                              build_user_prompt(state.health),
                                              max_tokens=48)
    except Exception:
        return None


def handle_line(line: bytes, state: AgentState) -> Optional[bytes]:
    """Process one OBC line; return the reply line, if any."""
    try:
        msg = obc_link.parse(line)
        if msg.type == "TELEMETRY":
            state.health = health_from_telemetry(obc_link.parse_telemetry(msg))
            return None
        if msg.type == "BEACON_REQ":
            req = obc_link.parse_beacon_req(msg)
            text = beacon_text(state.health, _generate(state), req.max_len)
            return obc_link.beacon_rsp(req.seq, text)
    except obc_link.CrcError:
        return obc_link.nack("CRC")
    except obc_link.LinkError:
        return obc_link.nack("FORMAT")
    return None


def _make_runner() -> Optional[object]:
    model = os.environ.get("LLAMA_MODEL", "/models/Llama-3.2-3B-Instruct-Q4_K_M.gguf")
    if not os.path.exists(model):
        return None
    return LlamaRunner(LlamaConfig(
        model_path=model,
        n_ctx=int(os.environ.get("LLAMA_CTX", "1024")),
        n_gpu_layers=int(os.environ.get("LLAMA_GPU_LAYERS", "0")),
        cache_type_k=os.environ.get("LLAMA_CACHE_K"),
        cache_type_v=os.environ.get("LLAMA_CACHE_V"),
    ))


def run_stdio_loop():
    state = AgentState(runner=_make_runner())
    for raw in sys.stdin.buffer:
        reply = handle_line(raw, state)
        if reply:
            sys.stdout.buffer.write(reply)
            sys.stdout.buffer.flush()


if __name__ == "__main__":
    run_stdio_loop()
