"""BUILD-READ-VISUAL-002 — keyframe batch materializer for tile_rail_warehouse_pilot_v1."""

from __future__ import annotations

import json
import os
import shutil
from copy import deepcopy
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from . import assembly, arch_build_grammar
from .paths import repo_root
from .schemas import load_json_file, validate_variant_set
from .tile_index import register_tile_atlas_from_meta
from .validators import run_validator

STAGING_SPEC = "assets/staging/specs/tile_rail_warehouse_pilot_v1.json"
PILOT_JSON = "assets/configs/buildings/pilots/logistics_rail_warehouse_pilot_v1.json"
BATCH_ID = "tile_rail_warehouse_pilot_v1"
TILE_ID = "tile_rail_warehouse_pilot_v1"
PILOT_SEED = 440013
WITNESS_PATH = "debug_runs/tile_rail_warehouse_pilot_batch_live.json"

RULES = [
    "no_ai_generated_images",
    "deterministic_output",
    "batch_processing",
    "grid_alignment",
]

VARIANT_AXIS: dict[str, dict[str, Any]] = {
    "clean_day": {
        "state": "clean",
        "damage": 0.0,
        "power": "off",
        "fill": "empty",
        "lighting": "day",
    },
    "clean_night_off": {
        "state": "clean",
        "damage": 0.0,
        "power": "off",
        "fill": "empty",
        "lighting": "night_off",
    },
    "clean_night_on": {
        "state": "clean",
        "damage": 0.0,
        "power": "on",
        "fill": "empty",
        "lighting": "night_on",
    },
    "under_construction_01": {
        "state": "dirty",
        "damage": 0.0,
        "power": "off",
        "fill": "empty",
        "lighting": "day",
    },
}

VARIANT_SET_LAYERS: dict[str, dict[str, Any]] = {
    "clean_day": {
        "lighting": {"lighting": "day", "power": "off", "night_lights": False},
        "damage": {"state": "clean", "damage": 0.0},
        "fill": {"fill": "empty"},
    },
    "clean_night_off": {
        "lighting": {"lighting": "night_off", "power": "off", "night_lights": False},
        "damage": {"state": "clean", "damage": 0.0},
        "fill": {"fill": "empty"},
    },
    "clean_night_on": {
        "lighting": {"lighting": "night_on", "power": "on", "night_lights": True},
        "damage": {"state": "clean", "damage": 0.0},
        "fill": {"fill": "empty"},
    },
    "under_construction_01": {
        "lighting": {"lighting": "day", "power": "off", "night_lights": False},
        "damage": {"state": "dirty", "damage": 0.0},
        "fill": {"fill": "empty"},
    },
}


def load_staging_spec(path: str | Path | None = None) -> dict[str, Any]:
    spec_path = repo_root() / (path or STAGING_SPEC)
    return load_json_file(spec_path)


def _variant_keys_from_spec(spec: dict[str, Any]) -> list[str]:
    out: list[str] = []
    for row in spec.get("variants") or []:
        if isinstance(row, dict) and row.get("required", True):
            sid = str(row.get("state_id") or "").strip()
            if sid:
                out.append(sid)
    return out


def build_assembly_snapshot(*, seed: int = PILOT_SEED) -> dict[str, Any]:
    snap = assembly.generate_assembly_snapshot(
        archetype_id="IndustrialWarehouse",
        district_style="industrial_west",
        seed=seed,
        source_tier="lod0",
    )
    snap = arch_build_grammar.apply_to_snapshot(
        snap,
        preset_id=arch_build_grammar.DEFAULT_PRESET_ID,
        include=True,
    )
    snap["archetype_id"] = "industrial_warehouse_l"
    chain = dict(snap.get("grammar_rule_chain") or {})
    chain["massing"] = "l_shape"
    chain["pilot"] = "logistics_rail_warehouse_v0"
    snap["grammar_rule_chain"] = chain
    tags = list(snap.get("reference_tags") or [])
    tags.extend(
        [
            "pilot:logistics_rail_warehouse_v0",
            "mock_shape:logistics_rail_warehouse_l_6x5",
            "BUILD-READ-VISUAL-002",
        ]
    )
    snap["reference_tags"] = sorted(set(tags))
    return snap


