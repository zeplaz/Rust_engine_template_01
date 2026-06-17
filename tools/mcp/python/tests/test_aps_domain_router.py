"""APS-EVO-E1/E2 — Option D lane router + landscape preset browse tests."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

APS_ROOT = Path(__file__).resolve().parents[2]
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))

from rust_engine_mcp.aps_domain_router import (
    APS_DOMAIN_ROUTER_WITNESS,
    refresh_aps_domain_router_witness,
    verify_catalog_source_switches,
)
from rust_engine_mcp.aps_landscape_preset_browse import (
    APS_LANDSCAPE_PRESET_BROWSE_WITNESS,
    refresh_aps_landscape_preset_browse_witness,
    verify_landscape_preset_browse,
)
from rust_engine_mcp.paths import repo_root


def test_catalog_source_ids_switch_per_lane() -> None:
    from art_pipeline_suite.domain_router import catalog_source_for

    assert catalog_source_for("buildings") == "building_modules"
    assert catalog_source_for("landscape") == "landscape_presets"
    assert catalog_source_for("buildings") != catalog_source_for("landscape")


def test_option_d_tab_labels() -> None:
    from art_pipeline_suite.domain_router import tab_labels_for

    assert tab_labels_for("buildings") == ("Catalog", "Materials", "Assembly", "Variants", "Atlas")


def test_option_d_pipeline_and_flow_lane_scoped() -> None:
    from art_pipeline_suite.domain_router import flow_verbs_for, pipeline_steps_for

    b_pipe = [k for k, _ in pipeline_steps_for("buildings")]
    l_pipe = [k for k, _ in pipeline_steps_for("landscape")]
    assert b_pipe == ["catalog", "materials", "assembly", "variants", "atlas"]
    assert l_pipe == ["presets", "grammar", "states", "atlas"]
    assert [k for k, _ in flow_verbs_for("landscape")] == [
        "generate_grammar",
        "bake_states",
        "pack_lg5_atlas",
    ]


def test_option_d_ia_contract_and_shell() -> None:
    from art_pipeline_suite import app as app_mod
    from art_pipeline_suite.domain_router import des_aps_e1_ia_verdict, verify_option_d_ia_contract

    contract = verify_option_d_ia_contract()
    assert contract["option_d_ia_contract_ok"] is True
    assert contract["tab_set_swap"] is True
    ia = des_aps_e1_ia_verdict()
    assert ia["verdict"] == "pass"
    assert app_mod.OPTION_D_DUAL_NOTEBOOK is True


def test_active_lane_prefs_roundtrip(tmp_path: Path) -> None:
    from art_pipeline_suite.domain_router import load_active_lane, save_active_lane

    save_active_lane("landscape", repo=tmp_path)
    assert load_active_lane(repo=tmp_path) == "landscape"


def test_verify_catalog_source_switches_headless() -> None:
    body = verify_catalog_source_switches()
    assert body["catalog_source_switches"] is True
    assert body["building_modules_count"] >= 1
    assert body["landscape_preset_ship_count"] >= 10


def test_refresh_domain_router_witness_green() -> None:
    body = refresh_aps_domain_router_witness()
    assert body.get("green") is True
    assert body.get("catalog_source_switches") is True
    assert body.get("des_aps_e1_ia_option_d_001", {}).get("verdict") == "pass"
    assert (repo_root() / APS_DOMAIN_ROUTER_WITNESS).is_file()
    written = json.loads((repo_root() / APS_DOMAIN_ROUTER_WITNESS).read_text(encoding="utf-8"))
    assert written.get("_agent_meta", {}).get("schema") == "aps_domain_router_live_v1"
    assert written.get("exit_predicate", {}).get("must")


def test_landscape_preset_browse_verify() -> None:
    body = verify_landscape_preset_browse()
    assert body["presets_listed"] >= 10
    assert body["validate_inline_green"] is True
    assert body["dedicated_landscape_panels"] is True
    assert body["des_aps_e1_ia_option_d_001"]["verdict"] == "pass"


def test_refresh_landscape_preset_browse_witness() -> None:
    body = refresh_aps_landscape_preset_browse_witness()
    assert body.get("green") is True
    assert (repo_root() / APS_LANDSCAPE_PRESET_BROWSE_WITNESS).is_file()
