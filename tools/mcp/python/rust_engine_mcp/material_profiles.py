"""Material profile catalog for APS material browser (ARCH-MATERIAL-AUTHORITY-001)."""

from __future__ import annotations

import json
import os
import platform
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .material_textures import PILOT_PROFILES, ProfileDef, generate_profile, texture_dir, write_registry
from .paths import repo_root

CATEGORY_ORDER = (
    "all",
    "industrial/steel",
    "industrial/concrete",
    "residential/brick",
    "residential/wood",
    "roof",
    "glass",
    "other",
)


@dataclass(frozen=True)
class MaterialProfileEntry:
    profile_id: str
    label: str
    generator: str
    category: str
    albedo_path: Path | None
    normal_path: Path | None
    roughness_path: Path | None
    metallic: float
    roughness_base: float
    in_registry: bool = False

    def display_label(self) -> str:
        return self.label or self.profile_id.replace("_", " ").title()

    def texture_status(self) -> str:
        maps = [self.albedo_path, self.normal_path, self.roughness_path]
        present = sum(1 for p in maps if p is not None and p.is_file())
        if present >= 3:
            return "ready"
        if present > 0:
            return "partial"
        return "missing"


def registry_path() -> Path:
    return repo_root() / "assets" / "materials" / "profiles" / "material_profiles_v1.json"


def _profile_label(profile_id: str) -> str:
    return profile_id.replace("_", " ").replace("-", " ").title()


def infer_category(profile_id: str) -> str:
    pid = profile_id.lower()
    if "steel" in pid or "metal" in pid:
        return "industrial/steel"
    if "brick" in pid:
        return "residential/brick"
    if "wood" in pid or "plank" in pid:
        return "residential/wood"
    if "glass" in pid:
        return "glass"
    if "roof" in pid:
        return "roof"
    if "concrete" in pid:
        return "industrial/concrete"
    return "other"


def infer_generator(profile_id: str) -> str:
    pid = profile_id.lower()
    if "steel" in pid or "metal" in pid:
        return "steel"
    if "brick" in pid:
        return "brick"
    if "wood" in pid or "plank" in pid:
        return "wood"
    if "glass" in pid:
        return "steel"
    if "roof" in pid and "tile" in pid:
        return "concrete"
    return "concrete"


def _seed_for_profile(profile_id: str, explicit: int | None = None) -> int:
    if explicit is not None:
        return int(explicit)
    return sum(ord(c) for c in profile_id) % 999_983


def _paths_from_registry_row(profile_id: str, row: dict[str, Any]) -> tuple[Path | None, Path | None, Path | None]:
    tex = row.get("textures") or {}
    root = repo_root()

    def _p(key: str) -> Path | None:
        rel = tex.get(key)
        if not rel:
            return None
        path = root / str(rel).replace("\\", "/")
        return path if path.is_file() else None

    albedo = _p("albedo")
    if albedo is None:
        cand = texture_dir(profile_id) / "albedo.png"
        albedo = cand if cand.is_file() else None
    normal = texture_dir(profile_id) / "normal.png"
    rough = texture_dir(profile_id) / "roughness.png"
    return albedo, normal if normal.is_file() else None, rough if rough.is_file() else None


def _row_from_registry(profile_id: str) -> dict[str, Any] | None:
    path = registry_path()
    if not path.is_file():
        return None
    data = json.loads(path.read_text(encoding="utf-8"))
    row = (data.get("profiles") or {}).get(profile_id)
    return row if isinstance(row, dict) else None


def _entry_from_row(
    profile_id: str,
    row: dict[str, Any],
    *,
    in_registry: bool,
) -> MaterialProfileEntry:
    albedo, normal, rough = _paths_from_registry_row(profile_id, row)
    return MaterialProfileEntry(
        profile_id=profile_id,
        label=str(row.get("label") or _profile_label(profile_id)),
        generator=str(row.get("generator") or infer_generator(profile_id)),
        category=str(row.get("category") or infer_category(profile_id)),
        albedo_path=albedo,
        normal_path=normal,
        roughness_path=rough,
        metallic=float(row.get("metallic") or 0.0),
        roughness_base=float(row.get("roughness_base") or 0.65),
        in_registry=in_registry,
    )


def _entry_from_pilot(defn: ProfileDef) -> MaterialProfileEntry:
    row = {
        "generator": defn.generator,
        "metallic": defn.metallic,
        "roughness_base": defn.roughness_base,
        "category": infer_category(defn.profile_id),
        "textures": {
            "albedo": f"assets/materials/textures/{defn.profile_id}/albedo.png",
            "normal": f"assets/materials/textures/{defn.profile_id}/normal.png",
            "roughness": f"assets/materials/textures/{defn.profile_id}/roughness.png",
        },
    }
    return _entry_from_row(defn.profile_id, row, in_registry=True)


