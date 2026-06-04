"""TILE-FIX-004 — assembly snapshot must reference real production module GLBs (not lod0 cubes)."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

from .report import ValidationIssue, ValidationReport

LOD0_MARKERS = ("kit_lod0", "_lod0_", "/lod0/")
MIN_UNIQUE_MODULES = 3
MIN_PLACEMENTS = 4


def _is_lod0_glb(path: str) -> bool:
    low = path.replace("\\", "/").lower()
    return any(m in low for m in LOD0_MARKERS) or "greybox" in low


def _is_production_glb(path: str) -> bool:
    low = path.replace("\\", "/").lower()
    if "assets/models/modules/" not in low:
        return False
    return (
        "_production_" in low
        or "production_run" in low
        or "_production_run" in low
    )


def validate_assembly_snapshot(
    snapshot: dict[str, Any],
    *,
    snapshot_path: str = "",
    ship: bool = True,
    strict_shell_modules: frozenset[str] | None = None,
    compression_level: int = 3,
) -> ValidationReport:
    issues: list[ValidationIssue] = []
    tier = str(snapshot.get("source_tier") or "").lower()
    if ship and tier != "production":
        issues.append(
            ValidationIssue(
                kind="TierMismatch",
                severity="error",
                file=snapshot_path,
                field="source_tier",
                hint="TILE-FIX-004: ship assembly requires source_tier: production",
                signature="assembly_production_tier",
            )
        )

    placements = list(snapshot.get("module_placements") or [])
    if len(placements) < MIN_PLACEMENTS:
        issues.append(
            ValidationIssue(
                kind="ModuleCount",
                severity="error",
                file=snapshot_path,
                field="module_placements",
                hint=f"assembly needs >= {MIN_PLACEMENTS} module placements (got {len(placements)})",
                signature="assembly_production_min_placements",
            )
        )

    if strict_shell_modules is None and ship:
        try:
            from rust_engine_mcp.building_definition import PRODUCTION_SHELL_MODULE_IDS

            strict_shell_modules = PRODUCTION_SHELL_MODULE_IDS
        except ImportError:
            strict_shell_modules = frozenset()

    unique_jobs: set[str] = set()
    lod0_shell = 0
    missing_glb = 0
    missing_material = 0
    for p in placements:
        module_id = str(p.get("module_id") or "")
        if ship and not str(p.get("material_profile") or "").strip():
            missing_material += 1
        shell_strict = bool(strict_shell_modules and module_id in strict_shell_modules)
        job = str(p.get("job_id") or "")
        if job:
            unique_jobs.add(job)
        glb_rel = str(p.get("glb_path") or "")
        if not glb_rel:
            issues.append(
                ValidationIssue(
                    kind="MissingField",
                    severity="error",
                    file=snapshot_path,
                    field="glb_path",
                    hint="each placement requires glb_path",
                    signature="assembly_production_missing_glb",
                )
            )
            continue
        if shell_strict and _is_lod0_glb(glb_rel):
            lod0_shell += 1
        if ship and shell_strict and not _is_production_glb(glb_rel):
            issues.append(
                ValidationIssue(
                    kind="NonProductionGlb",
                    severity="error",
                    file=glb_rel,
                    field="glb_path",
                    hint=f"shell module {module_id} requires production GLB under assets/models/modules/*production_run*",
                    signature="assembly_production_glb_path",
                )
            )
        glb_path = Path(glb_rel)
        if not glb_path.is_absolute():
            glb_path = repo_root() / glb_rel
        if not glb_path.is_file():
            missing_glb += 1

    if len(unique_jobs) < MIN_UNIQUE_MODULES:
        issues.append(
            ValidationIssue(
                kind="ModuleCount",
                severity="error",
                file=snapshot_path,
                field="module_placements",
                hint=f"need >= {MIN_UNIQUE_MODULES} unique job_ids (got {len(unique_jobs)})",
                signature="assembly_production_unique_modules",
            )
        )
    if lod0_shell > 0:
        issues.append(
            ValidationIssue(
                kind="Lod0Module",
                severity="error",
                file=snapshot_path,
                field="module_placements",
                hint=f"{lod0_shell} shell lod0/greybox GLB(s) — promote wall/roof production modules",
                signature="assembly_production_lod0_rejected",
            )
        )
    if missing_glb > 0:
        issues.append(
            ValidationIssue(
                kind="MissingFile",
                severity="error",
                file=snapshot_path,
                field="glb_path",
                hint=f"{missing_glb} GLB file(s) missing on disk — promote modules first",
                signature="assembly_production_glb_missing",
            )
        )
    if ship and missing_material > 0:
        issues.append(
            ValidationIssue(
                kind="MissingField",
                severity="error",
                file=snapshot_path,
                field="material_profile",
                hint=f"ARCH-003: {missing_material} placement(s) missing material_profile — use APS Assembly Editor",
                signature="assembly_graph_material_profile",
            )
        )

    status = "failed" if any(i.severity == "error" for i in issues) else "passed"
    return ValidationReport(
        validator="tile",
        status=status,
        compression_level=compression_level,
        summary=f"assembly_production: {len(issues)} issue(s)",
        error_count=sum(1 for i in issues if i.severity == "error"),
        errors=issues,
    )


def validate_assembly_snapshot_path(
    path: str | Path,
    *,
    ship: bool = True,
    compression_level: int = 3,
) -> ValidationReport:
    p = Path(path)
    if not p.is_file():
        return ValidationReport(
            validator="tile",
            status="failed",
            compression_level=compression_level,
            summary="snapshot not found",
            error_count=1,
            errors=[
                ValidationIssue(
                    kind="MissingFile",
                    severity="error",
                    file=str(p),
                    signature="assembly_production_snapshot_missing",
                )
            ],
        )
    snap = json.loads(p.read_text(encoding="utf-8"))
    rel = str(p.relative_to(repo_root())).replace("\\", "/")
    return validate_assembly_snapshot(
        snap, snapshot_path=rel, ship=ship, compression_level=compression_level
    )
