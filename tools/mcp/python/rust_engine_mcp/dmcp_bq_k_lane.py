"""BQ-K1/K2/K3 — designer-mcp charter sign-off witnesses."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.bq_k1_kitfill_catalog import BATCH_REL, CHARTER_REL as K1_CHARTER, K1_JOBS
from rust_engine_mcp.kit_coverage_audit import (
    CHARTER_REL as K2_CHARTER,
    WITNESS_REL as K2_WITNESS,
    audit_bq_k2_coverage,
    write_bq_k2_witness,
)
from rust_engine_mcp.paths import repo_root

K1_WITNESS = "debug_runs/bq_k1_kitfill_001_live.json"
K3_WITNESS = "debug_runs/bq_k3_grammar_001_live.json"
K3_MANIFEST = "tools/mcp/schemas/examples/bq_k3_grammar_enrichment_v1.json"
K3_CHARTER = "src/dev/design_bq_k3_grammar_enrichment_v1.md"
K_LANE_WITNESS = "debug_runs/bq_k_lane_charters_live.json"

K1_GATE = "BQ-K1-KITFILL-001"
K2_GATE = "BQ-K2-COVERAGE-001"
K3_GATE = "BQ-K3-GRAMMAR-001"


def _load_json(path: Path) -> dict[str, Any] | None:
    if not path.is_file():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def run_bq_k1_kitfill_audit(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    charter_ok = (root / K1_CHARTER).is_file()
    batch = _load_json(root / BATCH_REL)
    job_rows: list[dict[str, Any]] = []
    for job in K1_JOBS:
        mid = job["module_id"]
        spec_rel = f"assets/staging/specs/kit_fill/{mid}_production.json"
        geom_rel = f"tools/mcp/schemas/examples/geometry_job_{mid}_production_run001.json"
        spec_path = root / spec_rel
        geom_path = root / geom_rel
        spec_body = _load_json(spec_path) if spec_path.is_file() else None
        job_rows.append(
            {
                "module_id": mid,
                "material_family": job["material_family"],
                "category": job["category"],
                "spec_rel": spec_rel,
                "geometry_job_rel": geom_rel,
                "spec_on_disk": spec_path.is_file(),
                "geometry_on_disk": geom_path.is_file(),
                "contract_ref": bool(
                    spec_body
                    and "module_contract_v1" in str(spec_body.get("references") or [])
                ),
                "deterministic_seed": bool(
                    _load_json(geom_path) and (_load_json(geom_path) or {}).get("params", {}).get("seed")
                ),
            }
        )
    checks = {
        "charter_doc": charter_ok,
        "batch_manifest": batch is not None and batch.get("job_count") == len(K1_JOBS),
        "job_count_11": len(job_rows) == 11,
        "all_specs_on_disk": all(r["spec_on_disk"] for r in job_rows),
        "all_geometry_on_disk": all(r["geometry_on_disk"] for r in job_rows),
        "all_contract_refs": all(r["contract_ref"] for r in job_rows),
    }
    green = all(checks.values())
    return {
        "gate": K1_GATE,
        "program": "PLAN-BUILDING-QUALITY-v1",
        "charter_doc": K1_CHARTER,
        "batch_rel": BATCH_REL,
        "jobs": job_rows,
        "checks": checks,
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "handoff": {"coder_mcp": "kit_fill_bq_k1_001 batch bake + style_pack wire"},
    }


def run_bq_k3_grammar_audit(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    manifest = _load_json(root / K3_MANIFEST)
    patches = (manifest or {}).get("grammar_patches") or []
    age_map = (manifest or {}).get("age_band_aps_map") or {}
    massing_ok = all(len(p.get("add_massing_strategies") or []) >= 2 for p in patches)
    facade_ok = all(len(p.get("facade_by_massing_add") or []) >= 2 for p in patches)
    roof_ok = all(len(p.get("roof_by_massing_add") or []) >= 2 for p in patches)
    checks = {
        "charter_doc": (root / K3_CHARTER).is_file(),
        "manifest_on_disk": manifest is not None,
        "three_grammars": len(patches) == 3,
        "massing_additions": massing_ok,
        "facade_by_massing": facade_ok,
        "roof_by_massing": roof_ok,
        "age_aps_map_four_bands": len(age_map) >= 4,
        "reference_pattern_exists": (
            root / "assets/configs/buildings/grammars/industrial_warehouse_v1.ron"
        ).is_file(),
    }
    green = all(checks.values())
    return {
        "gate": K3_GATE,
        "program": "PLAN-BUILDING-QUALITY-v1",
        "charter_doc": K3_CHARTER,
        "manifest_rel": K3_MANIFEST,
        "grammar_patches": [p.get("grammar_id") for p in patches],
        "checks": checks,
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "handoff": {"coder": "merge grammar_patches into RON + BQ-H1 FacadeRule evaluator"},
    }


def _write_witness(body: dict[str, Any], rel: str, *, profile: str, source: str) -> dict[str, Any]:
    root = repo_root()
    body["_agent_meta"] = {
        "schema": f"{profile.lower()}_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": profile,
        "source_system": source,
        "relative_path": rel,
        "ritual": f"BLANG:WIT-HON→Q✓ {body.get('gate')}" if body.get("green") else "BLANG:WIT-HON FAIL",
        "agent": "designer-mcp",
    }
    out = root / rel
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = rel
    return body


def refresh_bq_k1_kitfill_witness(*, repo: Path | None = None) -> dict[str, Any]:
    body = run_bq_k1_kitfill_audit(repo=repo)
    return _write_witness(body, K1_WITNESS, profile="BQ_K1_KITFILL", source="dmcp_bq_k_lane")


def refresh_bq_k2_coverage_witness(*, repo: Path | None = None) -> dict[str, Any]:
    return write_bq_k2_witness(repo=repo)


def refresh_bq_k3_grammar_witness(*, repo: Path | None = None) -> dict[str, Any]:
    body = run_bq_k3_grammar_audit(repo=repo)
    return _write_witness(body, K3_WITNESS, profile="BQ_K3_GRAMMAR", source="dmcp_bq_k_lane")


def refresh_bq_k_lane_witness(*, repo: Path | None = None) -> dict[str, Any]:
    k1 = run_bq_k1_kitfill_audit(repo=repo)
    k2 = audit_bq_k2_coverage(repo=repo)
    k3 = run_bq_k3_grammar_audit(repo=repo)
    green = k1["green"] and k2["green"] and k3["green"]
    body: dict[str, Any] = {
        "gate": "BQ-K-LANE",
        "program": "PLAN-BUILDING-QUALITY-v1",
        "gates": {
            K1_GATE: {"green": k1["green"], "witness": K1_WITNESS},
            K2_GATE: {"green": k2["green"], "witness": K2_WITNESS},
            K3_GATE: {"green": k3["green"], "witness": K3_WITNESS},
        },
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "unblocks": ["coder_mcp kit bake", "APSR-Q2 kit panel", "BQ-H1 facade merge"],
    }
    refresh_bq_k1_kitfill_witness(repo=repo)
    refresh_bq_k2_coverage_witness(repo=repo)
    refresh_bq_k3_grammar_witness(repo=repo)
    return _write_witness(body, K_LANE_WITNESS, profile="BQ_K_LANE", source="dmcp_bq_k_lane")
