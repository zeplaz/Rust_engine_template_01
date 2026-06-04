"""MCP schema / job validators — structured output before tool execution."""

from __future__ import annotations

import json
from pathlib import Path

from rust_engine_mcp import schemas

from .knowledge import lookup_fixes
from .report import ValidationIssue, ValidationReport
from .tier import TierIssue, tier_issues_for_job, tier_issues_for_spec


def _issues_from_tier(rows: list[TierIssue], path: Path) -> list[ValidationIssue]:
    return [
        ValidationIssue(
            kind=ti.kind,
            severity=ti.severity,  # type: ignore[arg-type]
            file=str(path),
            symbol=ti.rule_id,
            hint=ti.hint,
            signature=ti.signature,
        )
        for ti in rows
    ]


def validate_mcp_spec(path: Path, *, compression_level: int = 3) -> ValidationReport:
    issues: list[ValidationIssue] = []
    data: dict = {}
    try:
        data = schemas.load_json_file(path)
        schemas.validate_asset_spec(data)
        issues.extend(_issues_from_tier(tier_issues_for_spec(data, path), path))
    except Exception as exc:  # noqa: BLE001
        issues.append(
            ValidationIssue(
                kind="SchemaInvalid",
                severity="error",
                file=str(path),
                hint=str(exc)[:240],
                signature="mcp_spec_invalid",
            )
        )
    status = "passed" if not any(i.severity == "error" for i in issues) else "failed"
    if status == "passed" and issues:
        status = "warning"
    report = ValidationReport(
        validator="mcp_schema",
        status=status,  # type: ignore[arg-type]
        compression_level=compression_level,
        summary=f"AssetSpec {path.name}: {status} tier={data.get('development_tier', '-')}",
        error_count=sum(1 for i in issues if i.severity == "error"),
        warning_count=sum(1 for i in issues if i.severity == "warning"),
        errors=issues,
        known_fixes=lookup_fixes(issues),
        confidence=1.0,
    )
    return report.compress(compression_level)


def validate_mcp_job(path: Path, *, compression_level: int = 3) -> ValidationReport:
    issues: list[ValidationIssue] = []
    data: dict = {}
    try:
        data = schemas.load_json_file(path)
        schemas.validate_geometry_job(data)
        params = data.get("params") or {}
        if "seed" not in params:
            issues.append(
                ValidationIssue(
                    kind="MissingField",
                    severity="warning",
                    file=str(path),
                    field="seed",
                    hint="Add params.seed for deterministic geometry jobs",
                    signature="mcp_missing_seed",
                )
            )
        issues.extend(_issues_from_tier(tier_issues_for_job(data, path), path))
    except Exception as exc:  # noqa: BLE001
        issues.append(
            ValidationIssue(
                kind="SchemaInvalid",
                severity="error",
                file=str(path),
                hint=str(exc)[:240],
            )
        )
    status = "passed" if not any(i.severity == "error" for i in issues) else "failed"
    if status == "passed" and issues:
        status = "warning"
    report = ValidationReport(
        validator="mcp_schema",
        status=status,  # type: ignore[arg-type]
        compression_level=compression_level,
        summary=f"GeometryJob {path.name}: {status}",
        error_count=sum(1 for i in issues if i.severity == "error"),
        warning_count=sum(1 for i in issues if i.severity == "warning"),
        errors=issues,
        known_fixes=lookup_fixes(issues),
        confidence=1.0,
    )
    return report.compress(compression_level)


def validate_mcp_json_text(text: str, *, kind: str = "asset_spec", compression_level: int = 3) -> ValidationReport:
    tmp = Path("_mcp_validate_inline.json")
    try:
        json.loads(text)
        tmp.write_text(text, encoding="utf-8")
        if kind == "geometry_job":
            return validate_mcp_job(tmp, compression_level=compression_level)
        return validate_mcp_spec(tmp, compression_level=compression_level)
    finally:
        if tmp.is_file():
            tmp.unlink()
