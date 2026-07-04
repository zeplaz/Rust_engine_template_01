"""G5 module library — register promoted modules and search the index."""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

from .paths import repo_root
from .validators.tier import infer_development_tier, infer_pbr_status, infer_stylepack_visible

ARCHETYPE_TO_CATEGORY: dict[str, str] = {
    "module_wall": "wall",
    "module_roof": "roof",
    "module_door": "door",
    "module_window": "window",
    "module_prop": "corner_prop",
}

KIT_GREYBOX_001_JOB_IDS = frozenset(
    {
        "wall_concrete_2u_run001",
        "roof_flat_2u_run001",
        "door_industrial_1u_run001",
        "wall_wood_1u_run001",
        "wall_steel_1u_run001",
        "wall_glass_1u_run001",
        "roof_pitched_2u_run001",
        "door_warehouse_2u_run001",
        "door_shop_1u_run001",
        "roof_industrial_shed_2u_run001",
    }
)

KIT_GREYBOX_002_JOB_IDS = frozenset(
    {
        "window_industrial_1u_run001",
        "window_shop_1u_run001",
        "window_warehouse_2u_run001",
        "window_glass_curtain_2u_run001",
        "corner_concrete_outer_run001",
        "corner_brick_outer_run001",
        "corner_steel_inner_run001",
        "prop_vent_roof_1u_run001",
        "prop_ac_unit_1u_run001",
        "corner_wood_porch_run001",
    }
)

KIT_GREYBOX_003_JOB_IDS = frozenset(
    {
        "wall_brick_2u_run001",
        "wall_concrete_1u_run001",
        "wall_wood_2u_run001",
        "wall_industrial_panel_2u_run001",
        "window_single_1u_run001",
        "window_double_1u_run001",
        "door_residential_1u_run001",
        "door_garage_2u_run001",
        "roof_sawtooth_2u_run001",
        "prop_chimney_1u_run001",
    }
)

KIT_LOD0_001_JOB_IDS = frozenset(
    {
        "wall_brick_1u_lod0_run001",
        "wall_concrete_1u_lod0_run001",
        "roof_pitched_gable_lod0_run001",
        "door_residential_lod0_run001",
        "win_single_1u_lod0_run001",
    }
)

KIT_LOD0_002_JOB_IDS = frozenset(
    {
        "wall_wood_1u_lod0_run001",
        "wall_steel_1u_lod0_run001",
        "roof_flat_lod0_run001",
        "door_shop_lod0_run001",
        "win_double_1u_lod0_run001",
    }
)

KIT_LOD0_003_JOB_IDS = frozenset(
    {
        "wall_concrete_2u_lod0_run001",
        "roof_sawtooth_lod0_run001",
        "door_warehouse_lod0_run001",
        "win_industrial_3u_lod0_run001",
        "prop_vent_lod0_run001",
    }
)

KIT_INDUSTRIAL_WEST_PRODUCTION_001_JOB_IDS = frozenset(
    {
        "corner_L_industrial_west_production_run001",
        "door_warehouse_production_run001",
        "win_industrial_3u_production_run001",
        "wall_concrete_2u_production_run001",
        "prop_vent_production_run001",
        "roof_shed_production_run001",
        "stack_chimney_1u_production_run001",
        "platform_dock_2u_production_run001",
    }
)

KIT_PRODUCTION_002_JOB_IDS = frozenset(
    {
        "roof_industrial_shed_2u_production_run001",
    }
)

KIT_UTILITY_POWER_PRODUCTION_001_JOB_IDS = frozenset(
    {
        "bus_bay_simplified_production_run001",
        "breaker_block_production_run001",
        "control_shack_1u_production_run001",
        "fence_chainlink_1u_production_run001",
        "gravel_pad_1u_production_run001",
        "warning_sign_1u_production_run001",
        "kit_substation_yard_production_run001",
        "prop_transformer_production_run001",
    }
)

KIT_NUCLEAR_PWR_PRODUCTION_001_JOB_IDS = frozenset(
    {
        "containment_dome_pwr_production_run001",
        "turbine_hall_1u_production_run001",
        "cooling_tower_1u_production_run001",
        "diesel_gen_pad_2x2_production_run001",
        "switchyard_edge_1u_production_run001",
        "warning_sign_nuclear_1u_production_run001",
        "kit_nuclear_pwr_production_run001",
        "fence_chainlink_1u_production_run001",
    }
)

# Greybox smoke module_id → canonical kit id when lod0 row supersedes harness art.
SMOKE_TO_CANONICAL_MODULE_ID: dict[str, str] = {
    "door_residential_1u": "door_residential",
    "window_single_1u": "win_single_1u",
    "door_shop_1u": "door_shop",
    "window_double_1u": "win_double_1u",
    "door_warehouse_2u": "door_warehouse",
    "roof_sawtooth_2u": "roof_sawtooth",
    "window_industrial_1u": "win_industrial_3u",
    "prop_vent_roof_1u": "prop_vent",
}


def _modules_root() -> Path:
    return repo_root() / "assets" / "models" / "modules"


