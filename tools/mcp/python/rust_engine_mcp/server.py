"""
Rust Engine Art MCP — thin wrappers over rust_engine_mcp.cli micro-tools.

Design rule: MCP tools delegate to CLI/functions; the LLM chooses tools, not mesh steps.
"""

from __future__ import annotations

import json

from pathlib import Path

try:
    from mcp.server.fastmcp import FastMCP
except ImportError as exc:  # pragma: no cover
    raise SystemExit(
        "Install: pip install -r tools/mcp/requirements.txt && pip install -e tools/mcp/python"
    ) from exc

from rust_engine_mcp import blender_runner, library, paths, promote, schemas, validate_glb
from rust_engine_mcp.tile_pipeline import (
    assembly_build_run,
    lod0_batch_run,
    tile_atlas_pack,
    tile_batch_run,
    tile_batch_status,
    tile_keyframe_export,
)
from rust_engine_mcp import assembly
from rust_engine_mcp import variant_set
from rust_engine_mcp.tile_index import register_tile_atlas_from_batch
from rust_engine_mcp.validators import run_validator
from rust_engine_mcp import grammar_build_set

mcp = FastMCP("rust-engine-art")


@mcp.tool()
def ping() -> str:
    """Health check: repo path and blender resolution."""
    try:
        blender = str(paths.blender_exe())
    except FileNotFoundError as e:
        blender = f"MISSING: {e}"
    return json.dumps({"ok": True, "repo": str(paths.repo_root()), "blender_exe": blender})


@mcp.tool()
def locate_blender() -> str:
    """Return resolved Blender executable path."""
    return json.dumps({"blender_exe": str(paths.blender_exe())})


@mcp.tool()
def spec_validate(spec_json: str) -> str:
    """Validate AssetSpec JSON string (schema only, no write)."""
    data = json.loads(spec_json)
    schemas.validate_asset_spec(data)
    return json.dumps({"valid": True, "asset_id": data.get("asset_id")})


@mcp.tool()
def spec_write(spec_json: str) -> str:
    """Validate and write AssetSpec to assets/staging/specs/<asset_id>.json."""
    data = json.loads(spec_json)
    schemas.validate_asset_spec(data)
    asset_id = data["asset_id"]
    out = paths.staging_root() / "specs" / f"{asset_id}.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(data, indent=2), encoding="utf-8")
    return json.dumps({"written": str(out), "asset_id": asset_id})


@mcp.tool()
def geometry_operations() -> str:
    """List geometry job operation ids supported by Blender headless."""
    return json.dumps(
        {
            "operations": [
                "module_wall",
                "module_roof",
                "module_door",
                "module_window",
                "module_prop",
            ],
            "example_job": "tools/mcp/schemas/examples/wall_job.example.json",
        }
    )


@mcp.tool()
def geometry_run_job(job_path: str) -> str:
    """Run a GeometryJob JSON file in Blender headless. Returns job status."""
    result = blender_runner.run_geometry_job(Path(job_path))
    return json.dumps(result.__dict__)


@mcp.tool()
def geometry_job_status(job_id: str) -> str:
    """Poll job status written by geometry_run_job."""
    st = blender_runner.read_status(job_id)
    if st is None:
        return json.dumps({"job_id": job_id, "status": "unknown"})
    return json.dumps(st)


@mcp.tool()
def validate_glb_asset(glb_path: str) -> str:
    """Validate a .glb file (header, vertices, budget)."""
    report = validate_glb.validate_glb(Path(glb_path))
    return json.dumps(report.to_dict())


@mcp.tool()
def list_staging() -> str:
    """List job folders under assets/staging/."""
    root = paths.staging_root()
    entries = []
    if root.is_dir():
        for p in sorted(root.iterdir()):
            if p.is_dir() and p.name != "specs":
                entries.append(
                    {
                        "job_id": p.name,
                        "glb_files": [str(x.relative_to(root)) for x in p.glob("**/*.glb")],
                    }
                )
    return json.dumps({"staging": entries})