def load_material_profile_catalog() -> list[MaterialProfileEntry]:
    """Sorted catalog: registry + pilot profiles + index-discovered ids."""
    by_id: dict[str, MaterialProfileEntry] = {}

    for pid, defn in PILOT_PROFILES.items():
        by_id[pid] = _entry_from_pilot(defn)

    reg_path = registry_path()
    if reg_path.is_file():
        data = json.loads(reg_path.read_text(encoding="utf-8"))
        for pid, row in (data.get("profiles") or {}).items():
            if pid in by_id:
                merged = dict(row if isinstance(row, dict) else {})
                merged.setdefault("category", infer_category(str(pid)))
                by_id[str(pid)] = _entry_from_row(str(pid), merged, in_registry=True)
                continue
            by_id[str(pid)] = _entry_from_row(
                str(pid), row if isinstance(row, dict) else {}, in_registry=True
            )

    try:
        from .assembly import load_index_json

        for row in load_index_json():
            pid = row.get("material_profile") or row.get("tileable_set_id")
            if not pid or str(pid) in by_id:
                continue
            pid = str(pid)
            by_id[pid] = MaterialProfileEntry(
                profile_id=pid,
                label=_profile_label(pid),
                generator=infer_generator(pid),
                category=infer_category(pid),
                albedo_path=(texture_dir(pid) / "albedo.png")
                if (texture_dir(pid) / "albedo.png").is_file()
                else None,
                normal_path=(texture_dir(pid) / "normal.png")
                if (texture_dir(pid) / "normal.png").is_file()
                else None,
                roughness_path=(texture_dir(pid) / "roughness.png")
                if (texture_dir(pid) / "roughness.png").is_file()
                else None,
                metallic=0.0,
                roughness_base=0.65,
                in_registry=False,
            )
    except Exception:
        pass

    return [by_id[k] for k in sorted(by_id)]


def profile_def_for_id(profile_id: str) -> ProfileDef:
    if profile_id in PILOT_PROFILES:
        return PILOT_PROFILES[profile_id]
    row = _row_from_registry(profile_id) or {}
    gen = str(row.get("generator") or infer_generator(profile_id))
    return ProfileDef(
        profile_id=profile_id,
        generator=gen,
        seed=_seed_for_profile(profile_id, row.get("seed")),
        metallic=float(row.get("metallic") or (0.85 if gen == "steel" else 0.0)),
        roughness_base=float(row.get("roughness_base") or (0.4 if gen == "steel" else 0.75)),
    )


def register_material_profile(
    profile_id: str,
    *,
    generator: str | None = None,
    category: str | None = None,
    seed: int | None = None,
    metallic: float | None = None,
    roughness_base: float | None = None,
) -> Path:
    """Add or update a profile row in material_profiles_v1.json."""
    pid = profile_id.strip()
    if not pid or any(c in pid for c in " /\\"):
        raise ValueError(f"Invalid profile_id {profile_id!r}")

    path = registry_path()
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.is_file():
        data = json.loads(path.read_text(encoding="utf-8"))
    else:
        data = {
            "schema_version": 1,
            "generator": "procedural_tile_v1",
            "note": "APS material library registry",
            "profiles": {},
        }

    gen = generator or infer_generator(pid)
    row = {
        "generator": gen,
        "seed": _seed_for_profile(pid, seed),
        "metallic": 0.85 if metallic is None and gen == "steel" else (metallic or 0.0),
        "roughness_base": roughness_base if roughness_base is not None else (0.4 if gen == "steel" else 0.75),
        "category": category or infer_category(pid),
        "label": _profile_label(pid),
        "textures": {
            "albedo": f"assets/materials/textures/{pid}/albedo.png",
            "normal": f"assets/materials/textures/{pid}/normal.png",
            "roughness": f"assets/materials/textures/{pid}/roughness.png",
        },
    }
    data.setdefault("profiles", {})[pid] = row
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    return path


def open_profile_folder(profile_id: str) -> Path:
    """Open texture folder in OS file manager; create dir if missing."""
    folder = texture_dir(profile_id)
    folder.mkdir(parents=True, exist_ok=True)
    if platform.system() == "Windows":
        os.startfile(folder)  # noqa: S606
    elif platform.system() == "Darwin":
        subprocess.run(["open", str(folder)], check=False)
    else:
        subprocess.run(["xdg-open", str(folder)], check=False)
    return folder


def open_registry_in_editor() -> Path:
    path = registry_path()
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.is_file():
        write_registry()
    if platform.system() == "Windows":
        os.startfile(path)  # noqa: S606
    elif platform.system() == "Darwin":
        subprocess.run(["open", str(path)], check=False)
    else:
        subprocess.run(["xdg-open", str(path)], check=False)
    return path


def ensure_profile_textures(profile_id: str, *, size: int = 512, force: bool = False) -> MaterialProfileEntry:
    """Generate deterministic textures if missing; register inferred profiles when needed."""
    pid = profile_id.strip()
    defn = profile_def_for_id(pid)
    albedo = texture_dir(pid) / "albedo.png"
    if force or not albedo.is_file():
        generate_profile(defn, size=size)
    if _row_from_registry(pid) is None and pid not in PILOT_PROFILES:
        register_material_profile(
            pid,
            generator=defn.generator,
            seed=defn.seed,
            metallic=defn.metallic,
            roughness_base=defn.roughness_base,
        )
    for entry in load_material_profile_catalog():
        if entry.profile_id == pid:
            return entry
    raise KeyError(f"Unknown material profile {profile_id!r}")


def generate_all_missing(*, size: int = 512) -> list[str]:
    """Generate textures for catalog entries that lack a full map set."""
    generated: list[str] = []
    for entry in load_material_profile_catalog():
        if entry.texture_status() == "ready":
            continue
        ensure_profile_textures(
            entry.profile_id,
            size=size,
            force=entry.texture_status() == "missing",
        )
        generated.append(entry.profile_id)
    return generated
