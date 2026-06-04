"""TILE-FIX-007 — BuildingDefinition drives variant × facing × frame bake matrix."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.variant_matrix_expand import VARIANT_BAKE

MINIMUM_G4_CELLS = 24
DEFAULT_MINIMUM_G4_FACINGS = tuple(range(8))
DEFAULT_MINIMUM_G4_VARIANTS = (
    "clean_day",
    "clean_night_on",
    "damaged_night_on",
)

PRODUCTION_SHELL_MODULE_IDS = frozenset({"wall_steel_1u", "roof_sawtooth", "wall_brick_1u", "roof_flat"})


@dataclass
class BakeCell:
    variant_key: str
    facing: int
    frame: int
    variant_params: dict[str, Any]


@dataclass
class BuildingDefinition:
    building_id: str
    modules: list[dict[str, Any]]
    variants: list[str]
    render_contract: dict[str, Any]
    assembly_snapshot: str = ""
    assembly_blend: str = ""
    fire_animation_frames: int = 0
    material_profiles: list[str] | None = None

    @property
    def facings(self) -> int:
        return int(self.render_contract.get("facings") or 8)

    @property
    def tile_px(self) -> int:
        return int(self.render_contract.get("tile_px") or 128)


def load_building_definition(path: str | Path) -> BuildingDefinition:
    p = Path(path)
    data = json.loads(p.read_text(encoding="utf-8"))
    if int(data.get("schema_version") or 0) != 1:
        raise ValueError(f"building_definition schema_version must be 1: {p}")
    return BuildingDefinition(
        building_id=str(data["building_id"]),
        modules=list(data.get("modules") or []),
        variants=[str(v) for v in data.get("variants") or []],
        render_contract=dict(data.get("render_contract") or {}),
        assembly_snapshot=str(data.get("assembly_snapshot") or ""),
        assembly_blend=str(data.get("assembly_blend") or ""),
        fire_animation_frames=int(data.get("fire_animation_frames") or 0),
        material_profiles=list(data.get("material_profiles") or []),
    )


def expand_bake_matrix(defn: BuildingDefinition) -> list[BakeCell]:
    """State × facing × animation frame grid for TILE-FIX-008 compile loop."""
    facings = defn.facings
    cells: list[BakeCell] = []
    for variant_key in defn.variants:
        params = dict(VARIANT_BAKE.get(variant_key) or {"state": "clean", "lighting": "day"})
        if variant_key.startswith("burning_"):
            frames = defn.fire_animation_frames or 8
            for frame in range(frames):
                for facing in range(facings):
                    p = dict(params)
                    p["fire_frame"] = frame
                    cells.append(BakeCell(variant_key, facing, frame, p))
        else:
            for facing in range(facings):
                cells.append(BakeCell(variant_key, facing, 0, params))
    return cells


def expand_bake_matrix_minimum(
    defn: BuildingDefinition,
    *,
    variant_keys: tuple[str, ...] | None = None,
    facings: tuple[int, ...] | None = None,
) -> list[BakeCell]:
    """TILE-FIX-09 pilot — 3 states × 8 facings = 24 cells (frame 0 only)."""
    want_variants = set(variant_keys or DEFAULT_MINIMUM_G4_VARIANTS)
    want_facings = set(facings or DEFAULT_MINIMUM_G4_FACINGS)
    return [
        c
        for c in expand_bake_matrix(defn)
        if c.variant_key in want_variants and c.facing in want_facings and c.frame == 0
    ]


def production_shell_modules_ready(defn: BuildingDefinition) -> tuple[bool, list[str]]:
    """TILE-FIX-010 — wall/roof (and brick rowhouse shell) must use production job_ids."""
    blockers: list[str] = []
    for mod in defn.modules:
        mid = str(mod.get("module_id") or "")
        if mid not in PRODUCTION_SHELL_MODULE_IDS:
            continue
        job = str(mod.get("job_id") or "")
        if "lod0" in job or "production" not in job:
            blockers.append(f"{mid} still on lod0 job_id {job!r}")
    return (not blockers, blockers)


def bake_matrix_summary(defn: BuildingDefinition) -> dict[str, Any]:
    cells = expand_bake_matrix(defn)
    return {
        "building_id": defn.building_id,
        "facings": defn.facings,
        "variant_count": len(defn.variants),
        "cell_count": len(cells),
        "module_slots": sum(int(m.get("count") or 1) for m in defn.modules),
    }


def default_rowhouse_production_definition() -> BuildingDefinition:
    root = repo_root()
    return BuildingDefinition(
        building_id="rowhouse_victorian",
        modules=[
            {"module_id": "corner_L", "job_id": "corner_L_production_run001", "count": 4},
            {"module_id": "wall_brick_1u", "job_id": "wall_brick_1u_production_run001", "count": 8},
            {"module_id": "door_standard", "job_id": "door_standard_production_run001", "count": 1},
            {"module_id": "roof_flat", "job_id": "roof_flat_production_run001", "count": 4},
            {"module_id": "window_standard", "job_id": "window_standard_production_run001", "count": 4},
        ],
        variants=[
            "clean_day",
            "clean_night_off",
            "clean_night_on",
            "damaged_day",
            "damaged_night_on",
            "under_construction_01",
            "under_construction_02",
            "under_construction_03",
            "abandoned",
            "ruined",
            *[f"burning_{i:02d}" for i in range(8)],
        ],
        render_contract={"facings": 8, "tile_px": 128, "quarter_turn_fallback": True},
        assembly_snapshot="tools/mcp/schemas/examples/assembly_snapshot_rowhouse_victorian_production_v1.json",
        assembly_blend="assets/staging/assemblies/victorian_4x3_s42_a7cb.blend",
        fire_animation_frames=8,
        material_profiles=["brick_red_01", "roof_tile_01", "wood_plank_01"],
    )


def default_warehouse_production_definition() -> BuildingDefinition:
    root = repo_root()
    return BuildingDefinition(
        building_id="warehouse_industrial",
        modules=[
            {"module_id": "corner_L", "job_id": "corner_L_production_run001", "count": 4},
            {"module_id": "wall_steel_1u", "job_id": "wall_steel_1u_production_run001", "count": 7},
            {"module_id": "door_shop", "job_id": "door_shop_lod0_run001", "count": 1},
            {"module_id": "roof_sawtooth", "job_id": "roof_sawtooth_production_run001", "count": 4},
        ],
        variants=[
            "clean_day",
            "clean_night_off",
            "clean_night_on",
            "damaged_day",
            "damaged_night_on",
            "under_construction_01",
            "under_construction_02",
            "abandoned",
            *[f"burning_{i:02d}" for i in range(8)],
        ],
        render_contract={"facings": 8, "tile_px": 128, "quarter_turn_fallback": True},
        assembly_snapshot=(
            "tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json"
        ),
        assembly_blend="assets/staging/assemblies/industrial_west_4x2_s43_a879.blend",
        fire_animation_frames=8,
        material_profiles=["steel_panel_01", "roof_metal_01"],
    )


def write_example_building_definition(path: str | Path | None = None) -> Path:
    out = path or (
        repo_root() / "tools/mcp/schemas/examples/building_definition_rowhouse_victorian_production_v1.json"
    )
    defn = default_rowhouse_production_definition()
    payload = {
        "schema_version": 1,
        "building_id": defn.building_id,
        "style_pack_id": "style_victorian",
        "assembly_snapshot": defn.assembly_snapshot,
        "assembly_blend": defn.assembly_blend,
        "modules": defn.modules,
        "variants": defn.variants,
        "fire_animation_frames": defn.fire_animation_frames,
        "render_contract": defn.render_contract,
        "material_profiles": defn.material_profiles,
    }
    out = Path(out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return out
