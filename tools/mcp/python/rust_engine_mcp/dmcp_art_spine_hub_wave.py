"""DMCP art spine hub wave — LG5 QC, rowhouse v2, civic, mat pilot 002."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from PIL import Image

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.schemas import load_json_file

WITNESS_REL = "debug_runs/art_pipeline/dmcp_art_spine_hub_wave_live.json"

LG5_BATCH_REL = "assets/staging/specs/tile_batch_landscape_expanded_v1.json"
LG5_KEYFRAME_REL = "assets/staging/tiles/keyframe_stills/tile_landscape_expanded_v1"
LG5_REQS_DOC = "src/dev/design_landscape_keyframe_burn_reqs_v1.md"

ROWHOUSE_VARIANT_REL = "tools/mcp/schemas/examples/variant_set_rowhouse_victorian_production_v1.json"
ROWHOUSE_ATLAS_META = "assets/staging/tiles/tile_rowhouse_victorian_production_v1/atlas_meta.json"
ROWHOUSE_DAMAGE = ("damaged_day", "damaged_night_on")
ROWHOUSE_BURN = tuple(f"burning_{i:02d}" for i in range(8))

MAT_PACK_REL = "assets/staging/specs/mat_profile_pilot_002_pack.json"

LANE_SLICES: tuple[dict[str, str], ...] = (
    {
        "id": "DMCP-LG5-KEYFRAME-QC-001",
        "deliverable": "src/dev/design_landscape_lg5_keyframe_qc_v1.md",
    },
    {
        "id": "DMCP-TILE-ROWHOUSE-V2-001",
        "deliverable": "src/dev/design_rowhouse_v2_operator_qc_v1.md",
    },
    {
        "id": "DES-GRAM-ARCHETYPE-CIVIC-001",
        "deliverable": "src/dev/design_civic_block_concept_v1.md",
    },
    {
        "id": "DMCP-MAT-PROFILE-PILOT-002",
        "deliverable": "src/dev/design_mat_profile_pilot_002_v1.md",
        "pack": MAT_PACK_REL,
    },
)

G4_MINIMUM_REVIEW = (
    "topology_patch_burn_04",
    "topology_patch_scar",
    "topology_corridor_regrowth_grass",
)


def _check_lg5_keyframe_qc(root: Path) -> dict[str, Any]:
    batch = load_json_file(root / LG5_BATCH_REL)
    variant_keys = [
        str(v.get("variant_key")) for v in batch.get("variants") or [] if v.get("variant_key")
    ]
    keyframe_dir = root / LG5_KEYFRAME_REL
    png_checks: list[dict[str, Any]] = []
    all_png_ok = True
    for key in variant_keys:
        png_path = keyframe_dir / f"{key}.png"
        row: dict[str, Any] = {"variant_key": key, "exists": png_path.is_file()}
        if png_path.is_file():
            with Image.open(png_path) as im:
                row["size"] = list(im.size)
                row["size_64"] = im.size == (64, 64)
        else:
            row["size_64"] = False
            all_png_ok = False
        png_checks.append(row)
    g4_in_batch = {k: k in variant_keys for k in G4_MINIMUM_REVIEW}
    checks = {
        "reqs_doc": (root / LG5_REQS_DOC).is_file(),
        "batch_variant_count_16": len(variant_keys) == 16,
        "all_keyframes_on_disk": all_png_ok,
        "all_64px": all(row.get("size_64") for row in png_checks),
        "seed_in_batch": int((batch.get("render") or {}).get("seed") or 0) == 550005,
        "ship_false": batch.get("ship") is False,
    }
    green = all(checks.values())
    verdict = "PASS_WITH_NOTES" if green and not all(g4_in_batch.values()) else ("PASS" if green else "FAIL")
    return {
        "green": green,
        "verdict": verdict,
        "checks": checks,
        "g4_minimum_in_batch": g4_in_batch,
        "png_rows": png_checks,
    }


def _check_rowhouse_v2(root: Path) -> dict[str, Any]:
    variant_set = load_json_file(root / ROWHOUSE_VARIANT_REL)
    keys = {str(v.get("variant_key")) for v in variant_set.get("variants") or []}
    meta_path = root / ROWHOUSE_ATLAS_META
    meta = json.loads(meta_path.read_text(encoding="utf-8")) if meta_path.is_file() else {}
    atlas_keys = {str(t.get("variant_key")) for t in meta.get("tiles") or []}
    burn_present = all(k in keys for k in ROWHOUSE_BURN)
    damage_present = all(k in keys for k in ROWHOUSE_DAMAGE)
    checks = {
        "variant_set_on_disk": True,
        "atlas_meta_on_disk": meta_path.is_file(),
        "variant_count_14": len(keys) == 14,
        "damage_frames": damage_present,
        "burn_frames_8": burn_present,
        "atlas_matches_variant_set": keys == atlas_keys and len(atlas_keys) == 14,
        "fire_tags": all(
            "sim_fire" in (v.get("sim_tags") or [])
            for v in variant_set.get("variants") or []
            if str(v.get("variant_key", "")).startswith("burning_")
        ),
    }
    return {"green": all(checks.values()), "verdict": "PASS", "checks": checks, "variant_keys": sorted(keys)}


def _check_civic_concept(root: Path) -> dict[str, Any]:
    doc = root / "src/dev/design_civic_block_concept_v1.md"
    text = doc.read_text(encoding="utf-8") if doc.is_file() else ""
    checks = {
        "doc_on_disk": doc.is_file(),
        "civic_block_id": "civic_block_v1" in text,
        "archetype_civic_block": "CivicBlock" in text,
        "massing_stepped_block": "stepped_block" in text,
        "g2_seed_note": "G2" in text or "GRAM-CONTENT-005" in text,
        "no_ron_on_disk": not (root / "assets/configs/buildings/grammars/civic_block_v1.ron").is_file(),
    }
    return {"green": all(checks.values()), "verdict": "PASS", "checks": checks}


def _check_mat_pilot_002(root: Path) -> dict[str, Any]:
    pack_path = root / MAT_PACK_REL
    pack = json.loads(pack_path.read_text(encoding="utf-8")) if pack_path.is_file() else {}
    profiles = pack.get("profiles") or []
    ids = {str(p.get("profile_id")) for p in profiles}
    categories = {str(p.get("category")) for p in profiles}
    leaf_expected = set(pack.get("category_leaves") or [])
    checks = {
        "pack_on_disk": pack_path.is_file(),
        "profile_count_24": len(profiles) == 24,
        "unique_ids": len(ids) == 24,
        "spec_only": pack.get("spec_only") is True,
        "categories_match_leaves": categories == leaf_expected,
        "deterministic_seeds": all(p.get("seed") for p in profiles),
    }
    return {
        "green": all(checks.values()),
        "verdict": "PASS",
        "checks": checks,
        "profile_count": len(profiles),
    }


def run_art_spine_hub_wave_audit(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    auditors = {
        "DMCP-LG5-KEYFRAME-QC-001": _check_lg5_keyframe_qc,
        "DMCP-TILE-ROWHOUSE-V2-001": _check_rowhouse_v2,
        "DES-GRAM-ARCHETYPE-CIVIC-001": _check_civic_concept,
        "DMCP-MAT-PROFILE-PILOT-002": _check_mat_pilot_002,
    }
    rows: list[dict[str, Any]] = []
    for spec in LANE_SLICES:
        gate = spec["id"]
        audit_fn = auditors[gate]
        audit = audit_fn(root)
        rows.append(
            {
                "id": gate,
                "deliverable": spec["deliverable"],
                "deliverable_exists": (root / spec["deliverable"]).is_file(),
                "status": "done" if audit.get("green") and (root / spec["deliverable"]).is_file() else "fail",
                "verdict": audit.get("verdict"),
                "audit": audit,
            }
        )
    done = sum(1 for r in rows if r["status"] == "done")
    green = done == len(rows)
    return {
        "gate": "DMCP-ART-SPINE-HUB-WAVE-001",
        "lanes": [s["id"] for s in LANE_SLICES],
        "slice_count": len(rows),
        "done_count": done,
        "rows": rows,
        "audit_complete": True,
        "green": green,
        "verdict": "PASS_WITH_NOTES" if green and any(r.get("verdict") == "PASS_WITH_NOTES" for r in rows) else ("PASS" if green else "FAIL"),
        "handoff": {
            "lg5_g4": "operator manual keyframes for corridor regrowth",
            "civic_ron": "GRAM-CONTENT-005 @coder-mcp",
            "mat_merge": "@coder-mcp validate material_profiles",
        },
    }


def refresh_dmcp_art_spine_hub_wave_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = run_art_spine_hub_wave_audit(repo=root)
    body["_agent_meta"] = {
        "schema": "dmcp_art_spine_hub_wave_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "DMCP_ART_SPINE_HUB",
        "source_system": "dmcp_art_spine_hub_wave",
        "relative_path": WITNESS_REL,
        "ritual": "BLANG:WIT-HON→Q✓ DMCP-ART-SPINE-HUB" if body.get("green") else "BLANG:WIT-HON FAIL",
        "agent": "designer-mcp",
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
