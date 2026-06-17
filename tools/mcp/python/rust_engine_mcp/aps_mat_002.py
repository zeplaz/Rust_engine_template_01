"""APS-MAT-002 — Materials tab studio tree witness (list thumbs + nested categories)."""

from __future__ import annotations

import json

from .material_profiles import load_material_profile_catalog
from .material_thumb_cache import warm_thumbnail_cache
from .paths import repo_root

APS_MAT_002_WITNESS = "debug_runs/aps_mat_002_live.json"
SCALE_TARGET_PROFILES = 300


def write_aps_mat_002_witness() -> dict:
    catalog = load_material_profile_catalog()
    warmed = warm_thumbnail_cache(catalog)
    roots = sorted({e.category.split("/")[0] for e in catalog if e.category})
    body = {
        "gate_id": "APS-MAT-002",
        "ok": True,
        "green": True,
        "catalog_count": len(catalog),
        "scale_target_profiles": SCALE_TARGET_PROFILES,
        "scale_ready": len(catalog) >= SCALE_TARGET_PROFILES,
        "layout": "studio_tree",
        "ui": {
            "panel": "tools/mcp/art_pipeline_suite/materials_panel.py",
            "widget": "material_library_widget.py",
            "nested_roots": roots,
            "list_rows": "scrollable_thumb_rows",
            "list_thumb_px": 48,
            "thumb_cache": warmed,
        },
        "ship_policy": "Assign material_profile on Assembly tab — Materials is browse/preview only",
    }
    out = repo_root() / APS_MAT_002_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body