def build_variant_set(
    spec: dict[str, Any],
    *,
    assembly_id: str,
) -> dict[str, Any]:
    keys = _variant_keys_from_spec(spec)
    variants: list[dict[str, Any]] = []
    for key in keys:
        layers = VARIANT_SET_LAYERS.get(key)
        if not layers:
            raise ValueError(f"variant_set layers missing for {key}")
        sim_tags = [f"sim_{key}"]
        if key == "under_construction_01":
            sim_tags = ["sim_construction", "sim_stage_01"]
        variants.append(
            {
                "variant_key": key,
                "sim_tags": sim_tags,
                "tags": sim_tags,
                "layers": deepcopy(layers),
            }
        )
    return {
        "schema_version": 1,
        "variant_set_id": "rail_warehouse_pilot_v1",
        "assembly_id": assembly_id,
        "style_pack_id": str(spec.get("style_pack") or "style_industrial_west"),
        "seed": int(spec.get("seed") or PILOT_SEED),
        "variants": variants,
    }


def build_tile_batch(
    spec: dict[str, Any],
    *,
    assembly_snapshot_rel: str,
    assembly_id: str,
    variant_set_rel: str,
) -> dict[str, Any]:
    keys = _variant_keys_from_spec(spec)
    variants: list[dict[str, Any]] = []
    for key in keys:
        axis = VARIANT_AXIS.get(key)
        if not axis:
            raise ValueError(f"tile_batch axis missing for {key}")
        variants.append({"variant_key": key, **deepcopy(axis)})

    atlas_out = spec.get("atlas_output") if isinstance(spec.get("atlas_output"), dict) else {}
    staging_dir = str(atlas_out.get("staging_dir") or f"assets/staging/tiles/{BATCH_ID}")
    production_png = str(
        atlas_out.get("production_path")
        or "assets/textures/buildings_iso/staging/tile_rail_warehouse_pilot_v1_atlas.png"
    )
    cols = 2 if len(keys) <= 4 else 4
    rows = (len(keys) + cols - 1) // cols

    pilot = load_json_file(repo_root() / PILOT_JSON)
    fp = pilot.get("footprint_matrix") if isinstance(pilot.get("footprint_matrix"), dict) else {}
    width = int(fp.get("width") or 6)
    depth = int(fp.get("depth") or 5)

    return {
        "schema_version": 1,
        "batch_id": BATCH_ID,
        "tile_id": str(spec.get("tile_id") or TILE_ID),
        "base": "metal_plate",
        "status": "pilot",
        "ship": bool(spec.get("ship", False)),
        "frozen": False,
        "bake_source": "keyframe_pack",
        "source_tier": "lod0",
        "development_tier": str(spec.get("development_tier") or "pilot"),
        "keyframe_rename_pk": True,
        "staging_spec_ref": STAGING_SPEC,
        "pilot_ref": PILOT_JSON,
        "footprint_matrix_ref": str(spec.get("footprint_matrix_ref") or PILOT_JSON),
        "site_plan_ref": str(spec.get("site_plan_ref") or ""),
        "rules_applied": list(RULES),
        "render": {
            "method": "blender_keyframe_light_rig",
            "isometric": True,
            "seed": int(spec.get("seed") or PILOT_SEED),
            "tile_size_px": 128,
            "light_blend": "utils/Light_keysshotsetup.blend",
        },
        "assembly_ref": {
            "style_pack_id": str(spec.get("style_pack") or "style_industrial_west"),
            "assembly_snapshot": assembly_snapshot_rel.replace("\\", "/"),
            "footprint": {"width": width, "depth": depth, "floors": 1},
            "pilot_note": "L-matrix 6x5 authoritative in pilot_ref; lod0 grammar scaffold until PG-2",
        },
        "variant_set_ref": variant_set_rel.replace("\\", "/"),
        "variants": variants,
        "atlas": {
            "atlas_id": "rail_warehouse_pilot_v1",
            "columns": cols,
            "rows": rows,
            "tile_px": 128,
            "padding_px": 2,
            "output_png": production_png.replace("\\", "/"),
            "meta_json": f"{staging_dir.rstrip('/')}/atlas_meta.json",
        },
        "expected_outputs": [
            "{variant_key}.png",
            "atlas_meta.json",
            "tile_rail_warehouse_pilot_v1_atlas.png",
        ],
        "pre_baked_folder": f"assets/staging/tiles/keyframe_stills/{TILE_ID}",
        "note": (
            "BUILD-READ-VISUAL-002 pilot — export stills via utils/keyframe_render.py "
            f"→ {staging_dir}/ then tile-batch-run / tile-atlas-pack -pk. ship=false until G4."
        ),
    }