@mcp.tool()
def promote_staging_module(
    job_id: str,
    force: bool = False,
    no_register: bool = False,
    allow_smoke: bool = False,
) -> str:
    """Copy validated staging glb to assets/models/modules/<job_id>/."""
    manifest = promote.promote_module(
        job_id,
        force=force,
        register=not no_register,
        allow_smoke=allow_smoke,
    )
    return json.dumps(manifest)


@mcp.tool()
def library_register(job_id: str = "", rebuild_all: bool = False) -> str:
    """Register promoted module(s) in assets/configs/buildings/_module_index.ron."""
    if rebuild_all:
        result = library.write_module_index()
    else:
        if not job_id:
            return json.dumps({"error": "job_id required unless rebuild_all=true"})
        result = library.register_module(job_id)
    return json.dumps(result)


@mcp.tool()
def library_search(
    tags: str = "",
    archetype: str = "",
    style_pack: str = "",
    category: str = "",
    batch_id: str = "",
) -> str:
    """Search _module_index.json by style tags, archetype, style_pack, category, or batch_id."""
    tag_list = [t.strip() for t in tags.split(",") if t.strip()] or None
    rows = library.search_modules(
        tags=tag_list,
        archetype=archetype or None,
        style_pack=style_pack or None,
        category=category or None,
        batch_id=batch_id or None,
    )
    return json.dumps({"count": len(rows), "modules": rows})


@mcp.tool()
def validate_report(
    validator: str,
    target: str = "",
    package: str = "",
    compress: int = 3,
    use_cached: bool = False,
) -> str:
    """Run a structured validator (cargo/bevy/mcp_spec/mcp_job/asset_glb/param_frame/massing_preview/building_grammar/…). Returns ValidationReport JSON — not raw logs."""
    report = run_validator(
        validator,
        target or None,
        package=package or None,
        compression_level=max(1, min(4, compress)),
        use_cached=use_cached,
    )
    return json.dumps(report.to_dict())


@mcp.tool()
def grammar_preset_pair_validate_tool(preset_id: str = "", path: str = "") -> str:
    """MCP-GRAMMAR-SET-002 — ARCH-DNA preset ↔ pilot row ↔ grammar_id parity."""
    body = grammar_build_set.grammar_preset_pair_validate(
        preset_id=preset_id or None,
        path=path or None,
    )
    return json.dumps(body)


@mcp.tool()
def grammar_eval_sweep_tool(
    archetype_id: str = "IndustrialWarehouse",
    district_style: str = "industrial_west",
    seeds_json: str = "",
) -> str:
    """MCP-GRAMMAR-SET-003 — seed sweep massing/roof histogram."""
    seeds = json.loads(seeds_json) if seeds_json.strip() else None
    body = grammar_build_set.grammar_eval_sweep(
        archetype_id=archetype_id,
        district_style=district_style,
        seeds=seeds,
    )
    return json.dumps(body)


@mcp.tool()
def building_set_coverage_report_tool(set_id: str = "") -> str:
    """MCP-BUILD-SET-002 — F-axis coverage + hardcode lint rollup."""
    body = grammar_build_set.building_set_coverage_report(set_id=set_id or None)
    return json.dumps(body)


@mcp.tool()
def grammar_rule_coverage_report_tool(write_staging: bool = True) -> str:
    """PCG-06 — golden-seed grammar rule coverage table (report only; optional staging write)."""
    from rust_engine_mcp import kit_coverage_audit

    if write_staging:
        body = kit_coverage_audit.write_pcg06_grammar_rule_coverage_report()
    else:
        body = kit_coverage_audit.run_grammar_rule_coverage_report()
    return json.dumps(body)


@mcp.tool()
def validate_cargo_report(package: str = "", compress: int = 3, use_cached: bool = False) -> str:
    """cargo check → classified ValidationReport (JSON diagnostics only)."""
    report = run_validator(
        "cargo",
        package=package or None,
        compression_level=max(1, min(4, compress)),
        use_cached=use_cached,
    )
    return json.dumps(report.to_dict())


@mcp.tool()
def validate_bevy_report(package: str = "", compress: int = 3) -> str:
    """Bevy API classifier on top of cargo diagnostics."""
    report = run_validator("bevy", package=package or None, compression_level=max(1, min(4, compress)))
    return json.dumps(report.to_dict())


