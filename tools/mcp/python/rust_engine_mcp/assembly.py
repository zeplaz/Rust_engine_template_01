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
    prefer_tier: str = "lod0",
) -> dict[str, dict[str, Any]]:
    """Prefer lod0_run* or production_run* rows when duplicate module_id keys exist."""
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
        for row in pool:
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


FootprintToken = str  # W | D | C | R


def _is_perimeter(x: int, y: int, width: int, depth: int) -> bool:
    return x == 0 or y == 0 or x + 1 == width or y + 1 == depth


def _is_corner(x: int, y: int, width: int, depth: int) -> bool:
    return (x == 0 or x + 1 == width) and (y == 0 or y + 1 == depth)


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
                if _is_corner(x, y, width, depth):
                    token: FootprintToken = "C"
                elif floor == 0 and y == 0 and x == door_x:
                    token = "D"
                else:
                    token = "W"
                cells.append({"x": x, "y": y, "floor": floor, "token": token})

    roof_floor = floors
    for y in range(depth):
        for x in range(width):
            if _is_perimeter(x, y, width, depth):
                cells.append({"x": x, "y": y, "floor": roof_floor, "token": "R"})
    return cells


SLOT_FOR_TOKEN = {
    "W": "wall_1u",
    "D": "door_default",
    "C": "corner_outer",
    "R": "roof_default",
}

# ARCH-003 — default placement tags by footprint token (APS Assembly Editor checkboxes).
TOKEN_PLACEMENT_TAGS: dict[str, list[str]] = {
    "W": ["exterior", "wall"],
    "D": ["exterior", "door"],
    "C": ["exterior", "corner"],
    "R": ["exterior", "roof"],
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
    source_tier: str = "lod0",
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
    tier = str(snapshot.get("source_tier") or "lod0")
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
        placements.append(enrich_placement(row, source_tier=str(out.get("source_tier") or "lod0")))
    if not found:
        raise KeyError(f"placement node_id not found: {node_id}")
    out["module_placements"] = placements
    validate_assembly_snapshot(out)
    return out


def _grid_to_position(x: int, y: int, floor: int) -> list[float]:
    """1m grid snap — south edge y=0, floor height 3m."""
    return [float(x), float(floor * 3.0), float(y)]


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
    base = {
        "module_id": str(row["module_id"]),
        "job_id": str(row["job_id"]),
        "slot_key": slot_key,
        "token": token,
        "grid_x": gx,
        "grid_y": gy,
        "floor": gf,
        "glb_path": str(glb.relative_to(repo_root())).replace("\\", "/"),
        "position": _grid_to_position(gx, gy, gf),
        "rotation_euler": [0.0, 0.0, 0.0],
    }
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

    def key(p: dict[str, Any]) -> tuple[int, int, int]:
        return (int(p.get("floor") or 0), int(p.get("grid_x") or 0), int(p.get("grid_y") or 0))

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
    source_tier: str = "lod0",
    reference_tags: list[str] | None = None,
    write: bool = True,
    archetype_id: str | None = None,
    district_style: str | None = None,
    grammar_result: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Footprint fill from W/D/C grid. When ``archetype_id`` + ``district_style`` set, grammar runs first."""
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

    for cell in cells:
        token = str(cell["token"])
        if token == "Y":
            continue
        slot_key = SLOT_FOR_TOKEN.get(token)
        if not slot_key:
            continue
        slot_key = slot_overrides.get(slot_key, slot_key)
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
        base = {
            "module_id": str(row["module_id"]),
            "job_id": str(row["job_id"]),
            "slot_key": slot_key,
            "token": token,
            "grid_x": gx,
            "grid_y": gy,
            "floor": gf,
            "glb_path": str(glb.relative_to(repo_root())).replace("\\", "/"),
            "position": _grid_to_position(gx, gy, gf),
            "rotation_euler": [0.0, 0.0, 0.0],
        }
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
    wdc = sum(1 for c in cells if c["token"] in ("W", "D", "C"))

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
