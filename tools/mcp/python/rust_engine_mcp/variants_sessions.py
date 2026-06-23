"""APS P0-A — deterministic variant session rows + witness (Variants tab)."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.reaction_territory import (
    CATALOG_EVENT_IDS,
    P0_EVENT_IDS,
    apply_tag_anchor_from_snapshot,
    build_reaction_session_rows,
)
from rust_engine_mcp.variant_matrix_expand import variant_set_rows

WITNESS_REL = "debug_runs/art_pipeline/variants_sessions_live.json"
DEFAULT_MATRIX_REL = "debug_runs/art_pipeline/variant_matrix_warehouse_v1.yaml"

# Minimum session set: clean / night / damaged / burning (deterministic keys).
SESSION_VARIANT_KEYS: tuple[str, ...] = (
    "clean_day",
    "clean_night_on",
    "damaged_day",
    "burning_00",
)


def session_variant_rows(*, style_pack_id: str, seed: int) -> list[dict[str, Any]]:
    rows = variant_set_rows(list(SESSION_VARIANT_KEYS))
    pack_tag = str(style_pack_id or "style_victorian").removeprefix("style_")
    for row in rows:
        tags = list(row.get("tags") or [])
        tags.extend(["session_default", f"stylepack_{pack_tag}", f"seed_{seed}"])
        row["tags"] = tags
    return rows


def build_variant_set_from_assembly(
    *,
    assembly_id: str,
    style_pack_id: str,
    seed: int,
    include_reaction_pack: bool = True,
    include_full_catalog: bool = False,
    assembly_snapshot: dict[str, Any] | None = None,
) -> dict[str, Any]:
    vsid = f"{assembly_id.replace('-', '_')}_variants"[:64]
    variants = session_variant_rows(style_pack_id=style_pack_id, seed=seed)
    if include_reaction_pack:
        event_ids = CATALOG_EVENT_IDS if include_full_catalog else P0_EVENT_IDS
        reaction_rows = build_reaction_session_rows(
            assembly_id=assembly_id,
            seed=seed,
            style_pack_id=style_pack_id,
            event_ids=event_ids,
        )
        if assembly_snapshot:
            apply_tag_anchor_from_snapshot(reaction_rows, assembly_snapshot)
        variants.extend(reaction_rows)
    return {
        "schema_version": 1,
        "variant_set_id": vsid,
        "assembly_id": assembly_id,
        "style_pack_id": style_pack_id,
        "seed": seed,
        "reaction_pack_included": include_reaction_pack,
        "reaction_event_scope": "full_catalog" if include_full_catalog else "p0_three",
        "axes": {
            "state": ["clean", "dirty", "damaged", "ruined"],
            "power": ["off", "partial", "on"],
            "fill": ["empty", "half", "full"],
            "lighting": ["day", "night_off", "night_on"],
        },
        "variants": variants,
    }


def refresh_variants_sessions_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    sample = build_variant_set_from_assembly(
        assembly_id="warehouse_industrial_west_production_v1",
        style_pack_id="style_industrial_west",
        seed=42,
    )
    keys = [str(v.get("variant_key")) for v in sample.get("variants") or []]
    module_ok = (root / "tools/mcp/art_pipeline_suite/variants_preview_panel.py").is_file()
    panel_ok = (root / "tools/mcp/art_pipeline_suite/variants_panel.py").is_file()
    matrix_ok = (root / DEFAULT_MATRIX_REL).is_file()
    reaction_ok = (root / "tools/mcp/schemas/examples/reaction_territory_events_v1.json").is_file()
    green = (
        len(keys) >= 4
        and all(k in keys for k in SESSION_VARIANT_KEYS)
        and module_ok
        and panel_ok
        and reaction_ok
    )
    body: dict[str, Any] = {
        "gate": "APS-P0-VARIANTS-SESSIONS-001",
        "green": green,
        "session_variant_keys": keys,
        "session_count": len(keys),
        "minimum_keys": list(SESSION_VARIANT_KEYS),
        "reaction_pack_included": sample.get("reaction_pack_included"),
        "reaction_catalog": reaction_ok,
        "variants_preview_panel": module_ok,
        "variants_panel_wired": panel_ok,
        "default_matrix": DEFAULT_MATRIX_REL,
        "default_matrix_exists": matrix_ok,
        "_agent_meta": {
            "schema": "variants_sessions_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "APS_VARIANTS_SESSIONS",
            "source_system": "variants_sessions",
            "relative_path": WITNESS_REL,
        },
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
