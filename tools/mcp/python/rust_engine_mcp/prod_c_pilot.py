"""MCP-PROD-C-PILOT — rowhouse production bpy profile witness."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

MCP_PROD_C_PILOT_WITNESS = "debug_runs/mcp_prod_c_pilot_live.json"
MANIFEST = repo_root() / "tools/mcp/schemas/examples/batch_kit_production_001.manifest.json"

ROWHOUSE_PROFILE_CASES: list[tuple[str, str, str, str]] = [
    ("wall_brick_1u", "wall_brick_1u_production_run001", "module_wall", "brick"),
    ("door_residential", "door_residential_production_run001", "module_door", "residential"),
    ("roof_pitched_gable", "roof_pitched_gable_production_run001", "module_roof", "pitched"),
]

WINDOW_REFERENCE = (
    "win_industrial_3u",
    "win_industrial_3u_production_run001",
    "module_window",
    "strip",
)


def _load_job(stem: str) -> dict[str, Any]:
    path = repo_root() / "tools/mcp/schemas/examples" / f"{stem}.json"
    if not path.is_file():
        raise FileNotFoundError(path)
    return json.loads(path.read_text(encoding="utf-8"))


def _profile_ok(job: dict[str, Any], operation: str, expected_profile: str) -> bool:
    if str(job.get("operation") or "") != operation:
        return False
    params = job.get("params") or {}
    profile = str(params.get("profile") or params.get("panel") or "").lower()
    return profile == expected_profile.lower()


def refresh_mcp_prod_c_pilot_witness() -> bool:
    """Validate rowhouse production jobs wire non-flat bpy profiles (Phase C pilot)."""
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    cases: dict[str, bool] = {}
    for module_id, job_stem, operation, profile in ROWHOUSE_PROFILE_CASES + [WINDOW_REFERENCE]:
        job = _load_job(job_stem)
        ok = _profile_ok(job, operation, profile)
        cases[f"{module_id}:{profile}"] = ok

    glb_checks: dict[str, bool] = {}
    for module_id, job_stem, _, _ in ROWHOUSE_PROFILE_CASES:
        glb = repo_root() / "assets/models/modules" / f"{job_stem}" / "model.glb"
        glb_checks[module_id] = glb.is_file()

    green = all(cases.values()) and all(glb_checks.values())
    payload = {
        "gate_id": "MCP-PROD-C-PILOT",
        "ok": green,
        "green": green,
        "batch_id": manifest.get("batch_id"),
        "profile_cases": cases,
        "promoted_glbs": glb_checks,
        "bpy_ops": ["module_wall", "module_door", "module_roof", "module_window"],
    }
    out: Path = repo_root() / MCP_PROD_C_PILOT_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return green
