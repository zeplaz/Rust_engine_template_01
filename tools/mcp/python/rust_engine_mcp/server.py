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
from rust_engine_mcp import mcp_productivity_p0
from rust_engine_mcp import ops_intelligence
from rust_engine_mcp.pilot_hardcode_lint import (
    pilot_hardcode_lint,
    validate_pilot_hardcode_lint,
    write_pilot_hardcode_lint_witness,
)
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
def validate_pilot_hardcode_lint_report(compress: int = 3) -> str:
    """MCP-GUARD-001 — scan for warehouse-shaped hardcode outside allowlists."""
    report = validate_pilot_hardcode_lint()
    report.compression_level = max(1, min(4, compress))
    return json.dumps(report.to_dict())


@mcp.tool()
def write_pilot_hardcode_lint_witness_tool() -> str:
    """Write debug_runs/pilot_hardcode_lint_live.json."""
    return json.dumps(write_pilot_hardcode_lint_witness())


@mcp.tool()
def grammar_set_brief(set_id: str = "") -> str:
    """MCP-GRAMMAR-SET-001 — pilot/grammar/preset inventory + F-axis gaps (≤50 lines)."""
    body = grammar_build_set.grammar_set_brief(set_id=set_id or None)
    return json.dumps(body)


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
def grammar_pilot_parity_tool() -> str:
    """MCP-GRAMMAR-SET-004 — catalog parity (≥4 grammar pilots, ≥8 total)."""
    return json.dumps(grammar_build_set.grammar_pilot_parity())


@mcp.tool()
def building_set_coverage_report_tool(set_id: str = "") -> str:
    """MCP-BUILD-SET-002 — F-axis coverage + hardcode lint rollup."""
    body = grammar_build_set.building_set_coverage_report(set_id=set_id or None)
    return json.dumps(body)


@mcp.tool()
def building_set_health_brief_tool() -> str:
    """MCP-BUILD-SET-003 — OPS/APS rollup: brief + coverage + parity + hardcode."""
    return json.dumps(grammar_build_set.building_set_health_brief())


@mcp.tool()
def write_grammar_set_brief_witness_tool() -> str:
    """Write debug_runs/grammar_set_brief_live.json."""
    return json.dumps(grammar_build_set.write_grammar_set_brief_witness())


@mcp.tool()
def grammar_set_tier_tool(write_witness: bool = False) -> str:
    """APS-GRAM-TIER-001 — G0–G4 maturity from registry + coverage guards."""
    if write_witness:
        return json.dumps(grammar_build_set.write_grammar_set_tier_witness())
    return json.dumps(grammar_build_set.grammar_set_tier())


@mcp.tool()
def designer_grammar_quality_loop_tool(mode: str = "fast", write_witness: bool = False) -> str:
    """Designer iteration loop — tier + brief + coverage + optional sweeps (compressed JSON)."""
    from rust_engine_mcp.designer_grammar_quality_loop import run_designer_grammar_quality_loop

    m = "full" if str(mode).strip().lower() == "full" else "fast"
    return json.dumps(
        run_designer_grammar_quality_loop(mode=m, write_witness=write_witness)
    )


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
    queue: str = "auto",
    mark_in_progress: bool = False,
) -> str:
    """Next drainable slice for @planner/@coder/@designer — work or drain fallback, never wait-only idle."""
    return json.dumps(
        agent_queue.agent_queue_next(
            agent, queue=queue, mark_in_progress=mark_in_progress
        )
    )


@mcp.tool()
def get_que(
    agent: str,
    track: str = "",
    demand: bool = False,
    minutes: int = 60,
    mark_in_progress: bool = False,
) -> str:
    """BLANG:Q+ — multi-parallel next slice + drain board. Say 'get que' at session start. Use demand=true for ~1h todo list."""
    return json.dumps(
        agent_queue.agent_get_que(
            agent,
            track=track,
            build_list=demand,
            minutes=minutes,
            mark_in_progress=mark_in_progress,
        )
    )


@mcp.tool()
def agent_queue_demand(
    agent: str,
    minutes: int = 60,
    max_slices: int = 8,
    track: str = "",
) -> str:
    """Ordered session todo list from ready rows across open tracks (~hour-scale)."""
    return json.dumps(
        agent_queue.agent_queue_demand(
            agent, minutes=minutes, max_slices=max_slices, track=track
        )
    )


@mcp.tool()
def agent_queue_update(
    slice_id: str,
    status: str,
    note: str = "",
    queue: str = "auto",
    enforce: bool = False,
) -> str:
    """Checkpoint a queue slice (ready|blocked|in_progress|done|deferred)."""
    return json.dumps(
        agent_queue.agent_queue_update(slice_id, status, note=note, queue=queue, enforce=enforce)
    )


