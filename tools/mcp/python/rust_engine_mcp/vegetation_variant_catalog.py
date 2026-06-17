"""DMCP-E3 — vegetation variant catalog authoring + schema validation."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

SCHEMA_REL = "tools/mcp/schemas/vegetation_variant_catalog_v1.schema.json"
CATALOG_RON_REL = "assets/configs/landscape/_vegetation_variant_catalog.ron"
WITNESS_REL = "debug_runs/art_pipeline/dmcp_e3_vegetation_catalog_live.json"

EXPANDED_ATLAS_ID = "landscape_lg5_expanded_v1"
PILOT_ATLAS_ID = "landscape_lg5_pilot_v1"
SEED = 550005

# 16-cell bake authority (tile_batch_landscape_expanded_v1.json)
ATLAS_BAKE_V1_KEYS: tuple[str, ...] = (
    "topology_patch",
    "topology_patch_scar",
    "topology_patch_burn_00",
    "topology_patch_burn_04",
    "topology_patch_regrowth_grass",
    "topology_patch_regrowth_shrub",
    "topology_corridor",
    "topology_corridor_scar",
    "topology_corridor_burn_00",
    "topology_corridor_burn_04",
    "topology_ring",
    "topology_ring_burn_00",
    "topology_cluster",
    "topology_cluster_regrowth_grass",
    "topology_fringe",
    "topology_fringe_regrowth_grass",
)

# Sparse ship rows — keyframe reqs / matrix §2 beyond 4×4 v1 bake (catalog authority pre-bake)
ATLAS_SPARSE_V1_KEYS: tuple[str, ...] = (
    "topology_patch_burn_07",
    "topology_patch_regrowth_canopy",
    "topology_corridor_burn_07",
    "topology_corridor_regrowth_grass",
    "topology_cluster_scar",
    "topology_cluster_regrowth_shrub",
)

TILE_BATCH_REL = "assets/staging/specs/tile_batch_landscape_expanded_v1.json"
KEYFRAME_REQS_REL = "src/dev/design_landscape_keyframe_burn_reqs_v1.md"


def _glyphs(planning: str, extract: str) -> dict[str, str]:
    return {"planning": planning, "extract": extract}


def _atlas(
    uv_slot: str,
    *,
    atlas_id: str = EXPANDED_ATLAS_ID,
    topology_kind: str | None = None,
    scale_band: str = "M",
) -> dict[str, Any]:
    row: dict[str, Any] = {
        "atlas_domain": "landscape",
        "atlas_id": atlas_id,
        "uv_slot": uv_slot,
        "scale_band": scale_band,
    }
    if topology_kind in ("Patch", "Corridor", "Ring"):
        row["topology_kind"] = topology_kind
    return row


def _expanded_topology_entries() -> list[dict[str, Any]]:
    """16 atlas cells — byte-aligned with tile_batch_landscape_expanded_v1.json."""
    rows: list[tuple[str, dict[str, Any], dict[str, str], list[str], str | None]] = [
        ("topology_patch", {"kind": "topology_kind", "topology_kind": "Patch"}, _glyphs("#", "#"), ["sim_day", "sim_topology_patch"], "Patch"),
        ("topology_patch_scar", {"kind": "succession_stage", "succession_stage": "BurnScar"}, _glyphs("x", "x"), ["sim_damaged", "sim_topology_patch"], "Patch"),
        ("topology_patch_burn_00", {"kind": "active_burn_frame", "frame_index": 0, "active_burn_heat_gt": 0.01}, _glyphs("⊗", "F"), ["sim_fire", "sim_topology_patch"], "Patch"),
        ("topology_patch_burn_04", {"kind": "active_burn_frame", "frame_index": 4, "active_burn_heat_gt": 0.01}, _glyphs("⊗", "F"), ["sim_fire", "sim_topology_patch"], "Patch"),
        ("topology_patch_regrowth_grass", {"kind": "succession_stage", "succession_stage": "Grass"}, _glyphs(",", ","), ["sim_regrowth", "sim_topology_patch"], "Patch"),
        ("topology_patch_regrowth_shrub", {"kind": "succession_stage", "succession_stage": "Shrub"}, _glyphs(".", "."), ["sim_regrowth", "sim_topology_patch"], "Patch"),
        ("topology_corridor", {"kind": "topology_kind", "topology_kind": "Corridor"}, _glyphs("=", "="), ["sim_day", "sim_topology_corridor"], "Corridor"),
        ("topology_corridor_scar", {"kind": "succession_stage", "succession_stage": "BurnScar"}, _glyphs("x", "x"), ["sim_damaged", "sim_topology_corridor"], "Corridor"),
        ("topology_corridor_burn_00", {"kind": "active_burn_frame", "frame_index": 0, "active_burn_heat_gt": 0.01}, _glyphs("⊗", "F"), ["sim_fire", "sim_topology_corridor"], "Corridor"),
        ("topology_corridor_burn_04", {"kind": "active_burn_frame", "frame_index": 4, "active_burn_heat_gt": 0.01}, _glyphs("⊗", "F"), ["sim_fire", "sim_topology_corridor"], "Corridor"),
        ("topology_ring", {"kind": "topology_kind", "topology_kind": "Ring"}, _glyphs("○", "O"), ["sim_day", "sim_topology_ring"], "Ring"),
        ("topology_ring_burn_00", {"kind": "active_burn_frame", "frame_index": 0, "active_burn_heat_gt": 0.01}, _glyphs("⊗", "F"), ["sim_fire", "sim_topology_ring"], "Ring"),
        ("topology_cluster", {"kind": "default"}, _glyphs("*", "*"), ["sim_day", "sim_topology_cluster"], None),
        ("topology_cluster_regrowth_grass", {"kind": "succession_stage", "succession_stage": "Grass"}, _glyphs(",", ","), ["sim_regrowth", "sim_topology_cluster"], None),
        ("topology_fringe", {"kind": "default"}, _glyphs(".", "."), ["sim_day", "sim_topology_fringe"], None),
        ("topology_fringe_regrowth_grass", {"kind": "succession_stage", "succession_stage": "Grass"}, _glyphs(",", ","), ["sim_regrowth", "sim_topology_fringe"], None),
    ]
    return _topology_rows_to_entries(rows)


def _sparse_topology_entries() -> list[dict[str, Any]]:
    """Sparse ship atlas rows — DMCP-E3 v1.1 extension per keyframe reqs charter."""
    rows: list[tuple[str, dict[str, Any], dict[str, str], list[str], str | None]] = [
        ("topology_patch_burn_07", {"kind": "active_burn_frame", "frame_index": 7, "active_burn_heat_gt": 0.01}, _glyphs("⊗", "F"), ["sim_fire", "sim_topology_patch"], "Patch"),
        ("topology_patch_regrowth_canopy", {"kind": "succession_stage", "succession_stage": "Canopy"}, _glyphs("*", "#"), ["sim_regrowth", "sim_topology_patch", "sim_canopy"], "Patch"),
        ("topology_corridor_burn_07", {"kind": "active_burn_frame", "frame_index": 7, "active_burn_heat_gt": 0.01}, _glyphs("⊗", "F"), ["sim_fire", "sim_topology_corridor"], "Corridor"),
        ("topology_corridor_regrowth_grass", {"kind": "succession_stage", "succession_stage": "Grass"}, _glyphs(",", ","), ["sim_regrowth", "sim_topology_corridor"], "Corridor"),
        ("topology_cluster_scar", {"kind": "succession_stage", "succession_stage": "BurnScar"}, _glyphs("x", "x"), ["sim_damaged", "sim_topology_cluster"], None),
        ("topology_cluster_regrowth_shrub", {"kind": "succession_stage", "succession_stage": "Shrub"}, _glyphs(".", "."), ["sim_regrowth", "sim_topology_cluster"], None),
    ]
    out = _topology_rows_to_entries(rows)
    for entry in out:
        entry["notes"] = "sparse_v1 — catalog ship row; bake when APS-E4 expands sheet or sparse keyframe lane"
    return out


def _topology_rows_to_entries(
    rows: list[tuple[str, dict[str, Any], dict[str, str], list[str], str | None]],
) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for key, resolver, glyphs, tags, topo in rows:
        out.append(
            {
                "variant_key": key,
                "resolver": resolver,
                "sim_tags": tags,
                "atlas": _atlas(key, topology_kind=topo),
                "glyphs": glyphs,
            }
        )
    return out


def _veg_resolver_entries() -> list[dict[str, Any]]:
    entries: list[dict[str, Any]] = [
        {
            "variant_key": "veg_clean_day",
            "resolver": {"kind": "default"},
            "sim_tags": ["sim_day", "sim_operational"],
            "notes": "Fallback when no burn overlay and non-OldGrowth succession",
        },
        {
            "variant_key": "veg_old_growth",
            "resolver": {"kind": "succession_stage", "succession_stage": "OldGrowth"},
            "sim_tags": ["sim_old_growth", "sim_day"],
        },
        {
            "variant_key": "veg_damaged",
            "resolver": {"kind": "regrowth_macro", "regrowth_macro_phase": "Scar"},
            "sim_tags": ["sim_damaged", "sim_day"],
            "notes": "Also covers RegrowthMacroPhase::Closing and succession BurnScar tint path",
        },
        {
            "variant_key": "veg_regrowth_nuclei",
            "resolver": {"kind": "regrowth_macro", "regrowth_macro_phase": "Nuclei"},
            "sim_tags": ["sim_regrowth", "sim_day"],
        },
        {
            "variant_key": "veg_regrowth_front",
            "resolver": {"kind": "regrowth_macro", "regrowth_macro_phase": "Front"},
            "sim_tags": ["sim_regrowth", "sim_day"],
        },
    ]
    for i in range(8):
        entries.append(
            {
                "variant_key": f"veg_burn_{i:02}",
                "resolver": {
                    "kind": "active_burn_frame",
                    "frame_index": i,
                    "active_burn_heat_gt": 0.01,
                },
                "sim_tags": ["sim_fire", "sim_burn_frame"],
            }
        )
    return entries


def build_catalog_body(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    entries = _veg_resolver_entries() + _expanded_topology_entries() + _sparse_topology_entries()
    return {
        "schema_version": 1,
        "catalog_id": "vegetation_variant_catalog_v1",
        "seed": SEED,
        "preset_scope": ["fire_recovery_v0", "old_growth_core_v0"],
        "_meta": {
            "gate": "DMCP-E3-VARIANT-KEY-SET-001",
            "ship_catalog_version": 2,
            "charter": "src/dev/plan_veg_variant_key_naming_v1.md",
            "matrix": "src/dev/design_landscape_lg5_expansion_matrix_v1.md",
            "keyframe_reqs": KEYFRAME_REQS_REL,
            "tile_batch": TILE_BATCH_REL,
            "pairs_with": "tools/mcp/schemas/examples/vegetation_variant_catalog_pilot_v1.json",
            "ship_tiers": {
                "engine_resolver_veg": 13,
                "atlas_bake_v1": len(ATLAS_BAKE_V1_KEYS),
                "atlas_sparse_v1": len(ATLAS_SPARSE_V1_KEYS),
            },
            "review": "DMCP-E3 ship catalog v1.1 — sparse atlas rows from keyframe reqs; veg byte parity unchanged",
        },
        "axes": {
            "burn_frame_count": 8,
            "succession_stages": ["Grass", "Shrub", "Sapling", "Canopy", "OldGrowth", "BurnScar"],
            "regrowth_macro_phases": ["None", "Scar", "Nuclei", "Front", "Closing", "Mature"],
        },
        "entries": entries,
    }


def review_ship_catalog(body: dict[str, Any], *, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.veg_resolver_parity import ENGINE_TOPOLOGY_STAMP_KEYS, ENGINE_VEG_RESOLVER_KEYS

    root = repo or repo_root()
    keys = {str(e.get("variant_key")) for e in body.get("entries") or []}
    batch = json.loads((root / TILE_BATCH_REL).read_text(encoding="utf-8"))
    batch_keys = [str(v.get("variant_key")) for v in batch.get("variants") or []]
    missing_bake = [k for k in batch_keys if k not in keys]
    missing_sparse = [k for k in ATLAS_SPARSE_V1_KEYS if k not in keys]
    missing_veg = [k for k in ENGINE_VEG_RESOLVER_KEYS if k not in keys]
    extra_veg = sorted(k for k in keys if k.startswith("veg_") and k not in ENGINE_VEG_RESOLVER_KEYS)
    missing_stamp = [k for k in ENGINE_TOPOLOGY_STAMP_KEYS if k not in keys]
    return {
        "atlas_bake_v1_aligned": missing_bake == [] and len(batch_keys) == len(ATLAS_BAKE_V1_KEYS),
        "atlas_sparse_v1_complete": missing_sparse == [],
        "engine_veg_byte_parity": missing_veg == [] and extra_veg == [],
        "topology_stamp_present": missing_stamp == [],
        "missing_bake_keys": missing_bake,
        "missing_sparse_keys": missing_sparse,
        "missing_veg_keys": missing_veg,
        "extra_veg_keys": extra_veg,
        "entry_count": len(keys),
        "topology_count": sum(1 for k in keys if k.startswith("topology_")),
    }


def validate_catalog_body(body: dict[str, Any], *, repo: Path | None = None) -> dict[str, Any]:
    import jsonschema

    root = repo or repo_root()
    schema = json.loads((root / SCHEMA_REL).read_text(encoding="utf-8"))
    jsonschema.validate(instance=body, schema=schema)
    keys = [str(e.get("variant_key")) for e in body.get("entries") or []]
    burn = [k for k in keys if k.startswith("veg_burn_")]
    review = review_ship_catalog(body, repo=root)
    return {
        "status": "passed",
        "entry_count": len(keys),
        "veg_burn_count": len(burn),
        "topology_count": sum(1 for k in keys if k.startswith("topology_")),
        "unique_keys": len(set(keys)),
        "ship_review": review,
        "ship_catalog_version": 2,
    }


def _ron_string(value: Any, indent: int = 0) -> str:
    pad = "    " * indent
    if isinstance(value, dict):
        if not value:
            return "()"
        inner = ",\n".join(
            f"{pad}    {k}: {_ron_string(v, indent + 1)}" for k, v in value.items()
        )
        return f"(\n{inner},\n{pad})"
    if isinstance(value, list):
        if not value:
            return "[]"
        inner = ",\n".join(f"{pad}    {_ron_string(v, indent + 1)}" for v in value)
        return f"[\n{inner},\n{pad}]"
    if isinstance(value, str):
        escaped = value.replace("\\", "\\\\").replace('"', '\\"')
        return f'"{escaped}"'
    if isinstance(value, bool):
        return "true" if value else "false"
    if value is None:
        return "()"
    return json.dumps(value)


def write_catalog_ron(body: dict[str, Any], *, repo: Path | None = None) -> Path:
    root = repo or repo_root()
    out = root / CATALOG_RON_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    text = "// DMCP-E3-VARIANT-KEY-SET-001 — authoritative veg + topology variant_key registry.\n"
    text += _ron_string(body) + "\n"
    out.write_text(text, encoding="utf-8")
    return out


def refresh_dmcp_e3_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = build_catalog_body(repo=root)
    validation = validate_catalog_body(body, repo=root)
    review = validation.get("ship_review") or {}
    written = write_catalog_ron(body, repo=root)
    green = (
        validation.get("status") == "passed"
        and validation.get("unique_keys") == validation.get("entry_count")
        and review.get("engine_veg_byte_parity")
        and review.get("atlas_bake_v1_aligned")
        and review.get("atlas_sparse_v1_complete")
    )
    witness: dict[str, Any] = {
        "gate": "DMCP-E3-VARIANT-KEY-SET-001",
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "ship_catalog_version": 2,
        "catalog": CATALOG_RON_REL,
        "schema": SCHEMA_REL,
        "validation": validation,
        "ship_review": review,
        "sparse_keys_added": list(ATLAS_SPARSE_V1_KEYS),
        "unblocks": ["APS-EVO-E3-VEG-STATE-AXIS-001", "CDR-B-VEG-RESOLVER-PARITY-001"],
        "_agent_meta": {
            "schema": "dmcp_e3_vegetation_catalog_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "DMCP_E3_VARIANT_KEY_SET",
            "source_system": "vegetation_variant_catalog",
            "relative_path": WITNESS_REL,
            "ritual": "BLANG:Q✓ DMCP-E3-VARIANT-KEY-SET-001" if green else None,
            "agent": "designer-mcp",
        },
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(witness, indent=2) + "\n", encoding="utf-8")
    witness["written"] = str(written.relative_to(root)).replace("\\", "/")
    return witness
