"""SPINE-RENDER-001 — headless blender-worker contract.

MCP submits ``render_variant`` jobs. Production / ship paths never accept
greybox ortho (``blender_orthographic_iso`` / ``smoke_ortho_headless``).
"""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

import jsonschema

from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness
from rust_engine_mcp.paths import repo_root, schemas_dir
from rust_engine_mcp.tile_promotion_honest import tile_promotion_honest_check
from rust_engine_mcp.validators.report import ValidationIssue, ValidationReport

TASK_ID = "SPINE-RENDER-001"
GATE = "SPINE-RENDER-001"
WITNESS_REL = "debug_runs/spine_render_001_live.json"
SCHEMA_NAME = "render_variant_job_v1.schema.json"
EXAMPLE_REL = "tools/mcp/schemas/examples/render_variant_warehouse_damage_heavy_v1.json"
EXAMPLE_BATCH_REL = (
    "tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_production_v1.json"
)
NEXT_RESIDUAL = "SPINE-BUILD-001"

PRODUCTION_METHODS = frozenset({"blender_keyframe_light_rig"})
SMOKE_METHODS = frozenset({"blender_orthographic_iso"})
PRODUCTION_BAKE = frozenset({"keyframe_pack"})
SMOKE_BAKE = frozenset({"smoke_ortho_headless"})


def _load_schema() -> dict[str, Any]:
    return json.loads((schemas_dir() / SCHEMA_NAME).read_text(encoding="utf-8"))


def validate_render_variant_job(data: dict[str, Any]) -> None:
    """JSON Schema + production/smoke role rules (raises jsonschema / ValueError)."""
    jsonschema.validate(instance=data, schema=_load_schema())
    errors = contract_errors(data)
    if errors:
        raise ValueError("; ".join(errors))


def contract_errors(job: dict[str, Any]) -> list[str]:
    """Return human-readable contract violations (empty = ok)."""
    errors: list[str] = []
    role = str(job.get("role") or "")
    ship = bool(job.get("ship"))
    render = job.get("render") if isinstance(job.get("render"), dict) else {}
    method = str(render.get("method") or "")
    bake = str(render.get("bake_source") or "")

    if role == "production":
        if method and method not in PRODUCTION_METHODS:
            errors.append(
                f"production role forbids render.method={method!r} "
                f"(need {sorted(PRODUCTION_METHODS)})"
            )
        if bake and bake not in PRODUCTION_BAKE:
            errors.append(
                f"production role forbids bake_source={bake!r} "
                f"(need {sorted(PRODUCTION_BAKE)})"
            )
    elif role == "smoke":
        if ship:
            errors.append("smoke role cannot ship=true — greybox ortho is CI-only")
        if method and method not in SMOKE_METHODS:
            errors.append(
                f"smoke role expects ortho method "
                f"(got {method!r}; allowed {sorted(SMOKE_METHODS)})"
            )

    if ship:
        if role != "production":
            errors.append("ship=true requires role=production")
        if method in SMOKE_METHODS:
            errors.append(
                "ship=true rejects blender_orthographic_iso — "
                "use blender_keyframe_light_rig + keyframe_pack"
            )
        if bake in SMOKE_BAKE:
            errors.append(
                "ship=true rejects smoke_ortho_headless bake_source — "
                "use keyframe_pack"
            )
        if method and method not in PRODUCTION_METHODS:
            errors.append(
                f"ship=true requires production render.method "
                f"(got {method!r})"
            )

    if job.get("seed") is None:
        errors.append("seed required for deterministic blender-worker jobs")

    return errors


def classify_render_job(job: dict[str, Any]) -> dict[str, Any]:
    errs = contract_errors(job)
    role = str(job.get("role") or "")
    method = str((job.get("render") or {}).get("method") or "")
    return {
        "role": role,
        "ship": bool(job.get("ship")),
        "method": method,
        "ok": not errs,
        "errors": errs,
        "production_capable": role == "production"
        and method in PRODUCTION_METHODS
        and not errs,
    }


