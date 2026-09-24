"""Assembly snapshot generation — StylePack + footprint → JSON (no Blender)."""

from __future__ import annotations

import hashlib
import json
import re
from copy import deepcopy
from pathlib import Path
from typing import Any

from .library import load_index_json
from .paths import repo_root, schemas_dir
from .schemas import load_json_file, validate_assembly_snapshot
from . import aps_tags


STYLE_PACKS_DIR = "assets/configs/buildings/style_packs"
SNAPSHOT_STAGING = "assets/staging/assemblies"
RULES_VERSION = "pg2_wdc_v1"
GRAMMAR_RULES_VERSION = "building_grammar_v1"
# BQ-C1 / Building Look v2 — must match module_contract_v1 + Rust GRID_UNIT_M / FLOOR_HEIGHT_M.
GRID_UNIT_M = 4.0
FLOOR_HEIGHT_M = 3.0
# Authored full-footprint roof kits (metal_low / pitched_gable) are 8×16 m = 2×4 cells.
ROOF_AUTHOR_SHORT_M = 8.0
ROOF_AUTHOR_LONG_M = 16.0


def full_footprint_roof_scale(width: int, depth: int) -> list[float]:
    """Non-uniform XY scale so 8×16 authored roofs cover arbitrary footprints.

    Mesh local X = short axis, Z = long axis; yaw π/2 when width≥depth maps
    long→street X. Scale is applied in mesh local space before rotation.
    """
    short_cells = max(1, min(int(width), int(depth)))
    long_cells = max(1, max(int(width), int(depth)))
    sx = (short_cells * GRID_UNIT_M) / ROOF_AUTHOR_SHORT_M
    sz = (long_cells * GRID_UNIT_M) / ROOF_AUTHOR_LONG_M
    return [sx, 1.0, sz]


def _style_packs_dir() -> Path:
    return repo_root() / STYLE_PACKS_DIR


def _parse_ron_slots(text: str) -> dict[str, str]:
    slots: dict[str, str] = {}
    m = re.search(r"slots:\s*\((.*?)\)\s*,", text, re.DOTALL)
    if not m:
        return slots
    block = m.group(1)
    for key, val in re.findall(r"(\w+):\s*\"([^\"]+)\"", block):
        slots[key] = val
    return slots


def load_style_pack(style_pack_id: str) -> dict[str, Any]:
    path = _style_packs_dir() / f"{style_pack_id}.ron"
    if not path.is_file():
        raise FileNotFoundError(f"StylePack not found: {path}")
    text = path.read_text(encoding="utf-8")
    m = re.search(r'style_pack_id:\s*"([^"]+)"', text)
    pack_id = m.group(1) if m else style_pack_id
    label_m = re.search(r'label:\s*"([^"]+)"', text)
    return {
        "style_pack_id": pack_id,
        "label": label_m.group(1) if label_m else pack_id,
        "slots": _parse_ron_slots(text),
    }


def list_style_packs() -> list[str]:
    out: list[str] = []
    root = _style_packs_dir()
    if not root.is_dir():
        return out
    for path in sorted(root.glob("style_*.ron")):
        if "_manifest" in path.name:
            continue
        out.append(path.stem)
    return out


def _index_by_module_id(
    index: list[dict[str, Any]],
    *,
    prefer_tier: str = "production",
) -> dict[str, dict[str, Any]]:
    """Prefer production_run* (default) or lod0_run* rows when duplicate module_id keys exist."""
    out: dict[str, dict[str, Any]] = {}
    for row in index:
        mid = str(row["module_id"])
        job = str(row.get("job_id") or "")
        tier = str(row.get("development_tier") or "")
        prev = out.get(mid)
        if prev is None:
            out[mid] = row
            continue
        prev_job = str(prev.get("job_id") or "")
        prev_tier = str(prev.get("development_tier") or "")
        if prefer_tier == "production":
            if tier == "production" and prev_tier != "production":
                out[mid] = row
            elif "production" in job and "production" not in prev_job:
                out[mid] = row
        elif "lod0" in job and "lod0" not in prev_job:
            out[mid] = row
    return out


# kit_production_001 promoted job_ids (MCP-PROD-KIT-001).
# corner_L excluded — Victorian production mesh drifts from style_industrial_west (PG-MODULE-AUDIT-001).
LOD0_TO_PRODUCTION_JOB: dict[str, str] = {
    "wall_brick_1u_lod0_run001": "wall_brick_1u_production_run001",
    "door_residential_lod0_run001": "door_residential_production_run001",
    "roof_pitched_gable_lod0_run001": "roof_pitched_gable_production_run001",
    "prop_chimney_lod0_run001": "prop_chimney_production_run001",
}


