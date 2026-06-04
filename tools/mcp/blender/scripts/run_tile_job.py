"""
Blender headless tile job runner.

Usage:
  blender --background --python tools/mcp/blender/scripts/run_tile_job.py -- --job path/to/job.json
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import bpy

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

from ops import assembly_import, tile_keyframe_bake, tile_ortho_bake  # noqa: E402


def _tile_variant_bake(job: dict, *, repo_root: Path) -> Path:
    render = dict(job.get("render") or {})
    if render.get("method") == "blender_keyframe_light_rig":
        return tile_keyframe_bake.bake(job, repo_root=repo_root)
    return tile_ortho_bake.bake(job, repo_root=repo_root)


_OPS = {
    "assembly_build": assembly_import.build,
    "tile_variant_bake": _tile_variant_bake,
}


def _argv_after_double_dash() -> list[str]:
    argv = sys.argv
    if "--" in argv:
        return argv[argv.index("--") + 1 :]
    return []


def _parse_job_path(args: list[str]) -> Path:
    if "--job" not in args:
        raise SystemExit("Usage: blender --background --python run_tile_job.py -- --job <job.json>")
    i = args.index("--job")
    if i + 1 >= len(args):
        raise SystemExit("Missing path after --job")
    return Path(args[i + 1]).resolve()


def _repo_from_job(job_path: Path) -> Path:
    repo_guess = job_path
    for _ in range(10):
        if (repo_guess / "Cargo.toml").is_file():
            return repo_guess
        if repo_guess.parent == repo_guess:
            break
        repo_guess = repo_guess.parent
    return job_path.parent


def main() -> None:
    job_path = _parse_job_path(_argv_after_double_dash())
    job = json.loads(job_path.read_text(encoding="utf-8"))
    operation = job.get("operation")
    if operation not in _OPS:
        raise SystemExit(f"Unknown operation: {operation!r}; known: {list(_OPS)}")

    repo = _repo_from_job(job_path)
    _OPS[operation](job, repo_root=repo)


if __name__ == "__main__":
    main()
