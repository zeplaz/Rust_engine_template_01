"""APS-VALIDATOR-PLAIN-002 + APS-MAT-003 witness tests."""

from __future__ import annotations

import json

from rust_engine_mcp.aps_validator_plain import (
    PLAIN_ENTRIES,
    fix_hint,
    plain_sentence,
    refresh_aps_validator_plain_witness,
)
from rust_engine_mcp.material_category_tree import (
    infer_category_from_tree,
    refresh_aps_mat_003_witness,
    tree_roots,
)
from rust_engine_mcp.paths import repo_root


def test_plain_map_covers_p0_signatures():
    assert len(PLAIN_ENTRIES) >= 22
    assert plain_sentence("grammar_verify_footprint_min") == (
        "Footprint is too small to read as a building."
    )
    assert "3×3" in fix_hint("grammar_verify_footprint_min")


def test_refresh_validator_plain_witness():
    assert refresh_aps_validator_plain_witness()
    body = json.loads(
        (repo_root() / "debug_runs/aps_validator_plain_002_live.json").read_text(encoding="utf-8")
    )
    assert body["green"] is True
    assert body["code_count"] >= 22


def test_category_tree_loaded():
    roots = tree_roots()
    assert len(roots) >= 5
    assert infer_category_from_tree("steel_panel_01") == "industrial/steel"
    assert infer_category_from_tree("brick_red_01") == "residential/brick"


def test_refresh_mat_003_witness():
    assert refresh_aps_mat_003_witness()
    body = json.loads(
        (repo_root() / "debug_runs/aps_mat_003_category_tree_live.json").read_text(encoding="utf-8")
    )
    assert body["green"] is True
    assert body["widget_wired"] is True