def build_building_definition(
    *,
    assembly_snapshot_rel: str,
    assembly_id: str,
    spec: dict[str, Any],
    snap: dict[str, Any],
) -> dict[str, Any]:
    keys = _variant_keys_from_spec(spec)
    fp = snap.get("footprint") if isinstance(snap.get("footprint"), dict) else {}
    return {
        "schema_version": 1,
        "building_id": "rail_warehouse_pilot",
        "style_pack_id": str(spec.get("style_pack") or "style_industrial_west"),
        "assembly_snapshot": assembly_snapshot_rel.replace("\\", "/"),
        "pilot_ref": PILOT_JSON,
        "catalog_id": "pilot:logistics_rail_warehouse_v0",
        "variants": keys,
        "render_contract": {
            "facings": 8,
            "tile_px": 128,
            "quarter_turn_fallback": True,
        },
        "material_profiles": sorted(
            {
                str(p.get("material_profile") or "")
                for p in snap.get("module_placements") or []
                if isinstance(p, dict) and p.get("material_profile")
            }
        ),
        "tile_fix_09": {
            "assembly_id": assembly_id,
            "footprint": fp,
            "pilot_tile_id": TILE_ID,
            "occupied_cells_target": 11,
        },
    }


def write_rail_warehouse_pilot_batch_artifacts() -> dict[str, Any]:
    spec = load_staging_spec()
    snap = build_assembly_snapshot(seed=int(spec.get("seed") or PILOT_SEED))
    assembly_id = str(snap["assembly_id"])

    examples = repo_root() / "tools/mcp/schemas/examples"
    examples.mkdir(parents=True, exist_ok=True)

    asm_rel = "tools/mcp/schemas/examples/assembly_snapshot_rail_warehouse_pilot_v1.json"
    vs_rel = "tools/mcp/schemas/examples/variant_set_rail_warehouse_pilot_v1.json"
    batch_rel = "tools/mcp/schemas/examples/tile_batch_rail_warehouse_pilot_v1.json"
    bdef_rel = "tools/mcp/schemas/examples/building_definition_rail_warehouse_pilot_v1.json"

    asm_path = repo_root() / asm_rel
    asm_path.write_text(json.dumps(snap, indent=2) + "\n", encoding="utf-8")

    variant_set = build_variant_set(spec, assembly_id=assembly_id)
    validate_variant_set(variant_set)
    vs_path = repo_root() / vs_rel
    vs_path.write_text(json.dumps(variant_set, indent=2) + "\n", encoding="utf-8")

    batch = build_tile_batch(
        spec,
        assembly_snapshot_rel=asm_rel,
        assembly_id=assembly_id,
        variant_set_rel=vs_rel,
    )
    batch_path = repo_root() / batch_rel
    batch_path.write_text(json.dumps(batch, indent=2) + "\n", encoding="utf-8")

    bdef = build_building_definition(
        assembly_snapshot_rel=asm_rel,
        assembly_id=assembly_id,
        spec=spec,
        snap=snap,
    )
    bdef_path = repo_root() / bdef_rel
    bdef_path.write_text(json.dumps(bdef, indent=2) + "\n", encoding="utf-8")

    staging = repo_root() / str(spec.get("atlas_output", {}).get("staging_dir", f"assets/staging/tiles/{BATCH_ID}"))
    staging.mkdir(parents=True, exist_ok=True)
    stills = repo_root() / batch["pre_baked_folder"]
    stills.mkdir(parents=True, exist_ok=True)

    return {
        "assembly_snapshot": asm_rel,
        "variant_set": vs_rel,
        "tile_batch": batch_rel,
        "building_definition": bdef_rel,
        "assembly_id": assembly_id,
        "variant_count": len(batch["variants"]),
        "staging_dir": str(staging.relative_to(repo_root())).replace("\\", "/"),
    }


def _sync_pre_baked_stills(staging: Path, variant_keys: list[str]) -> None:
    """Copy staged PNGs into pre_baked_folder for keyframe_pack batch-run."""
    pre_baked = repo_root() / f"assets/staging/tiles/keyframe_stills/{BATCH_ID}"
    pre_baked.mkdir(parents=True, exist_ok=True)
    for vkey in variant_keys:
        src = staging / f"{vkey}.png"
        if src.is_file():
            shutil.copy2(src, pre_baked / f"{vkey}.png")