@mcp.tool()
def validate_queue_integrity_report(queue_filter: str = "", compress: int = 3) -> str:
    """BLANG:WIT-HON — cross-queue contradiction + exit_predicate report."""
    from rust_engine_mcp.validators.queue_integrity import validate_queue_integrity

    report = validate_queue_integrity(
        queue_filter=queue_filter or None,
        compression_level=max(1, min(4, compress)),
    )
    return json.dumps(report.to_dict())


@mcp.tool()
def intel_officer_sweep(queue_filter: str = "", include_witness_scan: bool = True, compress: int = 3) -> str:
    """Intel officer surveillance — false-positive done/green cull candidates (report-only)."""
    from rust_engine_mcp import intel_officer

    return json.dumps(
        intel_officer.intel_officer_sweep(
            queue_filter=queue_filter,
            include_witness_scan=include_witness_scan,
            compression_level=max(1, min(4, compress)),
        )
    )


@mcp.tool()
def intel_officer_apply(
    ids: str,
    apply: bool = False,
    action: str = "reopen",
    note: str = "",
) -> str:
    """Supervised cull — reopen queue rows / demote dishonest witnesses (dry_run unless apply=true)."""
    from rust_engine_mcp import intel_officer

    id_list = [x.strip() for x in ids.split(",") if x.strip()]
    return json.dumps(
        intel_officer.intel_officer_apply(
            ids=id_list,
            dry_run=not apply,
            action=action,
            note=note,
        )
    )


@mcp.tool()
def agent_queue_board(agent: str = "", queue: str = "grammar") -> str:
    """Compressed queue board (tab-separated lines) — optional agent filter."""
    return json.dumps(agent_queue.agent_queue_board(queue=queue, agent=agent))


@mcp.tool()
def witness_brief(path: str, profile: str = "") -> str:
    """Witness JSON summary — profile=construction|map_pick|fire_product|honesty (failed rule ids only)."""
    prof = profile.strip() or None
    return json.dumps(agent_queue.witness_brief(path, profile=prof))


@mcp.tool()
def review_order_brief() -> str:
    """BLANG:REVIEW — REVIEW-ORDER P0 rows + phase4 status + VR compressed."""
    return json.dumps(ops_intelligence.review_order_brief())


@mcp.tool()
def slice_exec_brief(slice_id: str, queue: str = "") -> str:
    """BLANG:SLICE — one queue row: exit, witness, exec docs."""
    q = queue.strip() or None
    return json.dumps(agent_queue.slice_exec_brief(slice_id, queue=q))


@mcp.tool()
def validate_construction_report(path: str, compress: int = 3) -> str:
    """BLANG:PLACE — ValidationReport for construction/placement witness JSON."""
    from rust_engine_mcp.validators.construction_witness import validate_construction_witness_path

    report = validate_construction_witness_path(path, compression_level=max(1, min(4, compress)))
    return json.dumps(report.to_dict())


@mcp.tool()
def validate_witness_honesty_report(path: str, compress: int = 3, scan: bool = False) -> str:
    """BLANG:WIT-HON — ValidationReport for witness honesty (single path or scan dir when scan=True)."""
    from rust_engine_mcp.validators.witness_honesty import validate_witness_honesty_path, validate_witness_honesty_scan

    level = max(1, min(4, compress))
    if scan:
        report = validate_witness_honesty_scan(path or "debug_runs", compression_level=level)
    else:
        report = validate_witness_honesty_path(path, compression_level=level)
    return json.dumps(report.to_dict())


@mcp.tool()
def handoff_brief() -> str:
    """HANDOFF.md Goal/Blockers/Next only — not full handoff file."""
    return json.dumps(agent_queue.handoff_brief())


@mcp.tool()
def arch_dna_consumer_contract() -> str:
    """BUILD-READ-CONSUMER-MCP-001 — @coder snapshot field contract for DNA+β."""
    from rust_engine_mcp import arch_build_grammar

    return json.dumps(arch_build_grammar.consumer_contract())


@mcp.tool()
def arch_dna_snapshot_brief(path: str) -> str:
    """Compressed ARCH-DNA + β from assembly snapshot JSON."""
    from rust_engine_mcp import arch_build_grammar

    return json.dumps(arch_build_grammar.arch_dna_snapshot_brief(path))


@mcp.tool()
def ops_get_project_brief() -> str:
    """BLANG:OPS — ~20-token project orientation (ops_project_brief_v1); not HANDOFF + 80 witnesses."""
    return json.dumps(ops_intelligence.ops_get_project_brief())


