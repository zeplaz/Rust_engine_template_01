"""APS-ARTIST-TOOL-E2E-001 — no-Blender artist path witness (schema checks)."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path
from typing import Any

from .aps_atlas_qc import validate_atlas_folder
from .aps_catalog_preview import render_module_list_thumb
from .aps_mat_auth_ui import count_missing_material_profiles
from .aps_slot_preview import render_material_preview
from .material_profiles import load_material_profile_catalog
from .paths import repo_root

APS_ARTIST_TOOL_E2E_WITNESS = "debug_runs/aps_artist_tool_e2e_live.json"


def check_build_health() -> dict[str, Any]:
    """APS-WITNESS-INTEGRITY-001 — the witness must never be green over a broken tree.

    Gate 1: the Art Pipeline Suite must import (`art_pipeline_suite.app`).
    Gate 2: `pytest -k aps` must collect cleanly (no collection errors).
    If either fails, the E2E witness is forced ``green: false`` with a reason.
    """
    root = repo_root()
    py_root = root / "tools/mcp/python"
    suite_root = root / "tools/mcp"

    # Gate 1 — APS app import smoke (in a subprocess so a broken import cannot
    # poison this already-imported process and to mirror the real launch path).
    import_cmd = [
        sys.executable,
        "-c",
        "import sys; sys.path.insert(0, r'%s'); import art_pipeline_suite.app" % str(suite_root / "python"),
    ]
    # art_pipeline_suite lives under tools/mcp (one dir up from python/), so add it too.
    import_cmd[2] = (
        "import sys; "
        "sys.path.insert(0, r'%s'); "
        "sys.path.insert(0, r'%s'); "
        "import art_pipeline_suite.app" % (str(suite_root), str(py_root))
    )
    imp = subprocess.run(import_cmd, cwd=str(suite_root), capture_output=True, text=True)
    import_ok = imp.returncode == 0
    import_err = "" if import_ok else (imp.stderr or imp.stdout or "import failed").strip().splitlines()[-1:]
    import_err = import_err[0] if isinstance(import_err, list) and import_err else (import_err or "")

    # Gate 2 — pytest -k aps collection smoke (collect-only; fast, no test run).
    collect_cmd = [sys.executable, "-m", "pytest", "tests/", "-k", "aps", "--collect-only", "-q"]
    col = subprocess.run(collect_cmd, cwd=str(py_root), capture_output=True, text=True)
    out = (col.stdout or "") + (col.stderr or "")
    # pytest exits non-zero and prints "error" lines when a module fails to import at collection.
    collect_ok = col.returncode in (0, 5) and " error" not in out.lower() and "errors" not in out.lower()
    collect_summary = (out.strip().splitlines() or [""])[-1]

    ok = import_ok and collect_ok
    reason = ""
    if not ok:
        parts: list[str] = []
        if not import_ok:
            parts.append(f"APS app import failed: {import_err}")
        if not collect_ok:
            parts.append(f"pytest -k aps collection errors: {collect_summary}")
        reason = " ; ".join(parts)
    return {
        "ok": ok,
        "import_ok": import_ok,
        "import_error": import_err if not import_ok else None,
        "collect_ok": collect_ok,
        "collect_summary": collect_summary,
        "reason": reason,
    }
WAREHOUSE_SNAP = (
    "tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json"
)
PILOT_TILE = "assets/staging/tiles/tile_warehouse_industrial_west_pilot_v1"


def run_artist_tool_e2e(*, skip_build_health: bool = False) -> dict[str, Any]:
    root = repo_root()
    steps: list[dict[str, Any]] = []

    # APS-WITNESS-INTEGRITY-001 — build-health gate first. A green witness must
    # never survive a broken APS import or pytest -k aps collection errors.
    health = {"ok": True, "reason": ""} if skip_build_health else check_build_health()

    # 1 Catalog — module thumb
    glbs = sorted((root / "assets/models/modules").rglob("model.glb")) if (root / "assets/models/modules").is_dir() else []
    thumb_ok = False
    sample_mod = ""
    if glbs:
        sample_mod = glbs[0].parent.name
        img = render_module_list_thumb(glbs[0], module_id=sample_mod)
        thumb_ok = img is not None and img.size[0] > 0
    steps.append({"step": "catalog_thumb", "ok": thumb_ok, "module_id": sample_mod or None})

    # 2 Assembly — warehouse snapshot + grammar + materials
    snap_path = root / WAREHOUSE_SNAP
    snap_ok = snap_path.is_file()
    snap: dict[str, Any] = {}
    grammar_ok = False
    mat_ok = False
    if snap_ok:
        snap = json.loads(snap_path.read_text(encoding="utf-8"))
        chain = snap.get("grammar_rule_chain")
        grammar_ok = isinstance(chain, dict) and bool(chain.get("massing"))
        missing, total = count_missing_material_profiles(snap)
        mat_ok = total > 0 and missing == 0
    steps.append(
        {
            "step": "assembly_snapshot",
            "ok": snap_ok and grammar_ok,
            "path": WAREHOUSE_SNAP,
            "grammar_chain": grammar_ok,
            "material_profiles": mat_ok,
        }
    )

    # 3 Materials — catalog browse + preview
    catalog = load_material_profile_catalog()
    mat_preview_ok = False
    try:
        img = render_material_preview("steel_panel_01")
        mat_preview_ok = img.size[0] >= 64
    except Exception:
        pass
    steps.append(
        {
            "step": "materials_studio",
            "ok": len(catalog) >= 10 and mat_preview_ok,
            "catalog_count": len(catalog),
        }
    )

    # 4 Variants — example variant set schema
    var_example = root / "tools/mcp/schemas/examples/variant_set_warehouse_industrial_west_production_v1.json"
    var_ok = var_example.is_file()
    steps.append({"step": "variants_example", "ok": var_ok, "path": str(var_example.relative_to(root)).replace("\\", "/") if var_ok else None})

    # 5 Atlas — pilot folder + UI QC path (v2 validate reported separately)
    pilot = root / PILOT_TILE
    atlas_fixture_ok = pilot.is_dir() and (pilot / "atlas_meta.json").is_file()
    atlas_v2_ok = False
    atlas_status = "skipped"
    plain: list[str] = []
    if pilot.is_dir():
        report, plain = validate_atlas_folder(pilot)
        atlas_status = report.status if report else "missing_meta"
        atlas_v2_ok = report is not None and report.status == "passed"
    steps.append(
        {
            "step": "atlas_pilot_fixture",
            "ok": atlas_fixture_ok,
            "folder": PILOT_TILE,
            "meta_v2_validate": atlas_v2_ok,
            "validation": atlas_status,
            "plain_language": plain[:3],
        }
    )

    steps_ok = all(s.get("ok") for s in steps)
    # Build health is a hard precondition: never green over a broken tree.
    green = bool(health.get("ok")) and steps_ok
    body: dict[str, Any] = {
        "program_id": "APS-ARTIST-TOOL-E2E-001",
        "green": green,
        "honest_gate": "build_health+schema",
        "build_health": health,
        "steps_ok": steps_ok,
        "artist_path": "Catalog → Assembly → Materials → Variants → Atlas (no Blender)",
        "steps": steps,
        "designer_mcp_signoff": "pending",
        "track_b_deferred": "MCP-PILOT-GRAMMAR-001 manual keyframe",
    }
    if not health.get("ok"):
        body["not_green_reason"] = health.get("reason") or "build health gate failed"
    out = root / APS_ARTIST_TOOL_E2E_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body
