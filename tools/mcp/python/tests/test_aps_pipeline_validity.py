"""P1-4 / P1-5 regressions — pipeline-bar validity state + honest E2E build-health gate."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

APS_ROOT = Path(__file__).resolve().parents[2]  # tools/mcp
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))
if str(APS_ROOT / "python") not in sys.path:
    sys.path.insert(0, str(APS_ROOT / "python"))


# ---- P1-4: pipeline bar reflects validity, not just presence ----------------

def _pill_text(bar, key: str) -> str:
    _pill, lbl = bar._pills[key]
    return lbl.cget("text")


@pytest.mark.aps_gui
def test_assembly_step_three_states(tk_root) -> None:
    from art_pipeline_suite.pipeline_status_bar import PipelineStatusBar
    from art_pipeline_suite.state import SuiteState

    state = SuiteState()
    bar = PipelineStatusBar(tk_root, state)

    bar.refresh()
    assert _pill_text(bar, "assembly") == "○ Assembly pending"

    state.assembly_snapshot_path = "assets/staging/assemblies/x.json"
    state.assembly_p0_passed = None
    bar.refresh()
    txt = _pill_text(bar, "assembly")
    assert "✓" not in txt, f"unvalidated snapshot must not show ✓: {txt!r}"
    assert "saved" in txt and "not checked" in txt

    state.assembly_p0_passed = False
    bar.refresh()
    txt = _pill_text(bar, "assembly")
    assert "✓" not in txt, f"P0-failing snapshot must not show ✓: {txt!r}"
    assert "blocked" in txt

    state.assembly_p0_passed = True
    bar.refresh()
    assert _pill_text(bar, "assembly") == "✓ Assembly valid"


def test_suite_state_has_p0_field() -> None:
    from art_pipeline_suite.state import SuiteState

    assert SuiteState().assembly_p0_passed is None


# ---- P1-5: E2E witness must never be green over a broken tree ----------------

def test_e2e_build_health_gate_blocks_green(monkeypatch, tmp_path) -> None:
    import rust_engine_mcp.aps_artist_tool_e2e as e2e

    monkeypatch.setattr(
        e2e,
        "check_build_health",
        lambda: {
            "ok": False,
            "import_ok": False,
            "import_error": "cannot import name 'bind_aps_tooltip'",
            "collect_ok": False,
            "collect_summary": "9 errors",
            "reason": "APS app import failed: ImportError",
        },
    )
    monkeypatch.setattr(e2e, "APS_ARTIST_TOOL_E2E_WITNESS", "debug_runs/_aps_e2e_unittest_TEMP.json")
    body = e2e.run_artist_tool_e2e()
    assert body["green"] is False, "broken build must force green=false"
    assert body.get("not_green_reason"), "must record why it is not green"
    assert body["build_health"]["ok"] is False
    from rust_engine_mcp.paths import repo_root

    out = repo_root() / "debug_runs/_aps_e2e_unittest_TEMP.json"
    if out.is_file():
        out.unlink()


def test_e2e_build_health_check_shape() -> None:
    from rust_engine_mcp.aps_artist_tool_e2e import check_build_health

    health = check_build_health()
    for key in ("ok", "import_ok", "collect_ok", "reason"):
        assert key in health
    if not health["ok"]:
        assert health["reason"]
