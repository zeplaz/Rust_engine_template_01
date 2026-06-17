"""APS-OPTION-D-001 — E1 slice verification + witness rollup."""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness
from rust_engine_mcp.paths import repo_root

APS_OPTION_D_E1_WITNESS = "debug_runs/aps_option_d_e1_live.json"


def _ensure_aps_suite_path() -> None:
    suite_root = repo_root() / "tools/mcp"
    if str(suite_root) not in sys.path:
        sys.path.insert(0, str(suite_root))


def verify_e1_tab_swap(*, repo: Path | None = None) -> dict[str, Any]:
    _ = repo
    _ensure_aps_suite_path()
    from art_pipeline_suite.domain_router import (
        BUILDINGS_TAB_LABELS,
        LANDSCAPE_TAB_LABELS,
        verify_option_d_shell_implementation,
    )

    shell = verify_option_d_shell_implementation()
    return {
        "slice_id": "APS-E1-TAB-SWAP-001",
        "green": bool(shell.get("option_d_shell_ok")),
        "buildings_tab_labels": list(BUILDINGS_TAB_LABELS),
        "landscape_tab_labels": list(LANDSCAPE_TAB_LABELS),
        "landscape_tab_count": len(LANDSCAPE_TAB_LABELS),
        "materials_in_landscape_tabs": "Materials" in LANDSCAPE_TAB_LABELS,
        **shell,
    }


def verify_e1_flow_lane(*, repo: Path | None = None) -> dict[str, Any]:
    _ = repo
    _ensure_aps_suite_path()
    from art_pipeline_suite.domain_router import flow_verbs_for
    from art_pipeline_suite.state import ArtDomain

    landscape = [label for _, label in flow_verbs_for(ArtDomain.LANDSCAPE.value)]
    buildings = [label for _, label in flow_verbs_for(ArtDomain.BUILDINGS.value)]
    ok = landscape == ["Generate grammar", "Bake states", "Pack landscape atlas"]
    return {
        "slice_id": "APS-E1-FLOW-LANE-001",
        "green": ok,
        "landscape_flow_labels": landscape,
        "buildings_flow_labels": buildings,
    }


def verify_e1_pipeline_lane(*, repo: Path | None = None) -> dict[str, Any]:
    _ = repo
    _ensure_aps_suite_path()
    from art_pipeline_suite.domain_router import pipeline_steps_for
    from art_pipeline_suite.state import ArtDomain

    keys = [k for k, _ in pipeline_steps_for(ArtDomain.LANDSCAPE.value)]
    bkeys = [k for k, _ in pipeline_steps_for(ArtDomain.BUILDINGS.value)]
    ok = keys == ["presets", "grammar", "states", "atlas"] and bkeys == [
        "catalog",
        "materials",
        "assembly",
        "variants",
        "atlas",
    ]
    return {
        "slice_id": "APS-E1-PIPELINE-LANE-001",
        "green": ok,
        "landscape_pipeline_keys": keys,
        "buildings_pipeline_keys": bkeys,
        "stamp_folded_into_atlas": "stamp" not in keys,
        "has_stamp_step": False,
    }


def verify_e1_chrome(*, repo: Path | None = None) -> dict[str, Any]:
    _ = repo
    _ensure_aps_suite_path()
    from art_pipeline_suite import app as app_mod

    ok = bool(getattr(app_mod, "OPTION_D_DUAL_NOTEBOOK", False))
    return {
        "slice_id": "APS-E1-CHROME-001",
        "green": ok,
        "dual_notebook": ok,
        "design_ref": "src/dev/design_aps_chrome_mockup_spec_v1.md",
    }


def refresh_aps_option_d_e1_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    tab = verify_e1_tab_swap(repo=root)
    flow = verify_e1_flow_lane(repo=root)
    pipe = verify_e1_pipeline_lane(repo=root)
    chrome = verify_e1_chrome(repo=root)
    _ensure_aps_suite_path()
    from art_pipeline_suite.domain_router import des_aps_e1_ia_verdict

    ia = des_aps_e1_ia_verdict(repo=root)
    green = all(
        s.get("green") for s in (tab, flow, pipe, chrome)
    ) and ia.get("verdict") == "pass"
    body: dict[str, Any] = {
        "program_id": "APS-OPTION-D-001",
        "gate": "APS-E1-CRITICAL-PATH",
        "green": green,
        "slices": {
            "APS-E1-TAB-SWAP-001": tab,
            "APS-E1-FLOW-LANE-001": flow,
            "APS-E1-PIPELINE-LANE-001": pipe,
            "APS-E1-CHROME-001": chrome,
        },
        "des_aps_e1_ia_option_d_001": ia,
    }
    return write_aps_live_witness(
        body,
        APS_OPTION_D_E1_WITNESS,
        schema="aps_option_d_e1_live_v1",
        profile="APS_OPTION_D_E1",
        source_system="aps_option_d_e1",
        ritual="BLANG:WIT-HON APS-OPTION-D-E1" if green else None,
        exit_predicate_must=[{"path": "green", "eq": True}],
        repo=root,
    )
