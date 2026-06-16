#!/usr/bin/env python3
"""MCP-PILOT-GRAMMAR-001 Track B — prep + gates + blocked witness (no headless ship).

Ship requires real utils/keyframe_render.py in Blender UI (operator Phase 4–6).
See: docs/archive/2026-06-src-dev/plans/pilot_grammar_agent_orders_v1.md
"""

from __future__ import annotations

import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

_REPO = Path(__file__).resolve().parents[3]
if str(_REPO / "tools" / "mcp" / "python") not in sys.path:
    sys.path.insert(0, str(_REPO / "tools" / "mcp" / "python"))

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators import run_validator

PILOT_ASSEMBLY = "assets/staging/assemblies/industrial_west_7x5_s39_9fa1.json"
PILOT_BLEND = "assets/staging/assemblies/industrial_west_7x5_s39_9fa1.blend"
REJECTED_STAGING = "assets/staging/tiles/tile_warehouse_industrial_v2_minimum_g4"
MANUAL_STILLS = "assets/staging/tiles/keyframe_stills/warehouse_industrial_manual_v1"
WITNESS = "debug_runs/art_pipeline/mcp_pilot_grammar_001_live.json"
REJECTION = "debug_runs/art_pipeline/mcp_pilot_grammar_001_rejected_live.json"


def _brief_json(path: str) -> dict:
    p = repo_root() / path
    if not p.is_file():
        return {"path": path, "present": False}
    try:
        return {"path": path, "present": True, **json.loads(p.read_text(encoding="utf-8"))}
    except json.JSONDecodeError:
        return {"path": path, "present": True, "parse_error": True}


def main() -> int:
    root = repo_root()
    snap = root / PILOT_ASSEMBLY
    staging = root / REJECTED_STAGING
    marker = staging / "keyframe_manual.export"
    if marker.is_file():
        marker.unlink()

    gates: dict = {}
    p0 = run_validator("assembly_p0", PILOT_ASSEMBLY, compression_level=3)
    gates["assembly_p0"] = {"status": p0.status, "summary": p0.summary}
    grammar = run_validator("assembly_grammar", PILOT_ASSEMBLY, compression_level=3)
    gates["assembly_grammar"] = {"status": grammar.status, "summary": grammar.summary}

    cleanup = subprocess.run(
        [sys.executable, str(root / "tools/mcp/scripts/cleanup_assembly_blends.py")],
        cwd=root,
        capture_output=True,
        text=True,
    )
    gates["cleanup_assembly_blends"] = {"exit_code": cleanup.returncode}

    build = subprocess.run(
        [
            sys.executable,
            "-m",
            "rust_engine_mcp.cli",
            "assembly-build-run",
            str(snap),
        ],
        cwd=root / "tools/mcp/python",
        capture_output=True,
        text=True,
    )
    build_ok = build.returncode == 0
    try:
        build_body = json.loads(build.stdout.strip() or "{}")
    except json.JSONDecodeError:
        build_body = {"stdout": build.stdout[-500:], "stderr": build.stderr[-500:]}
    gates["assembly_build_run"] = {"ok": build_ok, "result": build_body}

    preview = _brief_json("debug_runs/aps_preview_002_live.json")
    build_worker = _brief_json("debug_runs/build_worker_001_live.json")
    grammar_e2e = _brief_json("debug_runs/pilot_grammar_001_grammar_e2e_live.json")
    rejected = _brief_json(REJECTION)

    pre_ok = (
        p0.status == "passed"
        and grammar.status == "passed"
        and build_ok
        and grammar_e2e.get("green") is True
    )

    body = {
        "slice_id": "MCP-PILOT-GRAMMAR-001",
        "track": "B",
        "green": False,
        "proceed_ship": False,
        "art_quality": "rejected_headless_procedural",
        "updated": datetime.now(timezone.utc).isoformat(),
        "pilot_assembly": PILOT_ASSEMBLY,
        "pilot_blend": PILOT_BLEND,
        "manual_stills_folder": MANUAL_STILLS,
        "rejection_witness": REJECTION,
        "gates": gates,
        "prerequisites": {
            "track_a_grammar_e2e": grammar_e2e.get("green"),
            "aps_preview_002": preview.get("green"),
            "build_worker_001": build_worker.get("status") == "done",
            "assembly_p0": p0.status == "passed",
            "prep_ready": pre_ok,
        },
        "blocked_by": [
            "real_keyframe_render_required",
            "operator_g4_eyeball",
        ],
        "operator_phase_4_6": [
            "READ docs/archive/2026-06-src-dev/plans/pilot_grammar_operator_runbook_v1.md (full steps)",
            f"Prep: powershell -File tools/mcp/scripts/designer_mcp_pilot_grammar_prep.ps1",
            f"Blender: open {PILOT_BLEND}, append Tile_iso_rig_v1 → TILE_ISO_RIG, keyframe_render addon → 24 PNGs",
            "PNG folder: assets/staging/tiles/tile_warehouse_industrial_v2_minimum_g4/ (clean_day_f0..f7, etc.)",
            "Finish: powershell -File tools/mcp/scripts/operator_warehouse_keyframe_finish.ps1",
            "NOT: tile_compile_minimum_bake / designer_mcp_pilot_grammar_keyframe.py for ship",
        ],
        "forbidden": [
            "tile_compile_minimum_bake.py",
            "designer_mcp_pilot_grammar_keyframe.py --force-bake for ship",
            "fake keyframe_manual.export on headless PNGs",
        ],
        "findings_from_rejection": rejected.get("findings"),
        "_agent_meta": {
            "agent": "designer-mcp",
            "lane": "MCP-PILOT-GRAMMAR-001",
            "policy": "docs/archive/2026-06-src-dev/plans/mcp_orchestrator_tile_fix_warehouse_slice_v2.md",
            "orders": "docs/archive/2026-06-src-dev/plans/pilot_grammar_agent_orders_v1.md",
        },
    }

    out = root / WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(body, indent=2))
    return 1  # blocked until operator Phase 4–6


if __name__ == "__main__":
    raise SystemExit(main())
