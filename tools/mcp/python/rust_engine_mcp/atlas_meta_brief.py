"""MCP-ATLAS-BRIEF-001 — ≤40-line artist summary of atlas folder (UV grid + missing lookups)."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .paths import repo_root
from .validators.atlas_meta import _required_lookup_keys, validate_atlas_meta_v2

MCP_ATLAS_BRIEF_WITNESS = "debug_runs/mcp_atlas_brief_001_live.json"

PILOT_V1_FOLDER = "assets/staging/tiles/tile_rowhouse_victorian_pilot_v1"
PRODUCTION_V2_FOLDER = "assets/staging/tiles/tile_warehouse_industrial_v2_minimum_g4"

_PLAIN_BY_SIGNATURE: dict[str, tuple[str, str, str]] = {
    "atlas_meta_v2_version": (
        "Atlas meta must be schema version 2 (v1 greybox is frozen).",
        "Regenerate meta with current tilemapgen",
        "ATL_SCHEMA_V2",
    ),
    "atlas_meta_v2_lookup_incomplete": (
        "Some variant/facing/frame cells are missing from lookups — bake or pack before register.",
        "Re-run tile batch + pack",
        "ATL_LOOKUP_GAP",
    ),
    "atlas_meta_v2_facings": (
        "render_contract.facings must be 4 or 8 so tile lookup matches the rig.",
        "Set facings to 4 or 8 in meta or rebake",
        "ATL_FACINGS",
    ),
    "atlas_meta_v2_parse": (
        "Could not read atlas_meta.json — check the file exists and is valid JSON.",
        "Open folder and re-run Pack",
        "ATL_PARSE",
    ),
    "atlas_meta_v2_jsonschema": (
        "Atlas meta is missing required fields — compare with a known-good pilot meta.",
        "See tile_warehouse_industrial_west_pilot_v1",
        "ATL_SCHEMA_FIELDS",
    ),
}

_V1_FROZEN = (
    "Atlas meta is schema v1 (greybox frozen) — production requires atlas_meta v2.",
    "Re-run tile batch + pack with v2 meta",
    "ATL_SCHEMA_V1_FROZEN",
)


def _resolve_folder(path: str | Path) -> Path:
    p = Path(path)
    if not p.is_absolute():
        p = repo_root() / p
    return p.resolve()


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


def atlas_meta_brief(
    atlas_folder: str | Path,
    *,
    batch_id: str | None = None,
) -> dict[str, Any]:
    """Artist-facing atlas folder summary — not full meta JSON."""
    folder = _resolve_folder(atlas_folder)
    if not folder.is_dir():
        return {
            "schema": "atlas_meta_brief_v1",
            "ok": False,
            "atlas_folder": str(atlas_folder),
            "error": f"Not a directory: {folder}",
            "artist_messages": [
                {
                    "sentence": "Atlas folder not found.",
                    "fix": "Check path or run tile batch first.",
                    "legend_code": "ATL_FOLDER_MISSING",
                }
            ],
            "plain_language_count": 1,
            "hint": "APS Atlas tab inline QC — not modal",
        }

    meta_path = folder / "atlas_meta.json"
    if not meta_path.is_file():
        return {
            "schema": "atlas_meta_brief_v1",
            "ok": False,
            "atlas_folder": _rel(folder),
            "atlas_meta_schema": None,
            "artist_messages": [
                {
                    "sentence": "No atlas_meta.json in this folder — run Pack atlas first.",
                    "fix": "Run tile-atlas-pack on keyframe PNG folder",
                    "legend_code": "ATL_META_MISSING",
                }
            ],
            "plain_language_count": 1,
            "hint": "APS Atlas tab inline QC — not modal",
        }

    try:
        data = json.loads(meta_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        sentence, fix, code = _PLAIN_BY_SIGNATURE["atlas_meta_v2_parse"]
        return {
            "schema": "atlas_meta_brief_v1",
            "ok": False,
            "atlas_folder": _rel(folder),
            "artist_messages": [{"sentence": sentence, "fix": fix, "legend_code": code}],
            "plain_language_count": 1,
            "hint": "APS Atlas tab inline QC — not modal",
        }

    schema_ver = int(data.get("schema_version") or 1)
    cols = int(data.get("columns") or 0)
    rows = int(data.get("rows") or 0)
    tile_px = int(data.get("tile_px") or 128)
    bid = batch_id or str(data.get("batch_id") or folder.name)
    uv_grid_summary = f"{cols}×{rows} grid · {tile_px}px cells" if cols and rows else "UV grid unavailable"

    if schema_ver != 2:
        sentence, fix, code = _V1_FROZEN
        return {
            "schema": "atlas_meta_brief_v1",
            "ok": False,
            "atlas_folder": _rel(folder),
            "batch_id": bid,
            "atlas_meta_schema": "v1",
            "facings": int((data.get("render_contract") or {}).get("facings") or 0) or None,
            "cells_expected": cols * rows if cols and rows else None,
            "cells_present": len(data.get("tiles") or data.get("lookups") or []),
            "missing_lookups": [],
            "uv_grid_summary": uv_grid_summary,
            "artist_messages": [{"sentence": sentence, "fix": fix, "legend_code": code}],
            "plain_language_count": 1,
            "hint": "APS Atlas tab inline QC — not modal",
        }

    report = validate_atlas_meta_v2(meta_path)
    render = data.get("render_contract") or {}
    facings = int(render.get("facings") or 0)
    lookups = data.get("lookups") or []
    present: set[tuple[str, int, int]] = set()
    for row in lookups:
        if not isinstance(row, dict):
            continue
        present.add(
            (
                str(row.get("variant") or ""),
                int(row.get("facing") or 0),
                int(row.get("frame") or 0),
            )
        )

    visual: dict[str, Any] | None = None
    vc_rel = str(data.get("visual_config") or "")
    if vc_rel:
        vc_path = repo_root() / vc_rel.replace("\\", "/")
        if vc_path.is_file():
            from .validators.visual_config import load_visual_config

            visual = load_visual_config(vc_path)

    minimum_g4 = bool(data.get("minimum_g4_ship")) or str(data.get("lookup_mode") or "") == "minimum_g4"
    cells_expected = 0
    missing_labels: list[str] = []
    if visual and facings in (4, 8):
        required = _required_lookup_keys(visual, facings=facings, minimum_g4=minimum_g4)
        cells_expected = len(required)
        missing = required - present
        missing_labels = _lookup_labels(missing)
    else:
        cells_expected = cols * rows if cols and rows else len(lookups)

    artist_messages: list[dict[str, str]] = []
    for issue in report.errors:
        if issue.severity != "error":
            continue
        sig = str(issue.signature or "")
        tpl = _PLAIN_BY_SIGNATURE.get(sig)
        if tpl:
            sentence, fix, code = tpl
        else:
            sentence = "Validation failed — see atlas_meta.json and log."
            fix = issue.hint or "Review atlas_meta.json"
            code = "ATL_UNKNOWN"
        artist_messages.append(
            {
                "sentence": sentence,
                "fix": fix,
                "legend_code": code,
                "signature": sig or issue.kind,
            }
        )

    if missing_labels and not any(m.get("legend_code") == "ATL_LOOKUP_GAP" for m in artist_messages):
        artist_messages.insert(
            0,
            {
                "sentence": f"{len(missing_labels)} facing or frame slots are missing from the atlas.",
                "fix": "Re-run tile batch for states: " + ", ".join(missing_labels[:4]),
                "legend_code": "ATL_LOOKUP_GAP",
            },
        )

    ok = report.status == "passed" and not missing_labels
    if ok and not artist_messages:
        artist_messages = [
            {
                "sentence": "Atlas meta looks complete — safe to proceed toward tile-atlas-register.",
                "fix": "APS Atlas tab — validate before register",
                "legend_code": "ATL_PASS",
            }
        ]

    return {
        "schema": "atlas_meta_brief_v1",
        "ok": ok,
        "atlas_folder": _rel(folder),
        "batch_id": bid,
        "atlas_meta_schema": "v2",
        "facings": facings or None,
        "cells_expected": cells_expected,
        "cells_present": len(lookups),
        "missing_lookups": missing_labels[:12],
        "uv_grid_summary": uv_grid_summary,
        "artist_messages": artist_messages[:8],
        "plain_language_count": len(artist_messages),
        "hint": "APS Atlas tab inline QC — not modal",
    }


def _rel(path: Path) -> str:
    try:
        return str(path.relative_to(repo_root())).replace("\\", "/")
    except ValueError:
        return str(path)


def refresh_mcp_atlas_brief_witness() -> bool:
    pilot = atlas_meta_brief(PILOT_V1_FOLDER)
    production = atlas_meta_brief(PRODUCTION_V2_FOLDER)
    green = bool(
        pilot.get("ok") is False
        and pilot.get("atlas_meta_schema") == "v1"
        and production.get("ok") is True
        and production.get("atlas_meta_schema") == "v2"
    )
    payload = {
        "gate_id": "MCP-ATLAS-BRIEF-001",
        "ok": green,
        "green": green,
        "pilot_v1_folder": PILOT_V1_FOLDER,
        "pilot_ok": pilot.get("ok"),
        "pilot_schema": pilot.get("atlas_meta_schema"),
        "production_v2_folder": PRODUCTION_V2_FOLDER,
        "production_ok": production.get("ok"),
        "production_schema": production.get("atlas_meta_schema"),
        "pilot_plain_count": pilot.get("plain_language_count"),
        "production_facings": production.get("facings"),
        "production_cells": production.get("cells_present"),
    }
    out = repo_root() / MCP_ATLAS_BRIEF_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return green
