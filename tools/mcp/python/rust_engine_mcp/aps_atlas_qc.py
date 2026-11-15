"""APS-ATLAS-PREVIEW-002 — plain-language atlas meta validation."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .paths import repo_root
from .validators.atlas_meta import validate_atlas_meta_v2
from .validators.report import ValidationIssue, ValidationReport

try:
    from PIL import Image
except ImportError:  # pragma: no cover
    Image = None  # type: ignore[misc, assignment]

APS_ATLAS_PREVIEW_WITNESS = "debug_runs/aps_atlas_preview_002_live.json"
ATL_SIGN_001_WITNESS = "debug_runs/atl_sign_001_live.json"

# ATL-SIGN-001 — production v2 path (not pilot v1 greybox)
PRODUCTION_V2_ATLAS_FOLDER = "assets/staging/tiles/tile_warehouse_industrial_v2_minimum_g4"
PILOT_V1_ATLAS_FOLDER = "assets/staging/tiles/tile_warehouse_industrial_west_pilot_v1"

APS_STATUS_PASS = "#0a6b0a"
APS_STATUS_FAIL = "#a00000"

TEXEL_DENSITY_TOLERANCE = 0.05

_PLAIN: dict[str, str] = {
    "atlas_meta_v2_parse": "Could not read atlas_meta.json — check the file exists and is valid JSON.",
    "atlas_meta_v2_version": "Atlas meta must be schema version 2 (v1 greybox is frozen).",
    "atlas_meta_v2_jsonschema": "Atlas meta is missing required fields — compare with a known-good pilot meta.",
    "atlas_meta_v2_facings": "render_contract.facings must be 4 or 8 so tile lookup matches the rig.",
    "atlas_meta_v2_lookup_incomplete": "Some variant/facing/frame cells are missing from lookups — bake or pack before register.",
    "atlas_qc_gutter_too_small": (
        "Tile gutter is too thin for the mip chain — widen padding to at least 2^(mip levels) px "
        "on each side before pack."
    ),
    "atlas_qc_gutter_not_extruded": (
        "Gutter pixels do not repeat the tile edge colour — re-pack with edge extrusion (bleed) "
        "so zoom-out does not show seams."
    ),
    "atlas_qc_texel_density_drift": (
        "Texels per metre drift more than 5% within this zoom band — match world scale or tile_px "
        "across variants."
    ),
    "atlas_qc_png_missing": "Atlas PNG path from meta is missing on disk — bake or pack before bleed QC.",
    "atlas_qc_png_read": "Could not read the atlas PNG for bleed QC — check the file is a valid image.",
}


def min_gutter_for_mip_levels(mip_levels: int) -> int:
    """RND-08 — minimum gutter per side for a given mip chain depth."""
    levels = max(0, int(mip_levels))
    return 2**levels


@dataclass(frozen=True)
class AtlasCellLayout:
    columns: int
    rows: int
    cell_w: int
    cell_h: int
    content_px: int
    gutter_px: int


def atlas_cell_layout(meta: dict[str, Any], image_size: tuple[int, int]) -> AtlasCellLayout:
    img_w, img_h = image_size
    cols = max(1, int(meta.get("columns") or 1))
    rows = max(1, int(meta.get("rows") or 1))
    if img_w % cols or img_h % rows:
        raise ValueError("atlas image size is not divisible by columns×rows")
    cell_w = img_w // cols
    cell_h = img_h // rows
    render = meta.get("render_contract") or {}
    content_px = int(
        meta.get("tile_px")
        or render.get("tile_px")
        or meta.get("content_px")
        or min(cell_w, cell_h)
    )
    gutter_px = max(0, (cell_w - content_px) // 2)
    return AtlasCellLayout(cols, rows, cell_w, cell_h, content_px, gutter_px)


def _atlas_qc_block(meta: dict[str, Any]) -> dict[str, Any]:
    block = meta.get("atlas_qc")
    return block if isinstance(block, dict) else {}


def mip_levels_for_meta(meta: dict[str, Any]) -> int | None:
    qc = _atlas_qc_block(meta)
    for src in (qc, meta.get("render_contract") or {}, meta):
        if isinstance(src, dict) and src.get("mip_levels") is not None:
            return int(src["mip_levels"])
    return None


def bleed_qc_requested(meta: dict[str, Any], layout: AtlasCellLayout) -> bool:
    qc = _atlas_qc_block(meta)
    if qc.get("check_bleed") is False:
        return False
    if qc.get("check_bleed") is True or mip_levels_for_meta(meta) is not None:
        return True
    padding = int(meta.get("padding_px") or qc.get("padding_px") or qc.get("gutter_px") or 0)
    if padding > 0:
        return True
    return layout.gutter_px > 0 or layout.cell_w > layout.content_px


def zoom_band_from_meta(meta: dict[str, Any]) -> str | None:
    qc = _atlas_qc_block(meta)
    for src in (qc, meta.get("render_contract") or {}, meta):
        if isinstance(src, dict) and src.get("zoom_band"):
            return str(src["zoom_band"])
    return None


def texel_density_qc_requested(meta: dict[str, Any]) -> bool:
    qc = _atlas_qc_block(meta)
    if qc.get("check_texel_density") is False:
        return False
    if qc.get("check_texel_density") is True:
        return True
    if zoom_band_from_meta(meta):
        return True
    lookups = meta.get("lookups") or []
    worlds = {
        float(row["world_metres_per_tile"])
        for row in lookups
        if isinstance(row, dict) and row.get("world_metres_per_tile") is not None
    }
    return len(worlds) > 1


def default_world_metres(meta: dict[str, Any]) -> float | None:
    qc = _atlas_qc_block(meta)
    for src in (qc, meta.get("render_contract") or {}, meta):
        if isinstance(src, dict) and src.get("world_metres_per_tile") is not None:
            return float(src["world_metres_per_tile"])
    return None


def _rgba_equal(a: tuple[int, ...], b: tuple[int, ...]) -> bool:
    return tuple(a[:4]) == tuple(b[:4])


def check_atlas_gutter_bleed(
    image: Image.Image,
    meta: dict[str, Any],
    *,
    file_label: str = "atlas.png",
) -> list[ValidationIssue]:
    """Verify gutter width and edge extrusion for RND-08."""
    layout = atlas_cell_layout(meta, image.size)
    if not bleed_qc_requested(meta, layout):
        return []

    mip = mip_levels_for_meta(meta)
    if mip is None:
        mip = 0
    min_gutter = min_gutter_for_mip_levels(mip)
    issues: list[ValidationIssue] = []

    if layout.gutter_px < min_gutter:
        issues.append(
            ValidationIssue(
                kind="AtlasBleed",
                severity="error",
                file=file_label,
                field="gutter_px",
                hint=(
                    f"gutter {layout.gutter_px}px < required {min_gutter}px "
                    f"for mip_levels={mip} (2^{mip})"
                ),
                signature="atlas_qc_gutter_too_small",
            )
        )
        return issues

    pixels = image.convert("RGBA")
    cols, rows = layout.columns, layout.rows
    tile_px = layout.content_px
    g = layout.gutter_px
    cell_w, cell_h = layout.cell_w, layout.cell_h

    def bad_extrusion(col: int, row: int) -> bool:
        base_x = col * cell_w
        base_y = row * cell_h
        cx = base_x + g
        cy = base_y + g
        # Left gutter
        if g > 0:
            for dy in range(tile_px):
                edge = pixels.getpixel((cx, cy + dy))
                for dx in range(g):
                    gutter_px = pixels.getpixel((base_x + dx, cy + dy))
                    if not _rgba_equal(edge, gutter_px):
                        return True
        # Right gutter
        if g > 0:
            for dy in range(tile_px):
                edge = pixels.getpixel((cx + tile_px - 1, cy + dy))
                for dx in range(g):
                    gutter_px = pixels.getpixel((cx + tile_px + dx, cy + dy))
                    if not _rgba_equal(edge, gutter_px):
                        return True
        # Top gutter
        if g > 0:
            for dx in range(tile_px):
                edge = pixels.getpixel((cx + dx, cy))
                for dy in range(g):
                    gutter_px = pixels.getpixel((cx + dx, base_y + dy))
                    if not _rgba_equal(edge, gutter_px):
                        return True
        # Bottom gutter
        if g > 0:
            for dx in range(tile_px):
                edge = pixels.getpixel((cx + dx, cy + tile_px - 1))
                for dy in range(g):
                    gutter_px = pixels.getpixel((cx + dx, cy + tile_px + dy))
                    if not _rgba_equal(edge, gutter_px):
                        return True
        return False

    for row in range(rows):
        for col in range(cols):
            if bad_extrusion(col, row):
                issues.append(
                    ValidationIssue(
                        kind="AtlasBleed",
                        severity="error",
                        file=file_label,
                        field="gutter_extrusion",
                        hint=f"cell grid[{col},{row}] gutter is not edge-extruded",
                        signature="atlas_qc_gutter_not_extruded",
                    )
                )
                return issues
    return issues


def check_texel_density_band(meta: dict[str, Any], *, file_label: str = "atlas_meta.json") -> list[ValidationIssue]:
    """Texels per metre must stay within 5% across tiles in one zoom band."""
    if not texel_density_qc_requested(meta):
        return []
    band = zoom_band_from_meta(meta) or "default"
    render = meta.get("render_contract") or {}
    tile_px = int(meta.get("tile_px") or render.get("tile_px") or 0)
    if tile_px <= 0:
        return [
            ValidationIssue(
                kind="AtlasTexelDensity",
                severity="error",
                file=file_label,
                field="tile_px",
                hint="tile_px missing — cannot compute texels per metre",
                signature="atlas_qc_texel_density_drift",
            )
        ]

    default_world = default_world_metres(meta)
    densities: list[float] = []
    lookups = meta.get("lookups") or []
    if lookups:
        for row in lookups:
            if not isinstance(row, dict):
                continue
            world = row.get("world_metres_per_tile", default_world)
            if world is None:
                continue
            world_f = float(world)
            if world_f <= 0:
                continue
            densities.append(tile_px / world_f)
    elif default_world and default_world > 0:
        densities.append(tile_px / default_world)

    if len(densities) < 2:
        return []

    ref = sorted(densities)[len(densities) // 2]
    if ref <= 0:
        return []
    spread = max(abs(d - ref) / ref for d in densities)
    if spread <= TEXEL_DENSITY_TOLERANCE:
        return []

    return [
        ValidationIssue(
            kind="AtlasTexelDensity",
            severity="error",
            file=file_label,
            field="zoom_band",
            hint=(
                f"zoom_band={band}: texel/m spread {spread * 100:.1f}% > "
                f"{TEXEL_DENSITY_TOLERANCE * 100:.0f}% tolerance"
            ),
            signature="atlas_qc_texel_density_drift",
        )
    ]


def validate_atlas_rnd08_qc(
    meta: dict[str, Any],
    image: Image.Image | None,
    *,
    folder: Path | None = None,
) -> list[ValidationIssue]:
    """RND-08 bleed + texel-density checks (PNG optional for density-only)."""
    label = str(folder / "atlas.png") if folder else "atlas.png"
    issues = check_texel_density_band(meta, file_label=label)
    if image is None:
        return issues
    issues.extend(check_atlas_gutter_bleed(image, meta, file_label=label))
    return issues


def _merge_reports(base: ValidationReport, extra_issues: list[ValidationIssue]) -> ValidationReport:
    if not extra_issues:
        return base
    merged = list(base.errors) + extra_issues
    status: str = "failed" if any(i.severity == "error" for i in merged) else base.status
    return ValidationReport(
        validator=base.validator,
        status=status,  # type: ignore[arg-type]
        compression_level=base.compression_level,
        summary=base.summary or "Atlas validation failed.",
        error_count=sum(1 for i in merged if i.severity == "error"),
        warning_count=sum(1 for i in merged if i.severity == "warning"),
        errors=merged,
        known_fixes=base.known_fixes,
        confidence=base.confidence,
    )


def validate_atlas_png_qc(folder: Path, meta: dict[str, Any], meta_path: Path) -> list[ValidationIssue]:
    """Load atlas PNG when needed and run RND-08 checks."""
    issues = check_texel_density_band(meta, file_label=str(meta_path))
    qc = _atlas_qc_block(meta)
    may_need_bleed = qc.get("check_bleed") is not False and (
        qc.get("check_bleed") is True
        or mip_levels_for_meta(meta) is not None
        or int(meta.get("padding_px") or qc.get("padding_px") or 0) > 0
    )
    if not may_need_bleed:
        return issues

    rel_png = str(meta.get("atlas_png") or "atlas.png")
    png_path = folder / Path(rel_png).name
    if not png_path.is_file():
        png_path = repo_root() / rel_png if not Path(rel_png).is_absolute() else Path(rel_png)
    if not png_path.is_file():
        issues.append(
                ValidationIssue(
                    kind="AtlasBleed",
                    severity="error",
                    file=str(meta_path),
                    field="atlas_png",
                    hint=f"missing {rel_png}",
                    signature="atlas_qc_png_missing",
                )
            )
        return issues
    if Image is None:
        return issues + [
            ValidationIssue(
                kind="AtlasBleed",
                severity="error",
                file=str(png_path),
                hint="Pillow not installed",
                signature="atlas_qc_png_read",
            )
        ]
    try:
        with Image.open(png_path) as im:
            image = im.convert("RGBA")
    except OSError as exc:
        return issues + [
            ValidationIssue(
                kind="AtlasBleed",
                severity="error",
                file=str(png_path),
                hint=str(exc),
                signature="atlas_qc_png_read",
            )
        ]
    layout = atlas_cell_layout(meta, image.size)
    if bleed_qc_requested(meta, layout):
        issues.extend(check_atlas_gutter_bleed(image, meta, file_label=str(png_path)))
    return issues


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
            n = len(meta.get("lookups") or meta.get("tiles") or [])
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
    try:
        meta = json.loads(meta_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        meta = {}
    rnd08 = validate_atlas_png_qc(folder, meta, meta_path)
    if rnd08:
        report = _merge_reports(report, rnd08)
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