"""Building visual_config validator — TILE-FIX-002 (.json authoritative for MCP)."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .report import ValidationIssue, ValidationReport

try:
    import jsonschema
except ImportError:  # pragma: no cover
    jsonschema = None  # type: ignore[assignment]


def _load_schema() -> dict[str, Any]:
    from rust_engine_mcp.paths import schemas_dir

    return json.loads((schemas_dir() / "visual_config_v1.schema.json").read_text(encoding="utf-8"))


def load_visual_config(path: Path) -> dict[str, Any]:
    """Load visual_config from .json, or sibling .json next to a .ron path."""
    if path.suffix.lower() == ".json":
        data = json.loads(path.read_text(encoding="utf-8"))
    elif path.suffix.lower() == ".ron":
        companion = path.with_suffix(".json")
        if not companion.is_file():
            raise ValueError(
                f"RON visual_config requires companion JSON for MCP validate: {companion}"
            )
        data = json.loads(companion.read_text(encoding="utf-8"))
    else:
        raise ValueError(f"unsupported visual_config extension: {path}")
    if not isinstance(data, dict):
        raise ValueError(f"visual_config root must be object: {path}")
    return data


def validate_visual_config(
    path: Path,
    *,
    compression_level: int = 3,
) -> ValidationReport:
    issues: list[ValidationIssue] = []
    try:
        data = load_visual_config(path)
    except Exception as exc:  # noqa: BLE001
        return ValidationReport(
            validator="visual_config",
            status="failed",
            errors=[
                ValidationIssue(
                    kind="SchemaInvalid",
                    severity="error",
                    file=str(path),
                    hint=str(exc),
                    signature="visual_config_parse",
                )
            ],
            known_fixes=[],
            summary=str(exc),
            compression_level=compression_level,
        )

    if int(data.get("schema_version") or 0) != 1:
        issues.append(
            ValidationIssue(
                kind="SchemaInvalid",
                severity="error",
                file=str(path),
                field="schema_version",
                hint="schema_version must be 1",
                signature="visual_config_version",
            )
        )

    if jsonschema is not None:
        try:
            jsonschema.validate(instance=data, schema=_load_schema())
        except jsonschema.ValidationError as exc:
            issues.append(
                ValidationIssue(
                    kind="SchemaInvalid",
                    severity="error",
                    file=str(path),
                    hint=str(exc.message),
                    signature="visual_config_jsonschema",
                )
            )

    rc = data.get("render_contract") or {}
    if int(rc.get("facings") or 0) not in (4, 8):
        issues.append(
            ValidationIssue(
                kind="MissingField",
                severity="error",
                file=str(path),
                field="render_contract.facings",
                hint="facings must be 4 or 8",
                signature="visual_config_facings",
            )
        )

    errors = [i for i in issues if i.severity == "error"]
    status = "failed" if errors else ("warning" if issues else "passed")
    return ValidationReport(
        validator="visual_config",
        status=status,
        errors=issues,
        known_fixes=[],
        summary=f"{path.name}: states={len(data.get('states') or [])}",
        compression_level=compression_level,
    )
