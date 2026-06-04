"""Agent-facing validators — structured reports, not raw logs."""

from __future__ import annotations

from pathlib import Path

from .asset import validate_asset_glb
from .bevy import validate_bevy
from .cargo import validate_cargo
from .mcp_schema import validate_mcp_job, validate_mcp_spec
from .atlas_meta import validate_atlas_meta_v2
from .tile_batch import validate_tile_batch
from .visual_config import validate_visual_config
from .assembly_production import validate_assembly_snapshot, validate_assembly_snapshot_path
from .assembly_grammar_verify import (
    validate_assembly_grammar_verify,
    validate_assembly_grammar_verify_path,
    validate_assembly_p0_gate,
)
from .material_textures import validate_material_textures, validate_material_textures_path
from .tile_promotion import validate_tile_promotion
from .report import ValidationReport

__all__ = [
    "validate_cargo",
    "validate_bevy",
    "validate_mcp_spec",
    "validate_mcp_job",
    "validate_asset_glb",
    "validate_tile_batch",
    "validate_atlas_meta_v2",
    "validate_visual_config",
    "validate_assembly_snapshot",
    "validate_assembly_snapshot_path",
    "validate_assembly_grammar_verify",
    "validate_assembly_grammar_verify_path",
    "validate_assembly_p0_gate",
    "validate_material_textures",
    "validate_material_textures_path",
    "validate_tile_promotion",
    "run_validator",
]


def run_validator(
    name: str,
    target: str | None = None,
    *,
    package: str | None = None,
    compression_level: int = 3,
    use_cached: bool = False,
) -> ValidationReport:
    from rust_engine_mcp.paths import repo_root

    def _resolve(p: str | None) -> Path | None:
        if not p:
            return None
        path = Path(p)
        if not path.is_absolute():
            path = repo_root() / path
        return path

    if name == "cargo":
        return validate_cargo(
            package=package,
            use_cached_orchestrator=use_cached,
            compression_level=compression_level,
        )
    if name == "bevy":
        return validate_bevy(package=package, compression_level=compression_level)
    if name == "mcp_spec":
        if not target:
            raise ValueError("target path required for mcp_spec")
        return validate_mcp_spec(_resolve(target), compression_level=compression_level)
    if name == "mcp_job":
        if not target:
            raise ValueError("target path required for mcp_job")
        return validate_mcp_job(_resolve(target), compression_level=compression_level)
    if name == "asset_glb":
        if not target:
            raise ValueError("target path required for asset_glb")
        return validate_asset_glb(_resolve(target), compression_level=compression_level)
    if name == "tile_batch":
        if not target:
            raise ValueError("target path required for tile_batch")
        return validate_tile_batch(_resolve(target), compression_level=compression_level)
    if name == "atlas_meta_v2":
        if not target:
            raise ValueError("target path required for atlas_meta_v2")
        resolved = _resolve(target)
        vc: Path | None = None
        if resolved and resolved.is_file():
            try:
                import json

                meta = json.loads(resolved.read_text(encoding="utf-8"))
                vc_rel = str(meta.get("visual_config") or "")
                if vc_rel:
                    vc = _resolve(vc_rel)
            except (OSError, json.JSONDecodeError):
                pass
        return validate_atlas_meta_v2(resolved, visual_config_path=vc, compression_level=compression_level)
    if name == "visual_config":
        if not target:
            raise ValueError("target path required for visual_config")
        return validate_visual_config(_resolve(target), compression_level=compression_level)
    if name == "tile_promotion":
        if not target:
            raise ValueError("target path required for tile_promotion (building_definition JSON)")
        return validate_tile_promotion(
            building_definition_path=_resolve(target),
            ship=True,
            compression_level=compression_level,
        )
    if name == "assembly_grammar":
        if not target:
            raise ValueError("target path required for assembly_grammar (assembly snapshot JSON)")
        return validate_assembly_grammar_verify_path(
            _resolve(target), ship=True, compression_level=compression_level, full_p0=False
        )
    if name == "assembly_p0":
        if not target:
            raise ValueError("target path required for assembly_p0 (assembly snapshot JSON)")
        return validate_assembly_grammar_verify_path(
            _resolve(target), ship=True, compression_level=compression_level, full_p0=True
        )
    if name == "assembly_production":
        if not target:
            raise ValueError("target path required for assembly_production (assembly snapshot JSON)")
        return validate_assembly_snapshot_path(_resolve(target), ship=True, compression_level=compression_level)
    raise ValueError(f"unknown validator: {name}")
