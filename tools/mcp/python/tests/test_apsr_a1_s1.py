"""Tests for APSR-A1-S1-001 — EventBus, SuiteStateWriter, AtlasService."""

from __future__ import annotations

import pytest

from rust_engine_mcp.apsr_a1_s1 import write_apsr_a1_s1_witness
from rust_engine_mcp.paths import repo_root


def test_suite_state_writer_enforces_atlas_owner() -> None:
    import sys

    mcp_dir = repo_root() / "tools/mcp"
    if str(mcp_dir) not in sys.path:
        sys.path.insert(0, str(mcp_dir))
    from art_pipeline_suite.aps_event_bus import EventBus
    from art_pipeline_suite.aps_services import AtlasService
    from art_pipeline_suite.aps_state_writer import SuiteStateWriteError, SuiteStateWriter
    from art_pipeline_suite.state import SuiteState

    state = SuiteState()
    bus = EventBus()
    writer = SuiteStateWriter(bus)
    atlas = AtlasService(state, writer)
    events: list[str] = []
    bus.subscribe("StateChanged", lambda p: events.append(str(p.get("field"))))

    atlas.set_atlas_folder("/tmp/atlas")
    assert state.atlas_folder == "/tmp/atlas"
    assert events == ["atlas_folder"]

    with pytest.raises(SuiteStateWriteError):
        writer.set(state, "atlas_folder", "/bad", owner="VariantsPanel")


def test_apsr_a1_s1_witness_green() -> None:
    body = write_apsr_a1_s1_witness(sync_allowlist=True)
    assert body["task_id"] == "APSR-A1-S1-001"
    assert body["green"] is True
    assert body["atlas_direct_write_sites"] == []
