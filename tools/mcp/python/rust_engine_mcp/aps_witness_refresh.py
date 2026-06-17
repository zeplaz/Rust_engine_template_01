"""APS-WITNESS-REFRESH-001 — refresh APS artist-tool witnesses + module manifest."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path
from typing import Any

from .aps_atlas_qc import write_aps_atlas_preview_witness
from .aps_catalog_preview import render_module_list_thumb, write_aps_preview_catalog_witness
from .aps_slot_preview import write_aps_preview_001_witness
from .material_studio_preview import write_material_studio_witness
from .paths import repo_root
from .aps_witness_honesty import write_aps_live_witness

APS_ARTIST_TOOL_MODULES_WITNESS = "debug_runs/aps_artist_tool_modules_live.json"


def _suite_modules() -> list[str]:
    suite = repo_root() / "tools/mcp/art_pipeline_suite"
    return sorted(p.name for p in suite.glob("*.py") if p.name != "__init__.py")


def _pytest_aps_smoke() -> dict[str, Any]:
    tests = [
        "tests/test_aps_preview_001.py",
        "tests/test_aps_atlas_preview.py",
        "tests/test_aps_grammar_labels.py",
    ]
    cmd = [sys.executable, "-m", "pytest", *tests, "-q", "--tb=no"]
    proc = subprocess.run(cmd, cwd=repo_root() / "tools/mcp/python", capture_output=True, text=True)
    tail = (proc.stdout or proc.stderr or "").strip().splitlines()
    summary = tail[-1] if tail else ""
    return {"ok": proc.returncode == 0, "summary": summary, "tests": tests}


def _pytest_aps_imports() -> dict[str, Any]:
    """MCP-WIT-042 — APS witness refresh refuses green when imports fail."""
    cmd = [sys.executable, "-m", "pytest", "tests/test_aps_imports.py", "-q", "--tb=no"]
    proc = subprocess.run(cmd, cwd=repo_root() / "tools/mcp/python", capture_output=True, text=True)
    tail = (proc.stdout or proc.stderr or "").strip().splitlines()
    summary = tail[-1] if tail else ""
    return {"ok": proc.returncode == 0, "summary": summary, "tests": ["tests/test_aps_imports.py"]}


def _sample_catalog_thumb() -> tuple[str, bool]:
    modules_dir = repo_root() / "assets/models/modules"
    if not modules_dir.is_dir():
        return "", False
    glbs = sorted(modules_dir.rglob("model.glb"))
    if not glbs:
        return "", False
    glb = glbs[0]
    mid = glb.parent.name
    img = render_module_list_thumb(glb, module_id=mid)
    return mid, img is not None


def refresh_aps_witnesses() -> dict[str, Any]:
    """Phase 1 bundle: preview witnesses + module manifest + pytest smoke."""
    mod_ok = True
    mat_ok = True
    comb_ok = True
    try:
        from .aps_slot_preview import render_combined_preview, render_material_preview, render_module_isolated

        render_material_preview("steel_panel_01")
        mod = render_module_isolated("assets/models/modules/nonexistent.glb")
        comb = render_combined_preview(mod, render_material_preview("steel_panel_01"))
        comb_ok = comb.size[0] > 0
    except Exception:
        mod_ok = mat_ok = comb_ok = False

    preview_001 = write_aps_preview_001_witness(
        module_ok=mod_ok, material_ok=mat_ok, combined_ok=comb_ok
    )
    sample_id, thumb_ok = _sample_catalog_thumb()
    preview_catalog = write_aps_preview_catalog_witness(sample_module_id=sample_id, thumb_ok=thumb_ok)
    atlas_witness = write_aps_atlas_preview_witness()
    material_studio = write_material_studio_witness()
    from rust_engine_mcp.aps_artist_tool_e2e import run_artist_tool_e2e

    e2e = run_artist_tool_e2e()
    from rust_engine_mcp.aps_mat_002 import write_aps_mat_002_witness
    from rust_engine_mcp.aps_bevy_preview_002 import run_aps_bevy_preview_002_smoke

    mat002 = write_aps_mat_002_witness()
    bevy002 = run_aps_bevy_preview_002_smoke(open_browser=False)

    modules = _suite_modules()
    pytest_result = _pytest_aps_smoke()
    imports_result = _pytest_aps_imports()
    gate_ok = pytest_result.get("ok", False) and imports_result.get("ok", False)
    modules_body = {
        "program_id": "APS-WITNESS-REFRESH-001",
        "green": gate_ok,
        "suite_modules": modules,
        "module_count": len(modules),
        "pytest": pytest_result,
        "aps_imports": imports_result,
        "witnesses": {
            "aps_preview_001": preview_001.relative_to(repo_root()).as_posix(),
            "aps_preview_catalog": preview_catalog.relative_to(repo_root()).as_posix(),
            "aps_atlas_preview_002": atlas_witness.relative_to(repo_root()).as_posix(),
            "aps_material_studio": "debug_runs/aps_material_studio_live.json",
            "aps_artist_tool_e2e": "debug_runs/aps_artist_tool_e2e_live.json",
            "aps_mat_002": "debug_runs/aps_mat_002_live.json",
            "aps_bevy_preview_002": "debug_runs/aps_bevy_preview_002_live.json",
        },
        "e2e_green": e2e.get("green"),
        "mat_002_ok": mat002.get("ok"),
        "bevy_preview_002_green": bevy002.get("green"),
    }
    out = repo_root() / APS_ARTIST_TOOL_MODULES_WITNESS
    modules_body = write_aps_live_witness(
        modules_body,
        APS_ARTIST_TOOL_MODULES_WITNESS,
        schema="aps_artist_tool_modules_live_v1",
        profile="APS_WITNESS_REFRESH",
        source_system="aps_witness_refresh",
        ritual="BLANG:WIT-HON APS-WITNESS-REFRESH-001" if gate_ok else None,
        exit_predicate_must=[
            {"path": "green", "eq": True},
        ],
    )
    return modules_body
