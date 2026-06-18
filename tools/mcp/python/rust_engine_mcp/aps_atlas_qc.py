"""APS-ATLAS-PREVIEW-002 — plain-language atlas meta validation."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .paths import repo_root
from .validators.atlas_meta import validate_atlas_meta_v2
from .validators.report import ValidationReport

APS_ATLAS_PREVIEW_WITNESS = "debug_runs/aps_atlas_preview_002_live.json"
ATL_SIGN_001_WITNESS = "debug_runs/atl_sign_001_live.json"

# ATL-SIGN-001 — production v2 path (not pilot v1 greybox)
PRODUCTION_V2_ATLAS_FOLDER = "assets/staging/tiles/tile_warehouse_industrial_v2_minimum_g4"
PILOT_V1_ATLAS_FOLDER = "assets/staging/tiles/tile_warehouse_industrial_west_pilot_v1"

APS_STATUS_PASS = "#0a6b0a"
APS_STATUS_FAIL = "#a00000"

_PLAIN: dict[str, str] = {
    "atlas_meta_v2_parse": "Could not read atlas_meta.json — check the file exists and is valid JSON.",
    "atlas_meta_v2_version": "Atlas meta must be schema version 2 (v1 greybox is frozen).",
    "atlas_meta_v2_jsonschema": "Atlas meta is missing required fields — compare with a known-good pilot meta.",
    "atlas_meta_v2_facings": "render_contract.facings must be 4 or 8 so tile lookup matches the rig.",
    "atlas_meta_v2_lookup_incomplete": "Some variant/facing/frame cells are missing from lookups — bake or pack before register.",
}


def plain_language_lines(report: ValidationReport) -> list[str]:
    if report.status == "passed":
        return ["Atlas meta looks complete — safe to proceed toward tile-atlas-register."]
    lines: list[str] = []
    for issue in report.errors:
        sig = issue.signature or issue.kind
        sentence = _PLAIN.get(sig, issue.hint or issue.kind)
        if sig not in _PLAIN and issue.field:
            sentence = f"{sentence} (field: {issue.field})"
        elif sig in _PLAIN and issue.field and issue.field not in sentence:
            sentence = f"{sentence} (field: {issue.field})"
        lines.append(sentence)
    if not lines:
        lines.append(report.summary or "Validation failed — see atlas_meta.json and log.")
    return lines


def format_atlas_qc_display(
    report: ValidationReport | None,
    lines: list[str],
    *,
    meta: dict[str, Any] | None = None,
    atlas_domain: str = "buildings",
) -> tuple[str, str]:
    """Return panel text + foreground color (PASS/FAIL prefix, not color-only)."""
    domain_label = "Landscape" if str(atlas_domain).lower() == "landscape" else "Buildings"
    if report is not None and report.status == "passed":
        detail = lines[0] if lines else "Atlas meta looks complete — safe to proceed toward tile-atlas-register."
        extra = ""
        if meta:
            cols = int(meta.get("columns") or 0)
            rows = int(meta.get("rows") or 0)
            n = len(meta.get("tiles") or [])
            if cols > 0 and rows > 0:
                extra = f" Grid {cols}×{rows} · {n} cells indexed · facings OK"
        register = (
            "_landscape_atlas_index"
            if str(atlas_domain).lower() == "landscape"
            else "_tile_atlas_index"
        )
        return (
            f"✓ valid — [{domain_label} → {register}] {detail}{extra}",
            APS_STATUS_PASS,
        )
    body = " · ".join(lines[:4]) if lines else "Validation failed — see atlas_meta.json."
    return f"✗ blocked — [{domain_label}] {body}", APS_STATUS_FAIL


def validate_atlas_folder(folder: Path) -> tuple[ValidationReport | None, list[str]]:
    meta_path = folder / "atlas_meta.json"
    if not meta_path.is_file():
        return None, ["No atlas_meta.json in this folder — run Pack atlas first."]
    report = validate_atlas_meta_v2(meta_path)
    return report, plain_language_lines(report)


def write_aps_atlas_preview_witness(folder: Path | None = None) -> Path:
    default = repo_root() / PRODUCTION_V2_ATLAS_FOLDER
    path = folder if folder and folder.is_dir() else default
    report, sentences = validate_atlas_folder(path) if path.is_dir() else (None, ["atlas folder missing"])
    grid_ok = False
    schema_version: int | None = None
    facings: int | None = None
    if path.is_dir():
        meta_path = path / "atlas_meta.json"
        if meta_path.is_file():
            try:
                meta = json.loads(meta_path.read_text(encoding="utf-8"))
                cols = int(meta.get("columns") or 0)
                rows = int(meta.get("rows") or 0)
                grid_ok = cols > 0 and rows > 0
                schema_version = int(meta.get("schema_version") or 0)
                facings = int((meta.get("render_contract") or {}).get("facings") or 0) or None
            except (OSError, json.JSONDecodeError, TypeError, ValueError):
                pass
    rel_folder = str(path.relative_to(repo_root())).replace("\\", "/") if path.is_dir() else None
    green = bool(report and report.status == "passed") and grid_ok and schema_version == 2
    out = repo_root() / APS_ATLAS_PREVIEW_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    body: dict[str, Any] = {
        "program_id": "APS-ATLAS-PREVIEW-002",
        "gate_id": "ATL-SIGN-001",
        "green": green,
        "folder": rel_folder,
        "atlas_meta_schema": f"v{schema_version}" if schema_version else None,
        "facings": facings,
        "validation_status": report.status if report else "skipped",
        "plain_language": sentences[:12],
        "uv_grid_overlay": grid_ok,
        "panel": "art_pipeline_suite.atlas_preview_panel.AtlasPreviewPanel",
    }
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return out


def refresh_atl_sign_001_witness() -> bool:
    """ATL-SIGN-001 — production atlas preview green on v2 folder + atlas_meta_brief."""
    from . import atlas_meta_brief

    preview_path = write_aps_atlas_preview_witness()
    preview = json.loads(preview_path.read_text(encoding="utf-8"))
    brief = atlas_meta_brief.atlas_meta_brief(PRODUCTION_V2_ATLAS_FOLDER)
    atlas_brief_path = repo_root() / atlas_meta_brief.MCP_ATLAS_BRIEF_WITNESS
    atlas_brief_body = (
        json.loads(atlas_brief_path.read_text(encoding="utf-8"))
        if atlas_brief_path.is_file()
        else {}
    )
    green = bool(
        preview.get("green")
        and preview.get("folder") == PRODUCTION_V2_ATLAS_FOLDER
        and preview.get("atlas_meta_schema") == "v2"
        and brief.get("ok")
        and atlas_brief_body.get("production_ok")
    )
    payload = {
        "gate_id": "ATL-SIGN-001",
        "ok": green,
        "green": green,
        "production_folder": PRODUCTION_V2_ATLAS_FOLDER,
        "pilot_folder": PILOT_V1_ATLAS_FOLDER,
        "aps_atlas_preview_002": preview,
        "atlas_meta_brief_production_ok": brief.get("ok"),
        "mcp_atlas_brief_001_green": atlas_brief_body.get("green"),
        "atl_star_criteria": "aps_atlas_preview_002 green + mcp_atlas_brief production v2 pass",
    }
    out = repo_root() / ATL_SIGN_001_WITNESS
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return green