def _rows_for_module(module_id: str, index: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [r for r in index if str(r.get("module_id") or "") == module_id]


def _row_glb_ready(row: dict[str, Any]) -> bool:
    return _module_glb_path(row).is_file()


def _resolve_module_row(
    module_id: str,
    index: list[dict[str, Any]],
    *,
    style_pack_id: str,
    source_tier: str,
    seen: frozenset[str] | None = None,
) -> dict[str, Any] | None:
    """Pick index row matching style_pack; prefer production tier when requested."""
    if seen is None:
        seen = frozenset()
    if module_id in seen:
        return None
    rows = _rows_for_module(module_id, index)
    if not rows:
        return None

    pack_rows = [r for r in rows if str(r.get("style_pack") or "") == style_pack_id]
    pool = pack_rows if pack_rows else rows

    if source_tier == "production":
        # Prefer same-pack production, then any-pack production, then same-pack lod0.
        # Never prefer same-pack lod0 over cross-pack production (KF-4 / BQ-PROD-DEFER-PACKS-001).
        for row in pool:
            tier = str(row.get("development_tier") or "")
            batch = str(row.get("batch_id") or "")
            if tier == "production" or batch.startswith(("kit_production", "kit_industrial_west_production")):
                if _row_glb_ready(row):
                    return row
        if pack_rows:
            for row in rows:
                tier = str(row.get("development_tier") or "")
                batch = str(row.get("batch_id") or "")
                if tier == "production" or batch.startswith(("kit_production", "kit_industrial_west_production")):
                    if _row_glb_ready(row):
                        return row
        for row in pool:
            job = str(row.get("job_id") or "")
            if "lod0" in job and _row_glb_ready(row):
                return row
        return None

    index_by_id = _index_by_module_id(index, prefer_tier="lod0")
    return _resolve_lod0_module(module_id, index_by_id, seen)


def _resolve_production_module(
    module_id: str,
    index_by_id: dict[str, dict[str, Any]],
    seen: frozenset[str] | None = None,
) -> dict[str, Any] | None:
    if seen is None:
        seen = frozenset()
    if module_id in seen:
        return None
    row = index_by_id.get(module_id)
    if not row:
        return None
    tier = str(row.get("development_tier") or "")
    batch = str(row.get("batch_id") or "")
    if tier != "production" and not batch.startswith(("kit_production", "kit_industrial_west_production")):
        return None
    glb = _module_glb_path(row)
    if not glb.is_file():
        return None
    return row


def _resolve_lod0_module(
    module_id: str,
    index_by_id: dict[str, dict[str, Any]],
    seen: frozenset[str] | None = None,
) -> dict[str, Any] | None:
    if seen is None:
        seen = frozenset()
    if module_id in seen:
        return None
    row = index_by_id.get(module_id)
    if not row:
        return None
    tier = str(row.get("development_tier") or "")
    batch = str(row.get("batch_id") or "")
    visible = row.get("stylepack_visible", True)
    if tier == "smoke" or batch.startswith("kit_greybox") or not visible:
        replaced = row.get("replaced_by")
        if replaced and str(replaced) != module_id and str(replaced) in index_by_id:
            return _resolve_lod0_module(str(replaced), index_by_id, seen | {module_id})
        return None
    return row


def _module_glb_path(row: dict[str, Any]) -> Path:
    job_id = str(row["job_id"])
    return repo_root() / "assets" / "models" / "modules" / job_id / "model.glb"


def explain_module_resolve(
    module_id: str,
    *,
    style_pack_id: str = "",
    source_tier: str = "production",
) -> dict[str, Any]:
    """CMCP-ASM-RESOLVE-HONEST-001 — inline reason when production GLB missing."""
    index = load_index_json()
    row = _resolve_module_row(
        module_id,
        index,
        style_pack_id=style_pack_id,
        source_tier=source_tier,
    )
    if not row:
        return {
            "module_id": module_id,
            "ok": False,
            "reason": "not_in_index",
            "label": f"✗ No module index row — validate or register {module_id}",
            "hint": "library-register or kit batch promote",
        }
    tier = str(row.get("development_tier") or "")
    glb = _module_glb_path(row)
    if not glb.is_file():
        return {
            "module_id": module_id,
            "ok": False,
            "reason": "glb_missing",
            "job_id": str(row.get("job_id") or ""),
            "label": f"✗ GLB missing — promote {row.get('job_id')}",
            "hint": "validate-report asset_glb then library-promote",
        }
    if source_tier == "production" and tier != "production":
        return {
            "module_id": module_id,
            "ok": True,
            "reason": "lod0_fallback",
            "resolved_tier": "lod0",
            "job_id": str(row.get("job_id") or ""),
            "glb_path": str(glb.relative_to(repo_root())).replace("\\", "/"),
            "label": f"◐ Production missing — showing {tier} ({row.get('job_id')})",
            "hint": "Run ship check or remap to kit_production_001",
        }
    return {
        "module_id": module_id,
        "ok": True,
        "reason": "ok_production" if tier == "production" else "ok_lod0",
        "resolved_tier": tier or source_tier,
        "job_id": str(row.get("job_id") or ""),
        "glb_path": str(glb.relative_to(repo_root())).replace("\\", "/"),
        "label": f"✓ {tier} · {row.get('job_id')}",
        "hint": "",
    }


FootprintToken = str  # W | D | C | R | O (opening/window)


def _is_perimeter(x: int, y: int, width: int, depth: int) -> bool:
    return x == 0 or y == 0 or x + 1 == width or y + 1 == depth


def _is_corner(x: int, y: int, width: int, depth: int) -> bool:
    return (x == 0 or x + 1 == width) and (y == 0 or y + 1 == depth)


def exterior_faces(x: int, y: int, width: int, depth: int) -> list[str]:
    """Cardinal exterior edges this perimeter cell sits on (S/N/W/E).

    Corners return *two* faces so dual wall placements close the envelope —
    diagonal single-yaw corners left diamond bay gaps / skeletal pierces on depth=2.
    """
    faces: list[str] = []
    if y == 0:
        faces.append("S")
    if y + 1 == depth:
        faces.append("N")
    if x == 0:
        faces.append("W")
    if x + 1 == width:
        faces.append("E")
    return faces


def outward_yaw_for_face(face: str) -> float:
    """Yaw about +Y for a cardinal face (default module normal glTF −Z)."""
    import math

    key = str(face or "S").upper()
    if key == "N":
        return math.pi
    if key == "W":
        return math.pi / 2.0
    if key == "E":
        return -math.pi / 2.0
    return 0.0  # S


def outward_yaw_rad(x: int, y: int, width: int, depth: int) -> float:
    """Yaw about +Y so the module face points off the footprint (Building Look v2).

    After BUILDING-LOOK-V2-ART Z-up authorship + export_yup, wall face normals default
    to glTF −Z (Blender +Y → −Z). South edge (y=0) therefore uses yaw 0.

    When a cell has multiple exterior faces (corner), prefer street/south then
    north — callers that need a closed envelope should place one module per face
    via ``exterior_faces`` + ``outward_yaw_for_face``.
    """
    faces = exterior_faces(x, y, width, depth)
    if not faces:
        return 0.0
    for prefer in ("S", "N", "W", "E"):
        if prefer in faces:
            return outward_yaw_for_face(prefer)
    return outward_yaw_for_face(faces[0])


def footprint_grid(width: int, depth: int, floors: int) -> list[dict[str, Any]]:
    width = max(2, width)
    depth = max(2, depth)
    floors = max(1, floors)
    door_x = width // 2
    cells: list[dict[str, Any]] = []

    for floor in range(floors):
        for y in range(depth):
            for x in range(width):
                if not _is_perimeter(x, y, width, depth):
                    continue
                faces = exterior_faces(x, y, width, depth)
                # Dual-face corners: one bay per cardinal edge (closes gaps / pierces).
                for face in faces:
                    if face == "S":
                        if floor == 0 and x == door_x:
                            token = "D"
                        elif x != door_x or floor > 0:
                            # Street openings — ground non-door + all upper (readable at distance).
                            token = "O"
                        else:
                            token = "W"
                    elif face == "N":
                        # Soft openings on north — iso_ne QC camera sees N/E, not only street S.
                        if x != door_x or floor > 0:
                            token = "O"
                        else:
                            token = "W"
                    elif face in ("E", "W") and y == (depth // 2):
                        # Mid long-face window band — reads from iso_se / side views.
                        token = "O"
                    else:
                        token = "W"
                    cells.append(
                        {
                            "x": x,
                            "y": y,
                            "floor": floor,
                            "token": token,
                            "face": face,
                        }
                    )

    # Roof seat: ONE full-footprint module at plan center — per-bay gable caps
    # read as sawtooth collage (Look v2). Mesh authored 8×16 (4×2); larger footprints scale.
    roof_floor = floors
    cells.append(
        {
            "x": max(0, width // 2),
            "y": max(0, (depth - 1) // 2),
            "floor": roof_floor,
            "token": "R",
            "face": "R",
            "roof_mode": "full_footprint",
        }
    )
    return cells


SLOT_FOR_TOKEN = {
    "W": "wall_1u",
    "D": "door_default",
    "C": "corner_outer",
    "R": "roof_default",
    "O": "window_1u",
}

# ARCH-003 — default placement tags by footprint token (APS Assembly Editor checkboxes).
TOKEN_PLACEMENT_TAGS: dict[str, list[str]] = {
    "W": ["exterior", "wall"],
    "D": ["exterior", "door"],
    "C": ["exterior", "corner"],
    "R": ["exterior", "roof"],
    "O": ["exterior", "window_band"],
}

COMMON_PLACEMENT_TAGS = (
    "exterior",
    "interior",
    "wall",
    "door",
    "corner",
    "roof",
    "industrial",
    "weathered",
    "clean",
    "damaged",
    "night",
)

COMMON_VARIANT_TAGS = ("clean", "damaged", "night", "construction", "fire")


def placement_node_id(placement: dict[str, Any]) -> str:
    """Stable AssemblyNode id for UI selection and patches."""
    existing = placement.get("node_id")
    if existing:
        return str(existing)
    mid = str(placement.get("module_id") or "mod")
    gx = int(placement.get("grid_x") or 0)
    gy = int(placement.get("grid_y") or 0)
    fl = int(placement.get("floor") or 0)
    face = str(placement.get("face") or "").upper()
    if face and face != "R":
        return f"{mid}_{gx}_{gy}_f{fl}_{face}"
    return f"{mid}_{gx}_{gy}_f{fl}"


def _material_profile_from_index_row(row: dict[str, Any]) -> str | None:
    profile = row.get("material_profile") or row.get("tileable_set_id")
    if profile:
        return str(profile)
    return None


def _default_lod_policy(row: dict[str, Any], source_tier: str) -> str:
    tier = str(row.get("development_tier") or source_tier or "lod0")
    if tier == "production":
        return "production"
    return "lod0"


def enrich_placement(
    placement: dict[str, Any],
    *,
    source_tier: str = "production",
    index_row: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """ARCH-003 — ensure node_id, material_profile, tags, lod_policy on one placement."""
    out = dict(placement)
    out["node_id"] = placement_node_id(out)
    row = index_row
    if row is None and out.get("module_id"):
        index = load_index_json()
        prefer = "production" if source_tier == "production" else "lod0"
        row = _index_by_module_id(index, prefer_tier=prefer).get(str(out["module_id"]))
    if row and not out.get("material_profile"):
        profile = _material_profile_from_index_row(row)
        if profile:
            out["material_profile"] = profile
    token = str(out.get("token") or "W")
    if not out.get("placement_tags"):
        out["placement_tags"] = list(TOKEN_PLACEMENT_TAGS.get(token, ["exterior"]))
    if not out.get("variant_tags"):
        out["variant_tags"] = ["clean"]
    if not out.get("lod_policy"):
        out["lod_policy"] = _default_lod_policy(row or {}, source_tier)
    return aps_tags.sync_placement_tags(out)


def enrich_snapshot(snapshot: dict[str, Any]) -> dict[str, Any]:
    """Apply ARCH-003 fields to all placements (idempotent)."""
    tier = str(snapshot.get("source_tier") or "production")
    index = load_index_json()
    index_by_id = _index_by_module_id(
        index, prefer_tier="production" if tier == "production" else "lod0"
    )
    placements = [
        enrich_placement(p, source_tier=tier, index_row=index_by_id.get(str(p.get("module_id") or "")))
        for p in snapshot.get("module_placements") or []
    ]
    out = dict(snapshot)
    out["module_placements"] = placements
    return out


def list_material_profiles() -> list[str]:
    """Known material_profile ids for Assembly Editor / MCP."""
    try:
        from .material_profiles import load_material_profile_catalog

        return [e.profile_id for e in load_material_profile_catalog()]
    except Exception:
        pass
    seen: set[str] = set()
    try:
        from .material_textures import PILOT_PROFILES

        seen.update(PILOT_PROFILES.keys())
    except ImportError:
        pass
    reg = repo_root() / "assets" / "materials" / "profiles" / "material_profiles_v1.json"
    if reg.is_file():
        data = json.loads(reg.read_text(encoding="utf-8"))
        seen.update((data.get("profiles") or {}).keys())
    for row in load_index_json():
        profile = row.get("material_profile") or row.get("tileable_set_id")
        if profile:
            seen.add(str(profile))
    return sorted(seen)


def save_assembly_snapshot(
    snapshot: dict[str, Any],
    *,
    path: str | Path | None = None,
) -> Path:
    """Write enriched snapshot; validates schema."""
    out = enrich_snapshot(snapshot)
    validate_assembly_snapshot(out)
    assembly_id = str(out["assembly_id"])
    out_path = Path(path) if path else default_snapshot_path(assembly_id)
    if not out_path.is_absolute():
        out_path = repo_root() / out_path
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
    out["written_path"] = str(out_path.relative_to(repo_root())).replace("\\", "/")
    return out_path


def footprint_cells_for_snapshot(snapshot: dict[str, Any]) -> list[dict[str, Any]]:
    """Plan-view cells for APS footprint grid (grammar-aware when metadata present)."""
    fp = snapshot.get("footprint") or {}
    width = int(fp.get("width") or 4)
    depth = int(fp.get("depth") or 3)
    floors = int(fp.get("floors") or 2)
    seed = int(snapshot.get("seed") or 42)
    archetype = snapshot.get("archetype_id")
    district = snapshot.get("district_style")
    if archetype and district:
        from . import building_grammar

        grammar = building_grammar.generate(str(archetype), str(district), seed)
        return building_grammar.footprint_grid_from_grammar(grammar)
    return footprint_grid(width, depth, floors)


def update_placement(
    snapshot: dict[str, Any],
    node_id: str,
    *,
    material_profile: str | None = None,
    placement_tags: list[str] | None = None,
    semantic_tags: dict[str, list[str]] | None = None,
    variant_tags: list[str] | None = None,
    lod_policy: str | None = None,
    module_id: str | None = None,
) -> dict[str, Any]:
    """Patch one placement by node_id; returns updated snapshot."""
    out = enrich_snapshot(snapshot)
    found = False
    placements: list[dict[str, Any]] = []
    for p in out.get("module_placements") or []:
        row = dict(p)
        if placement_node_id(row) != node_id:
            placements.append(row)
            continue
        found = True
        if material_profile is not None:
            row["material_profile"] = material_profile.strip() or None
            if not row["material_profile"]:
                row.pop("material_profile", None)
        if semantic_tags is not None:
            row["semantic_tags"] = {k: list(v) for k, v in semantic_tags.items() if v}
            row["placement_tags"] = aps_tags.flatten_semantic_tags(row["semantic_tags"])
        elif placement_tags is not None:
            row["placement_tags"] = list(placement_tags)
            row["semantic_tags"] = aps_tags.semantic_tags_from_flat(row["placement_tags"])
        if variant_tags is not None:
            row["variant_tags"] = list(variant_tags)
        if lod_policy is not None:
            row["lod_policy"] = lod_policy
        if module_id is not None:
            row["module_id"] = module_id
            style_pack = str(out.get("style_pack_id") or "")
            tier = str(out.get("source_tier") or "production")
            mod_row = _resolve_module_row(
                module_id,
                load_index_json(),
                style_pack_id=style_pack,
                source_tier=tier,
            )
            if mod_row:
                glb = _module_glb_path(mod_row)
                row["job_id"] = str(mod_row.get("job_id") or "")
                if glb.is_file():
                    row["glb_path"] = str(glb.relative_to(repo_root())).replace("\\", "/")
                else:
                    row.pop("glb_path", None)
                if tier == "production" and str(mod_row.get("development_tier") or "") != "production":
                    row["mesh_tier_fallback"] = "lod0"
                else:
                    row.pop("mesh_tier_fallback", None)
            else:
                row.pop("glb_path", None)
                row.pop("job_id", None)
        placements.append(enrich_placement(row, source_tier=str(out.get("source_tier") or "production")))
    if not found:
        raise KeyError(f"placement node_id not found: {node_id}")
    out["module_placements"] = placements
    validate_assembly_snapshot(out)
    return out


def _grid_to_position(x: int, y: int, floor: int) -> list[float]:
    """World metres — cell × GRID_UNIT_M (4), floor × FLOOR_HEIGHT_M (3). South edge y=0."""
    return [
        float(x) * GRID_UNIT_M,
        float(floor) * FLOOR_HEIGHT_M,
        float(y) * GRID_UNIT_M,
    ]


def _wall_position(x: int, y: int, floor: int, face: str) -> list[float]:
    """Seat wall on the exterior edge of the cell (closes envelope; avoids center-pier look)."""
    pos = _grid_to_position(x, y, floor)
    half = GRID_UNIT_M * 0.5
    key = str(face or "").upper()
    if key == "S":
        pos[2] -= half
    elif key == "N":
        pos[2] += half
    elif key == "W":
        pos[0] -= half
    elif key == "E":
        pos[0] += half
    return pos


def _roof_position(width: int, depth: int, floor: int) -> list[float]:
    """Center of footprint plan — single continuous roof seat."""
    return [
        (max(width, 1) - 1) * GRID_UNIT_M * 0.5,
        float(floor) * FLOOR_HEIGHT_M,
        (max(depth, 1) - 1) * GRID_UNIT_M * 0.5,
    ]


def _rotation_for_cell(
    token: str,
    x: int,
    y: int,
    width: int,
    depth: int,
    *,
    face: str | None = None,
) -> list[float]:
    """Euler XYZ radians. Roofs stay axis-aligned; walls/doors/openings yaw per face."""
    if token == "R":
        return [0.0, 0.0, 0.0]
    if face and str(face).upper() in ("S", "N", "W", "E"):
        return [0.0, outward_yaw_for_face(str(face)), 0.0]
    return [0.0, outward_yaw_rad(x, y, width, depth), 0.0]


def _module_width_cells(row: dict[str, Any] | None) -> int:
    """Grid cells spanned along X by a roof/wall module (≥1). 2u roofs → stride-2 ridge."""
    if not row:
        return 1
    for key in ("footprint_w", "grid_w", "width_cells"):
        if row.get(key) is not None:
            return max(1, int(row[key]))
    job_id = str(row.get("job_id") or "")
    if job_id:
        job_path = repo_root() / "tools/mcp/schemas/examples" / f"{job_id}.json"
        if job_path.is_file():
            try:
                job = json.loads(job_path.read_text(encoding="utf-8"))
                wm = (job.get("params") or {}).get("width_m")
                if wm is not None:
                    return max(1, int(round(float(wm) / GRID_UNIT_M)))
            except (OSError, json.JSONDecodeError, TypeError, ValueError):
                pass
    mid = str(row.get("module_id") or job_id or "")
    if "_2u" in mid or mid.endswith("2u"):
        return 2
    return 1


def _assembly_id(style_pack_id: str, width: int, depth: int, floors: int, seed: int) -> str:
    raw = f"{style_pack_id}:{width}x{depth}x{floors}:s{seed}"
    digest = hashlib.sha256(raw.encode()).hexdigest()[:4]
    pack_suffix = style_pack_id.removeprefix("style_")
    return f"{pack_suffix}_{width}x{depth}_s{seed}_{digest}"


def remap_assembly_snapshot_to_production(
    snapshot: dict[str, Any],
    *,
    reference_tags: list[str] | None = None,
    kit_batch_id: str = "kit_production_001",
) -> dict[str, Any]:
    """Remap lod0 placements to kit_production_001 job_ids + production tier metadata."""
    out = json.loads(json.dumps(snapshot))
    out["source_tier"] = "production"
    tags = list(reference_tags or out.get("reference_tags") or [])
    for tag in (f"ref:kit:{kit_batch_id}",):
        if tag not in tags:
            tags.append(tag)
    out["reference_tags"] = tags
    placements: list[dict[str, Any]] = []
    lod0_fallbacks = 0
    for p in out.get("module_placements") or []:
        row = dict(p)
        old_job = str(row.get("job_id") or "")
        new_job = LOD0_TO_PRODUCTION_JOB.get(old_job)
        if not new_job and old_job.endswith("_lod0_run001"):
            new_job = old_job.replace("_lod0_run001", "_production_run001")
        if new_job:
            row["job_id"] = new_job
            glb = repo_root() / "assets" / "models" / "modules" / new_job / "model.glb"
            if not glb.is_file() and old_job:
                lod0_glb = repo_root() / "assets" / "models" / "modules" / old_job / "model.glb"
                if lod0_glb.is_file():
                    row["job_id"] = old_job
                    row["glb_path"] = str(lod0_glb.relative_to(repo_root())).replace("\\", "/")
                    row["mesh_tier_fallback"] = "lod0"
                    lod0_fallbacks += 1
                else:
                    raise FileNotFoundError(
                        f"production GLB missing for {new_job} and lod0 {old_job}"
                    )
            else:
                row["glb_path"] = str(glb.relative_to(repo_root())).replace("\\", "/")
        placements.append(row)
    if lod0_fallbacks:
        out["mesh_tier_fallback_count"] = lod0_fallbacks
    out["module_placements"] = placements
    validate_assembly_snapshot(out)
    return out


def _placement_for_cell(
    cell: dict[str, Any],
    *,
    pack: dict[str, Any],
    index: list[dict[str, Any]],
    slot_overrides: dict[str, str],
    grammar: dict[str, Any] | None,
    source_tier: str,
    default_placement_tags: list[str],
    default_variant_tags: list[str],
    footprint_width: int,
    footprint_depth: int,
) -> dict[str, Any] | None:
    from . import building_grammar

    token = str(cell["token"])
    if token == "Y":
        return None
    slot_key = SLOT_FOR_TOKEN.get(token)
    if not slot_key:
        return None
    slot_key = slot_overrides.get(slot_key, slot_key)
    module_id = pack["slots"].get(slot_key)
    # Window slot hole → solid wall (still an envelope; style kit fill is separate art work).
    if not module_id and token == "O":
        token = "W"
        slot_key = slot_overrides.get("wall_1u", "wall_1u")
        module_id = pack["slots"].get(slot_key)
    if not module_id:
        return None
    row = _resolve_module_row(
        module_id,
        index,
        style_pack_id=str(pack["style_pack_id"]),
        source_tier=source_tier,
    )
    if not row:
        return None
    glb = _module_glb_path(row)
    if not glb.is_file():
        return None
    gx, gy, gf = int(cell["x"]), int(cell["y"]), int(cell["floor"])
    face = str(cell.get("face") or "")
    if token == "R":
        span = _module_width_cells(row)
        if span > 1 and (gx % span) != 0 and str(cell.get("roof_mode") or "") != "full_footprint":
            return None
    if token == "R" and str(cell.get("roof_mode") or "") == "full_footprint":
        position = _roof_position(footprint_width, footprint_depth, gf)
        # Mesh authored ridge-along-Z; yaw so ridge follows long (X) street axis.
        rotation = [0.0, 0.0, 0.0]
        if footprint_width >= footprint_depth:
            import math

            rotation = [0.0, math.pi / 2.0, 0.0]
        scale = full_footprint_roof_scale(footprint_width, footprint_depth)
    elif face and face != "R":
        position = _wall_position(gx, gy, gf, face)
        rotation = _rotation_for_cell(
            token, gx, gy, footprint_width, footprint_depth, face=face
        )
        scale = None
    else:
        position = _grid_to_position(gx, gy, gf)
        rotation = _rotation_for_cell(
            token, gx, gy, footprint_width, footprint_depth, face=face or None
        )
        scale = None
    base = {
        "module_id": str(row["module_id"]),
        "job_id": str(row["job_id"]),
        "slot_key": slot_key,
        "token": token,
        "grid_x": gx,
        "grid_y": gy,
        "floor": gf,
        "glb_path": str(glb.relative_to(repo_root())).replace("\\", "/"),
        "position": position,
        "rotation_euler": rotation,
    }
    if scale is not None and (abs(scale[0] - 1.0) > 1e-6 or abs(scale[2] - 1.0) > 1e-6):
        base["scale"] = scale
    if face and face != "R":
        base["face"] = face
    if str(cell.get("roof_mode") or ""):
        base["roof_mode"] = str(cell.get("roof_mode"))
    if source_tier == "production" and str(row.get("development_tier") or "") != "production":
        base["mesh_tier_fallback"] = "lod0"
    enriched = enrich_placement(base, source_tier=source_tier, index_row=row)
    if grammar is not None:
        prof = building_grammar.material_profile_for_slot(grammar, slot_key)
        if prof:
            enriched["material_profile"] = prof
        enriched["weathering"] = str(grammar.get("weathering") or "medium")
    if default_placement_tags and not enriched.get("placement_tags"):
        enriched["placement_tags"] = list(default_placement_tags)
    if default_variant_tags:
        enriched["variant_tags"] = list(default_variant_tags)
    return enriched


def refresh_placements_for_tokens(
    snapshot: dict[str, Any],
    grammar: dict[str, Any],
    tokens: frozenset[str],
) -> dict[str, Any]:
    """GRAMMAR-002 — replace placements for footprint tokens only; preserve others."""
    from . import building_grammar

    out = enrich_snapshot(deepcopy(snapshot))
    source_tier = str(out.get("source_tier") or "production")
    style_pack_id = str(out.get("style_pack_id") or grammar.get("style_pack_id") or "")
    pack = load_style_pack(style_pack_id)
    index = load_index_json()
    slot_overrides = dict(grammar.get("slot_overrides") or {})
    default_placement_tags = list(grammar.get("placement_tags") or [])
    default_variant_tags = list(grammar.get("variant_tags") or ["clean"])
    cells = building_grammar.footprint_grid_from_grammar(grammar)
    fp = out.get("footprint") or {}
    fw = int(fp.get("width") or grammar.get("width") or 2)
    fd = int(fp.get("depth") or grammar.get("depth") or 2)

    def key(p: dict[str, Any]) -> tuple[int, int, int, str]:
        return (
            int(p.get("floor") or 0),
            int(p.get("grid_x") or 0),
            int(p.get("grid_y") or 0),
            str(p.get("face") or p.get("token") or ""),
        )

    existing = {key(p): dict(p) for p in out.get("module_placements") or []}
    for cell in cells:
        token = str(cell.get("token") or "")
        if token not in tokens:
            continue
        placement = _placement_for_cell(
            cell,
            pack=pack,
            index=index,
            slot_overrides=slot_overrides,
            grammar=grammar,
            source_tier=source_tier,
            default_placement_tags=default_placement_tags,
            default_variant_tags=default_variant_tags,
            footprint_width=fw,
            footprint_depth=fd,
        )
        if placement:
            existing[key(placement)] = placement

    out["module_placements"] = list(existing.values())
    out["grammar_rule_chain"] = building_grammar.grammar_rule_chain_snapshot(grammar)
    return enrich_snapshot(out)


def generate_assembly_snapshot(
    *,
    style_pack_id: str | None = None,
    width: int | None = None,
    depth: int | None = None,
    floors: int | None = None,
    seed: int = 42,
    source_tier: str = "production",
    reference_tags: list[str] | None = None,
    write: bool = True,
    archetype_id: str | None = None,
    district_style: str | None = None,
    grammar_result: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Footprint fill from W/D/C grid. When ``archetype_id`` + ``district_style`` set, grammar runs first.

    Default ``source_tier=production`` (BQ-ASSEMBLE-PREF-PROD-001) — pass ``lod0`` only for
    explicit greybox/pilot paths.
    """
    from . import building_grammar

    grammar: dict[str, Any] | None = grammar_result
    if grammar is None and archetype_id and district_style:
        grammar = building_grammar.generate(archetype_id, district_style, seed)

    if grammar is not None:
        style_pack_id = style_pack_id or str(grammar["style_pack_id"])
        width = int(grammar["width"]) if width is None else width
        depth = int(grammar["depth"]) if depth is None else depth
        floors = int(grammar["floors"]) if floors is None else floors
        ref = list(reference_tags or [])
        for tag in building_grammar.grammar_reference_tags(grammar):
            if tag not in ref:
                ref.append(tag)
        reference_tags = ref
        rules_version = GRAMMAR_RULES_VERSION
        cells = building_grammar.footprint_grid_from_grammar(grammar)
        slot_overrides = dict(grammar.get("slot_overrides") or {})
        default_placement_tags = list(grammar.get("placement_tags") or [])
        default_variant_tags = list(grammar.get("variant_tags") or ["clean"])
    else:
        if style_pack_id is None or width is None or depth is None:
            raise ValueError(
                "style_pack_id, width, depth required when not using grammar"
            )
        floors = floors if floors is not None else 2
        rules_version = RULES_VERSION
        cells = footprint_grid(width, depth, floors)
        slot_overrides = {}
        default_placement_tags = []
        default_variant_tags = ["clean"]

    pack = load_style_pack(style_pack_id)
    index = load_index_json()
    prefer = "production" if source_tier == "production" else "lod0"
    index_by_id = _index_by_module_id(index, prefer_tier=prefer)

    placements: list[dict[str, Any]] = []
    lod0_fallbacks = 0
    assert width is not None and depth is not None and floors is not None
    fw, fd = int(width), int(depth)

    for cell in cells:
        token = str(cell["token"])
        if token == "Y":
            continue
        slot_key = SLOT_FOR_TOKEN.get(token)
        if not slot_key:
            continue
        slot_key = slot_overrides.get(slot_key, slot_key)
        module_id = pack["slots"].get(slot_key)
        use_token = token
        if not module_id and token == "O":
            use_token = "W"
            slot_key = slot_overrides.get("wall_1u", "wall_1u")
            module_id = pack["slots"].get(slot_key)
        if not module_id:
            continue
        row = _resolve_module_row(
            module_id,
            index,
            style_pack_id=style_pack_id,
            source_tier=source_tier,
        )
        if not row:
            continue
        glb = _module_glb_path(row)
        if not glb.is_file():
            continue
        gx, gy, gf = int(cell["x"]), int(cell["y"]), int(cell["floor"])
        face = str(cell.get("face") or "")
        roof_mode = str(cell.get("roof_mode") or "")
        # 2u roofs on every bay read as pierce/collage — stride by module span.
        if use_token == "R" and roof_mode != "full_footprint":
            span = _module_width_cells(row)
            if span > 1 and (gx % span) != 0:
                continue
        if use_token == "R" and roof_mode == "full_footprint":
            position = _roof_position(fw, fd, gf)
            rotation = [0.0, 0.0, 0.0]
            if fw >= fd:
                import math

                rotation = [0.0, math.pi / 2.0, 0.0]
            scale = full_footprint_roof_scale(fw, fd)
        elif face and face != "R":
            position = _wall_position(gx, gy, gf, face)
            rotation = _rotation_for_cell(use_token, gx, gy, fw, fd, face=face)
            scale = None
        else:
            position = _grid_to_position(gx, gy, gf)
            rotation = _rotation_for_cell(
                use_token, gx, gy, fw, fd, face=face or None
            )
            scale = None
        base = {
            "module_id": str(row["module_id"]),
            "job_id": str(row["job_id"]),
            "slot_key": slot_key,
            "token": use_token,
            "grid_x": gx,
            "grid_y": gy,
            "floor": gf,
            "glb_path": str(glb.relative_to(repo_root())).replace("\\", "/"),
            "position": position,
            "rotation_euler": rotation,
        }
        if scale is not None and (abs(scale[0] - 1.0) > 1e-6 or abs(scale[2] - 1.0) > 1e-6):
            base["scale"] = scale
        if face and face != "R":
            base["face"] = face
        if roof_mode:
            base["roof_mode"] = roof_mode
        if source_tier == "production" and str(row.get("development_tier") or "") != "production":
            base["mesh_tier_fallback"] = "lod0"
            lod0_fallbacks += 1
        enriched = enrich_placement(base, source_tier=source_tier, index_row=row)
        if grammar is not None:
            prof = building_grammar.material_profile_for_slot(grammar, slot_key)
            if prof:
                enriched["material_profile"] = prof
            enriched["weathering"] = str(grammar.get("weathering") or "medium")
        if default_placement_tags and not enriched.get("placement_tags"):
            enriched["placement_tags"] = list(default_placement_tags)
        if default_variant_tags:
            enriched["variant_tags"] = list(default_variant_tags)
        placements.append(enriched)

    if lod0_fallbacks:
        pass  # counted on snapshot below

    if not placements:
        raise ValueError(
            f"No resolvable {source_tier} placements for {style_pack_id} {width}x{depth}"
        )

    assembly_id = _assembly_id(style_pack_id, width, depth, floors, seed)
    wdc = sum(1 for c in cells if c["token"] in ("W", "D", "C", "O"))

    snapshot: dict[str, Any] = {
        "schema_version": 1,
        "assembly_id": assembly_id,
        "style_pack_id": style_pack_id,
        "source_tier": source_tier,
        "procedural_rules_version": rules_version,
        "reference_tags": reference_tags or [],
        "seed": seed,
        "footprint": {
            "width": width,
            "depth": depth,
            "floors": floors,
            "wdc_cell_count": wdc,
        },
        "module_placements": placements,
        "building_look_v2": {
            "grid_unit_m": GRID_UNIT_M,
            "floor_height_m": FLOOR_HEIGHT_M,
            "ridge_row_roof": False,
            "full_footprint_roof": True,
            "outward_yaw": True,
            "corners_as_walls": True,
            "dual_face_corners": True,
            "street_openings_ground": True,
            "wall_edge_offset": True,
        },
    }
    if lod0_fallbacks:
        snapshot["mesh_tier_fallback_count"] = lod0_fallbacks
    if grammar is not None:
        snapshot["archetype_id"] = str(grammar.get("archetype_id") or archetype_id or "")
        snapshot["district_style"] = str(grammar.get("district_style") or district_style or "")
        snapshot["grammar_rule_chain"] = building_grammar.grammar_rule_chain_snapshot(grammar)
    snapshot = enrich_snapshot(snapshot)
    validate_assembly_snapshot(snapshot)

    if write:
        out_dir = repo_root() / SNAPSHOT_STAGING
        out_dir.mkdir(parents=True, exist_ok=True)
        out_path = out_dir / f"{assembly_id}.json"
        out_path.write_text(json.dumps(snapshot, indent=2) + "\n", encoding="utf-8")
        snapshot["written_path"] = str(out_path.relative_to(repo_root())).replace("\\", "/")

    return snapshot


def default_snapshot_path(assembly_id: str) -> Path:
    return repo_root() / SNAPSHOT_STAGING / f"{assembly_id}.json"


def load_assembly_snapshot(path: str | Path, *, enrich: bool = True) -> dict[str, Any]:
    data = load_json_file(Path(path))
    if enrich:
        data = enrich_snapshot(data)
    validate_assembly_snapshot(data)
    return data


def example_snapshot_path() -> Path:
    return schemas_dir() / "examples" / "assembly_snapshot_rowhouse_victorian_v1.json"