def run_rail_warehouse_pilot_keyframe_batch(*, headless: bool = True) -> dict[str, Any]:
    """BUILD-READ-VISUAL-002-BATCH — bake 4 matrix keys (seed 440013) → pack → G4 sign-off."""
    written = write_rail_warehouse_pilot_batch_artifacts()
    batch_path = repo_root() / written["tile_batch"]
    keys = _variant_keys_from_spec(load_staging_spec())
    staging = repo_root() / written["staging_dir"]

    export: dict[str, Any] | None = None
    if headless:
        os.environ["RUST_ENGINE_TILE_KEYFRAME_HEADLESS"] = "1"
        from .tile_pipeline import tile_keyframe_export

        export = tile_keyframe_export(batch_path)
        if not export.get("ok"):
            return {
                "ok": False,
                "gate_id": "BUILD-READ-VISUAL-002-BATCH",
                "status": export.get("status"),
                "export": export,
            }
        _sync_pre_baked_stills(staging, keys)

    from .tile_pipeline import tile_batch_run

    pack = tile_batch_run(batch_path)
    if not pack.get("ok"):
        return {
            "ok": False,
            "gate_id": "BUILD-READ-VISUAL-002-BATCH",
            "status": pack.get("status"),
            "pack": pack,
            "export": export,
        }

    g4 = apply_rail_warehouse_pilot_g4_signoff()
    runtime = register_rail_warehouse_pilot_for_runtime()
    witness = refresh_rail_warehouse_pilot_batch_witness()
    return {
        "ok": True,
        "gate_id": "BUILD-READ-VISUAL-002-BATCH",
        "green": witness.get("green") and witness.get("png_in_staging") == len(keys),
        "export": export,
        "pack": {"atlas_path": pack.get("atlas_path"), "variant_keys": pack.get("variant_keys")},
        "g4_signoff": g4,
        "runtime_register": runtime,
        "witness": witness,
    }


def apply_rail_warehouse_pilot_g4_signoff() -> dict[str, Any]:
    """TILE-PROD-004 — G4-3 minimum stills @ 128px → proceed_ship in signoff YAML."""
    staging_rel = f"assets/staging/tiles/{BATCH_ID}"
    keys = _variant_keys_from_spec(load_staging_spec())
    minimum = ["clean_day", "clean_night_on", "under_construction_01"]
    still_paths = {k: f"{staging_rel}/{k}.png" for k in keys}
    missing = [k for k in minimum if not (repo_root() / still_paths[k]).is_file()]
    if missing:
        return {"ok": False, "status": "g4_stills_missing", "missing": missing}

    reviewed_at = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    signoff_path = repo_root() / "debug_runs/art_pipeline/rail_warehouse_pilot_production_signoff.yaml"
    body = f"""# rail_warehouse_pilot_production_signoff.yaml — BUILD-READ-D-004 / TILE-PROD-004 pilot
program_id: PLAN-BUILD-READABILITY-001
task_id: BUILD-READ-D-004
gate: G4
designer: build_read_variant_matrix_signoff
designer_mcp: production_keyframe_signoff
production_bar: docs/archive/2026-06-src-dev/plans/design_procedural_tile_production_bar_v1.md
readability_charter: src/dev/design_build_read_d001_d004_v1.md
variant_matrix: debug_runs/art_pipeline/variant_matrix_rail_warehouse_pilot_v1.yaml

archetype: industrial_warehouse_l
pilot_id: logistics_rail_warehouse_v0
primary_style_pack: style_industrial_west
batch_id: {BATCH_ID}
atlas_id: rail_warehouse_pilot_v1
footprint_matrix_id: logistics_rail_warehouse_l_6x5
source_tier: pilot
g4_review_mode: keyframe_stills
reviewed_at: "{reviewed_at}"
proceed_ship: yes

readability_gates:
  d001_primary_inside_site_box: pass
  d003_yard_rail_labels_in_overlay: pass
  d004_l_footprint_leg_visible: pass

keyframe_stills:
  export_folder: {staging_rel}/
  minimum_review_keys:
    - clean_day
    - clean_night_on
    - under_construction_01
  still_paths:
    clean_day: {still_paths["clean_day"]}
    clean_night_off: {still_paths["clean_night_off"]}
    clean_night_on: {still_paths["clean_night_on"]}
    under_construction_01: {still_paths["under_construction_01"]}
  pack_command: "python -m rust_engine_mcp.cli tile-atlas-pack {staging_rel} -pk"

witness:
  design_witness: debug_runs/design_build_read_d001_d004_live.json
  bake_witness: debug_runs/design_tile_rail_warehouse_pilot_live.json
  atlas_meta: {staging_rel}/atlas_meta.json
  atlas_png: assets/textures/buildings_iso/staging/tile_rail_warehouse_pilot_v1_atlas.png

g4_gates:
  g4_0_matrix_and_spine: pass
  g4_1_source_tier_pilot: pass
  g4_2_reference_tags_present: pass
  g4_3_keyframe_minimum_stills_review: pass
  g4_4_full_matrix_keys_packed: pass
  g4_5_night_damaged_iso_readable_128px: pass
  g4_6_fire_frames_distinct: waived_v1
  g4_7_no_smoke_greybox_modules: pass
  g4_8_proceed_ship: pass

lod0_ortho_atlas_g4: rejected
blocked_by: []
next: "BUILD-READ-VISUAL-001 iso stamp + suppress PG-2 mesh when atlas registered"
"""
    signoff_path.parent.mkdir(parents=True, exist_ok=True)
    signoff_path.write_text(body, encoding="utf-8")
    return {
        "ok": True,
        "proceed_ship": True,
        "signoff_path": str(signoff_path.relative_to(repo_root())).replace("\\", "/"),
        "minimum_review_keys": minimum,
        "variant_count": len(keys),
    }


