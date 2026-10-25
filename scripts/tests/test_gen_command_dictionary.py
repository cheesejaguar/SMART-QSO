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


def test_c_header_defines_the_id_enum():
    header = gen.render_h(gen.validate(dictionary(ENTRY)))
    assert "CMD_COMM_TX_DISABLE = 0x32," in header
    assert "#define CMD_DICTIONARY_COUNT 1U" in header


def test_rust_enum_and_table():
    source = gen.render_rust(gen.validate(dictionary(ENTRY)))
    assert "    TxDisable = 0x32," in source
    assert "0x32 => Some(Command::TxDisable)," in source
    assert ("Command::TxDisable => CommandSpec { min_args: 0, max_args: 0, "
            "key: KeyClass::Any, without_key: false },") in source


def test_rust_variant_names():
    assert gen.rust_variant("SET_CONTACT_TIMEOUT") == "SetContactTimeout"
    assert gen.rust_variant("NOP") == "Nop"


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


def test_without_key_flag_emitted():
    source = gen.render_rust(gen.validate(dictionary({**ENTRY, "without_key": True})))
    assert "key: KeyClass::Any, without_key: true }" in source
    module = gen.render_python(gen.validate(dictionary({**ENTRY, "without_key": True})))
    assert "without_key=True" in module


def test_without_key_requires_any_key():
    with pytest.raises(ValueError, match="without_key"):
        gen.validate(dictionary({**ENTRY, "key": "operational", "without_key": True}))


def test_duplicate_id_rejected():
    other = {**ENTRY, "name": "OTHER", "c_name": "CMD_OTHER"}
    with pytest.raises(ValueError, match="duplicate"):
        gen.validate(dictionary(ENTRY, other))


def test_source_is_valid_json():
    data = json.loads(gen.SOURCE.read_text())
    assert gen.validate(data)