@mcp.tool()
def ops_get_retry_guidance(task_id: str) -> str:
    """BLANG:OPS retry — phase3/phase4 row + exec_doc + hotfix_steps."""
    return json.dumps(ops_intelligence.ops_get_retry_guidance(task_id))


@mcp.tool()
def ops_get_active_blockers() -> str:
    """BLANG:OPS blockers — open gates from master_chain_tensor_v1.json."""
    return json.dumps(ops_intelligence.ops_get_active_blockers())


@mcp.tool()
def landscape_grammar_presets_witness() -> str:
    """MCP-LG-VALID-PRESET-001 — batch validate ship presets + refresh witnesses."""
    from rust_engine_mcp import landscape_grammar_presets

    batch = landscape_grammar_presets.write_landscape_grammar_presets_witness()
    sign = landscape_grammar_presets.refresh_mcp_landscape_grammar_sign_witness()
    return json.dumps(
        {
            "green": bool(batch.get("green")) and bool(sign.get("green")),
            "batch_witness": batch.get("written"),
            "sign_witness": sign.get("written"),
        }
    )


@mcp.tool()
def coder_drain_brief(coder: str = "c") -> str:
    """MCP-CODER-DRAIN-001 — @coder A/B/C open vs stale slices before dispatch paste."""
    return json.dumps(agent_queue.coder_drain_brief(coder))


@mcp.tool()
def simulation_queue_brief() -> str:
    """MCP-SIM-QUEUE-001 — weather simulation train open/done rows."""
    return json.dumps(agent_queue.simulation_queue_brief())


@mcp.tool()
def coder_mcp_drain_brief() -> str:
    """MCP-CODER-MCP-DRAIN-001 — all open @coder-mcp slices + recommended drain order."""
    return json.dumps(agent_queue.coder_mcp_drain_brief())


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
def pipeline_preflight(queue: str = "grammar") -> str:
    """MCP-PREFLIGHT-001 — Blender, schemas, repo paths, queue stale rows (one call)."""
    return json.dumps(mcp_productivity_p0.pipeline_preflight(queue=queue))


@mcp.tool()
def snapshot_digest(path: str) -> str:
    """MCP-SNAPSHOT-DIGEST-001 — placements, materials, grammar one-liner (no full JSON)."""
    return json.dumps(mcp_productivity_p0.snapshot_digest(path))


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
def validate_p0_gate_plain(path: str, ship: bool = True, compression_level: int = 4) -> str:
    """MCP-P0-PLAIN-001 — P0 gate with artist sentences + fix hints."""
    return json.dumps(
        mcp_productivity_p0.validate_p0_gate_plain(
            path, ship=ship, compression_level=compression_level
        )
    )


@mcp.tool()
def agent_doc_touch(
    path: str,
    agent: str = "coder-mcp",
    intent: str = "ref",
    max_lines: int = 40,
    session_hint: str = "",
) -> str:
    """MCP-DOC-READ-001 — ledger doc read + file digest (BLANG ref/orient/implement)."""
    from . import agent_doc_read

    return json.dumps(
        agent_doc_read.agent_doc_touch(
            path,
            agent=agent,
            intent=intent,
            max_lines=max_lines,
            session_hint=session_hint,
        )
    )


@mcp.tool()
def agent_doc_reads_brief(min_reads: int = 2, tail_rows: int = 500) -> str:
    """MCP-DOC-READ-003 — aggregate doc_reads.jsonl: hot paths + repeat-in-session."""
    from . import agent_doc_read

    return json.dumps(
        agent_doc_read.agent_doc_reads_brief(min_reads=min_reads, tail_rows=tail_rows)
    )


@mcp.tool()
def agent_doc_promote_hot_reads(min_reads: int = 3, max_promote: int = 8) -> str:
    """MCP-DOC-READ-004 — promote hot ledger paths to tools/mcp/cache/agent_doc_digests/."""
    from . import agent_doc_read

    return json.dumps(
        agent_doc_read.agent_doc_promote_hot_reads(min_reads=min_reads, max_promote=max_promote)
    )


@mcp.tool()
def agent_doc_digest_cached(path: str, max_lines: int = 120) -> str:
    """Return MCP digest cache for path when source mtime unchanged."""
    from . import agent_doc_read

    return json.dumps(agent_doc_read.agent_doc_digest_cached(path, max_lines=max_lines))


@mcp.tool()
def agent_session_bootstrap(
    agent: str,
    session_hint: str = "SESSION-START",
    max_lines: int = 60,
    touch_role_reads: bool = False,
) -> str:
    """MCP-DOC-READ-005 — session start: brief stack + ledger + hot-read stats."""
    from . import agent_doc_read

    return json.dumps(
        agent_doc_read.agent_session_bootstrap(
            agent,
            session_hint=session_hint,
            max_lines=max_lines,
            touch_role_reads=touch_role_reads,
        )
    )


