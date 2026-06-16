"""MCP-P2-HONEST-BAKE-001 — reject headless / ortho smoke as ship art before G4."""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any

from .building_definition import MINIMUM_G4_CELLS
from .paths import repo_root
from .tile_pipeline import tile_dry_run_enabled
from .validators.report import ValidationIssue, ValidationReport

MCP_P2_HONEST_BAKE_WITNESS = "debug_runs/mcp_p2_honest_bake_001_live.json"
EXAMPLE_BATCH = "tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_production_v1.json"


def _resolve(path: str | Path) -> Path:
    p = Path(path)
    if not p.is_absolute():
        p = repo_root() / p
    return p.resolve()


def staging_for_batch(batch: dict[str, Any], batch_path: Path) -> Path:
    batch_id = str(batch.get("batch_id") or batch_path.stem.removeprefix("tile_batch_"))
    explicit = str(batch.get("staging_dir") or "").strip()
    if explicit:
        return _resolve(explicit)
    return repo_root() / "assets" / "staging" / "tiles" / batch_id


def tile_promotion_honest_check(
    *,
    batch_path: Path,
    staging: Path | None = None,
    ship: bool = True,
    honest_bake: bool = True,
) -> dict[str, Any]:
    """Pre-G4 honest bake gate — hard fail on ortho/dry-run/invalid bake_source for ship."""
    warnings: list[str] = []
    errors: list[str] = []
    batch_path = _resolve(batch_path)

    if not honest_bake:
        return {"ok": True, "warnings": ["honest_bake disabled"], "errors": []}

    if ship and tile_dry_run_enabled():
        errors.append("RUST_ENGINE_TILE_DRY_RUN is set — cannot ship ortho smoke bake.")

    if ship and os.environ.get("RUST_ENGINE_TILE_KEYFRAME_HEADLESS") == "1":
        errors.append(
            "RUST_ENGINE_TILE_KEYFRAME_HEADLESS is set — headless keyframe is CI/schema only, not ship art."
        )

    batch: dict[str, Any] = {}
    try:
        batch = json.loads(batch_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        errors.append(f"Could not read tile batch JSON: {exc}")
        return {"ok": False, "artist_message": errors[0], "warnings": warnings, "errors": errors}

    stage = staging if staging is not None else staging_for_batch(batch, batch_path)
    bake_source = str(batch.get("bake_source") or "")
    render_method = str((batch.get("render") or {}).get("method") or "")

    if ship and bake_source != "keyframe_pack":
        errors.append(f"bake_source must be keyframe_pack for ship (got {bake_source!r}).")

    if ship and render_method == "blender_orthographic_iso":
        errors.append(
            "render.method blender_orthographic_iso is smoke/CI only — ship requires manual keyframe_render + G4."
        )

    if ship and stage.is_dir():
        pngs = sorted(stage.glob("*.png"))
        if len(pngs) < MINIMUM_G4_CELLS:
            errors.append(
                f"Keyframe folder has {len(pngs)} PNGs; need {MINIMUM_G4_CELLS} for ship bake."
            )

    meta_path = stage / "atlas_meta.json"
    if meta_path.is_file():
        try:
            meta = json.loads(meta_path.read_text(encoding="utf-8"))
            if ship and int(meta.get("schema_version") or 1) < 2:
                errors.append("atlas_meta schema v2 required for ship.")
        except (OSError, json.JSONDecodeError):
            warnings.append("atlas_meta.json unreadable for honest check.")
    elif ship:
        warnings.append("atlas_meta.json missing — defer full atlas validate until pack step.")

    if errors:
        return {
            "ok": False,
            "artist_message": errors[0],
            "warnings": warnings,
            "errors": errors,
            "batch_path": str(batch_path),
            "staging": str(stage),
        }
    return {
        "ok": True,
        "warnings": warnings or ["honest bake checks passed"],
        "errors": [],
        "batch_path": str(batch_path),
        "staging": str(stage),
        "bake_source": bake_source,
        "render_method": render_method,
    }


def validate_tile_promotion_honest_path(
    batch_path: Path,
    *,
    ship: bool = True,
    honest_bake: bool = True,
    compression_level: int = 3,
) -> ValidationReport:
    check = tile_promotion_honest_check(
        batch_path=_resolve(batch_path),
        ship=ship,
        honest_bake=honest_bake,
    )
    issues: list[ValidationIssue] = []
    for msg in check.get("errors") or []:
        issues.append(
            ValidationIssue(
                kind="HonestBakeViolation",
                severity="error",
                file=str(batch_path),
                hint=str(msg),
                signature="tile_promotion_honest_error",
            )
        )
    for msg in check.get("warnings") or []:
        if check.get("ok"):
            issues.append(
                ValidationIssue(
                    kind="HonestBakeWarn",
                    severity="warning",
                    file=str(batch_path),
                    hint=str(msg),
                    signature="tile_promotion_honest_warn",
                )
            )
    status = "passed" if check.get("ok") else "failed"
    summary = (
        f"tile_promotion_honest: {check.get('bake_source') or 'batch'}"
        if status == "passed"
        else f"tile_promotion_honest blocked: {check.get('artist_message', 'failed')}"
    )
    return ValidationReport(
        validator="tile",
        status=status,  # type: ignore[arg-type]
        compression_level=compression_level,
        summary=summary,
        error_count=sum(1 for i in issues if i.severity == "error"),
        warning_count=sum(1 for i in issues if i.severity == "warning"),
        errors=issues,
    ).compress(compression_level)


def write_mcp_p2_honest_bake_001_witness() -> dict[str, Any]:
    batch = repo_root() / EXAMPLE_BATCH
    smoke_batch = {
        "batch_id": "test_smoke_ortho",
        "bake_source": "smoke_ortho_headless",
        "render": {"method": "blender_orthographic_iso", "seed": 1},
    }
    smoke_path = repo_root() / "assets/staging/tiles/_honest_witness_smoke.json"
    smoke_path.parent.mkdir(parents=True, exist_ok=True)
    smoke_path.write_text(json.dumps(smoke_batch, indent=2) + "\n", encoding="utf-8")

    production_check = tile_promotion_honest_check(batch_path=batch, ship=True, honest_bake=True)
    smoke_check = tile_promotion_honest_check(batch_path=smoke_path, ship=True, honest_bake=True)

    prev_dry = os.environ.get("RUST_ENGINE_TILE_DRY_RUN")
    os.environ["RUST_ENGINE_TILE_DRY_RUN"] = "1"
    try:
        dry_check = tile_promotion_honest_check(batch_path=batch, ship=True, honest_bake=True)
    finally:
        if prev_dry is None:
            os.environ.pop("RUST_ENGINE_TILE_DRY_RUN", None)
        else:
            os.environ["RUST_ENGINE_TILE_DRY_RUN"] = prev_dry

    prod_policy_errors = [
        e
        for e in (production_check.get("errors") or [])
        if any(k in e for k in ("keyframe_pack", "orthographic", "DRY_RUN", "HEADLESS"))
    ]
    body: dict[str, Any] = {
        "gate_id": "MCP-P2-HONEST-BAKE-001",
        "ok": (
            not smoke_check.get("ok")
            and not dry_check.get("ok")
            and len(prod_policy_errors) == 0
        ),
        "green": (
            not smoke_check.get("ok")
            and not dry_check.get("ok")
            and len(prod_policy_errors) == 0
        ),
        "checks": {
            "production_keyframe_pack": production_check,
            "smoke_ortho_rejected": smoke_check,
            "dry_run_rejected": dry_check,
        },
        "cli": "validate-report tile_promotion_honest <tile_batch.json>",
        "spine_hook": "tile_spine_run honest_bake step",
    }
    out = repo_root() / MCP_P2_HONEST_BAKE_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body
