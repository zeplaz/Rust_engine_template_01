"""RT-BRIEF-001 — runtime lookup brief from index row + atlas meta (≤40 lines)."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .paths import repo_root
from .rt_registry import (
    ROWHOUSE_PRODUCTION_ATLAS_ID,
    lookup_stamp_from_meta,
    rt_registry_001_green,
)
from .schemas import load_json_file
from .tile_index import load_tile_atlas_index
from .validators.atlas_meta import _required_lookup_keys, validate_atlas_meta_v2

RT_LOOKUP_BRIEF_WITNESS = "debug_runs/rt_lookup_brief_001_live.json"


def _rel(path: Path) -> str:
    try:
        return str(path.relative_to(repo_root())).replace("\\", "/")
    except ValueError:
        return str(path)


def _lookup_labels(missing: set[tuple[str, int, int]]) -> list[str]:
    labels: list[str] = []
    for variant, facing, frame in sorted(missing):
        if frame:
            labels.append(f"{variant}_f{frame}")
        elif facing:
            labels.append(f"{variant}_f{facing}")
        else:
            labels.append(variant)
    return labels


def runtime_lookup_brief(atlas_id: str = ROWHOUSE_PRODUCTION_ATLAS_ID) -> dict[str, Any]:
    """Engine-facing lookup summary — index row + meta stamp + missing cells."""
    entry = next(
        (e for e in load_tile_atlas_index() if str(e.get("atlas_id") or "") == atlas_id),
        None,
    )
    if entry is None:
        return {
            "schema": "runtime_lookup_brief_v1",
            "ok": False,
            "atlas_id": atlas_id,
            "artist_messages": [
                {
                    "sentence": f"No index row for atlas_id {atlas_id} — run tile-atlas-register first.",
                    "fix": "rust-engine-mcp tile-atlas-register <batch>",
                    "legend_code": "RT_INDEX_MISSING",
                }
            ],
            "plain_language_count": 1,
            "hint": "Runtime registry — not APS Atlas tab",
        }

    meta_rel = str(entry.get("meta_json") or "")
    meta_path = Path(meta_rel) if Path(meta_rel).is_absolute() else repo_root() / meta_rel
    if not meta_path.is_file():
        return {
            "schema": "runtime_lookup_brief_v1",
            "ok": False,
            "atlas_id": atlas_id,
            "meta_json": meta_rel,
            "artist_messages": [
                {
                    "sentence": "Index row points to missing atlas_meta.json.",
                    "fix": "Re-run tile batch + pack, then register",
                    "legend_code": "RT_META_MISSING",
                }
            ],
            "plain_language_count": 1,
            "hint": "Runtime registry — not APS Atlas tab",
        }

    meta = load_json_file(meta_path)
    stamp = lookup_stamp_from_meta(meta)
    schema_version = int(meta.get("schema_version") or 1)
    artist_messages: list[dict[str, str]] = []
    missing_labels: list[str] = []

    if schema_version >= 2:
        report = validate_atlas_meta_v2(meta_path)
        required = _required_lookup_keys(meta)
        present = {
            (
                str(row.get("variant_key") or row.get("key") or ""),
                int(row.get("facing") or 0),
                int(row.get("frame") or 0),
            )
            for row in (meta.get("lookups") or [])
            if isinstance(row, dict)
        }
        missing_labels = _lookup_labels(required - present)
        if report.status != "passed":
            artist_messages.append(
                {
                    "sentence": "Atlas meta v2 validation failed — engine lookup may miss facings.",
                    "fix": "See atlas-meta-brief on staging folder",
                    "legend_code": "RT_META_V2_FAIL",
                }
            )
    else:
        artist_messages.append(
            {
                "sentence": f"Atlas meta is v1 with {stamp['lookup_count']} variant tiles — acceptable for rowhouse production register.",
                "fix": "Warehouse Track B uses v2; rowhouse production stays on v1 stamp",
                "legend_code": "RT_META_V1_OK",
            }
        )

    if missing_labels:
        artist_messages.insert(
            0,
            {
                "sentence": f"{len(missing_labels)} lookup cells missing from atlas meta.",
                "fix": "Re-run tile batch for: " + ", ".join(missing_labels[:4]),
                "legend_code": "RT_LOOKUP_GAP",
            },
        )

    atlas_png = Path(str(entry.get("atlas_png") or ""))
    if not atlas_png.is_absolute():
        atlas_png = repo_root() / atlas_png
    png_ok = atlas_png.is_file()
    if not png_ok:
        artist_messages.append(
            {
                "sentence": "Atlas PNG path from index is missing on disk.",
                "fix": "Re-run atlas pack or fix atlas_png in index",
                "legend_code": "RT_PNG_MISSING",
            }
        )

    ok = (
        bool(entry.get("ship_allowed"))
        and stamp["lookup_count"] > 0
        and png_ok
        and not missing_labels
        and (schema_version < 2 or validate_atlas_meta_v2(meta_path).status == "passed")
    )
    if ok and len(artist_messages) == 1 and artist_messages[0].get("legend_code") == "RT_META_V1_OK":
        pass
    elif ok and not artist_messages:
        artist_messages = [
            {
                "sentence": "Runtime lookup table is complete — engine may load this atlas row.",
                "fix": "Proceed to RT-ENG-001 registry load demo",
                "legend_code": "RT_LOOKUP_PASS",
            }
        ]

    return {
        "schema": "runtime_lookup_brief_v1",
        "ok": ok,
        "atlas_id": atlas_id,
        "batch_id": entry.get("batch_id"),
        "tile_id": entry.get("tile_id"),
        "meta_json": _rel(meta_path),
        "atlas_meta_schema": f"v{schema_version}",
        "lookup_stamp": stamp,
        "missing_lookups": missing_labels[:12],
        "ship_allowed": bool(entry.get("ship_allowed")),
        "atlas_png_present": png_ok,
        "artist_messages": artist_messages[:8],
        "plain_language_count": len(artist_messages),
        "hint": "Runtime registry — not APS Atlas tab",
    }


def refresh_rt_lookup_brief_001_witness(*, atlas_id: str | None = None) -> bool:
    aid = atlas_id or ROWHOUSE_PRODUCTION_ATLAS_ID
    brief = runtime_lookup_brief(aid)
    green = bool(brief.get("ok")) and rt_registry_001_green()
    payload = {
        "gate_id": "RT-BRIEF-001",
        "green": green,
        "ok": green,
        "atlas_id": aid,
        "lookup_stamp": brief.get("lookup_stamp"),
        "missing_lookups": brief.get("missing_lookups"),
        "plain_language_count": brief.get("plain_language_count"),
        "registry_witness_green": rt_registry_001_green(),
        "brief": brief,
    }
    out = repo_root() / RT_LOOKUP_BRIEF_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return green
