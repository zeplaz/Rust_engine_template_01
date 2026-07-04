"""One-shot generator for BQ-K1 kit fill AssetSpec + geometry job examples."""

from __future__ import annotations

import json
from pathlib import Path

from rust_engine_mcp.bq_k1_kitfill_catalog import BATCH_REL, CHARTER_REL, K1_JOBS
from rust_engine_mcp.paths import repo_root


def main() -> None:
    root = repo_root()
    spec_dir = root / "assets/staging/specs/kit_fill"
    spec_dir.mkdir(parents=True, exist_ok=True)
    jobs_out: list[dict] = []
    for i, job in enumerate(K1_JOBS, start=1):
        mid = job["module_id"]
        job_id = f"{mid}_production_run001"
        spec_rel = f"assets/staging/specs/kit_fill/{mid}_production.json"
        geom_rel = f"tools/mcp/schemas/examples/geometry_job_{mid}_production_run001.json"
        spec = {
            "schema_version": 1,
            "asset_id": mid,
            "archetype": f"module_{job['category']}"
            if job["category"] != "window"
            else "module_window",
            "batch_id": "kit_fill_bq_k1_001",
            "style_pack": job["style_packs"][0],
            "style_packs": job["style_packs"],
            "development_tier": "production",
            "pbr_status": "planned",
            "material_profile": job["material_profile"],
            "material_family": job["material_family"],
            "module": {
                "grid_units": job["grid_units"],
                "snap": (
                    "roof_ridge"
                    if job["category"] == "roof"
                    else ("floor_edge" if job["category"] == "door" else "wall_center")
                ),
                "pivot": "bottom_center",
            },
            "dimensions_m": job["dimensions_m"],
            "references": [
                "ref:charter:BQ-K1-KITFILL-001",
                f"ref:contract:{CHARTER_REL}",
                "ref:contract:tools/mcp/schemas/module_contract_v1.json",
            ],
            "_meta": {
                "teaches": [job["material_family"], job["category"], "kit_fill_bq_k1"]
            },
        }
        geom = {
            "schema_version": 1,
            "job_id": job_id,
            "batch_id": "kit_fill_bq_k1_001",
            "development_tier": "production",
            "spec_ref": spec_rel,
            "operation": job["operation"],
            "params": {
                **job["dimensions_m"],
                "profile": job["material_family"],
                "seed": 550_100 + i,
            },
            "output": {
                "glb": f"assets/staging/{mid}_production_run001/model.glb",
                "thumbnail": f"assets/staging/{mid}_production_run001/preview.png",
            },
        }
        (root / spec_rel).write_text(json.dumps(spec, indent=2) + "\n", encoding="utf-8")
        (root / geom_rel).write_text(json.dumps(geom, indent=2) + "\n", encoding="utf-8")
        jobs_out.append(
            {
                "module_id": mid,
                "material_family": job["material_family"],
                "category": job["category"],
                "style_packs": job["style_packs"],
                "spec_rel": spec_rel,
                "geometry_job_rel": geom_rel,
                "job_id": job_id,
                "replaces_slots": job["replaces_slots"],
            }
        )
    batch = {
        "schema": "bq_k1_kitfill_batch_v1",
        "gate": "BQ-K1-KITFILL-001",
        "charter": CHARTER_REL,
        "job_count": len(jobs_out),
        "jobs": jobs_out,
    }
    (root / BATCH_REL).write_text(json.dumps(batch, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {len(jobs_out)} kit fill specs")


if __name__ == "__main__":
    main()