def index_ron_path() -> Path:
    return repo_root() / "assets" / "configs" / "buildings" / "_module_index.ron"


def index_json_path() -> Path:
    return repo_root() / "assets" / "configs" / "buildings" / "_module_index.json"


def _ron_str(value: str) -> str:
    escaped = value.replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'


def _infer_batch_id(job_id: str, module_id: str) -> str:
    manifest_path = _modules_root() / job_id / "manifest.json"
    if manifest_path.is_file():
        try:
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            batch = manifest.get("batch_id")
            if batch:
                return str(batch)
        except (json.JSONDecodeError, OSError):
            pass
    if job_id in KIT_GREYBOX_001_JOB_IDS:
        return "kit_greybox_001"
    if job_id in KIT_GREYBOX_002_JOB_IDS:
        return "kit_greybox_002"
    if job_id in KIT_GREYBOX_003_JOB_IDS:
        return "kit_greybox_003"
    if job_id in KIT_LOD0_001_JOB_IDS:
        return "kit_lod0_001"
    if job_id in KIT_LOD0_002_JOB_IDS:
        return "kit_lod0_002"
    if job_id in KIT_LOD0_003_JOB_IDS:
        return "kit_lod0_003"
    if job_id in KIT_INDUSTRIAL_WEST_PRODUCTION_001_JOB_IDS:
        return "kit_industrial_west_production_001"
    if job_id in KIT_PRODUCTION_002_JOB_IDS:
        return "kit_production_002"
    if job_id in KIT_UTILITY_POWER_PRODUCTION_001_JOB_IDS:
        return "kit_utility_power_production_001"
    if job_id in KIT_NUCLEAR_PWR_PRODUCTION_001_JOB_IDS:
        return "kit_nuclear_pwr_production_001"
    return ""


def _category_for(archetype: str, module_id: str) -> str:
    if archetype == "module_prop":
        if module_id.startswith("corner_"):
            return "corner_prop"
        if module_id.startswith("prop_"):
            return "prop"
        return "corner_prop"
    return ARCHETYPE_TO_CATEGORY.get(archetype, "wall")


def _style_tags_from_spec(spec: dict[str, Any]) -> list[str]:
    tags: list[str] = []
    mat = str(spec.get("material_profile") or "")
    if mat:
        parts = [p for p in re.split(r"[_\-]+", mat) if p and p not in ("01", "02")]
        tags.extend(parts[:3])
    asset_id = str(spec.get("asset_id") or "")
    for token in ("brick", "concrete", "wood", "steel", "glass", "industrial", "rural", "modern"):
        if token in asset_id and token not in tags:
            tags.append(token)
    style_pack = str(spec.get("style_pack") or "")
    if style_pack.startswith("style_"):
        pack_tag = style_pack.removeprefix("style_")
        if pack_tag and pack_tag not in tags:
            tags.append(pack_tag)
    return tags[:6]


def entry_from_promoted(job_dir: Path) -> dict[str, Any] | None:
    glb = job_dir / "model.glb"
    if not glb.is_file():
        return None

    sidecars = list(job_dir.glob("*.module.json"))
    if not sidecars:
        return None
    spec = json.loads(sidecars[0].read_text(encoding="utf-8"))
    job_id = job_dir.name
    module = spec.get("module") or {}
    grid = module.get("grid_units") or [1, 1]
    if len(grid) < 2:
        grid = [1, 1]

    module_id = str(spec["asset_id"])
    archetype = str(spec.get("archetype", "module_wall"))
    glb_rel = str(glb.relative_to(repo_root())).replace("\\", "/")
    batch_id = _infer_batch_id(job_id, module_id)
    if spec.get("batch_id"):
        batch_id = str(spec["batch_id"])
    tier = infer_development_tier(spec, batch_id)
    pbr = infer_pbr_status(spec, tier)

    return {
        "module_id": module_id,
        "job_id": job_id,
        "category": _category_for(archetype, module_id),
        "glb": glb_rel,
        "glb_path": glb_rel,
        "archetype": archetype,
        "style_pack": str(spec.get("style_pack", "")),
        "grid_units": [int(grid[0]), int(grid[1])],
        "snap": str(module.get("snap", "floor_edge")),
        "material_profile": str(spec.get("material_profile", "")),
        "style_tags": _style_tags_from_spec(spec),
        "batch_id": batch_id,
        "development_tier": tier,
        "pbr_status": pbr,
        "stylepack_visible": infer_stylepack_visible(tier),
    }


def _apply_replaced_by(entries: list[dict[str, Any]]) -> None:
    """Set replaced_by on smoke rows superseded by lod0/production canonical modules."""
    lod0_by_module: dict[str, str] = {}
    for row in entries:
        tier = str(row.get("development_tier") or "")
        if tier in ("lod0", "production"):
            lod0_by_module[str(row["module_id"])] = str(row["module_id"])

    for row in entries:
        if str(row.get("development_tier") or "") != "smoke":
            continue
        smoke_id = str(row["module_id"])
        canonical = SMOKE_TO_CANONICAL_MODULE_ID.get(smoke_id, smoke_id)
        if canonical in lod0_by_module:
            row["replaced_by"] = canonical