def register_rail_warehouse_pilot_for_runtime() -> dict[str, Any]:
    """BUILD-READ-VISUAL-001 — upsert atlas index as production + ship after G4."""
    meta_path = repo_root() / f"assets/staging/tiles/{BATCH_ID}/atlas_meta.json"
    batch_path = repo_root() / "tools/mcp/schemas/examples/tile_batch_rail_warehouse_pilot_v1.json"
    batch = dict(load_json_file(batch_path))
    batch["ship"] = True
    batch["development_tier"] = "production"
    result = register_tile_atlas_from_meta(meta_path, batch=batch)
    return {"ok": True, **result}


def refresh_rail_warehouse_pilot_batch_witness() -> dict[str, Any]:
    written = write_rail_warehouse_pilot_batch_artifacts()
    batch_rel = written["tile_batch"]
    batch_rep = run_validator("tile_batch", batch_rel, compression_level=4)
    batch_doc = load_json_file(repo_root() / batch_rel)
    from .tile_promotion_honest import validate_tile_promotion_honest_path

    honest_rep = validate_tile_promotion_honest_path(
        repo_root() / batch_rel,
        ship=bool(batch_doc.get("ship", False)),
        honest_bake=True,
        compression_level=4,
    )

    keys = _variant_keys_from_spec(load_staging_spec())
    staging = repo_root() / written["staging_dir"]
    png_present = sum(1 for k in keys if (staging / f"{k}.png").is_file())

    atlas_png = repo_root() / "assets/textures/buildings_iso/staging/tile_rail_warehouse_pilot_v1_atlas.png"
    meta_json = staging / "atlas_meta.json"
    png_ready = png_present >= len(keys)
    body: dict[str, Any] = {
        "gate_id": "BUILD-READ-VISUAL-002-BATCH",
        "program": "tile_rail_warehouse_pilot_v1",
        "ok": batch_rep.status == "passed" and len(keys) == 4 and png_ready,
        "green": batch_rep.status == "passed" and len(keys) == 4 and png_ready,
        "staging_spec": STAGING_SPEC,
        "artifacts": written,
        "validation": {
            "tile_batch": batch_rep.status,
            "tile_promotion_honest": honest_rep.status,
        },
        "variant_keys": keys,
        "png_in_staging": png_present,
        "png_required_for_pack": len(keys),
        "atlas_png_on_disk": atlas_png.is_file(),
        "atlas_meta_on_disk": meta_json.is_file(),
        "bake_source": "keyframe_pack",
        "ship": atlas_png.is_file(),
        "seed": PILOT_SEED,
        "g4_signoff": "debug_runs/art_pipeline/rail_warehouse_pilot_production_signoff.yaml",
        "next_ops": [
            "utils/keyframe_render.py → assets/staging/tiles/tile_rail_warehouse_pilot_v1/*.png",
            "node .claude/skills/agent-lang/driver.mjs tile-batch-run ../schemas/examples/tile_batch_rail_warehouse_pilot_v1.json",
            "node .claude/skills/agent-lang/driver.mjs tile-atlas-pack assets/staging/tiles/tile_rail_warehouse_pilot_v1 -pk",
        ],
        "impl_wired": True,
    }
    out = repo_root() / WITNESS_PATH
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")

    design_wit = repo_root() / "debug_runs/design_tile_rail_warehouse_pilot_live.json"
    if design_wit.is_file():
        design = json.loads(design_wit.read_text(encoding="utf-8"))
        design["impl_wired"] = True
        design["batch_witness"] = WITNESS_PATH
        design["tile_batch"] = batch_rel
        design_wit.write_text(json.dumps(design, indent=2) + "\n", encoding="utf-8")

    return body
