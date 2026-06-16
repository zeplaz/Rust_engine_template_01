"""APS-MAT-002 / APS-BEVY-PREVIEW-002 witness smoke."""

from __future__ import annotations

from rust_engine_mcp.aps_bevy_preview_002 import APS_BEVY_PREVIEW_002_WITNESS, run_aps_bevy_preview_002_smoke
from rust_engine_mcp.aps_mat_002 import APS_MAT_002_WITNESS, write_aps_mat_002_witness
from rust_engine_mcp.paths import repo_root


def test_mat_002_witness() -> None:
    body = write_aps_mat_002_witness()
    assert body.get("gate_id") == "APS-MAT-002"
    assert body.get("layout") == "studio_tree"
    assert (repo_root() / APS_MAT_002_WITNESS).is_file()


def test_bevy_preview_002_witness() -> None:
    body = run_aps_bevy_preview_002_smoke(open_browser=False)
    assert body.get("gate_id") == "APS-BEVY-PREVIEW-002"
    assert body.get("context_thumb_pipe", {}).get("wired") is True
    assert (repo_root() / APS_BEVY_PREVIEW_002_WITNESS).is_file()