@mcp.tool()
def validate_asset_report(glb_path: str, compress: int = 3) -> str:
    """Structured GLB validation report for agents."""
    report = run_validator("asset_glb", glb_path, compression_level=max(1, min(4, compress)))
    return json.dumps(report.to_dict())


@mcp.tool()
def tile_atlas_pack_tool(folder: str, keyframe_rename: bool = False) -> str:
    """Pack a folder of PNG tile stills into one atlas via utils/tilemapgen (manual keyframe workflow)."""
    return json.dumps(tile_atlas_pack(folder, keyframe_rename=keyframe_rename))


@mcp.tool()
def lod0_batch_run_tool(batch_id: str, phase: str = "full") -> str:
    """Run kit_lod0 batch pipeline (g0g1|geometry|promote|full|all) — same as module viewer Pipeline tab."""
    return json.dumps(lod0_batch_run(batch_id, phase=phase))


@mcp.tool()
def tile_batch_validate(tile_batch_path: str) -> str:
    """Validate tile_batch_v1 JSON — returns ValidationReport JSON."""
    report = run_validator("tile_batch", tile_batch_path)
    return json.dumps(report.to_dict())


@mcp.tool()
def tile_batch_run(tile_batch_path: str) -> str:
    """Run full automated tile batch pipeline (validate → bake → atlas → witness)."""
    return json.dumps(tile_batch_run(tile_batch_path))


@mcp.tool()
def assembly_snapshot_generate(
    style_pack: str = "style_victorian",
    footprint: str = "4x3",
    floors: int = 2,
    seed: int = 42,
    source_tier: str = "production",
) -> str:
    """Generate assembly_snapshot_v1 JSON from StylePack + footprint (no Blender).

    Default source_tier=production (BQ-ASSEMBLE-PREF-PROD-001) — lod0 only when explicit.
    """
    w, d = footprint.lower().split("x")
    tier = (source_tier or "production").strip().lower()
    if tier not in ("production", "lod0"):
        tier = "production"
    snap = assembly.generate_assembly_snapshot(
        style_pack_id=style_pack,
        width=int(w),
        depth=int(d),
        floors=floors,
        seed=seed,
        source_tier=tier,
    )
    return json.dumps(snap)


@mcp.tool()
def assembly_build_run(snapshot_path: str) -> str:
    """Headless Blender assembly_build from snapshot JSON."""
    return json.dumps(assembly_build_run(snapshot_path))


@mcp.tool()
def tile_batch_status(batch_id: str) -> str:
    """Read tile batch status JSON from staging."""
    return json.dumps(tile_batch_status(batch_id))


@mcp.tool()
def variant_set_validate(variant_set_path: str) -> str:
    """Validate variant_set_v1 JSON/RON against schema."""
    return json.dumps(variant_set.variant_set_validate(variant_set_path))


@mcp.tool()
def variant_set_patch(variant_set_path: str, patch_json: str) -> str:
    """Apply RFC6902-style patch to variant_set_v1 (deterministic)."""
    patch = json.loads(patch_json)
    if isinstance(patch, dict) and "patch" in patch:
        patch = patch["patch"]
    return json.dumps(variant_set.variant_set_patch(variant_set_path, patch))


@mcp.tool()
def variant_bake(variant_set_path: str, variant_key: str, seed: int = 0) -> str:
    """Bake one variant_key — PNG + bake.status on variant row."""
    return json.dumps(
        variant_set.variant_bake(
            variant_set_path,
            variant_key,
            seed=seed or None,
        )
    )


@mcp.tool()
def variant_agent_request(request_json: str) -> str:
    """Stub agent callback — returns suggested variant_set_patch proposal (no LLM)."""
    body = json.loads(request_json)
    return json.dumps(variant_set.variant_agent_request(body))


@mcp.tool()
def tile_atlas_register(batch_id: str, batch_json: str = "") -> str:
    """Upsert atlas row in assets/configs/buildings/_tile_atlas_index.ron from batch staging."""
    return json.dumps(
        register_tile_atlas_from_batch(batch_id, tile_batch_path=batch_json or None)
    )


