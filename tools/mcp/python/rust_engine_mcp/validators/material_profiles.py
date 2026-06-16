"""APS-MAT-008 — assembly snapshot material_profile + PBR map gates."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.material_profiles import load_material_profile_catalog, profile_def_for_id
from rust_engine_mcp.material_textures import PILOT_PROFILES, generate_profile, texture_dir, write_registry
from rust_engine_mcp.paths import repo_root

from .report import ValidationIssue, ValidationReport

REQUIRED_MAPS_SHIP = ("albedo", "normal", "roughness")
REQUIRED_MAPS_WARN = ("albedo",)


def _textures_status(profile_id: str) -> tuple[bool, list[str], list[str]]:
    root = texture_dir(profile_id)
    missing_required: list[str] = []
    missing_optional: list[str] = []
    for name in REQUIRED_MAPS_SHIP:
        if not (root / f"{name}.png").is_file():
            if name in REQUIRED_MAPS_WARN:
                missing_required.append(f"{name}.png")
            else:
                missing_optional.append(f"{name}.png")
    ship_ok = not missing_required
    full_ok = ship_ok and not missing_optional
    return full_ok, missing_required, missing_optional


def validate_assembly_material_profiles(
    snapshot: dict[str, Any],
    *,
    snapshot_path: str = "",
    ship: bool = True,
    require_full_pbr: bool = True,
    compression_level: int = 3,
) -> ValidationReport:
    """Every placement must have material_profile; ship requires resolvable PBR maps."""
    issues: list[ValidationIssue] = []
    placements = list(snapshot.get("module_placements") or [])
    known_ids = {e.profile_id for e in load_material_profile_catalog()}
    seen_profiles: set[str] = set()

    for i, row in enumerate(placements):
        pid = str((row or {}).get("material_profile") or "").strip()
        node = str((row or {}).get("node_id") or f"placement_{i}")
        if not pid:
            issues.append(
                ValidationIssue(
                    kind="MissingMaterialProfile",
                    severity="error",
                    file=snapshot_path,
                    field=f"module_placements[{i}].material_profile",
                    hint=f"{node}: ARCH-003 / APS-MAT-008 requires material_profile on every placement",
                    signature="material_profiles_placement_missing",
                )
            )
            continue
        seen_profiles.add(pid)
        if pid not in known_ids and pid not in PILOT_PROFILES:
            issues.append(
                ValidationIssue(
                    kind="UnknownMaterialProfile",
                    severity="warning",
                    file=snapshot_path,
                    field="material_profile",
                    hint=f"{pid} not in registry or PILOT_PROFILES — generate or register in APS Materials tab",
                    signature="material_profiles_unknown_id",
                )
            )
        if not ship:
            continue
        full_ok, missing_req, missing_opt = _textures_status(pid)
        if missing_req:
            issues.append(
                ValidationIssue(
                    kind="MissingTexture",
                    severity="error",
                    file=str(texture_dir(pid)),
                    field="material_profile",
                    hint=f"{pid}: missing {', '.join(missing_req)} — run material-studio-witness or Generate in APS",
                    signature="material_profiles_missing_albedo",
                )
            )
        elif require_full_pbr and missing_opt:
            issues.append(
                ValidationIssue(
                    kind="MissingTexture",
                    severity="warning",
                    file=str(texture_dir(pid)),
                    field="material_profile",
                    hint=f"{pid}: missing optional {', '.join(missing_opt)}",
                    signature="material_profiles_missing_normal_roughness",
                )
            )

    status = "failed" if any(i.severity == "error" for i in issues) else (
        "warning" if any(i.severity == "warning" for i in issues) else "passed"
    )
    return ValidationReport(
        validator="material_profiles",
        status=status,
        compression_level=compression_level,
        summary=(
            f"material_profiles: {len(placements)} placements, "
            f"{len(seen_profiles)} unique profiles, {len(issues)} issue(s)"
        ),
        error_count=sum(1 for i in issues if i.severity == "error"),
        warning_count=sum(1 for i in issues if i.severity == "warning"),
        errors=issues,
    )


def validate_assembly_material_profiles_path(
    path: str | Path,
    *,
    ship: bool = True,
    require_full_pbr: bool = True,
    compression_level: int = 3,
) -> ValidationReport:
    p = Path(path)
    if not p.is_file():
        return ValidationReport(
            validator="material_profiles",
            status="failed",
            compression_level=compression_level,
            summary="snapshot not found",
            error_count=1,
            errors=[
                ValidationIssue(
                    kind="MissingFile",
                    severity="error",
                    file=str(p),
                    signature="material_profiles_snapshot_missing",
                )
            ],
        )
    snap = json.loads(p.read_text(encoding="utf-8"))
    try:
        rel = str(p.relative_to(repo_root())).replace("\\", "/")
    except ValueError:
        rel = str(p)
    return validate_assembly_material_profiles(
        snap,
        snapshot_path=rel,
        ship=ship,
        require_full_pbr=require_full_pbr,
        compression_level=compression_level,
    )


def ensure_placement_material_textures(
    snapshot: dict[str, Any],
    *,
    size: int = 512,
) -> dict[str, Any]:
    """Generate missing pilot/registry textures for profiles used on snapshot."""
    write_registry()
    generated: list[str] = []
    missing: list[str] = []
    seen: set[str] = set()
    for row in snapshot.get("module_placements") or []:
        pid = str((row or {}).get("material_profile") or "").strip()
        if not pid or pid in seen:
            continue
        seen.add(pid)
        full_ok, missing_req, _ = _textures_status(pid)
        if full_ok:
            continue
        if missing_req:
            try:
                generate_profile(profile_def_for_id(pid), size=size)
                generated.append(pid)
            except Exception:
                missing.append(pid)
    return {"generated": generated, "missing": missing, "ok": not missing, "profiles": sorted(seen)}


def write_material_validation_witness(report: ValidationReport, *, snapshot_path: str = "") -> Path:
    out = repo_root() / "debug_runs/material_validation_live.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    body = {
        "gate_id": "APS-MAT-008",
        "ok": report.status == "passed",
        "status": report.status,
        "snapshot": snapshot_path,
        "summary": report.summary,
        "error_count": report.error_count,
        "errors": [e.to_dict() for e in report.errors[:20]],
    }
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return out
