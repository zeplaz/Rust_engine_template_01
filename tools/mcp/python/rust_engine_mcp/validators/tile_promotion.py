"""TILE-FIX-010 — promotion gates (geometry, materials, minimum bake, atlas v2, shell GLBs)."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.building_definition import (
    MINIMUM_G4_CELLS,
    expand_bake_matrix_minimum,
    load_building_definition,
    production_shell_modules_ready,
)
from rust_engine_mcp.paths import repo_root

from .assembly_production import validate_assembly_snapshot_path
from .atlas_meta import validate_atlas_meta_v2
from .material_textures import validate_material_textures
from .report import ValidationIssue, ValidationReport


def _variant_facings_pixel_identical(staging: Path, variant_key: str, facings: int) -> bool:
    """True when facings f1..fN match f0 (rotation not baked — visually same tile)."""
    try:
        from PIL import Image
    except ImportError:
        return False

    ref_path = staging / f"{variant_key}_f0.png"
    if not ref_path.is_file():
        return False
    ref = Image.open(ref_path).convert("RGBA")
    ref_bytes = ref.tobytes()
    for facing in range(1, facings):
        png = staging / f"{variant_key}_f{facing}.png"
        if not png.is_file():
            return False
        other = Image.open(png).convert("RGBA")
        if other.size != ref.size:
            return False
        if other.tobytes() != ref_bytes:
            # Allow PNG metadata / 1-pixel compression noise only.
            diff = sum(1 for a, b in zip(ref_bytes, other.tobytes()) if a != b)
            if diff > 16:
                return False
    return True


def validate_tile_promotion(
    *,
    building_definition_path: str | Path,
    batch: dict[str, Any] | None = None,
    meta_path: str | Path | None = None,
    staging_dir: str | Path | None = None,
    ship: bool = True,
    compression_level: int = 3,
) -> ValidationReport:
    issues: list[ValidationIssue] = []
    root = repo_root()
    bdef_path = Path(building_definition_path)
    if not bdef_path.is_absolute():
        bdef_path = root / bdef_path

    if not bdef_path.is_file():
        return ValidationReport(
            validator="tile",
            status="failed",
            compression_level=compression_level,
            summary="building_definition missing",
            error_count=1,
            errors=[
                ValidationIssue(
                    kind="MissingFile",
                    severity="error",
                    file=str(bdef_path),
                    signature="tile_promotion_bdef_missing",
                )
            ],
        )

    defn = load_building_definition(bdef_path)
    shell_ok, shell_blockers = production_shell_modules_ready(defn)

    stage = Path(staging_dir) if staging_dir else root / "assets/staging/tiles" / f"tile_{defn.building_id}_v2_minimum_g4"
    if not stage.is_absolute():
        stage = root / stage
    meta = Path(meta_path) if meta_path else stage / "atlas_meta.json"
    if not meta.is_absolute():
        meta = root / meta

    if batch is None:
        batch = {
            "batch_id": stage.name,
            "tile_id": defn.building_id,
            "dry_run": False,
            "visual_config_ref": "assets/configs/buildings/visual_config_warehouse_industrial_west_v2.json",
        }
    if ship and not shell_ok:
        for hint in shell_blockers:
            issues.append(
                ValidationIssue(
                    kind="Lod0Shell",
                    severity="error",
                    file=str(bdef_path),
                    field="modules",
                    hint=f"TILE-FIX-010: promote production wall/roof GLBs — {hint}",
                    signature="tile_promotion_shell_lod0",
                )
            )

    if defn.assembly_snapshot:
        snap = root / defn.assembly_snapshot
        asm_rep = validate_assembly_snapshot_path(snap, ship=ship)
        issues.extend(asm_rep.errors)

    for profile in defn.material_profiles or []:
        mat_rep = validate_material_textures(
            {"development_tier": "production", "material_profile": profile},
            ship=ship,
        )
        issues.extend(mat_rep.errors)

    minimum_cells = expand_bake_matrix_minimum(defn)
    if len(minimum_cells) != MINIMUM_G4_CELLS:
        issues.append(
            ValidationIssue(
                kind="MatrixIncomplete",
                severity="error",
                file=str(bdef_path),
                hint=f"minimum G4 matrix must be {MINIMUM_G4_CELLS} cells (got {len(minimum_cells)})",
                signature="tile_promotion_minimum_cell_count",
            )
        )

    from rust_engine_mcp.atlas_meta_v2_pack import cell_png_basename

    missing_png = [
        cell_png_basename(c)
        for c in minimum_cells
        if not (stage / cell_png_basename(c)).is_file()
    ]
    if missing_png and ship:
        issues.append(
            ValidationIssue(
                kind="MissingBake",
                severity="error",
                file=str(stage),
                hint=f"{len(missing_png)} minimum cell PNG(s) missing — run tile_compile_minimum_bake or export manual keyframes",
                signature="tile_promotion_minimum_png_missing",
            )
        )

    vc_rel = str(batch.get("visual_config_ref") or "")
    if meta.is_file() and not vc_rel:
        try:
            meta_data = json.loads(meta.read_text(encoding="utf-8"))
            vc_rel = str(meta_data.get("visual_config") or "")
        except json.JSONDecodeError:
            pass
    vc_path = root / vc_rel if vc_rel else None
    if meta.is_file():
        meta_rep = validate_atlas_meta_v2(meta, visual_config_path=vc_path)
        issues.extend(meta_rep.errors)
        try:
            meta_data = json.loads(meta.read_text(encoding="utf-8"))
            if len(meta_data.get("lookups") or []) < MINIMUM_G4_CELLS:
                issues.append(
                    ValidationIssue(
                        kind="LookupIncomplete",
                        severity="error",
                        file=str(meta),
                        hint=f"atlas_meta v2 needs >= {MINIMUM_G4_CELLS} lookups for minimum G4 ship",
                        signature="tile_promotion_minimum_lookups",
                    )
                )
            if ship and not bool(meta_data.get("minimum_g4_ship")):
                issues.append(
                    ValidationIssue(
                        kind="MinimumG4ShipFlag",
                        severity="error",
                        file=str(meta),
                        hint="atlas_meta.minimum_g4_ship must be true for minimum G4 promotion",
                        signature="tile_promotion_minimum_g4_ship_flag",
                    )
                )
        except json.JSONDecodeError:
            pass

    if ship:
        atlas_out = ""
        if batch:
            atlas_out = str((batch.get("atlas") or {}).get("output_png") or "")
        if "buildings_iso/production" in atlas_out.replace("\\", "/"):
            issues.append(
                ValidationIssue(
                    kind="ForbiddenShipPath",
                    severity="error",
                    file=str(bdef_path),
                    hint=(
                        "ship atlas must not live under buildings_iso/production "
                        "(headless debug only) — use staging + manual keyframe_render"
                    ),
                    signature="tile_promotion_forbidden_production_path",
                )
            )
        render_method = str((batch or {}).get("render", {}).get("method") or "")
        if render_method == "blender_keyframe_light_rig":
            import os

            if os.environ.get("RUST_ENGINE_TILE_KEYFRAME_HEADLESS") == "1":
                issues.append(
                    ValidationIssue(
                        kind="HeadlessNotShipArt",
                        severity="error",
                        file=str(bdef_path),
                        hint=(
                            "headless tile_keyframe_bake is schema/CI only — "
                            "ship requires manual keyframe_render + designer G4"
                        ),
                        signature="tile_promotion_headless_not_ship",
                    )
                )

    if ship and stage.is_dir():
        pilot_variants = ("clean_day", "clean_night_on", "burning_00")
        facings = int((defn.render_contract or {}).get("facings") or 8)
        for variant_key in pilot_variants:
            if _variant_facings_pixel_identical(stage, variant_key, facings):
                issues.append(
                    ValidationIssue(
                        kind="FacingRotationMissing",
                        severity="error",
                        file=str(stage / f"{variant_key}_f0.png"),
                        hint=(
                            f"{variant_key}: f0..f{facings - 1} are pixel-identical — "
                            "facing yaw was not applied at bake (not shippable)"
                        ),
                        signature="tile_promotion_facing_rotation_missing",
                    )
                )

    if ship and batch and not bool(batch.get("dry_run", True)):
        pass
    elif ship and batch is None:
        issues.append(
            ValidationIssue(
                kind="MissingBatch",
                severity="warning",
                file=str(bdef_path),
                hint="pass batch with dry_run:false after bake for ship witness",
                signature="tile_promotion_batch_context",
            )
        )

    errors = [i for i in issues if i.severity == "error"]
    status = "failed" if errors else ("warning" if issues else "passed")
    return ValidationReport(
        validator="tile",
        status=status,
        compression_level=compression_level,
        summary=f"tile_promotion: {len(errors)} error(s), shell_ok={shell_ok}",
        error_count=len(errors),
        errors=issues,
    )
