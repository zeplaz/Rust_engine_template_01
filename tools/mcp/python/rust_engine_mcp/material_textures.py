"""Deterministic tileable PBR texture generation for module material profiles."""

from __future__ import annotations

import json
import random
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

from .paths import repo_root


def _category_for_profile(profile_id: str) -> str:
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

try:
    from PIL import Image
except ImportError as exc:  # pragma: no cover
    raise ImportError("Install Pillow: pip install Pillow") from exc

SIZE = 512
NORMAL_FLAT = (128, 128, 255)


@dataclass(frozen=True)
class ProfileDef:
    profile_id: str
    generator: str
    seed: int
    metallic: float = 0.0
    roughness_base: float = 0.65


PILOT_PROFILES: dict[str, ProfileDef] = {
    "brick_red_01": ProfileDef("brick_red_01", "brick", 42, 0.0, 0.82),
    "concrete_grey_01": ProfileDef("concrete_grey_01", "concrete", 42, 0.0, 0.88),
    "wood_plank_01": ProfileDef("wood_plank_01", "wood", 42, 0.0, 0.72),
    "steel_door_01": ProfileDef("steel_door_01", "steel", 42, 0.85, 0.35),
    "steel_door_warehouse_01": ProfileDef("steel_door_warehouse_01", "steel", 42012, 0.88, 0.32),
    "steel_corner_01": ProfileDef("steel_corner_01", "steel", 42013, 0.85, 0.4),
    "roof_tile_01": ProfileDef("roof_tile_01", "concrete", 44, 0.0, 0.75),
    "glass_panel_01": ProfileDef("glass_panel_01", "steel", 45, 0.1, 0.15),
    "steel_panel_01": ProfileDef("steel_panel_01", "steel", 42010, 0.85, 0.42),
    "roof_metal_01": ProfileDef("roof_metal_01", "steel", 42011, 0.9, 0.38),
    "metal_roof_01": ProfileDef("metal_roof_01", "steel", 42011, 0.9, 0.38),
}


def profiles_registry_path() -> Path:
    return repo_root() / "assets" / "materials" / "profiles" / "material_profiles_v1.json"


def texture_dir(profile_id: str) -> Path:
    return repo_root() / "assets" / "materials" / "textures" / profile_id


def write_registry() -> Path:
    out = profiles_registry_path()
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema_version": 1,
        "generator": "procedural_tile_v1",
        "note": "Deterministic tileable sets — replace with Material Maker CLI when Tier 3 ships",
        "profiles": {
            pid: {
                "generator": p.generator,
                "seed": p.seed,
                "metallic": p.metallic,
                "roughness_base": p.roughness_base,
                "category": _category_for_profile(pid),
                "label": pid.replace("_", " ").title(),
                "textures": {
                    "albedo": f"assets/materials/textures/{pid}/albedo.png",
                    "normal": f"assets/materials/textures/{pid}/normal.png",
                    "roughness": f"assets/materials/textures/{pid}/roughness.png",
                },
            }
            for pid, p in PILOT_PROFILES.items()
        },
    }
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return out


def _noise(rng: random.Random, x: int, y: int) -> float:
    return rng.random()