@mcp.tool()
def agent_run_append(event_json: str, agent: str = "") -> str:
    """MCP-DOC-READ-002 — append session telemetry to debug_runs/agent_ops/run_events.jsonl."""
    from . import agent_doc_read

    event = json.loads(event_json)
    return json.dumps(agent_doc_read.agent_run_append(event, agent=agent or None))


@mcp.tool()
def snapshot_diff_brief(before_path: str, after_path: str) -> str:
    """MCP-SNAPSHOT-DIFF-001 — compact diff between two assembly snapshots."""
    from . import agent_doc_read

    return json.dumps(agent_doc_read.snapshot_diff_brief(before_path, after_path))


@mcp.tool()
def grammar_iterate(
    request_path: str,
    write_snapshot: bool = False,
    write_witness: str = "",
) -> str:
    """MCP-GRAMMAR-ITER-TOOL — grammar iterate (CLI parity with grammar-iterate)."""
    from . import agent_doc_read

    return json.dumps(
        agent_doc_read.grammar_iterate_mcp(
            request_path,
            write_snapshot=write_snapshot,
            write_witness=write_witness or None,
        )
    )


@mcp.tool()
def tile_spine_run_tool(request_path: str) -> str:
    """MCP-SPINE-CHAIN-001 — WRK→ATL chain with per-step witness (CLI parity)."""
    from . import tile_spine_run

    return json.dumps(tile_spine_run.tile_spine_run(request_path))


@mcp.tool()
def atlas_meta_brief_tool(atlas_folder: str, batch_id: str = "") -> str:
    """MCP-ATLAS-BRIEF-001 — ≤40-line atlas folder artist summary."""
    from . import atlas_meta_brief

    return json.dumps(
        atlas_meta_brief.atlas_meta_brief(atlas_folder, batch_id=batch_id or None)
    )


@mcp.tool()
def rt_registry_tool(batch_id: str = "") -> str:
    """RT-REG-001 — register rowhouse production batch + lookup stamp."""
    from . import rt_registry

    return json.dumps(
        rt_registry.rt_registry_register_rowhouse_production(batch_id=batch_id or None)
    )


@mcp.tool()
def runtime_lookup_brief_tool(atlas_id: str = "rowhouse_victorian_production_v1") -> str:
    """RT-BRIEF-001 — runtime lookup brief from index row + atlas meta."""
    from . import runtime_lookup_brief

    return json.dumps(runtime_lookup_brief.runtime_lookup_brief(atlas_id))


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
                "coder-drain-brief [--coder a|b|c]",
                "simulation-queue-brief",
                "file-digest <path>",
                "orchestrator-brief",
                "token-savings-guide",
                "pipeline-preflight [--queue grammar]",
                "snapshot-digest <assembly.json>",
                "material-profile-brief <profile_id>",
                "material-catalog-brief [--max-rows 12]",
                "validate-p0-gate-plain <assembly.json>",
                "agent-doc-touch <path> [--intent ref|orient|implement]",
                "agent-doc-reads-brief [--min-reads 2]",
                "agent-doc-promote-hot-reads [--min-reads 3]",
                "agent-doc-digest-cached <path>",
                "agent-session-bootstrap <agent> [--session-hint SESSION-START]",
                "agent-run-append '<event json>'",
                "snapshot-diff-brief <before.json> <after.json>",
                "grammar-iterate <request.json> [--write-snapshot]",
                "tile-spine-run <request.json>",
                "tile-spine-run-witness",
                "atlas-meta-brief <folder> [--batch-id X]",
                "atlas-meta-brief-witness",
            ],
            "mcp_agent_tools": [
                "agent_queue_next",
                "get_que",
                "agent_queue_demand",
                "agent_queue_update",
                "agent_queue_board",
                "witness_brief",
                "review_order_brief",
                "slice_exec_brief",
                "validate_construction_report",
                "handoff_brief",
                "coder_drain_brief",
                "simulation_queue_brief",
                "file_digest",
                "orchestrator_brief",
                "token_savings_guide",
                "pipeline_preflight",
                "snapshot_digest",
                "material_profile_brief",
                "material_catalog_brief",
                "validate_p0_gate_plain",
                "agent_doc_touch",
                "agent_doc_reads_brief",
                "agent_doc_promote_hot_reads",
                "agent_doc_digest_cached",
                "agent_session_bootstrap",
                "agent_run_append",
                "snapshot_diff_brief",
                "grammar_iterate",
                "tile_spine_run_tool",
                "atlas_meta_brief_tool",
                "rt_registry_tool",
                "runtime_lookup_brief_tool",
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
