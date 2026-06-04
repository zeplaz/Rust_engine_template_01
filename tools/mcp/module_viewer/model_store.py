"""Load module index entries and resolve on-disk artifact paths."""

from __future__ import annotations

import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

from rust_engine_mcp.library import load_index_json, write_module_index
from rust_engine_mcp.paths import blender_exe, repo_root
from rust_engine_mcp.validate_glb import validate_glb


@dataclass
class ModuleRecord:
    module_id: str
    job_id: str
    glb_path: Path
    module_dir: Path
    index_row: dict
    manifest: dict | None
    sidecar_path: Path | None
    sidecar: dict | None


def _modules_root() -> Path:
    return repo_root() / "assets" / "models" / "modules"


def list_modules(
    *,
    batch_id: str | None = None,
    category: str | None = None,
    archetype: str | None = None,
) -> list[ModuleRecord]:
    rows = load_index_json()
    out: list[ModuleRecord] = []
    for row in rows:
        if batch_id and row.get("batch_id") != batch_id:
            continue
        if category and row.get("category") != category:
            continue
        if archetype and row.get("archetype") != archetype:
            continue
        rec = resolve_record(row)
        if rec is not None:
            out.append(rec)
    return out


def resolve_record(row: dict) -> ModuleRecord | None:
    job_id = str(row.get("job_id") or "")
    module_dir = _modules_root() / job_id
    glb_rel = str(row.get("glb") or row.get("glb_path") or "")
    glb_path = repo_root() / glb_rel.replace("/", "\\") if glb_rel else module_dir / "model.glb"
    if not glb_path.is_file():
        glb_path = module_dir / "model.glb"
    if not glb_path.is_file():
        return None

    manifest = None
    manifest_path = module_dir / "manifest.json"
    if manifest_path.is_file():
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

    sidecar_path = None
    sidecar = None
    sidecars = sorted(module_dir.glob("*.module.json"))
    if sidecars:
        sidecar_path = sidecars[0]
        sidecar = json.loads(sidecar_path.read_text(encoding="utf-8"))

    return ModuleRecord(
        module_id=str(row.get("module_id") or ""),
        job_id=job_id,
        glb_path=glb_path.resolve(),
        module_dir=module_dir.resolve(),
        index_row=row,
        manifest=manifest,
        sidecar_path=sidecar_path,
        sidecar=sidecar,
    )


def validate_record(rec: ModuleRecord) -> dict:
    report = validate_glb(rec.glb_path)
    return report.to_dict()


def save_sidecar(rec: ModuleRecord, data: dict) -> Path:
    if rec.sidecar_path is None:
        path = rec.module_dir / f"{rec.job_id}.module.json"
    else:
        path = rec.sidecar_path
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    rec.sidecar_path = path
    rec.sidecar = data
    return path


def reindex_library() -> dict:
    return write_module_index()


def open_path(path: Path) -> None:
    if sys.platform == "win32":
        import os

        os.startfile(str(path))  # noqa: S606
    elif sys.platform == "darwin":
        subprocess.run(["open", str(path)], check=False)
    else:
        subprocess.run(["xdg-open", str(path)], check=False)


def open_in_blender(glb_path: Path) -> None:
    """Import GLB in Blender GUI — passing .glb as argv opens as unsupported .blend."""
    script = repo_root() / "tools" / "mcp" / "blender" / "scripts" / "view_glb.py"
    subprocess.Popen(  # noqa: S603
        [
            str(blender_exe()),
            "--python",
            str(script),
            "--",
            "--glb",
            str(glb_path.resolve()),
        ],
        cwd=str(repo_root()),
    )


def preview_trimesh(glb_path: Path) -> str | None:
    try:
        import trimesh  # noqa: F401
    except ImportError:
        return (
            "trimesh not installed for THIS Python.\n\n"
            f'  "{sys.executable}" -m pip install -r tools/mcp/module_viewer/requirements.txt\n\n'
            "Or use Preview in browser (no extra deps)."
        )
    import trimesh

    try:
        loaded = trimesh.load(str(glb_path), force="mesh")
    except Exception as exc:  # noqa: BLE001
        return f"trimesh failed to load GLB:\n{exc}"
    if isinstance(loaded, trimesh.Scene):
        loaded = loaded.dump(concatenate=True)
    try:
        loaded.show(caption=glb_path.name)
    except ImportError as exc:
        msg = str(exc)
        if "pyglet<2" in msg or "pyglet" in msg.lower():
            return (
                "trimesh 3D preview needs pyglet 1.x (not 2.x).\n\n"
                f'  "{sys.executable}" -m pip install "pyglet>=1.5.27,<2" trimesh\n\n'
                "Or use Preview in browser (recommended on Windows)."
            )
        return f"trimesh viewer import failed:\n{exc}"
    except Exception as exc:  # noqa: BLE001
        return f"trimesh viewer failed (try Preview in browser):\n{exc}"
    return None
