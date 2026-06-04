"""Asset GLB validator — tier-aware structured report (TIER-001..006)."""

from __future__ import annotations

from pathlib import Path

from rust_engine_mcp.validate_glb import validate_glb

from .knowledge import lookup_fixes
from .report import ValidationIssue, ValidationReport
from .tier import resolve_asset_context, tier_issues_for_asset


def validate_asset_glb(path: Path, *, compression_level: int = 3) -> ValidationReport:
    raw = validate_glb(path)
    ctx = resolve_asset_context(path)
    issues: list[ValidationIssue] = []

    if not raw.valid:
        for msg in raw.issues:
            kind = "EmptyMesh" if "no vertices" in msg else "GltfInvalid"
            issues.append(
                ValidationIssue(
                    kind=kind,
                    severity="error",
                    file=str(path),
                    hint=msg,
                    signature=f"glb_{kind}",
                )
            )

    if raw.vertex_count is not None and raw.vertex_count > 50_000:
        issues.append(
            ValidationIssue(
                kind="VertexBudgetExceeded",
                severity="warning",
                file=str(path),
                hint=f"vertex_count={raw.vertex_count} exceeds 50000",
            )
        )

    for ti in tier_issues_for_asset(ctx, vertex_count=raw.vertex_count):
        issues.append(
            ValidationIssue(
                kind=ti.kind,
                severity=ti.severity,  # type: ignore[arg-type]
                file=str(path),
                symbol=ti.rule_id,
                field="development_tier",
                hint=ti.hint,
                signature=ti.signature,
            )
        )

    tier = ctx.effective_tier()
    status = "passed" if raw.valid and not any(i.severity == "error" for i in issues) else "failed"
    if status == "passed" and issues:
        status = "warning"

    summary = (
        f"{path.name}: verts={raw.vertex_count} tier={tier} "
        f"arch={ctx.archetype or '?'} profile={ctx.profile or '-'}"
    )
    report = ValidationReport(
        validator="asset_glb",
        status=status,  # type: ignore[arg-type]
        compression_level=compression_level,
        summary=summary,
        error_count=sum(1 for i in issues if i.severity == "error"),
        warning_count=sum(1 for i in issues if i.severity == "warning"),
        errors=issues,
        known_fixes=lookup_fixes(issues),
        confidence=1.0,
    )
    return report.compress(compression_level)
