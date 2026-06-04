"""Tile batch v1 validator — schema-only (MCP-T0-002, no execution)."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .report import ValidationIssue, ValidationReport

REQUIRED_RULES = frozenset(
    {
        "no_ai_generated_images",
        "deterministic_output",
        "batch_processing",
        "grid_alignment",
    }
)

VALID_BASES = frozenset({"wood", "stone", "concrete", "dirt", "asphalt", "metal_plate"})
VALID_STATES = frozenset({"clean", "dirty", "damaged", "ruined"})
VALID_POWER = frozenset({"off", "partial", "on"})
VALID_FILL = frozenset({"empty", "quarter", "half", "full"})
VALID_LIGHTING = frozenset({"day", "night_off", "night_on"})
VALID_BAKE_SOURCES = frozenset({"keyframe_pack", "smoke_ortho_headless"})
VALID_RENDER_METHODS = frozenset(
    {"blender_keyframe_light_rig", "blender_orthographic_iso"}
)


def validate_tile_batch(path: Path, *, compression_level: int = 3) -> ValidationReport:
    issues: list[ValidationIssue] = []
    data: dict[str, Any] = {}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        issues.append(
            ValidationIssue(
                kind="SchemaInvalid",
                severity="error",
                file=str(path),
                hint=str(exc),
                signature="tile_batch_parse",
            )
        )
        return ValidationReport(
            validator="tile_batch",
            status="failed",
            errors=issues,
            known_fixes=[],
            summary=str(exc),
            compression_level=compression_level,
        )

    if data.get("schema_version") != 1:
        issues.append(
            ValidationIssue(
                kind="SchemaInvalid",
                severity="error",
                file=str(path),
                field="schema_version",
                hint="schema_version must be 1",
                signature="tile_batch_schema_version",
            )
        )

    batch_id = str(data.get("batch_id") or data.get("tile_id") or "")
    if not batch_id:
        issues.append(
            ValidationIssue(
                kind="MissingField",
                severity="error",
                file=str(path),
                field="batch_id",
                hint="batch_id required",
                signature="tile_batch_missing_batch_id",
            )
        )

    tile_id = str(data.get("tile_id") or data.get("tile") or "")
    if not tile_id:
        issues.append(
            ValidationIssue(
                kind="MissingField",
                severity="error",
                file=str(path),
                field="tile_id",
                hint="tile_id required",
                signature="tile_batch_missing_tile_id",
            )
        )

    base = str(data.get("base") or "")
    if base not in VALID_BASES:
        issues.append(
            ValidationIssue(
                kind="InvalidEnum",
                severity="error",
                file=str(path),
                field="base",
                hint=f"base must be one of {sorted(VALID_BASES)}",
                signature="tile_batch_invalid_base",
            )
        )

    rules = {str(r) for r in (data.get("rules_applied") or [])}
    missing_rules = REQUIRED_RULES - rules
    for rule in sorted(missing_rules):
        issues.append(
            ValidationIssue(
                kind="MissingField",
                severity="error",
                file=str(path),
                field="rules_applied",
                hint=f"missing rule {rule}",
                signature="tile_batch_missing_rule",
            )
        )

    bake_source = str(data.get("bake_source") or "smoke_ortho_headless")
    if bake_source not in VALID_BAKE_SOURCES:
        issues.append(
            ValidationIssue(
                kind="InvalidEnum",
                severity="error",
                file=str(path),
                field="bake_source",
                hint=f"bake_source must be one of {sorted(VALID_BAKE_SOURCES)}",
                signature="tile_batch_invalid_bake_source",
            )
        )

    render = data.get("render") or {}
    render_method = str(render.get("method") or "")
    if render_method and render_method not in VALID_RENDER_METHODS:
        issues.append(
            ValidationIssue(
                kind="InvalidEnum",
                severity="error",
                file=str(path),
                field="render.method",
                hint=f"render.method must be one of {sorted(VALID_RENDER_METHODS)}",
                signature="tile_batch_render_method",
            )
        )
    if "seed" not in render:
        issues.append(
            ValidationIssue(
                kind="MissingField",
                severity="error",
                file=str(path),
                field="render.seed",
                hint="render.seed required (deterministic_output)",
                signature="tile_batch_missing_seed",
            )
        )

    variants = data.get("variants") or []
    if len(variants) < 2:
        issues.append(
            ValidationIssue(
                kind="BatchTooSmall",
                severity="error",
                file=str(path),
                field="variants",
                hint="min 2 variants per batch",
                signature="tile_batch_variants_min",
            )
        )

    for i, var in enumerate(variants):
        if not isinstance(var, dict):
            continue
        state = str(var.get("state") or "")
        if state and state not in VALID_STATES:
            issues.append(
                ValidationIssue(
                    kind="InvalidEnum",
                    severity="error",
                    file=str(path),
                    field=f"variants[{i}].state",
                    hint=f"invalid state {state!r}",
                    signature="tile_batch_variant_state",
                )
            )
        power = str(var.get("power") or "")
        if power and power not in VALID_POWER:
            issues.append(
                ValidationIssue(
                    kind="InvalidEnum",
                    severity="error",
                    file=str(path),
                    field=f"variants[{i}].power",
                    hint=f"invalid power {power!r}",
                    signature="tile_batch_variant_power",
                )
            )
        fill = str(var.get("fill") or "empty")
        if fill not in VALID_FILL:
            issues.append(
                ValidationIssue(
                    kind="InvalidEnum",
                    severity="warning",
                    file=str(path),
                    field=f"variants[{i}].fill",
                    hint=f"fill should be in {sorted(VALID_FILL)}",
                    signature="tile_batch_variant_fill",
                )
            )
        lighting = str(var.get("lighting") or "")
        if lighting and lighting not in VALID_LIGHTING:
            issues.append(
                ValidationIssue(
                    kind="InvalidEnum",
                    severity="error",
                    file=str(path),
                    field=f"variants[{i}].lighting",
                    hint=f"invalid lighting {lighting!r}",
                    signature="tile_batch_variant_lighting",
                )
            )

    if str(data.get("status") or "").upper() == "PLANNED" and not issues:
        issues.append(
            ValidationIssue(
                kind="PlannedDraft",
                severity="warning",
                file=str(path),
                hint="draft marked PLANNED — validate-only, no tile.generate",
                signature="tile_batch_planned_draft",
            )
        )

    ship = bool(data.get("ship"))
    source_tier = str(data.get("source_tier") or data.get("development_tier") or "lod0").lower()
    if ship and bake_source != "keyframe_pack":
        issues.append(
            ValidationIssue(
                kind="BakeSourceMismatch",
                severity="error",
                file=str(path),
                field="bake_source",
                hint=(
                    "ship: true requires bake_source keyframe_pack "
                    "(keyframe_render PNG folder → tile-atlas-pack); "
                    "see design_tile_bake_spine_convergence_v1.md"
                ),
                signature="tile_batch_ship_requires_keyframe_pack",
            )
        )
    if ship and render_method == "blender_orthographic_iso":
        issues.append(
            ValidationIssue(
                kind="BakeSourceMismatch",
                severity="error",
                file=str(path),
                field="render.method",
                hint="ship batches must use render.method blender_keyframe_light_rig",
                signature="tile_batch_ship_ortho_render_rejected",
            )
        )
    if ship and source_tier == "lod0":
        issues.append(
            ValidationIssue(
                kind="TierMismatch",
                severity="error",
                file=str(path),
                field="source_tier",
                hint="ship: true batches must use source_tier/development_tier production (PT-2-003)",
                signature="tile_batch_ship_lod0_rejected",
            )
        )

    if bool(data.get("frozen")):
        issues.append(
            ValidationIssue(
                kind="FrozenBatch",
                severity="error",
                file=str(path),
                hint="frozen batch — TILE-FIX-001 greybox v1; set ship: false and use v2 pipeline",
                signature="tile_batch_frozen",
            )
        )

    atlas_schema_version = int(data.get("atlas_schema_version") or 1)
    if ship and atlas_schema_version < 2:
        issues.append(
            ValidationIssue(
                kind="SchemaInvalid",
                severity="error",
                file=str(path),
                field="atlas_schema_version",
                hint="ship: true requires atlas_schema_version >= 2 (variant×facing×frame)",
                signature="tile_batch_ship_requires_atlas_v2",
            )
        )

    visual_config_ref = str(data.get("visual_config_ref") or "")
    if ship and not visual_config_ref:
        issues.append(
            ValidationIssue(
                kind="MissingField",
                severity="error",
                file=str(path),
                field="visual_config_ref",
                hint="ship batches require visual_config_ref (.json)",
                signature="tile_batch_ship_visual_config",
            )
        )

    render_contract = data.get("render_contract") or {}
    if ship:
        facings = int(render_contract.get("facings") or 0)
        if facings not in (4, 8):
            issues.append(
                ValidationIssue(
                    kind="MissingField",
                    severity="error",
                    file=str(path),
                    field="render_contract.facings",
                    hint="ship batches require render_contract.facings 4 or 8",
                    signature="tile_batch_ship_facings",
                )
            )

    if ship:
        from rust_engine_mcp.paths import repo_root

        from .assembly_production import validate_assembly_snapshot_path

        snap_rel = str(data.get("assembly_snapshot") or "")
        if snap_rel:
            snap_path = Path(snap_rel)
            if not snap_path.is_absolute():
                snap_path = repo_root() / snap_rel
            asm_rep = validate_assembly_snapshot_path(snap_path, ship=True)
            issues.extend(asm_rep.errors)

        bdef_rel = str(data.get("building_definition") or "")
        if bdef_rel:
            try:
                from rust_engine_mcp.building_definition import load_building_definition
                from rust_engine_mcp.tile_compile_loop import validate_compile_preconditions

                defn = load_building_definition(repo_root() / bdef_rel)
                for hint in validate_compile_preconditions(defn, ship=True):
                    issues.append(
                        ValidationIssue(
                            kind="CompilePrecondition",
                            severity="error",
                            file=str(path),
                            field="building_definition",
                            hint=hint,
                            signature="tile_batch_compile_precondition",
                        )
                    )
            except (OSError, json.JSONDecodeError, ValueError) as exc:
                issues.append(
                    ValidationIssue(
                        kind="SchemaInvalid",
                        severity="error",
                        file=str(path),
                        field="building_definition",
                        hint=str(exc),
                        signature="tile_batch_building_definition",
                    )
                )

    errors = [i for i in issues if i.severity == "error"]
    status = "failed" if errors else ("warning" if issues else "passed")
    return ValidationReport(
        validator="tile_batch",
        status=status,
        errors=issues,
        known_fixes=[],
        summary=f"{path.name}: variants={len(variants)} base={base} tier=schema-only",
        compression_level=compression_level,
    )
