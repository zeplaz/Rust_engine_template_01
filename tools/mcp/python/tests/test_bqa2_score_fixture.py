from __future__ import annotations
import json
from pathlib import Path
from rust_engine_mcp.building_quality_qc import overall_score_from_parts
from rust_engine_mcp.paths import repo_root

def test_bqa2_score_fixture_matches_python_weights() -> None:
    path = repo_root() / "tools/mcp/schemas/examples/bqa2_score_fixture_v1.json"
    doc = json.loads(path.read_text(encoding="utf-8"))
    assert doc.get("schema") == "bqa2_score_fixture_v1"
    for case in doc["cases"]:
        got = overall_score_from_parts(
            style_purity_pct=float(case["style_purity_pct"]),
            silhouette_continuity_pct=float(case["silhouette_continuity_pct"]),
            missing_slot_count=int(case["missing_slot_count"]),
            adjacency_violation_count=int(case["adjacency_violation_count"]),
        )
        assert abs(got - float(case["overall_score"])) < 1e-6, (case["id"], got, case["overall_score"])

def test_snapshot_honors_adjacency_field() -> None:
    from rust_engine_mcp.building_quality_qc import compute_snapshot_quality_score
    snap = {
        "module_placements": [{"module_id": "a", "slot_key": "wall_1u", "hidden": False}],
        "adjacency_violation_count": 2,
    }
    score = compute_snapshot_quality_score(snap)
    assert score["adjacency_violation_count"] == 2
    assert score["overall_score"] < 100.0
