"""EffectSpec v1 validator — VSS-T4-003 validate_report lane.

R-SCHEMA-1 (allOf spawn_hook conditional):
  effect_spec_v1.schema.json uses JSON Schema Draft 2020-12 ``if``/``then`` branches
  on root ``spawn_hook`` to require the matching ``spawn_hook_config`` sub-key
  (world_xy | attach_entity | field_sample). Those conditionals live in the
  schema ``allOf`` array — they are NOT enforced by a plain ``properties`` map.

  Risk: calling ``jsonschema.validate()`` without an explicit Draft 2020-12
  validator can silently skip ``if``/``then`` (wrong metaschema / legacy default),
  letting e.g. ``spawn_hook: world_xy`` pass while ``spawn_hook_config`` omits
  ``world_xy`` or carries only ``field_sample``.

  Mitigation in this module:
    1. ``schemas.validate_effect_spec`` uses ``Draft202012Validator`` only.
    2. ``_spawn_hook_config_issues`` re-checks hook↔config key parity so a
       schema-engine regression still fails closed before promote (VSS-T4-003).
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from rust_engine_mcp import schemas

from .knowledge import lookup_fixes
from .report import ValidationIssue, ValidationReport


def _spawn_hook_config_issues(data: dict[str, Any], path: Path) -> list[ValidationIssue]:
    """Belt-and-suspenders for R-SCHEMA-1 — hook kind must own config branch."""
    hook = data.get("spawn_hook")
    if not isinstance(hook, str) or not hook:
        return []
    cfg = data.get("spawn_hook_config")
    if not isinstance(cfg, dict):
        return [
            ValidationIssue(
                kind="MissingField",
                severity="error",
                file=str(path),
                field="spawn_hook_config",
                hint=f"spawn_hook={hook!r} requires spawn_hook_config.{hook}",
                signature="effect_spec_spawn_hook_config",
            )
        ]
    if hook not in cfg:
        return [
            ValidationIssue(
                kind="SchemaInvalid",
                severity="error",
                file=str(path),
                field="spawn_hook_config",
                hint=f"spawn_hook={hook!r} requires spawn_hook_config[{hook!r}]",
                signature="effect_spec_spawn_hook_mismatch",
            )
        ]
    extra = {k for k in cfg if k in ("world_xy", "attach_entity", "field_sample") and k != hook}
    if extra:
        return [
            ValidationIssue(
                kind="SchemaInvalid",
                severity="warning",
                file=str(path),
                field="spawn_hook_config",
                hint=f"unexpected branches for spawn_hook={hook!r}: {sorted(extra)}",
                signature="effect_spec_spawn_hook_extra",
            )
        ]
    return []


def validate_effect_spec(path: Path, *, compression_level: int = 3) -> ValidationReport:
    issues: list[ValidationIssue] = []
    data: dict[str, Any] = {}
    try:
        data = schemas.load_json_file(path)
        schemas.validate_effect_spec(data)
        issues.extend(_spawn_hook_config_issues(data, path))
    except Exception as exc:  # noqa: BLE001
        issues.append(
            ValidationIssue(
                kind="SchemaInvalid",
                severity="error",
                file=str(path),
                hint=str(exc)[:240],
                signature="effect_spec_invalid",
            )
        )

    rc = data.get("rules_check") if isinstance(data, dict) else None
    if isinstance(rc, dict) and rc.get("passed") is False:
        issues.append(
            ValidationIssue(
                kind="RulesCheck",
                severity="warning",
                file=str(path),
                field="rules_check.passed",
                hint="rules_check.passed is false — production promote blocked",
                signature="effect_spec_rules_check",
            )
        )

    errors = [i for i in issues if i.severity == "error"]
    status = "failed" if errors else ("warning" if issues else "passed")
    effect_id = str(data.get("effect_id") or path.stem)
    tier = str(data.get("development_tier") or "-")
    report = ValidationReport(
        validator="effect_spec",  # type: ignore[arg-type]
        status=status,  # type: ignore[arg-type]
        compression_level=compression_level,
        summary=f"EffectSpec {effect_id}: {status} tier={tier}",
        error_count=sum(1 for i in issues if i.severity == "error"),
        warning_count=sum(1 for i in issues if i.severity == "warning"),
        errors=issues,
        known_fixes=lookup_fixes(issues),
        confidence=1.0,
    )
    return report.compress(compression_level)