def collect_entries() -> list[dict[str, Any]]:
    root = _modules_root()
    if not root.is_dir():
        return []
    entries: list[dict[str, Any]] = []
    for job_dir in sorted(root.iterdir()):
        if not job_dir.is_dir():
            continue
        row = entry_from_promoted(job_dir)
        if row is not None:
            entries.append(row)
    _apply_replaced_by(entries)
    entries.sort(key=lambda e: e["module_id"])
    return entries


def format_index_ron(entries: list[dict[str, Any]]) -> str:
    lines = [
        "// Auto-generated by rust_engine_mcp.library — do not hand-edit entries.",
        "(",
        "    schema_version: 1,",
        "    entries: [",
    ]
    for e in entries:
        gu0, gu1 = e["grid_units"]
        tags = e.get("style_tags") or []
        tag_ron = ", ".join(_ron_str(t) for t in tags)
        batch = e.get("batch_id") or ""
        tier = e.get("development_tier") or "smoke"
        pbr = e.get("pbr_status") or "none"
        stylepack_vis = "true" if e.get("stylepack_visible", tier != "smoke") else "false"
        replaced = e.get("replaced_by")
        from rust_engine_mcp.city_palette_g2 import palette_fields_for_entry

        palette = palette_fields_for_entry(e)
        e = {**e, **palette}
        lines.extend(
            [
                "        (",
                f"            module_id: {_ron_str(e['module_id'])},",
                f"            job_id: {_ron_str(e['job_id'])},",
                f"            category: {_ron_str(e['category'])},",
                f"            glb: {_ron_str(e['glb'])},",
                f"            grid_units: ({gu0}, {gu1}),",
                f"            style_tags: [{tag_ron}],",
                f"            batch_id: {_ron_str(batch)},",
                f"            development_tier: {_ron_str(tier)},",
                f"            pbr_status: {_ron_str(pbr)},",
                f"            stylepack_visible: {stylepack_vis},",
                f"            archetype: {_ron_str(e['archetype'])},",
                f"            style_pack: {_ron_str(e['style_pack'])},",
                f"            snap: {_ron_str(e['snap'])},",
                f"            material_profile: {_ron_str(e['material_profile'])},",
            ]
        )
        if palette.get("palette_family"):
            lines.append(f"            palette_family: {_ron_str(str(palette['palette_family']))},")
            lines.append(
                f"            palette_variation_count: {int(palette.get('palette_variation_count') or 1)},"
            )
            lines.append(
                f"            default_variation_id: {_ron_str(str(palette.get('default_variation_id') or ''))},"
            )
        if replaced:
            lines.append(f"            replaced_by: Some({_ron_str(str(replaced))}),")
        lines.append("        ),")
    lines.extend(["    ],", ")", ""])
    return "\n".join(lines)


def write_module_index() -> dict[str, Any]:
    entries = collect_entries()
    ron_out = index_ron_path()
    json_out = index_json_path()
    ron_out.parent.mkdir(parents=True, exist_ok=True)
    ron_out.write_text(format_index_ron(entries), encoding="utf-8")
    json_payload = {
        "schema_version": 1,
        "entries": entries,
    }
    json_out.write_text(json.dumps(json_payload, indent=2) + "\n", encoding="utf-8")
    return {
        "written": str(ron_out),
        "json_mirror": str(json_out),
        "entry_count": len(entries),
        "module_ids": [e["module_id"] for e in entries],
    }


def load_index_json() -> list[dict[str, Any]]:
    path = index_json_path()
    if not path.is_file():
        write_module_index()
    data = json.loads(path.read_text(encoding="utf-8"))
    return list(data.get("entries") or [])


def register_module(job_id: str) -> dict[str, Any]:
    job_dir = _modules_root() / job_id
    if not job_dir.is_dir():
        raise FileNotFoundError(f"Promoted module folder not found: {job_dir}")
    row = entry_from_promoted(job_dir)
    if row is None:
        raise ValueError(f"Cannot register {job_id}: missing model.glb or *.module.json sidecar")
    index = write_module_index()
    index["registered"] = row["module_id"]
    index["job_id"] = job_id
    index["entry"] = row
    return index


def search_modules(
    *,
    tags: list[str] | None = None,
    archetype: str | None = None,
    style_pack: str | None = None,
    category: str | None = None,
    batch_id: str | None = None,
) -> list[dict[str, Any]]:
    entries = load_index_json()
    tag_set = {t.lower() for t in (tags or [])}

    def matches(row: dict[str, Any]) -> bool:
        if archetype and row.get("archetype") != archetype:
            return False
        if style_pack and row.get("style_pack") != style_pack:
            return False
        if category and row.get("category") != category:
            return False
        if batch_id and row.get("batch_id") != batch_id:
            return False
        if tag_set:
            row_tags = {str(t).lower() for t in (row.get("style_tags") or [])}
            if not tag_set.issubset(row_tags):
                return False
        return True

    return [e for e in entries if matches(e)]
