"""Validator for arch_build_grammar_v0 preset JSON."""

from __future__ import annotations

from pathlib import Path

from rust_engine_mcp import schemas

from .report import ValidationIssue, ValidationReport


def validate_arch_build_grammar_path(path: Path, *, compression_level: int = 3) -> ValidationReport:
    issues: list[ValidationIssue] = []
    data: dict = {}
    try:
        data = schemas.load_json_file(path)
        schemas.validate_arch_build_grammar(data)
    except Exception as exc:  # noqa: BLE001
        issues.append(
            ValidationIssue(
                kind="SchemaInvalid",
                severity="error",
                file=str(path),
                symbol="arch_build_grammar_v0",
                hint=str(exc),
                signature="arch_build_grammar_v0",
            )
        )
    status = "passed" if not issues else "failed"
    preset_id = str(data.get("preset_id") or path.stem)
    summary = f"arch_build_grammar_v0 preset={preset_id}" if status == "passed" else "arch_build_grammar_v0 invalid"
    return ValidationReport(
        validator="mcp_schema",
        status=status,  # type: ignore[arg-type]
        compression_level=compression_level,
        summary=summary,
        error_count=sum(1 for i in issues if i.severity == "error"),
        warning_count=sum(1 for i in issues if i.severity == "warning"),
        errors=issues,
    ).compress(compression_level)
