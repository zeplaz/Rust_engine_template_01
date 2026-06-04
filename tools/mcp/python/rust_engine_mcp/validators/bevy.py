"""Bevy-specific validator — classifies deprecated ECS/render APIs from cargo diagnostics."""

from __future__ import annotations

import re

from .cargo import validate_cargo
from .knowledge import lookup_fixes
from .report import ValidationIssue, ValidationReport

_BEVY_PATTERNS: list[tuple[re.Pattern[str], str, str, str]] = [
    (re.compile(r"\badd_system\b"), "add_system", "add_systems", "BevyVersionIssue"),
    (re.compile(r"add_system_to_stage"), "add_system_to_stage", "add_systems(CoreSystemSet::...)", "BevyVersionIssue"),
    (re.compile(r"\bApp::add_stage\b"), "add_stage", "configure_sets + add_systems", "BevyVersionIssue"),
    (re.compile(r"SingleThreaded\b.*Schedule"), "SingleThreaded", "ExecutorKind::SingleThreaded on Schedule", "ScheduleIssue"),
    (re.compile(r"Query<[^>]+>\s+.*\.get_single"), "get_single", "single() or get_single() per Bevy 0.14+", "BevyQueryIssue"),
]


def _scan_bevy_issues(cargo_report: ValidationReport) -> list[ValidationIssue]:
    extra: list[ValidationIssue] = []
    for issue in cargo_report.errors:
        hay = f"{issue.hint} {issue.symbol}"
        for pattern, api, replacement, kind in _BEVY_PATTERNS:
            if pattern.search(hay):
                extra.append(
                    ValidationIssue(
                        kind=kind,
                        severity="error" if issue.severity == "error" else "warning",
                        file=issue.file,
                        line=issue.line,
                        api=api,
                        replacement=replacement,
                        hint=f"Replace {api} with {replacement}",
                        signature=f"bevy_{api}",
                    )
                )
    return extra


def validate_bevy(*, package: str | None = None, compression_level: int = 3) -> ValidationReport:
    base = validate_cargo(package=package, compression_level=1)
    bevy_issues = _scan_bevy_issues(base)
    merged = bevy_issues + [i for i in base.errors if i.kind not in {b.kind for b in bevy_issues}]
    errors = [i for i in merged if i.severity == "error"]
    warnings = [i for i in merged if i.severity == "warning"]
    status = base.status if not bevy_issues else ("failed" if errors else "warning")
    known = lookup_fixes(bevy_issues + errors[:5])
    report = ValidationReport(
        validator="bevy",
        status=status,  # type: ignore[arg-type]
        compression_level=compression_level,
        summary=f"bevy scan: {len(bevy_issues)} bevy-specific, {len(errors)} total errors",
        error_count=len(errors),
        warning_count=len(warnings),
        errors=merged,
        known_fixes=known,
        raw_log_path=base.raw_log_path,
        confidence=0.9,
    )
    return report.compress(compression_level)
