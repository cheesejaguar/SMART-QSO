"""Tests for the command dictionary generator (scripts/gen_command_dictionary.py)."""

import json
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import gen_command_dictionary as gen  # noqa: E402

ENTRY = {"name": "TX_DISABLE", "id": "0x32", "c_name": "CMD_COMM_TX_DISABLE",
         "min_args": 0, "max_args": 0, "key": "any", "summary": "Latch TX-OFF"}


def dictionary(*entries):
    return {"version": 1, "commands": list(entries)}


def test_repository_outputs_are_current():
    """The checked-in generated files match the JSON source."""
    assert gen.check() == []


def test_c_table_uses_enum_symbols():
    source = gen.render_c(gen.validate(dictionary(ENTRY)))
    assert "(uint8_t)CMD_COMM_TX_DISABLE, 0U, 0U, CMD_KEY_ANY" in source


def test_python_module_round_trips():
    module = gen.render_python(gen.validate(dictionary(ENTRY)))
    namespace = {}
    exec(compile(module, "command_dictionary.py", "exec"), namespace)
    spec = namespace["COMMANDS"]["TX_DISABLE"]
    assert (spec.cmd_id, spec.min_args, spec.max_args, spec.key) == (0x32, 0, 0, "any")


@pytest.mark.parametrize("change, message", [
    ({"id": "0x100"}, "id"),
    ({"min_args": 2, "max_args": 1}, "args"),
    ({"max_args": 65}, "args"),
    ({"key": "master"}, "key"),
    ({"c_name": "tx_disable"}, "c_name"),
])
def test_invalid_entries_rejected(change, message):
    with pytest.raises(ValueError, match=message):
        gen.validate(dictionary({**ENTRY, **change}))


def test_duplicate_id_rejected():
    other = {**ENTRY, "name": "OTHER", "c_name": "CMD_OTHER"}
    with pytest.raises(ValueError, match="duplicate"):
        gen.validate(dictionary(ENTRY, other))


def test_source_is_valid_json():
    data = json.loads(gen.SOURCE.read_text())
    assert gen.validate(data)
