"""MCP-MAT-BRIEF-001 — compressed material profile briefs for MCP agents."""

from __future__ import annotations

import json
from typing import Any

from .material_profiles import infer_category, load_material_profile_catalog
from .paths import repo_root

MCP_MAT_BRIEF_WITNESS = "debug_runs/mcp_mat_brief_001_live.json"


def material_profile_brief(profile_id: str) -> dict[str, Any]:
    catalog = {e.profile_id: e for e in load_material_profile_catalog()}
    entry = catalog.get(profile_id)
    known = entry is not None
    category = entry.category if entry else infer_category(profile_id)
    status = entry.texture_status() if entry else "missing"
    hint = (
        "Textures ready — assign on Assembly snapshot placements."
        if status == "ready"
        else "Generate textures on Materials tab before ship."
    )
    return {
        "ok": True,
        "schema": "material_profile_brief_v1",
        "profile_id": profile_id,
        "known": known,
        "category_path": category,
        "texture_status": status,
        "hint": hint,
    }


def material_catalog_brief(*, max_rows: int = 50) -> dict[str, Any]:
    catalog = load_material_profile_catalog()
    counts = {"ready": 0, "partial": 0, "missing": 0}
    rows: list[dict[str, Any]] = []
    for entry in catalog[:max_rows]:
        st = entry.texture_status()
        counts[st] = counts.get(st, 0) + 1
        rows.append(
            {
                "profile_id": entry.profile_id,
                "category": entry.category,
                "texture_status": st,
            }
        )
    return {
        "ok": True,
        "schema": "material_catalog_brief_v1",
        "total": len(catalog),
        "counts": counts,
        "rows": rows,
    }


def refresh_mcp_mat_brief_witness() -> bool:
    sample = material_profile_brief("steel_panel_01")
    catalog = material_catalog_brief(max_rows=5)
    body = {
        "gate_id": "MCP-MAT-BRIEF-001",
        "ok": sample.get("ok") and catalog.get("ok"),
        "green": sample.get("ok") and catalog.get("ok"),
        "sample_profile": sample.get("profile_id"),
        "sample_category_path": sample.get("category_path"),
        "catalog_total": catalog.get("total"),
        "catalog_counts": catalog.get("counts"),
    }
    out = repo_root() / MCP_MAT_BRIEF_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return bool(body["green"])
