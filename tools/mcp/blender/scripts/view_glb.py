"""Open a GLB in Blender GUI via glTF import (do not pass .glb as a .blend path).

Usage:
  blender --python tools/mcp/blender/scripts/view_glb.py -- --glb path/to/model.glb
"""

from __future__ import annotations

import sys
from pathlib import Path

import bpy


def _args_after_dash() -> list[str]:
    argv = sys.argv
    if "--" not in argv:
        return []
    return argv[argv.index("--") + 1 :]


def _import_gltf(filepath: str) -> None:
    try:
        bpy.ops.wm.gltf_import(filepath=filepath)
        return
    except Exception:
        pass
    bpy.ops.import_scene.gltf(filepath=filepath)


def main() -> None:
    args = _args_after_dash()
    if "--glb" not in args:
        raise SystemExit("Usage: blender --python view_glb.py -- --glb <model.glb>")
    i = args.index("--glb")
    if i + 1 >= len(args):
        raise SystemExit("Missing path after --glb")
    glb = Path(args[i + 1]).resolve()
    if not glb.is_file():
        raise SystemExit(f"GLB not found: {glb}")

    bpy.ops.wm.read_factory_settings(use_empty=True)
    _import_gltf(str(glb))

    for area in bpy.context.screen.areas:
        if area.type == "VIEW_3D":
            for space in area.spaces:
                if space.type == "VIEW_3D":
                    space.shading.type = "MATERIAL"
            break

    print(f"VIEW_OK {glb}")


if __name__ == "__main__":
    main()
