"""Studio DCC schema validators — ParamFrame, massing_preview, building_grammar_v1.

PLAN-REF-BOOKS-UPGRADE-v3 S5: structured ValidationReport for agents (no raw jsonschema logs).
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from rust_engine_mcp import schemas

from .knowledge import lookup_fixes
from .report import ValidationIssue, ValidationReport


def _schema_report(
    *,
    validator: str,
    path: Path,
    data: dict[str, Any],
    issues: list[ValidationIssue],
    label: str,
    id_field: str,
    compression_level: int,
) -> ValidationReport:
    errors = [i for i in issues if i.severity == "error"]
    status = "failed" if errors else ("warning" if issues else "passed")
    entity_id = str(data.get(id_field) or path.stem)
    report = ValidationReport(
        validator=validator,  # type: ignore[arg-type]
        status=status,  # type: ignore[arg-type]
        compression_level=compression_level,
        summary=f"{label} {entity_id}: {status}",
        error_count=sum(1 for i in issues if i.severity == "error"),
        warning_count=sum(1 for i in issues if i.severity == "warning"),
        errors=issues,
        known_fixes=lookup_fixes(issues) if issues else [],
        confidence=1.0,
    )
    return report.compress(compression_level)


def _load_fail(path: Path, exc: Exception, signature: str) -> list[ValidationIssue]:
    return [
        ValidationIssue(
            kind="SchemaInvalid",
            severity="error",
            file=str(path),
            hint=str(exc)[:240],
            signature=signature,
        )
    ]


def validate_param_frame_path(path: Path, *, compression_level: int = 3) -> ValidationReport:
    issues: list[ValidationIssue] = []
    data: dict[str, Any] = {}
    try:
        data = schemas.load_json_file(path)
        schemas.validate_param_frame(data)
    except Exception as exc:  # noqa: BLE001
        issues = _load_fail(path, exc, "param_frame_invalid")
    return _schema_report(
        validator="param_frame",
        path=path,
        data=data,
        issues=issues,
        label="ParamFrame",
        id_field="frame_id",
        compression_level=compression_level,
    )


def validate_massing_preview_path(path: Path, *, compression_level: int = 3) -> ValidationReport:
    issues: list[ValidationIssue] = []
    data: dict[str, Any] = {}
    try:
        data = schemas.load_json_file(path)
        schemas.validate_massing_preview(data)
        cols = data.get("massing_columns") or []
        if isinstance(cols, list) and len(cols) == 0:
            issues.append(
                ValidationIssue(
                    kind="SchemaInvalid",
                    severity="warning",
                    file=str(path),
                    field="massing_columns",
                    hint="Presentation massing has zero columns — voxel ghost will be empty",
                    signature="massing_preview_empty_columns",
                )
            )
    except Exception as exc:  # noqa: BLE001
        issues = _load_fail(path, exc, "massing_preview_invalid")
    return _schema_report(
        validator="massing_preview",
        path=path,
        data=data,
        issues=issues,
        label="massing_preview",
        id_field="title",
        compression_level=compression_level,
    )


def validate_veg_species_path(path: Path | None = None, *, compression_level: int = 3) -> ValidationReport:
    """VEG-SHAPE-001 — veg species RON structural QC (CQ-11: honors `path`)."""
    from rust_engine_mcp.paths import repo_root
    from rust_engine_mcp.veg_species_table import VEG_SPECIES_REL, brief_veg_species_table

    resolved = Path(path) if path is not None else (repo_root() / VEG_SPECIES_REL)
    issues: list[ValidationIssue] = []
    try:
        brief = brief_veg_species_table(path=resolved)
        if not brief.get("green"):
            for msg in brief.get("issues") or []:
                issues.append(
                    ValidationIssue(
                        kind="VegSpeciesTable",
                        severity="error",
                        file=str(resolved),
                        hint=str(msg),
                        signature="veg_species_invalid",
                    )
                )
        data = brief
    except Exception as exc:  # noqa: BLE001
        issues = _load_fail(resolved, exc, "veg_species_load")
        data = {}
    return _schema_report(
        validator="veg_species",
        path=resolved,
        data=data,
        issues=issues,
        label="veg_species_v1",
        id_field="path",
        compression_level=compression_level,
    )


def validate_building_grammar_path(path: Path, *, compression_level: int = 3) -> ValidationReport:
    issues: list[ValidationIssue] = []
    data: dict[str, Any] = {}
    try:
        data = schemas.load_json_file(path)
        schemas.validate_building_grammar(data)
    except Exception as exc:  # noqa: BLE001
        issues = _load_fail(path, exc, "building_grammar_invalid")
    grammar_id = str(data.get("grammar_id") or data.get("archetype_id") or path.stem)
    return _schema_report(
        validator="building_grammar",
        path=path,
        data=data,
        issues=issues,
        label="building_grammar_v1",
        id_field="grammar_id",
        compression_level=compression_level,
    )