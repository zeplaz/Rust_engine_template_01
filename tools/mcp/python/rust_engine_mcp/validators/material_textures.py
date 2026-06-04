"""TILE-FIX-005 — production ship requires resolved PBR textures (no greybox auto-fallback)."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.material_textures import texture_dir
from rust_engine_mcp.paths import repo_root

from .report import ValidationIssue, ValidationReport

REQUIRED_MAPS = ("albedo", "normal", "roughness")


def _resolve_profile_id(spec: dict[str, Any]) -> str:
    return str(
        spec.get("material_profile")
        or spec.get("tileable_set_id")
        or spec.get("material_id")
        or ""
    ).strip()


def _textures_present(profile_id: str) -> tuple[bool, list[str]]:
    root = texture_dir(profile_id)
    missing = []
    for name in REQUIRED_MAPS:
        if not (root / f"{name}.png").is_file():
            missing.append(f"{name}.png")
    return (not missing, missing)


def validate_material_textures(
    spec: dict[str, Any],
    *,
    spec_path: str = "",
    ship: bool = True,
    compression_level: int = 3,
) -> ValidationReport:
    """Fail production ship when PBR maps are absent and no explicit shipped manifest."""
    issues: list[ValidationIssue] = []
    tier = str(spec.get("development_tier") or spec.get("source_tier") or "").lower()
    if tier != "production" and not ship:
        return ValidationReport(
            validator="tile",
            status="passed",
            compression_level=compression_level,
            summary="skipped (non-production)",
        )

    profile_id = _resolve_profile_id(spec)
    if not profile_id:
        issues.append(
            ValidationIssue(
                kind="MissingMaterial",
                severity="error",
                file=spec_path,
                field="material_profile",
                hint="production requires material_profile or tileable_set_id",
                signature="material_textures_missing_profile",
            )
        )
    else:
        if spec.get("material_fallback") or spec.get("greybox_fallback"):
            issues.append(
                ValidationIssue(
                    kind="GreyboxFallback",
                    severity="error",
                    file=spec_path,
                    field="material_fallback",
                    hint="TILE-FIX-005: ship forbids greybox/material_fallback on production",
                    signature="material_textures_greybox_fallback_forbidden",
                )
            )
        ok, missing = _textures_present(profile_id)
        if not ok:
            issues.append(
                ValidationIssue(
                    kind="MissingTexture",
                    severity="error",
                    file=str(texture_dir(profile_id)),
                    field="textures",
                    hint=f"missing {', '.join(missing)} — run material_textures.generate_profile or Material Maker",
                    signature="material_textures_missing_maps",
                )
            )

    status = "failed" if any(i.severity == "error" for i in issues) else "passed"
    return ValidationReport(
        validator="tile",
        status=status,
        compression_level=compression_level,
        summary=f"material_textures: {len(issues)} issue(s)",
        error_count=sum(1 for i in issues if i.severity == "error"),
        errors=issues,
    )


def validate_material_textures_path(
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
            summary="spec not found",
            error_count=1,
            errors=[
                ValidationIssue(
                    kind="MissingFile",
                    severity="error",
                    file=str(p),
                    signature="material_textures_spec_missing",
                )
            ],
        )
    spec = json.loads(p.read_text(encoding="utf-8"))
    return validate_material_textures(
        spec, spec_path=str(p.relative_to(repo_root())).replace("\\", "/"), ship=ship, compression_level=compression_level
    )
