"""Automation wrappers — same logic as rust_engine_mcp CLI/MCP tools."""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

from rust_engine_mcp.paths import blender_exe, repo_root
from rust_engine_mcp.tile_pipeline import light_setup_blend_path, lod0_batch_run, tile_atlas_pack, tile_batch_run


def light_setup_blend() -> Path:
    return light_setup_blend_path()


def pack_tile_folder(folder: Path, *, keyframe_rename: bool = False) -> tuple[int, str]:
    result = tile_atlas_pack(folder, keyframe_rename=keyframe_rename)
    log = result.get("log") or result.get("error") or ""
    if result.get("atlas_path"):
        log += f"\nAtlas: {result['atlas_path']}"
    code = 0 if result.get("ok") else int(result.get("exit_code") or 1)
    return code, log.strip() or f"exit {code}"


def run_lod0_batch(batch_id: str, *, step: str = "g0g1") -> tuple[int, str]:
    result = lod0_batch_run(batch_id, phase=step)
    log = result.get("log") or result.get("error") or ""
    code = 0 if result.get("ok") else int(result.get("exit_code") or 1)
    return code, log.strip() or f"exit {code}"


def run_tile_batch(tile_batch_json: Path) -> tuple[int, str]:
    result = tile_batch_run(tile_batch_json)
    log = json_dumps(result)
    code = 0 if result.get("ok") else 1
    return code, log


def json_dumps(obj: object) -> str:
    import json

    return json.dumps(obj, indent=2)


def art_debug_gui_enabled() -> bool:
    return os.environ.get("RUST_ENGINE_ART_DEBUG_GUI", "").strip() in ("1", "true", "TRUE")


def open_light_blend() -> tuple[int, str]:
    blend = light_setup_blend_path()
    if not blend.is_file():
        return 1, (
            f"Light setup blend not found: {blend}\n"
            "Expected utils/Light_keysshotsetup.blend (legacy Keyshot-style rig)."
        )
    exe = blender_exe()
    subprocess.Popen([exe, str(blend)], cwd=str(blend.parent))
    return 0, f"Launched Blender: {blend}"


def open_keyframe_render_addon() -> tuple[int, str]:
    script = repo_root() / "utils" / "keyframe_render.py"
    if not script.is_file():
        return 1, f"Missing: {script}"
    exe = blender_exe()
    subprocess.Popen([exe, "--python", str(script)], cwd=str(repo_root()))
    return 0, "Launched Blender + keyframe_render.py — use Output → Keyframes panel."


def find_latest_atlas_in(folder: Path) -> Path | None:
    matches = sorted(folder.glob("tile_map_*.png"), key=lambda p: p.stat().st_mtime)
    return matches[-1] if matches else None
