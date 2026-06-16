"""BLANG:PLACE — compressed ValidationReport for construction / placement witnesses."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from .report import KnownFix, ValidationIssue, ValidationReport

KNOWN_FIXES = (
    KnownFix(
        signature="construction_footprint_projection",
        fix="Hotfix A: stop skipping egui footprint_tiles when gpu_path_active — src/construction/visual_authority.rs",
        confidence=0.92,
    ),
    KnownFix(
        signature="construction_map_pick_delta",
        fix="Align pick (ConstructionMapProjection) with paint path — see plan_build_footprint_vm09_exec_v1.md",
        confidence=0.88,
    ),
    KnownFix(
        signature="construction_parametric_gate",
        fix="Run cargo test -p proc_A_dine01 --lib construction_parametric — refresh construction_stage_live.json",
        confidence=0.85,
    ),
)


def _resolve(path: str | Path) -> Path:
    p = Path(path)
    if not p.is_absolute():
        p = repo_root() / p
    return p


def _footprint_projection_ok(data: dict[str, Any]) -> bool | None:
    if "footprint_projection_ok" in data:
        return bool(data["footprint_projection_ok"])
    mp = data.get("map_pick_closure_001")
    if isinstance(mp, dict) and "footprint_projection_ok" in mp:
        return bool(mp["footprint_projection_ok"])
    mzc = data.get("map_zoom_coherence_001")
    if isinstance(mzc, dict) and mzc.get("green") is True:
        return True
    if data.get("map_pick_closure_math_ok") is True:
        return True
    return None


def _map_pick_closure_green(data: dict[str, Any]) -> bool | None:
    block = data.get("map_pick_closure_001")
    if isinstance(block, dict) and "green" in block:
        return bool(block["green"])
    mzc = data.get("map_zoom_coherence_001")
    if isinstance(mzc, dict) and "green" in mzc:
        return bool(mzc["green"])
    if data.get("map_pick_closure_math_ok") is True:
        return True
    return None


def validate_construction_witness(
    data: dict[str, Any],
    *,
    witness_path: str = "",
    ship: bool = True,
    compression_level: int = 3,
) -> ValidationReport:
    """Validate construction_stage_live.json or construction_placement_live.json rollup."""
    issues: list[ValidationIssue] = []
    rel = witness_path.replace("\\", "/")

    footprint_ok = _footprint_projection_ok(data)
    if footprint_ok is False:
        issues.append(
            ValidationIssue(
                kind="FootprintProjection",
                severity="error",
                file=rel,
                hint="Ghost footprint mis-projected — pick vs paint authority drift",
                signature="construction_footprint_projection",
            )
        )
    elif footprint_ok is None and ship:
        issues.append(
            ValidationIssue(
                kind="FootprintProjection",
                severity="warning",
                file=rel,
                hint="footprint_projection_ok missing — export from placement_debug or map_zoom witness",
                signature="construction_footprint_projection",
            )
        )

    pick_green = _map_pick_closure_green(data)
    if pick_green is False:
        issues.append(
            ValidationIssue(
                kind="MapPickClosure",
                severity="error",
                file=rel,
                hint="MAP-PICK closure failed — pick_delta or ghost_screen_delta over threshold",
                signature="construction_map_pick_delta",
            )
        )

    param = data.get("construction_parametric_placement_001")
    if isinstance(param, dict) and param.get("green") is False:
        issues.append(
            ValidationIssue(
                kind="ParametricPlacement",
                severity="error",
                file=rel,
                hint="construction_parametric_placement_001 gate red",
                signature="construction_parametric_gate",
            )
        )

    cursor_delta = data.get("cursor_delta_px") or data.get("cursor_reproject_delta_px")
    if cursor_delta is not None and float(cursor_delta) > 8.0:
        issues.append(
            ValidationIssue(
                kind="MapPickClosure",
                severity="error",
                file=rel,
                hint=f"cursor_delta_px={cursor_delta} > 8",
                signature="construction_map_pick_delta",
            )
        )

    errors = [i for i in issues if i.severity == "error"]
    status = "failed" if errors else ("warning" if issues else "passed")
    fixes = [k for k in KNOWN_FIXES if any(i.signature == k.signature for i in issues)]
    if not fixes and errors:
        fixes = list(KNOWN_FIXES[:2])

    return ValidationReport(
        validator="test",
        status=status,
        compression_level=compression_level,
        summary=f"construction: {len(errors)} error(s), footprint_ok={footprint_ok}, pick_green={pick_green}",
        error_count=len(errors),
        warning_count=len(issues) - len(errors),
        errors=issues[:8],
        known_fixes=fixes,
        confidence=0.9 if not errors else 0.82,
    )


def validate_construction_witness_path(
    path: str | Path,
    *,
    ship: bool = True,
    compression_level: int = 3,
) -> ValidationReport:
    resolved = _resolve(path)
    if not resolved.is_file():
        return ValidationReport(
            validator="test",
            status="failed",
            compression_level=compression_level,
            summary="construction witness not found",
            error_count=1,
            errors=[
                ValidationIssue(
                    kind="MissingFile",
                    severity="error",
                    file=str(resolved),
                    hint="Run sim witness export or use debug_runs/construction_stage_live.json",
                    signature="construction_witness_missing",
                )
            ],
        )
    data = json.loads(resolved.read_text(encoding="utf-8"))
    try:
        rel = str(resolved.relative_to(repo_root())).replace("\\", "/")
    except ValueError:
        rel = str(resolved)
    return validate_construction_witness(
        data,
        witness_path=rel,
        ship=ship,
        compression_level=compression_level,
    )
