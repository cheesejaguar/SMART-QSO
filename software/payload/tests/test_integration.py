"""
End to end: the deployed agent process speaks the ICD line protocol.

Feeds OBC lines (testdata/obc_lines.txt, the golden vectors shared with the
flight tests) to `python -m software.payload.agent` and checks the reply.
"""

import os
import subprocess
import sys
from pathlib import Path

import obc_link

REPO_ROOT = Path(__file__).resolve().parents[3]


def test_agent_answers_beacon_request():
    env = os.environ.copy()
    env.setdefault("LLAMA_MODEL", "/nonexistent/model.gguf")  # force the fallback path
    lines = (Path(__file__).parent.parent / "testdata" / "obc_lines.txt").read_bytes()
    p = subprocess.run(
        [sys.executable, "-m", "software.payload.agent"],
        input=lines, capture_output=True, env=env, cwd=REPO_ROOT, timeout=20,
    )
    assert p.returncode == 0, p.stderr.decode()
    replies = [line + b"\n" for line in p.stdout.splitlines() if line]
    assert len(replies) == 1
    msg = obc_link.parse(replies[0])
    assert msg.type == "BEACON_RSP"
    assert msg.fields[0] == "42"
    assert 0 < len(msg.fields[1]) <= obc_link.TEXT_MAX