def _gen_brick(rng: random.Random, size: int) -> Image.Image:
    img = Image.new("RGB", (size, size))
    px = img.load()
    mortar = (0.58, 0.55, 0.52)
    brick_w, brick_h = size // 4, size // 8
    mortar_px = max(2, size // 64)
    for y in range(size):
        row = y // brick_h
        offset = (brick_w // 2) if row % 2 else 0
        for x in range(size):
            bx = (x + offset) % brick_w
            by = y % brick_h
            if bx < mortar_px or by < mortar_px:
                c = mortar
            else:
                n = _noise(rng, x, y) * 0.12
                c = (0.62 + n, 0.28 + n * 0.5, 0.22 + n * 0.3)
            px[x, y] = (int(c[0] * 255), int(c[1] * 255), int(c[2] * 255))
    return img


def _gen_concrete(rng: random.Random, size: int) -> Image.Image:
    img = Image.new("RGB", (size, size))
    px = img.load()
    base = (0.52, 0.52, 0.50)
    for y in range(size):
        for x in range(size):
            n = (_noise(rng, x, y) - 0.5) * 0.18
            speck = 0.04 if rng.random() > 0.97 else 0.0
            c = (base[0] + n + speck, base[1] + n + speck, base[2] + n + speck * 0.5)
            px[x, y] = tuple(int(max(0, min(1, v)) * 255) for v in c)
    return img


def _gen_wood(rng: random.Random, size: int) -> Image.Image:
    img = Image.new("RGB", (size, size))
    px = img.load()
    plank_h = max(8, size // 16)
    for y in range(size):
        plank = y // plank_h
        base = 0.42 + (plank % 3) * 0.04
        for x in range(size):
            grain = (_noise(rng, x // 2, y) - 0.5) * 0.08
            ring = 0.03 * ((y % plank_h) / plank_h - 0.5)
            c = (base + grain + ring, base * 0.75 + grain, base * 0.45 + grain * 0.5)
            px[x, y] = tuple(int(max(0, min(1, v)) * 255) for v in c)
    return img


def _gen_steel(rng: random.Random, size: int) -> Image.Image:
    """Brushed panel steel — blue-gray tint, seams, speckle (distinct from APS error swatch ~#60686e)."""
    img = Image.new("RGB", (size, size))
    px = img.load()
    panel_w = max(24, size // 3)
    seam = max(2, size // 128)
    for y in range(size):
        panel_band = (y // panel_w) % 2
        for x in range(size):
            along = (_noise(rng, x // 3, y) - 0.5) * 0.14
            brush = (_noise(rng, x, y // 6) - 0.5) * 0.08
            seam_dark = -0.12 if (x % panel_w) < seam or (y % panel_w) < seam else 0.0
            speck = 0.09 if rng.random() > 0.992 else 0.0
            band = 0.03 if panel_band else -0.02
            base_r, base_g, base_b = 0.46, 0.52, 0.58
            r = base_r + along + brush + seam_dark + speck + band
            g = base_g + along * 0.9 + brush + seam_dark + speck * 0.8 + band
            b = base_b + along * 1.1 + brush * 0.7 + seam_dark + speck + band + 0.04
            px[x, y] = tuple(int(max(0, min(1, v)) * 255) for v in (r, g, b))
    return img


def _normal_from_albedo(albedo: Image.Image, *, strength: float = 2.2) -> Image.Image:
    """Lightweight height-from-luminance normal map (Tier-2; replace with authored maps later)."""
    grey = albedo.convert("L")
    w, h = grey.size
    px = grey.load()
    out = Image.new("RGB", (w, h), NORMAL_FLAT)
    po = out.load()

    def lum(x: int, y: int) -> float:
        x = max(0, min(w - 1, x))
        y = max(0, min(h - 1, y))
        return px[x, y] / 255.0

    for y in range(h):
        for x in range(w):
            dx = (lum(x + 1, y) - lum(x - 1, y)) * strength
            dy = (lum(x, y + 1) - lum(x, y - 1)) * strength
            nx = max(-1.0, min(1.0, -dx))
            ny = max(-1.0, min(1.0, -dy))
            nz = max(0.35, min(1.0, 1.0 - abs(dx) * 0.25 - abs(dy) * 0.25))
            po[x, y] = (
                int((nx * 0.5 + 0.5) * 255),
                int((ny * 0.5 + 0.5) * 255),
                int(nz * 255),
            )
    return out


_GENERATORS: dict[str, Callable[[random.Random, int], Image.Image]] = {
    "brick": _gen_brick,
    "concrete": _gen_concrete,
    "wood": _gen_wood,
    "steel": _gen_steel,
}


def _roughness_map(albedo: Image.Image, base: float, rng: random.Random) -> Image.Image:
    grey = albedo.convert("L")
    out = Image.new("L", grey.size)
    px_in = grey.load()
    px_out = out.load()
    for y in range(grey.size[1]):
        for x in range(grey.size[0]):
            v = px_in[x, y] / 255.0
            r = base + (0.5 - v) * 0.15 + (rng.random() - 0.5) * 0.04
            px_out[x, y] = int(max(0, min(255, r * 255)))
    return out


def generate_profile(profile: ProfileDef, *, size: int = SIZE) -> dict[str, str]:
    rng = random.Random(profile.seed)
    gen = _GENERATORS.get(profile.generator)
    if gen is None:
        raise ValueError(f"Unknown generator {profile.generator!r}")

    out_dir = texture_dir(profile.profile_id)
    out_dir.mkdir(parents=True, exist_ok=True)

    albedo = gen(rng, size)
    if profile.generator == "steel":
        normal = _normal_from_albedo(albedo, strength=2.8)
    elif profile.generator in ("brick", "concrete", "wood"):
        normal = _normal_from_albedo(albedo, strength=1.6)
    else:
        normal = Image.new("RGB", (size, size), NORMAL_FLAT)
    rough = _roughness_map(albedo, profile.roughness_base, rng)

    albedo_path = out_dir / "albedo.png"
    normal_path = out_dir / "normal.png"
    rough_path = out_dir / "roughness.png"
    albedo.save(albedo_path)
    normal.save(normal_path)
    rough.save(rough_path)

    manifest = {
        "profile_id": profile.profile_id,
        "seed": profile.seed,
        "generator": profile.generator,
        "size": size,
        "paths": {
            "albedo": str(albedo_path.relative_to(repo_root())).replace("\\", "/"),
            "normal": str(normal_path.relative_to(repo_root())).replace("\\", "/"),
            "roughness": str(rough_path.relative_to(repo_root())).replace("\\", "/"),
        },
    }
    (out_dir / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return manifest


def generate_pilot_profiles(profile_ids: list[str] | None = None) -> list[dict]:
    write_registry()
    ids = profile_ids or list(PILOT_PROFILES.keys())
    results = []
    for pid in ids:
        if pid not in PILOT_PROFILES:
            raise KeyError(f"Unknown profile {pid}")
        results.append(generate_profile(PILOT_PROFILES[pid]))
    return results


def load_profile_manifest(profile_id: str) -> dict | None:
    path = texture_dir(profile_id) / "manifest.json"
    if not path.is_file():
        return None
    return json.loads(path.read_text(encoding="utf-8"))