@mcp.tool()
def tile_keyframe_export_tool(tile_batch_path: str) -> str:
    """Optional headless keyframe still export (Light rig). Requires RUST_ENGINE_TILE_KEYFRAME_HEADLESS=1."""
    return json.dumps(tile_keyframe_export(tile_batch_path))


@mcp.tool()
def variant_matrix_expand(
    matrix_path: str,
    minimum_only: bool = False,
    include_fire_row: bool = True,
    write_batch: bool = False,
) -> str:
    """Expand variant_matrix YAML → variant keys + sim_tags (PT-3)."""
    from .variant_matrix_expand import variant_matrix_expand as expand

    return json.dumps(
        expand(
            matrix_path,
            minimum_only=minimum_only,
            include_fire_row=include_fire_row,
            write_batch=write_batch,
        )
    )


@mcp.tool()
def validate_effect_spec_report(path: str, compress: int = 3) -> str:
    """VSS-T4-003 — ValidationReport for effect_spec_v1 JSON (Draft 2020-12 spawn_hook allOf)."""
    from rust_engine_mcp.validators.effect_spec import validate_effect_spec
    from rust_engine_mcp.paths import repo_root

    p = Path(path)
    if not p.is_absolute():
        p = repo_root() / p
    report = validate_effect_spec(p, compression_level=max(1, min(4, compress)))
    return json.dumps(report.to_dict())


@mcp.tool()
def effect_promote(
    spec_path: str = "",
    batch_id: str = "",
    phase: str = "full",
    force: bool = False,
    write_witness: bool = False,
) -> str:
    """VSS-T4-003 — pack wgsl_pack shaders to staging and/or promote to assets/effects/registry/.

    force=True remains the operator override for a pending production honest_gate
    (PCI-28). It does not override dishonest_gate and does not mark the witness honest.
    """
    from rust_engine_mcp import effect_promote as ep

    result = ep.effect_promote(
        spec_path=spec_path,
        batch_id=batch_id,
        phase=phase,
        force=force,
        write_witness=write_witness,
    )
    return json.dumps(result)


@mcp.tool()
def effect_preview_capture(
    spec_path: str = "",
    batch_id: str = "",
    pack_first: bool = True,
    force: bool = False,
    write_witness: bool = False,
) -> str:
    """VSS-T4 residual — deterministic staging_pack_digest frames → honest_gate=honest (Assign unlock)."""
    from rust_engine_mcp import effect_preview_capture as epc

    result = epc.effect_preview_capture(
        spec_path=spec_path,
        batch_id=batch_id,
        pack_first=pack_first,
        force=force,
        write_witness=write_witness,
    )
    return json.dumps(result)


@mcp.tool()
def arch_dna_snapshot_brief(path: str) -> str:
    """Compressed ARCH-DNA + β from assembly snapshot JSON."""
    from rust_engine_mcp import arch_build_grammar

    return json.dumps(arch_build_grammar.arch_dna_snapshot_brief(path))


@mcp.tool()
def material_profile_brief(profile_id: str) -> str:
    """MCP-MAT-BRIEF-001 — texture status + category path for one material profile."""
    from . import material_brief

    return json.dumps(material_brief.material_profile_brief(profile_id))


@mcp.tool()
def material_catalog_brief(max_rows: int = 12) -> str:
    """MAT node roll-up — profile counts by texture_status (no full catalog Read)."""
    from . import material_brief

    return json.dumps(material_brief.material_catalog_brief(max_rows=max_rows))




@mcp.tool()
def atlas_meta_brief_tool(atlas_folder: str, batch_id: str = "") -> str:
    """MCP-ATLAS-BRIEF-001 — ≤40-line atlas folder artist summary."""
    from . import atlas_meta_brief

    return json.dumps(
        atlas_meta_brief.atlas_meta_brief(atlas_folder, batch_id=batch_id or None)
    )


@mcp.tool()
def micro_tool_help() -> str:
    """List micro CLI commands from the live argparse catalog (token-efficient groups).

    Never hand-lists commands — same source as ``python -m rust_engine_mcp.cli commands``.
    """
    from .cli import build_parser
    from .cli_catalog import mcp_help_payload

    return json.dumps(mcp_help_payload(build_parser()))


def main() -> None:
    mcp.run()


if __name__ == "__main__":
    main()
