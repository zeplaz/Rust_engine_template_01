"""APS-EVO-E1-DOMAIN-ROUTER-001 — domain router witness + headless checks."""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness
from rust_engine_mcp.landscape_preset_browse import list_landscape_presets
from rust_engine_mcp.paths import repo_root

APS_DOMAIN_ROUTER_WITNESS = "debug_runs/aps_domain_router_live.json"


def _ensure_aps_suite_path() -> None:
    suite_root = repo_root() / "tools/mcp"
    if str(suite_root) not in sys.path:
        sys.path.insert(0, str(suite_root))


def verify_catalog_source_switches(*, repo: Path | None = None) -> dict[str, Any]:
    """Headless proof that catalog source id changes per lane."""
    _ensure_aps_suite_path()
    from art_pipeline_suite.domain_router import catalog_source_for
    from module_viewer.model_store import list_modules

    root = repo or repo_root()
    buildings_src = catalog_source_for("buildings")
    landscape_src = catalog_source_for("landscape")
    modules_ok = len(list_modules()) >= 1
    presets = list_landscape_presets(repo=root)
    presets_ok = presets.get("ship_count", 0) >= 10
    switches = buildings_src != landscape_src and buildings_src == "building_modules" and landscape_src == "landscape_presets"
    return {
        "catalog_source_switches": switches,
        "buildings_catalog_source": buildings_src,
        "landscape_catalog_source": landscape_src,
        "building_modules_count": len(list_modules()),
        "landscape_preset_ship_count": presets.get("ship_count"),
        "modules_ok": modules_ok,
        "presets_ok": presets_ok,
    }


def refresh_aps_domain_router_witness(*, repo: Path | None = None) -> dict[str, Any]:
    _ensure_aps_suite_path()
    from art_pipeline_suite.domain_router import (
        BUILDINGS_TAB_LABELS,
        LANDSCAPE_TAB_LABELS,
        des_aps_e1_ia_verdict,
        load_active_lane,
        tab_labels_for,
    )

    root = repo or repo_root()
    check = verify_catalog_source_switches(repo=root)
    ia = des_aps_e1_ia_verdict(repo=root)
    green = bool(
        check.get("catalog_source_switches")
        and check.get("modules_ok")
        and check.get("presets_ok")
        and tab_labels_for("buildings") == BUILDINGS_TAB_LABELS
        and tab_labels_for("landscape") == LANDSCAPE_TAB_LABELS
        and ia.get("verdict") == "pass"
    )
    body: dict[str, Any] = {
        "program_id": "APS-EVO-E1-DOMAIN-ROUTER-001",
        "gate": "APS-E1-DOMAIN-ROUTER",
        "green": green,
        "active_lane_default": load_active_lane(repo=root),
        "tab_labels": {
            "buildings": list(BUILDINGS_TAB_LABELS),
            "landscape": list(LANDSCAPE_TAB_LABELS),
        },
        "des_aps_e1_ia_option_d_001": ia,
        **check,
        "design_ref": "src/dev/design_aps_domain_ia_sign_v1.md",
    }
    return write_aps_live_witness(
        body,
        APS_DOMAIN_ROUTER_WITNESS,
        schema="aps_domain_router_live_v1",
        profile="APS_E1_DOMAIN_ROUTER",
        source_system="aps_domain_router",
        ritual="BLANG:WIT-HON APS-EVO-E1-DOMAIN-ROUTER-001" if green else None,
        exit_predicate_must=[
            {"path": "green", "eq": True},
            {"path": "catalog_source_switches", "eq": True},
            {"path": "des_aps_e1_ia_option_d_001.verdict", "eq": "pass"},
        ],
        repo=root,
    )
