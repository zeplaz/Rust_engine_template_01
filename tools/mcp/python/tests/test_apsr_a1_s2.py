"""Tests for APSR-A1-S2-ASM-001 — AssemblyService, no shadow _snapshot."""

from __future__ import annotations

import pytest

from rust_engine_mcp.apsr_a1_s2 import write_apsr_a1_s2_witness
from rust_engine_mcp.paths import repo_root


def test_assembly_service_enforces_field_owner() -> None:
    import sys

    mcp_dir = repo_root() / "tools/mcp"
    if str(mcp_dir) not in sys.path:
        sys.path.insert(0, str(mcp_dir))
    from art_pipeline_suite.aps_event_bus import EventBus
    from art_pipeline_suite.aps_services import AssemblyService
    from art_pipeline_suite.aps_state_writer import SuiteStateWriteError, SuiteStateWriter
    from art_pipeline_suite.state import SuiteState

    state = SuiteState()
    writer = SuiteStateWriter(EventBus())
    asm = AssemblyService(state, writer)

    asm.set_snapshot_data({"assembly_id": "asm_test", "module_placements": []})
    assert state.assembly_id == "asm_test"

    with pytest.raises(SuiteStateWriteError):
        writer.set(state, "assembly_snapshot_data", {"assembly_id": "bad"}, owner="AssemblyPanel")


def test_apsr_a1_s2_witness_green() -> None:
    body = write_apsr_a1_s2_witness(sync_allowlist=True)
    assert body["task_id"] == "APSR-A1-S2-ASM-001"
    assert body["green"] is True
    assert body["assembly_direct_write_sites"] == []
