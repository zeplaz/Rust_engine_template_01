"""
Blender headless geometry job runner.

Usage (from repo root):
  blender --background --python tools/mcp/blender/scripts/run_job.py -- --job path/to/job.json
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import bpy

# ops live beside this file
_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

from ops.export_glb import export_glb  # noqa: E402
from ops import module_door, module_prop, module_roof, module_wall, module_window  # noqa: E402

_OPS = {
    "module_wall": module_wall.build,
    "module_roof": module_roof.build,
    "module_door": module_door.build,
    "module_window": module_window.build,
    "module_prop": module_prop.build,
}


def _argv_after_double_dash() -> list[str]:
    argv = sys.argv
    if "--" in argv:
        return argv[argv.index("--") + 1 :]
    return []


def _parse_job_path(args: list[str]) -> Path:
    if "--job" not in args:
        raise SystemExit("Usage: blender --background --python run_job.py -- --job <job.json>")
    i = args.index("--job")
    if i + 1 >= len(args):
        raise SystemExit("Missing path after --job")
    return Path(args[i + 1]).resolve()


def _reset_scene() -> None:
    bpy.ops.wm.read_factory_settings(use_empty=True)


def main() -> None:
    job_path = _parse_job_path(_argv_after_double_dash())
    job = json.loads(job_path.read_text(encoding="utf-8"))
    operation = job.get("operation")
    if operation not in _OPS:
        raise SystemExit(f"Unknown operation: {operation!r}; known: {list(_OPS)}")

    params = dict(job.get("params") or {})
    params.setdefault("name", job.get("job_id", operation))

    out = job.get("output") or {}
    glb_rel = out.get("glb")
    if not glb_rel:
        raise SystemExit("job.output.glb required")

    # Resolve output relative to repo if not absolute
    repo_guess = job_path
    for _ in range(8):
        if (repo_guess / "Cargo.toml").is_file():
            break
        if repo_guess.parent == repo_guess:
            break
        repo_guess = repo_guess.parent
    glb_path = Path(glb_rel)
    if not glb_path.is_absolute():
        glb_path = (repo_guess / glb_rel).resolve()
    glb_path.parent.mkdir(parents=True, exist_ok=True)

    _reset_scene()
    _OPS[operation](params)
    material_profile = params.get("material_profile")
    export_glb(str(glb_path), material_profile=material_profile, repo_root=repo_guess)
    print(f"EXPORT_OK {glb_path}")


if __name__ == "__main__":
    main()
