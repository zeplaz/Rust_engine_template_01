"""GRAMMAR-GEN-VERIFY-001 (P0) — block bad grammar assemblies before bake/preview ship."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

from .assembly_production import validate_assembly_snapshot
from .report import ValidationIssue, ValidationReport

GATE_ID = "GRAMMAR-GEN-VERIFY-001"

# Industrial warehouse pilot minimum readable footprint (not a thin fence).
MIN_WAREHOUSE_WIDTH = 4
MIN_WAREHOUSE_DEPTH = 3
MIN_FOOTPRINT_WIDTH = 3
MIN_FOOTPRINT_DEPTH = 3

WALL_MODULE_HINTS = ("wall",)
ROOF_MODULE_HINTS = ("roof",)
CORNER_MODULE_HINTS = ("corner",)


def _perimeter_cell_count(width: int, depth: int, floors: int) -> int:
    width = max(2, width)
    depth = max(2, depth)
    floors = max(1, floors)
    ring = 2 * (width + depth) - 4
    return ring * floors + ring


def _index_by_job_id() -> dict[str, dict[str, Any]]:
    from rust_engine_mcp.library import load_index_json

    out: dict[str, dict[str, Any]] = {}
    for row in load_index_json():
        job = str(row.get("job_id") or "")
        if job:
            out[job] = row
    return out


def _module_category(module_id: str) -> str:
    low = module_id.lower()
    if "roof" in low:
        return "roof"
    if "corner" in low:
        return "corner"
    if "door" in low:
        return "door"
    if "wall" in low:
        return "wall"
    return "other"


def validate_assembly_grammar_verify(
    snapshot: dict[str, Any],
    *,
    snapshot_path: str = "",
    ship: bool = True,
    compression_level: int = 3,
) -> ValidationReport:
    """P0 silhouette + style-pack + footprint coherence gate."""
    issues: list[ValidationIssue] = []
    placements = list(snapshot.get("module_placements") or [])
    fp = snapshot.get("footprint") or {}
    width = int(fp.get("width") or 0)
    depth = int(fp.get("depth") or 0)
    floors = int(fp.get("floors") or 1)
    style_pack = str(snapshot.get("style_pack_id") or "")
    ref_tags = [str(t) for t in snapshot.get("reference_tags") or []]
    rules_ver = str(snapshot.get("procedural_rules_version") or "")

    if width < MIN_FOOTPRINT_WIDTH or depth < MIN_FOOTPRINT_DEPTH:
        issues.append(
            ValidationIssue(
                kind="FootprintTooSmall",
                severity="error",
                file=snapshot_path,
                field="footprint",
                hint=(
                    f"footprint {width}x{depth} too small — minimum {MIN_FOOTPRINT_WIDTH}x{MIN_FOOTPRINT_DEPTH} "
                    "for a readable building shell"
                ),
                signature="grammar_verify_footprint_min",
            )
        )

    is_warehouse = any("IndustrialWarehouse" in t for t in ref_tags) or any(
        "industrial_warehouse" in t for t in ref_tags
    )
    if ship and is_warehouse and (width < MIN_WAREHOUSE_WIDTH or depth < MIN_WAREHOUSE_DEPTH):
        issues.append(
            ValidationIssue(
                kind="WarehouseFootprintThin",
                severity="error",
                file=snapshot_path,
                field="footprint",
                hint=(
                    f"warehouse footprint {width}x{depth} — need at least "
                    f"{MIN_WAREHOUSE_WIDTH}x{MIN_WAREHOUSE_DEPTH} for hall massing (4x2 reads as janky fence)"
                ),
                signature="grammar_verify_warehouse_footprint",
            )
        )

    expected = _perimeter_cell_count(width, depth, floors)
    if placements and len(placements) < int(expected * 0.85):
        issues.append(
            ValidationIssue(
                kind="PerimeterIncomplete",
                severity="error",
                file=snapshot_path,
                field="module_placements",
                hint=(
                    f"expected ~{expected} perimeter/roof placements for {width}x{depth}x{floors}, "
                    f"got {len(placements)} — hollow or incomplete shell"
                ),
                signature="grammar_verify_perimeter_count",
            )
        )

    categories = {_module_category(str(p.get("module_id") or "")) for p in placements}
    if ship and "wall" not in categories:
        issues.append(
            ValidationIssue(
                kind="MissingWallModule",
                severity="error",
                file=snapshot_path,
                hint="no wall module in placements — not a building shell",
                signature="grammar_verify_missing_wall",
            )
        )
    if ship and "roof" not in categories:
        issues.append(
            ValidationIssue(
                kind="MissingRoofModule",
                severity="error",
                file=snapshot_path,
                hint="no roof module in placements — open slab stack",
                signature="grammar_verify_missing_roof",
            )
        )

    if rules_ver == "building_grammar_v1":
        chain = snapshot.get("grammar_rule_chain")
        chain_ok = False
        if isinstance(chain, list) and len(chain) >= 4:
            chain_ok = True
        elif isinstance(chain, dict) and len(chain) >= 4:
            chain_ok = True
        elif any(str(t).startswith("chain:") for t in ref_tags):
            chain_ok = True
        if not chain_ok:
            issues.append(
                ValidationIssue(
                    kind="GrammarChainMissing",
                    severity="error",
                    file=snapshot_path,
                    field="grammar_rule_chain",
                    hint="building_grammar_v1 snapshot requires grammar_rule_chain (list or dict) or chain: reference_tags",
                    signature="grammar_verify_grammar_chain",
                )
            )

    if ship and style_pack:
        by_job = _index_by_job_id()
        drift: list[str] = []
        for p in placements:
            job = str(p.get("job_id") or "")
            module_id = str(p.get("module_id") or "")
            row = by_job.get(job)
            if not row:
                continue
            row_pack = str(row.get("style_pack") or "")
            if row_pack and row_pack != style_pack:
                drift.append(f"{module_id}({job}→{row_pack})")
        if drift:
            issues.append(
                ValidationIssue(
                    kind="StylePackDrift",
                    severity="error",
                    file=snapshot_path,
                    field="style_pack_id",
                    hint=(
                        f"{len(drift)} module(s) from wrong style pack for {style_pack}: "
                        + ", ".join(drift[:6])
                        + ("…" if len(drift) > 6 else "")
                    ),
                    signature="grammar_verify_style_pack_drift",
                )
            )

    errors = [i for i in issues if i.severity == "error"]
    status = "failed" if errors else "passed"
    return ValidationReport(
        validator="assembly_grammar",
        status=status,
        compression_level=compression_level,
        summary=f"{GATE_ID}: {len(errors)} error(s) footprint={width}x{depth}x{floors} placements={len(placements)}",
        error_count=len(errors),
        errors=issues,
    )


def validate_assembly_p0_gate(
    snapshot: dict[str, Any],
    *,
    snapshot_path: str = "",
    ship: bool = True,
    compression_level: int = 3,
) -> ValidationReport:
    """Production GLB/material gate + P0 grammar verify (single report)."""
    prod = validate_assembly_snapshot(
        snapshot, snapshot_path=snapshot_path, ship=ship, compression_level=compression_level
    )
    gram = validate_assembly_grammar_verify(
        snapshot, snapshot_path=snapshot_path, ship=ship, compression_level=compression_level
    )
    issues = list(prod.errors) + list(gram.errors)
    errors = [i for i in issues if i.severity == "error"]
    status = "failed" if errors else "passed"
    return ValidationReport(
        validator="assembly_p0",
        status=status,
        compression_level=compression_level,
        summary=f"P0 gate: production={prod.status} grammar={gram.status}",
        error_count=len(errors),
        errors=issues,
    )


def validate_assembly_grammar_verify_path(
    path: str | Path,
    *,
    ship: bool = True,
    compression_level: int = 3,
    full_p0: bool = False,
) -> ValidationReport:
    p = Path(path)
    if not p.is_file():
        return ValidationReport(
            validator="assembly_grammar",
            status="failed",
            compression_level=compression_level,
            summary="snapshot not found",
            error_count=1,
            errors=[
                ValidationIssue(
                    kind="MissingFile",
                    severity="error",
                    file=str(p),
                    signature="grammar_verify_snapshot_missing",
                )
            ],
        )
    snap = json.loads(p.read_text(encoding="utf-8"))
    try:
        rel = str(p.relative_to(repo_root())).replace("\\", "/")
    except ValueError:
        rel = str(p)
    if full_p0:
        return validate_assembly_p0_gate(snap, snapshot_path=rel, ship=ship, compression_level=compression_level)
    return validate_assembly_grammar_verify(
        snap, snapshot_path=rel, ship=ship, compression_level=compression_level
    )
