import os
import subprocess
import sys
from pathlib import Path


def run_agent_with_input(input_text: str) -> str:
    env = os.environ.copy()
    # Force fallback path (no llama_cpp installed)
    env.setdefault("LLAMA_MODEL", "/nonexistent/model.gguf")
    p = subprocess.Popen(
        [sys.executable, "-m", "software.payload.agent"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=env,
        text=True,
    )
    out, err = p.communicate(input=input_text, timeout=10)
    assert p.returncode == 0, (
        f"agent exited with {p.returncode}: {err}"
    )
    return out.strip()


def test_agent_processes_sample_jsonl():
    sample = Path(__file__).parent.parent / "testdata" / "health_sample.jsonl"
    data = sample.read_text().strip().splitlines()
    outputs = []
    for line in data:
        outputs.append(run_agent_with_input(line))
    # Ensure outputs look like AX.25 info field with ' | ' and tail keys
    for out in outputs:
        assert " | " in out
        ok_tail = (("T=" in out) and ("M=" in out) and ("SOC=" in out))
        assert ok_tail
