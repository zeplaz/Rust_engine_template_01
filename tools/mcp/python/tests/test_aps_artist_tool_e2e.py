"""APS-ARTIST-TOOL-E2E-001 smoke."""

from __future__ import annotations

from rust_engine_mcp.aps_artist_tool_e2e import APS_ARTIST_TOOL_E2E_WITNESS, run_artist_tool_e2e
from rust_engine_mcp.aps_mat_auth_ui import plain_validation_lines, save_hint
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators.report import ValidationReport, ValidationIssue


def test_save_hint_counts_missing() -> None:
    snap = {"module_placements": [{"material_profile": "steel_panel_01"}, {}]}
    assert "missing material_profile" in save_hint(snap)


def test_plain_validation_passed() -> None:
    rep = ValidationReport(validator="assembly_p0", status="passed")
    assert "passed" in plain_validation_lines(rep)[0].lower()


def test_plain_validation_error() -> None:
    rep = ValidationReport(
        validator="assembly_p0",
        status="failed",
        errors=[
            ValidationIssue(
                kind="MissingMaterialProfile",
                severity="error",
                signature="material_profiles_placement_missing",
                hint="raw",
            )
        ],
    )
    lines = plain_validation_lines(rep)
    assert any("material profile" in ln.lower() for ln in lines)


def test_e2e_witness_written() -> None:
    body = run_artist_tool_e2e()
    assert body.get("program_id") == "APS-ARTIST-TOOL-E2E-001"
    assert (repo_root() / APS_ARTIST_TOOL_E2E_WITNESS).is_file()
    assert len(body.get("steps") or []) >= 5
    assert body.get("import_guard_pass") is True
    assert body.get("_agent_meta", {}).get("schema") == "aps_artist_tool_e2e_live_v1"
    assert body.get("exit_predicate", {}).get("must")
