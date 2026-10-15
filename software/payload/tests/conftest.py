"""
Pytest configuration for the payload test suite.

The deployed agent runs as ``python -m software.payload.agent`` from the
repository root (see ansible/deploy_payload.yml), so it imports
``software.payload.*``; the supervisor/health modules and the unit tests use
bare module imports. Put both roots on sys.path so the suite runs from any
working directory.
"""

import os
import sys

_PAYLOAD_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
_REPO_ROOT = os.path.dirname(os.path.dirname(_PAYLOAD_DIR))

for _path in (_REPO_ROOT, _PAYLOAD_DIR):
    if _path not in sys.path:
        sys.path.insert(0, _path)
