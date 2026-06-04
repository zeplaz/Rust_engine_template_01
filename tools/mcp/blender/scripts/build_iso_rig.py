"""
Build utils/Tile_iso_rig_v1.blend — camera + lights only (no meshes).

Usage:
  blender --background --python tools/mcp/blender/scripts/build_iso_rig.py -- --repo <repo_root>
  blender --background --python tools/mcp/blender/scripts/build_iso_rig.py -- --procedural-only
"""

from __future__ import annotations

import sys
from pathlib import Path

import bpy

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

from ops.iso_rig import (  # noqa: E402
    DEFAULT_ISO_RIG_REL,
    build_procedural_iso_rig,
    extract_iso_rig_from_legacy,
    save_iso_rig_blend,
)


def _argv_after_double_dash() -> list[str]:
    argv = sys.argv
    if "--" in argv:
        return argv[argv.index("--") + 1 :]
    return []


def _repo_root(args: list[str]) -> Path:
    if "--repo" in args:
        i = args.index("--repo")
        if i + 1 < len(args):
            return Path(args[i + 1]).resolve()
    guess = _SCRIPT_DIR
    for _ in range(12):
        if (guess / "Cargo.toml").is_file():
            return guess
        if guess.parent == guess:
            break
        guess = guess.parent
    return _SCRIPT_DIR.parents[3]


def main() -> None:
    args = _argv_after_double_dash()
    root = _repo_root(args)
    dest = root / DEFAULT_ISO_RIG_REL
    legacy = root / "utils" / "Light_keysshotsetup.blend"
    procedural_only = "--procedural-only" in args

    if procedural_only or not legacy.is_file():
        build_procedural_iso_rig()
    else:
        extract_iso_rig_from_legacy(legacy)

    save_iso_rig_blend(dest)


if __name__ == "__main__":
    main()