def validate_render_variant_path(
    path: Path | str,
    *,
    compression_level: int = 3,
) -> ValidationReport:
    p = Path(path)
    if not p.is_absolute():
        p = repo_root() / p
    issues: list[ValidationIssue] = []
    try:
        data = json.loads(p.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        issues.append(
            ValidationIssue(
                kind="RenderVariantLoadError",
                severity="error",
                file=str(p),
                hint=str(exc),
                signature="render_variant_load_error",
            )
        )
        return ValidationReport(
            validator="render_variant",
            status="failed",
            compression_level=compression_level,
            summary=f"render_variant load failed: {exc}",
            error_count=1,
            warning_count=0,
            errors=issues,
        ).compress(compression_level)

    try:
        jsonschema.validate(instance=data, schema=_load_schema())
    except jsonschema.ValidationError as exc:
        issues.append(
            ValidationIssue(
                kind="RenderVariantSchema",
                severity="error",
                file=str(p),
                hint=exc.message,
                signature="render_variant_schema",
            )
        )

    for msg in contract_errors(data):
        issues.append(
            ValidationIssue(
                kind="RenderVariantContract",
                severity="error",
                file=str(p),
                hint=msg,
                signature="render_variant_contract",
            )
        )

    status = "passed" if not issues else "failed"
    summary = (
        f"render_variant: {data.get('job_id') or data.get('asset') or p.name}"
        if status == "passed"
        else f"render_variant blocked: {issues[0].hint}"
    )
    return ValidationReport(
        validator="render_variant",
        status=status,  # type: ignore[arg-type]
        compression_level=compression_level,
        summary=summary,
        error_count=sum(1 for i in issues if i.severity == "error"),
        warning_count=sum(1 for i in issues if i.severity == "warning"),
        errors=issues,
    ).compress(compression_level)


def _smoke_ortho_job() -> dict[str, Any]:
    return {
        "schema_version": 1,
        "job": "render_variant",
        "job_id": "render_variant_smoke_ortho_reject",
        "asset": "warehouse_industrial",
        "variant": "clean",
        "facing": 0,
        "frame": 0,
        "seed": 1,
        "role": "smoke",
        "ship": True,
        "render": {
            "method": "blender_orthographic_iso",
            "bake_source": "smoke_ortho_headless",
        },
    }


def run_spine_render_audit(*, repo: Path | None = None) -> dict[str, Any]:
    """Prove schema + contract: production example ok; ship+ortho rejected."""
    root = repo or repo_root()
    example_path = root / EXAMPLE_REL
    example = json.loads(example_path.read_text(encoding="utf-8"))

    schema_ok = False
    schema_error: str | None = None
    try:
        validate_render_variant_job(example)
        schema_ok = True
    except (jsonschema.ValidationError, ValueError) as exc:
        schema_error = str(exc)

    prod_class = classify_render_job(example)
    smoke_job = _smoke_ortho_job()
    smoke_class = classify_render_job(smoke_job)
    smoke_rejected = not smoke_class["ok"]

    # Honest bake gate still rejects ortho tile_batch ship paths.
    smoke_batch = {
        "batch_id": "spine_render_ortho_reject",
        "bake_source": "smoke_ortho_headless",
        "render": {"method": "blender_orthographic_iso", "seed": 1},
        "ship": True,
    }
    smoke_batch_path = root / "assets/staging/tiles/_spine_render_ortho_reject.json"
    smoke_batch_path.parent.mkdir(parents=True, exist_ok=True)
    smoke_batch_path.write_text(json.dumps(smoke_batch, indent=2) + "\n", encoding="utf-8")
    honest = tile_promotion_honest_check(
        batch_path=smoke_batch_path, ship=True, honest_bake=True
    )
    honest_rejects_ortho = not bool(honest.get("ok"))

    batch_path = root / EXAMPLE_BATCH_REL
    batch = json.loads(batch_path.read_text(encoding="utf-8"))
    batch_method = str((batch.get("render") or {}).get("method") or "")
    batch_bake = str(batch.get("bake_source") or "")
    example_batch_not_ortho = (
        batch_method not in SMOKE_METHODS and batch_bake not in SMOKE_BAKE
    )

    green = bool(
        schema_ok
        and prod_class.get("ok")
        and smoke_rejected
        and honest_rejects_ortho
        and example_batch_not_ortho
    )

    return {
        "gate": GATE,
        "task_id": TASK_ID,
        "green": green,
        "ok": green,
        "schema": SCHEMA_NAME,
        "example": EXAMPLE_REL.replace("\\", "/"),
        "checks": {
            "example_schema_and_contract": {
                "ok": schema_ok and prod_class.get("ok"),
                "schema_ok": schema_ok,
                "schema_error": schema_error,
                "classification": prod_class,
            },
            "ship_ortho_job_rejected": {
                "ok": smoke_rejected,
                "classification": smoke_class,
            },
            "tile_promotion_honest_rejects_ortho": {
                "ok": honest_rejects_ortho,
                "errors": honest.get("errors") or [],
            },
            "example_tile_batch_not_ortho": {
                "ok": example_batch_not_ortho,
                "batch": EXAMPLE_BATCH_REL.replace("\\", "/"),
                "render_method": batch_method,
                "bake_source": batch_bake,
            },
        },
        "worker_roles": {
            "production": sorted(PRODUCTION_METHODS),
            "smoke": sorted(SMOKE_METHODS),
        },
        "cli": "spine-render-001 / validate-report render_variant <job.json>",
        "next_residual": NEXT_RESIDUAL,
    }


def write_spine_render_001_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = run_spine_render_audit(repo=root)
    meta_extra = {
        "track": "PLAN-MCP-APS-TOOLING-FINISH-001",
        "task_id": TASK_ID,
        "proceed_ship": False,
        "art_quality": "contract_only_no_ship_art",
        "agent": "coder-mcp",
    }
    out = write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="spine_render_001_live_v1",
        profile="SPINE_RENDER",
        source_system="spine_render_contract",
        ritual=f"BLANG:WIT-HON→Q✓ {TASK_ID}" if body.get("green") else None,
        exit_predicate_must=[
            {"field": "checks.example_schema_and_contract.ok", "equals": True},
            {"field": "checks.ship_ortho_job_rejected.ok", "equals": True},
            {"field": "checks.tile_promotion_honest_rejects_ortho.ok", "equals": True},
        ],
        repo=root,
    )
    out.pop("_agent_meta_extra", None)
    meta = dict(out.get("_agent_meta") or {})
    meta.update(meta_extra)
    meta["written_at_epoch_secs"] = int(time.time())
    if out.get("green"):
        meta["ritual"] = f"BLANG:WIT-HON→Q✓ {TASK_ID}"
    out["_agent_meta"] = meta
    (root / WITNESS_REL).write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
    out["written"] = WITNESS_REL
    return out
