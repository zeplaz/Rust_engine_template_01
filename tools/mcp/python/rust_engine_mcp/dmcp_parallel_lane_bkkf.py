"""DMCP parallel lane — LG-5 expand bake + FactoryCluster + kit002 concept."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/art_pipeline/dmcp_parallel_lane_bkkf_live.json"

LANE_SLICES: tuple[dict[str, str], ...] = (
    {
        "id": "DMCP-LG5-EXPAND-BAKE-001",
        "deliverable": "src/dev/design_landscape_lg5_expand_bake_v1.md",
        "batch": "assets/staging/specs/tile_batch_landscape_expanded_v1.json",
        "charter": "src/dev/design_landscape_lg5_expansion_matrix_v1.md",
    },
    {
        "id": "DMCP-GRAM-ARCHETYPE-FACTORY-001",
        "deliverable": "src/dev/design_factory_cluster_concept_v1.md",
        "grammar": "assets/configs/buildings/grammars/factory_cluster_v1.ron",
    },
    {
        "id": "DMCP-MODULE-KIT002-001",
        "deliverable": "src/dev/design_kit_production_002_concept_v1.md",
        "manifest": "tools/mcp/schemas/examples/batch_kit_production_002.manifest.json",
        "g1": "src/dev/design_kit_production_002_g1_v1.md",
    },
)


def _check_factory_grammar(root: Path) -> dict[str, Any]:
    ron = root / "assets/configs/buildings/grammars/factory_cluster_v1.ron"
    if not ron.is_file():
        return {"green": False, "checks": {"ron_on_disk": False}}
    text = ron.read_text(encoding="utf-8")
    checks = {
        "ron_on_disk": True,
        "archetype_id": 'id: "FactoryCluster"' in text,
        "double_hall": "double_hall" in text,
        "manufacturing_row": "manufacturing_row" in text,
    }
    return {"green": all(checks.values()), "checks": checks}


def _check_kit002_manifest(root: Path) -> dict[str, Any]:
    path = root / "tools/mcp/schemas/examples/batch_kit_production_002.manifest.json"
    if not path.is_file():
        return {"green": False, "checks": {"manifest_on_disk": False}}
    body = json.loads(path.read_text(encoding="utf-8"))
    modules = body.get("modules") or []
    ids = {str(m.get("module_id")) for m in modules}
    expected = {
        "wall_steel_1u",
        "door_warehouse",
        "win_industrial_3u",
        "corner_L",
        "roof_industrial_shed_2u",
        "stack_chimney_1u",
    }
    checks = {
        "manifest_on_disk": True,
        "batch_id": body.get("batch_id") == "kit_production_002",
        "module_count_6": len(modules) == 6,
        "module_ids": ids == expected,
        "production_tier": body.get("development_tier") == "production",
    }
    return {"green": all(checks.values()), "checks": checks, "module_ids": sorted(ids)}


def _check_expanded_batch(root: Path) -> dict[str, Any]:
    batch_path = root / "assets/staging/specs/tile_batch_landscape_expanded_v1.json"
    if not batch_path.is_file():
        return {"green": False, "checks": {"batch_on_disk": False}}
    body = json.loads(batch_path.read_text(encoding="utf-8"))
    variants = body.get("variants") or []
    checks = {
        "batch_on_disk": True,
        "variant_count_16": len(variants) == 16,
        "bake_source_keyframe_pack": body.get("bake_source") == "keyframe_pack",
        "atlas_domain_landscape": body.get("atlas_domain") == "landscape",
        "matrix_ref": "design_landscape_lg5_expansion_matrix" in str(body.get("matrix_ref") or ""),
    }
    return {"green": all(checks.values()), "checks": checks}


def run_parallel_lane_bkkf_audit(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    rows: list[dict[str, Any]] = []
    for spec in LANE_SLICES:
        gate = spec["id"]
        deliverable = root / spec["deliverable"]
        row: dict[str, Any] = {
            "id": gate,
            "deliverable": spec["deliverable"],
            "deliverable_exists": deliverable.is_file(),
            "status": "done" if deliverable.is_file() else "fail",
        }
        if gate == "DMCP-LG5-EXPAND-BAKE-001":
            row["batch_audit"] = _check_expanded_batch(root)
            if not row["batch_audit"].get("green"):
                row["status"] = "fail"
            row["verdict"] = "PASS_WITH_NOTES"
        elif gate == "DMCP-GRAM-ARCHETYPE-FACTORY-001":
            row["grammar_audit"] = _check_factory_grammar(root)
            if not row["grammar_audit"].get("green"):
                row["status"] = "fail"
            row["verdict"] = "PASS"
        elif gate == "DMCP-MODULE-KIT002-001":
            row["manifest_audit"] = _check_kit002_manifest(root)
            if not row["manifest_audit"].get("green"):
                row["status"] = "fail"
            row["verdict"] = "PASS_WITH_NOTES"
            row["g4_open"] = True
        rows.append(row)

    done = sum(1 for r in rows if r["status"] == "done")
    green = done == len(rows)
    return {
        "gate": "DMCP-PARALLEL-LANE-BKKF-001",
        "lanes": ["LG-5 expanded bake", "FactoryCluster", "kit002 concept"],
        "slice_count": len(rows),
        "done_count": done,
        "rows": rows,
        "audit_complete": True,
        "green": green,
        "verdict": "PASS_WITH_NOTES" if green else "FAIL",
        "handoff": {
            "coder_mcp_lg5": "python -m rust_engine_mcp.landscape_lg5_expanded_batch",
            "coder_mcp_kit_g4": "operator keyframes → kit-production-002-g4-evaluate",
            "designer": "DES-STYLE-INDUSTRIAL-WEST-001 parallel",
        },
    }


def refresh_dmcp_parallel_lane_bkkf_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = run_parallel_lane_bkkf_audit(repo=root)
    body["_agent_meta"] = {
        "schema": "dmcp_parallel_lane_bkkf_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "DMCP_PARALLEL_BKKF",
        "source_system": "dmcp_parallel_lane_bkkf",
        "relative_path": WITNESS_REL,
        "ritual": "BLANG:WIT-HON→Q✓ DMCP-PARALLEL-LANE-BKKF" if body.get("green") else "BLANG:WIT-HON FAIL",
        "agent": "designer-mcp",
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
