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

from rust_engine_mcp import blender_runner, library, paths, promote, schemas, validate_glb, witness
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
from rust_engine_mcp import agent_queue

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
def write_witness(batch_id: str) -> str:
    """Rebuild debug_runs/art_pipeline/<batch_id>_live.json from disk + index."""
    return json.dumps(witness.write_batch_witness(batch_id))


@mcp.tool()
def validate_report(
    validator: str,
    target: str = "",
    package: str = "",
    compress: int = 3,
    use_cached: bool = False,
) -> str:
    """Run a structured validator (cargo/bevy/mcp_spec/mcp_job/asset_glb). Returns ValidationReport JSON — not raw logs."""
    report = run_validator(
        validator,
        target or None,
        package=package or None,
        compression_level=max(1, min(4, compress)),
        use_cached=use_cached,
    )
    return json.dumps(report.to_dict())


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
def write_tile_fix_designer_g4_witness(
    building_definition: str = "tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json",
    allow_headless: bool = False,
) -> str:
    """TILE-FIX Phase C — validate-report chain + designer G4 witness (same as CLI)."""
    from rust_engine_mcp.tile_compile_loop import run_designer_warehouse_phase_c

    return json.dumps(
        run_designer_warehouse_phase_c(
            building_definition,
            require_manual_art=not allow_headless,
        ),
        indent=2,
    )


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
) -> str:
    """Generate assembly_snapshot_v1 JSON from StylePack + footprint (no Blender)."""
    w, d = footprint.lower().split("x")
    snap = assembly.generate_assembly_snapshot(
        style_pack_id=style_pack,
        width=int(w),
        depth=int(d),
        floors=floors,
        seed=seed,
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
def agent_queue_next(
    agent: str,
    queue: str = "grammar",
    mark_in_progress: bool = False,
) -> str:
    """Next drainable slice for @planner/@coder/@designer — work or drain fallback, never wait-only idle."""
    return json.dumps(
        agent_queue.agent_queue_next(
            agent, queue=queue, mark_in_progress=mark_in_progress
        )
    )


@mcp.tool()
def agent_queue_update(
    slice_id: str,
    status: str,
    note: str = "",
    queue: str = "grammar",
) -> str:
    """Checkpoint a queue slice (ready|blocked|in_progress|done|deferred)."""
    return json.dumps(agent_queue.agent_queue_update(slice_id, status, note=note, queue=queue))


@mcp.tool()
def agent_queue_board(agent: str = "", queue: str = "grammar") -> str:
    """Compressed queue board (tab-separated lines) — optional agent filter."""
    return json.dumps(agent_queue.agent_queue_board(queue=queue, agent=agent))


@mcp.tool()
def witness_brief(path: str) -> str:
    """Witness JSON summary (green, blockers, errors cap) — not full file."""
    return json.dumps(agent_queue.witness_brief(path))


@mcp.tool()
def handoff_brief() -> str:
    """HANDOFF.md Goal/Blockers/Next only — not full handoff file."""
    return json.dumps(agent_queue.handoff_brief())


@mcp.tool()
def file_digest(path: str, max_lines: int = 40) -> str:
    """File head + line count — avoid reading huge sources into context."""
    return json.dumps(agent_queue.file_digest(path, max_lines=max_lines))


@mcp.tool()
def orchestrator_brief(use_cached: bool = True) -> str:
    """Last orchestrator last_run.json summary — pair with validate_cargo_report(cached)."""
    return json.dumps(agent_queue.orchestrator_brief(use_cached=use_cached))


@mcp.tool()
def token_savings_guide() -> str:
    """Which MCP tools to use instead of raw logs / full-file reads (token policy)."""
    return json.dumps(agent_queue.token_savings_guide())


@mcp.tool()
def micro_tool_help() -> str:
    """List micro CLI commands designers can run in terminal (same logic as MCP)."""
    return json.dumps(
        {
            "cli": "python -m rust_engine_mcp.cli",
            "commands": [
                "ping",
                "locate-blender",
                "validate-spec <path>",
                "write-spec <path>",
                "run-geometry <job.json>",
                "job-status <job_id>",
                "validate-glb <path.glb>",
                "list-staging",
                "promote <job_id>",
                "library-register <job_id>",
                "library-register --rebuild-all",
                "library-search [--style-pack X] [--batch-id Y]",
                "write-witness <batch_id>",
                "validate-report <cargo|bevy|mcp_spec|mcp_job|asset_glb|tile_batch> [path]",
                "tile-atlas-pack <png_folder> [-pk]",
                "lod0-batch-run --batch kit_lod0_003 --phase geometry",
                "tile-batch-run <tile_batch_v1.json>",
                "assembly-snapshot-generate --style-pack style_victorian --footprint 4x3",
                "assembly-build-run <assembly_snapshot.json>",
                "tile-batch-status <batch_id>",
                "variant-set-validate <variant_set.json>",
                "variant-set-patch <variant_set.json> <patch.json>",
                "variant-bake <variant_set.json> <variant_key>",
                "variant-agent-request <request.json>",
                "tile-atlas-register <batch_id>",
                "tile-keyframe-export <tile_batch_v1.json>",
                "variant-matrix-expand <variant_matrix.yaml>",
                "write-procedural-tiles-production-bake-witness",
                "agent-queue-next <agent> [--queue grammar]",
                "agent-queue-update <slice_id> <status>",
                "agent-queue-board [--agent planner]",
                "witness-brief <debug_runs/...json>",
                "handoff-brief",
                "file-digest <path>",
                "orchestrator-brief",
            ],
            "mcp_agent_tools": [
                "agent_queue_next",
                "agent_queue_update",
                "agent_queue_board",
                "witness_brief",
                "handoff_brief",
                "file_digest",
                "orchestrator_brief",
                "token_savings_guide",
            ],
            "mcp_tile_tools": [
                "tile_atlas_pack_tool",
                "lod0_batch_run_tool",
                "tile_batch_validate",
                "tile_batch_run",
                "assembly_snapshot_generate",
                "assembly_build_run",
                "tile_batch_status",
                "variant_set_validate",
                "variant_set_patch",
                "variant_bake",
                "variant_agent_request",
                "tile_atlas_register",
                "tile_keyframe_export_tool",
                "variant_matrix_expand",
            ],
            "validators": [
                "validate_report",
                "write-tile-fix-10-witness --building <building_definition>",
                "write-tile-fix-designer-g4-witness --building <building_definition>",
                "validate_cargo_report",
                "validate_bevy_report",
                "validate_asset_report",
            ],
            "philosophy": "Validators return ValidationReport JSON; agents must not read raw cargo/build logs when a validator exists.",
        }
    )


def main() -> None:
    mcp.run()


if __name__ == "__main__":
    main()
