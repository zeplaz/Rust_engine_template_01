"""APS-EVO-E2-PRESET-BROWSE-001 — landscape preset browse witness."""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness
from rust_engine_mcp.landscape_preset_browse import list_landscape_presets, validate_landscape_preset
from rust_engine_mcp.paths import repo_root

APS_LANDSCAPE_PRESET_BROWSE_WITNESS = "debug_runs/aps_landscape_preset_browse_live.json"


def _ensure_aps_suite_path() -> None:
    suite_root = repo_root() / "tools/mcp"
    if str(suite_root) not in sys.path:
        sys.path.insert(0, str(suite_root))


def verify_landscape_preset_browse(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    listed = list_landscape_presets(repo=root)
    ship = [str(x) for x in (listed.get("ship_presets") or []) if x]
    validate_inline_green = True
    sample_id = ship[0] if ship else ""
    sample_status = "skipped"
    if sample_id:
        report = validate_landscape_preset(sample_id, repo=root)
        sample_status = report.status
        validate_inline_green = report.status == "passed"
    _ensure_aps_suite_path()
    from art_pipeline_suite.domain_router import des_aps_e1_ia_verdict

    ia = des_aps_e1_ia_verdict(repo=root)
    panels_ok = False
    try:
        from art_pipeline_suite.landscape_presets_panel import LandscapePresetsPanel
        from art_pipeline_suite.landscape_grammar_panel import LandscapeGrammarPanel
        from art_pipeline_suite.landscape_states_panel import LandscapeStatesPanel

        panels_ok = all(
            cls.__name__
            in ("LandscapePresetsPanel", "LandscapeGrammarPanel", "LandscapeStatesPanel")
            for cls in (LandscapePresetsPanel, LandscapeGrammarPanel, LandscapeStatesPanel)
        )
    except ImportError:
        panels_ok = False
    presets_listed = int(listed.get("ship_count") or 0)
    presets_listed_ok = presets_listed >= 10
    return {
        "presets_listed": presets_listed,
        "presets_listed_ok": presets_listed_ok,
        "validate_inline_green": validate_inline_green,
        "sample_preset_id": sample_id,
        "sample_validate_status": sample_status,
        "dedicated_landscape_panels": panels_ok,
        "des_aps_e1_ia_option_d_001": ia,
        "index_path": listed.get("index_path"),
    }


def refresh_aps_landscape_preset_browse_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    check = verify_landscape_preset_browse(repo=root)
    ia = check.get("des_aps_e1_ia_option_d_001") or {}
    green = bool(
        check.get("presets_listed_ok")
        and check.get("validate_inline_green")
        and check.get("dedicated_landscape_panels")
        and ia.get("verdict") == "pass"
    )
    body: dict[str, Any] = {
        "gate": "APS-EVO-E2-PRESET-BROWSE-001",
        "program_id": "APS-E2",
        "green": green,
        **check,
        "design_ref": "src/dev/design_aps_preset_qc_criteria_v1.md",
    }
    return write_aps_live_witness(
        body,
        APS_LANDSCAPE_PRESET_BROWSE_WITNESS,
        schema="aps_landscape_preset_browse_live_v1",
        profile="APS_E2_PRESET_BROWSE",
        source_system="aps_landscape_preset_browse",
        ritual="BLANG:WIT-HON APS-EVO-E2-PRESET-BROWSE-001" if green else None,
        exit_predicate_must=[
            {"path": "presets_listed_ok", "eq": True},
            {"path": "validate_inline_green", "eq": True},
            {"path": "dedicated_landscape_panels", "eq": True},
            {"path": "des_aps_e1_ia_option_d_001.verdict", "eq": "pass"},
        ],
        repo=root,
    )
