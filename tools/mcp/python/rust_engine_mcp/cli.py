"""
Micro CLI — deterministic steps; MCP and designers call these instead of LLM improvisation.

  python -m rust_engine_mcp.cli ping
  python -m rust_engine_mcp.cli locate-blender
  python -m rust_engine_mcp.cli validate-spec path.json
  python -m rust_engine_mcp.cli write-spec path.json
  python -m rust_engine_mcp.cli run-geometry path/job.json
  python -m rust_engine_mcp.cli job-status JOB_ID
  python -m rust_engine_mcp.cli validate-glb path.glb
  python -m rust_engine_mcp.cli list-staging
  python -m rust_engine_mcp.cli promote JOB_ID
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from . import blender_runner, library, material_textures, paths, promote, schemas, validate_glb, witness
from .tile_pipeline import (
    assembly_build_run,
    lod0_batch_run,
    tile_atlas_pack,
    tile_batch_run,
    tile_batch_status,
    tile_keyframe_export,
)
from . import assembly
from . import assembly_build_worker
from . import assembly_preview
from . import variant_set
from .tile_index import register_tile_atlas_from_batch
from .validators import run_validator
from . import agent_queue


def _cmd_ping(_: argparse.Namespace) -> int:
    print(json.dumps({"ok": True, "repo": str(paths.repo_root())}))
    return 0


def _cmd_locate_blender(_: argparse.Namespace) -> int:
    print(json.dumps({"blender_exe": str(paths.blender_exe())}))
    return 0


def _cmd_validate_spec(args: argparse.Namespace) -> int:
    data = schemas.load_json_file(Path(args.path))
    schemas.validate_asset_spec(data)
    print(json.dumps({"valid": True, "asset_id": data.get("asset_id")}))
    return 0


def _cmd_write_spec(args: argparse.Namespace) -> int:
    data = schemas.load_json_file(Path(args.path))
    schemas.validate_asset_spec(data)
    asset_id = data["asset_id"]
    out = paths.staging_root() / "specs" / f"{asset_id}.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(data, indent=2), encoding="utf-8")
    print(json.dumps({"written": str(out)}))
    return 0


def _cmd_run_geometry(args: argparse.Namespace) -> int:
    result = blender_runner.run_geometry_job(Path(args.path))
    print(json.dumps(result.__dict__))
    return 0 if result.status == "done" else 1


def _cmd_job_status(args: argparse.Namespace) -> int:
    st = blender_runner.read_status(args.job_id)
    if st is None:
        print(json.dumps({"error": "not found", "job_id": args.job_id}))
        return 1
    print(json.dumps(st))
    return 0


def _cmd_validate_glb_cmd(args: argparse.Namespace) -> int:
    report = validate_glb.validate_glb(Path(args.path))
    print(json.dumps(report.to_dict()))
    return 0 if report.valid else 1


def _cmd_list_staging(_: argparse.Namespace) -> int:
    root = paths.staging_root()
    entries = []
    if root.is_dir():
        for p in sorted(root.iterdir()):
            if p.is_dir() and p.name != "specs":
                glbs = list(p.glob("**/*.glb"))
                entries.append({"job_id": p.name, "glb_count": len(glbs)})
    print(json.dumps({"staging": entries}))
    return 0


def _cmd_promote(args: argparse.Namespace) -> int:
    manifest = promote.promote_module(
        args.job_id,
        force=args.force,
        register=not args.no_register,
        allow_smoke=args.allow_smoke,
    )
    print(json.dumps(manifest))
    return 0


def _cmd_library_register(args: argparse.Namespace) -> int:
    if getattr(args, "rebuild_all", False):
        result = library.write_module_index()
    else:
        result = library.register_module(args.job_id)
    print(json.dumps(result))
    return 0


def _cmd_library_search(args: argparse.Namespace) -> int:
    tags = [t.strip() for t in (args.tags or "").split(",") if t.strip()] or None
    rows = library.search_modules(
        tags=tags,
        archetype=args.archetype or None,
        style_pack=args.style_pack or None,
        category=args.category or None,
        batch_id=args.batch_id or None,
    )
    print(json.dumps({"count": len(rows), "modules": rows}))
    return 0


def _cmd_write_witness(args: argparse.Namespace) -> int:
    result = witness.write_batch_witness(args.batch_id)
    print(json.dumps(result))
    return 0


def _cmd_generate_material_textures(args: argparse.Namespace) -> int:
    if args.all_pilot or not args.profile:
        results = material_textures.generate_pilot_profiles()
    else:
        if args.profile not in material_textures.PILOT_PROFILES:
            raise KeyError(f"Unknown profile {args.profile}")
        results = [material_textures.generate_profile(material_textures.PILOT_PROFILES[args.profile])]
    print(json.dumps({"generated": results}))
    return 0


def _cmd_tile_atlas_pack(args: argparse.Namespace) -> int:
    result = tile_atlas_pack(args.folder, keyframe_rename=args.keyframe_rename)
    print(json.dumps(result, indent=2))
    return 0 if result.get("ok") else 1


def _cmd_lod0_batch_run(args: argparse.Namespace) -> int:
    result = lod0_batch_run(args.batch_id, phase=args.phase)
    print(json.dumps(result, indent=2))
    return 0 if result.get("ok") else 1


def _cmd_tile_batch_run(args: argparse.Namespace) -> int:
    result = tile_batch_run(args.path or "")
    print(json.dumps(result, indent=2))
    return 0 if result.get("ok") else 1


def _cmd_assembly_snapshot_generate(args: argparse.Namespace) -> int:
    snap = assembly.generate_assembly_snapshot(
        style_pack_id=args.style_pack,
        width=int(args.footprint.split("x")[0]),
        depth=int(args.footprint.split("x")[1]),
        floors=int(args.floors),
        seed=int(args.seed),
    )
    print(json.dumps(snap, indent=2))
    return 0


def _cmd_build_iso_rig(args: argparse.Namespace) -> int:
    result = blender_runner.build_iso_rig_blend(procedural_only=args.procedural_only)
    print(json.dumps(result, indent=2))
    return 0 if result.get("ok") else 1


def _cmd_assembly_build_run(args: argparse.Namespace) -> int:
    if getattr(args, "ensure_materials", False):
        result = assembly_build_worker.assembly_build_with_materials(
            args.path,
            ensure_textures=True,
            render_still=getattr(args, "render_still", False),
            building_definition_path=getattr(args, "building_definition", None),
            write_witness=getattr(args, "write_witness", False),
        )
    else:
        result = assembly_build_run(args.path)
    print(json.dumps(result, indent=2))
    ok = result.get("ok")
    if ok is None:
        ok = result.get("status") == "done"
    return 0 if ok else 1


def _cmd_pg_module_audit_002_run(args: argparse.Namespace) -> int:
    from rust_engine_mcp.pg_module_audit_002 import run_pg_module_audit_002

    priorities = tuple(p.strip() for p in str(args.priorities or "P0,P1").split(",") if p.strip())
    result = run_pg_module_audit_002(
        phase=args.phase,
        priorities=priorities,
        use_blender=not args.no_blender,
    )
    print(json.dumps(result, indent=2))
    return 0 if result.get("ok") else 1


def _cmd_keyframe_matrix_export(args: argparse.Namespace) -> int:
    from rust_engine_mcp.tile_compile_loop import run_minimum_cell_bakes, write_compile_plan_json

    raw = Path(args.path or "")
    defn_path = raw if raw.is_file() else paths.repo_root() / raw
    if getattr(args, "plan_only", False):
        out = write_compile_plan_json(defn_path, minimum_only=True)
        print(json.dumps({"ok": True, "plan": str(out)}, indent=2))
        return 0
    result = run_minimum_cell_bakes(
        defn_path,
        skip_existing=not getattr(args, "force", False),
        require_blender=not getattr(args, "dry_run", False),
    )
    print(json.dumps(result, indent=2))
    return 0 if result.get("ok") else 1


def _cmd_preview_assembly(args: argparse.Namespace) -> int:
    serve = float(args.serve_seconds or 0)
    if serve <= 0 and not args.no_browser:
        serve = 120.0
    elif serve <= 0 and args.no_browser:
        serve = 300.0
    result = assembly_preview.preview_assembly(
        args.path,
        out_png=args.out or None,
        open_browser=not args.no_browser,
        try_bevy=not args.skip_bevy,
        serve_seconds=serve,
    )
    if args.write_witness:
        assembly_preview.write_aps_preview_002_witness(result)
    print(json.dumps(result, indent=2))
    if result.get("preview_url"):
        print(
            f"\nPreview server stopped after {serve:.0f}s. "
            "Re-run with --serve-seconds 600 to keep alive longer.",
            flush=True,
        )
    return 0 if result.get("green") else 1


def _cmd_tile_batch_status(args: argparse.Namespace) -> int:
    result = tile_batch_status(args.batch_id)
    print(json.dumps(result, indent=2))
    return 0


def _cmd_variant_set_validate(args: argparse.Namespace) -> int:
    result = variant_set.variant_set_validate(args.path)
    print(json.dumps(result, indent=2))
    return 0


def _cmd_variant_set_patch(args: argparse.Namespace) -> int:
    patch = json.loads(Path(args.patch_json).read_text(encoding="utf-8"))
    if isinstance(patch, dict) and "patch" in patch:
        patch = patch["patch"]
    result = variant_set.variant_set_patch(args.path, patch, write=not args.dry_run)
    print(json.dumps({k: v for k, v in result.items() if k != "document"}, indent=2))
    return 0


def _cmd_variant_bake(args: argparse.Namespace) -> int:
    result = variant_set.variant_bake(args.path, args.variant_key, seed=args.seed or None)
    print(json.dumps(result, indent=2))
    return 0 if result.get("ok") else 1


def _cmd_variant_agent_request(args: argparse.Namespace) -> int:
    body = json.loads(Path(args.request_json).read_text(encoding="utf-8"))
    result = variant_set.variant_agent_request(body, write=not args.no_write)
    print(json.dumps(result, indent=2))
    return 0


def _cmd_tile_atlas_register(args: argparse.Namespace) -> int:
    result = register_tile_atlas_from_batch(
        args.batch_id,
        tile_batch_path=args.batch_json or None,
    )
    print(json.dumps(result, indent=2))
    return 0 if result.get("ok") else 1


def _cmd_rail_warehouse_pilot_batch_run(args: argparse.Namespace) -> int:
    from rust_engine_mcp import rail_warehouse_pilot_batch

    result = rail_warehouse_pilot_batch.run_rail_warehouse_pilot_keyframe_batch(
        headless=not bool(args.no_headless),
    )
    print(json.dumps(result, indent=2))
    return 0 if result.get("ok") else 1


def _cmd_tile_keyframe_export(args: argparse.Namespace) -> int:
    result = tile_keyframe_export(args.path or "")
    print(json.dumps(result, indent=2))
    return 0 if result.get("ok") else 1


def _cmd_variant_matrix_expand(args: argparse.Namespace) -> int:
    from .variant_matrix_expand import variant_matrix_expand

    result = variant_matrix_expand(
        args.path,
        minimum_only=bool(args.minimum_only),
        include_fire_row=not bool(args.no_fire_row),
        write_batch=bool(args.write_batch),
    )
    print(json.dumps(result, indent=2))
    return 0 if result.get("ok") else 1


def _cmd_write_procedural_tiles_production_bake_witness(_: argparse.Namespace) -> int:
    result = witness.write_procedural_tiles_production_bake_witness()
    print(json.dumps(result, indent=2))
    return 0 if result.get("green") else 1


def _cmd_write_tile_fix_10_witness(args: argparse.Namespace) -> int:
    from rust_engine_mcp.tile_compile_loop import write_tile_fix_10_witness

    result = write_tile_fix_10_witness(args.building)
    print(json.dumps(result, indent=2))
    return 0 if result.get("green") else 1


def _cmd_write_tile_fix_designer_g4_witness(args: argparse.Namespace) -> int:
    from rust_engine_mcp.tile_compile_loop import run_designer_warehouse_phase_c

    result = run_designer_warehouse_phase_c(
        args.building,
        require_manual_art=not bool(args.allow_headless),
    )
    print(json.dumps(result, indent=2))
    return 0 if result.get("proceed_ship") else 1


def _cmd_agent_queue_next(args: argparse.Namespace) -> int:
    out = agent_queue.agent_queue_next(
        args.agent,
        queue=args.queue,
        mark_in_progress=bool(args.mark_in_progress),
    )
    print(json.dumps(out, indent=2))
    return 0 if out.get("action") == "work" else 1


def _cmd_get_que(args: argparse.Namespace) -> int:
    out = agent_queue.agent_get_que(
        args.agent,
        track=args.track or "",
        build_list=bool(args.demand),
        minutes=int(args.minutes),
        mark_in_progress=bool(args.mark_in_progress),
    )
    print(json.dumps(out, indent=2))
    return 0 if out.get("action") == "work" else 1


def _cmd_agent_queue_demand(args: argparse.Namespace) -> int:
    out = agent_queue.agent_queue_demand(
        args.agent,
        minutes=int(args.minutes),
        max_slices=int(args.max_slices),
        track=args.track or "",
    )
    print(json.dumps(out, indent=2))
    return 0 if out.get("action") == "work" else 1


def _cmd_agent_queue_update(args: argparse.Namespace) -> int:
    out = agent_queue.agent_queue_update(
        args.slice_id,
        args.status,
        note=args.note or "",
        queue=args.queue,
        enforce=bool(getattr(args, "enforce", False)),
    )
    print(json.dumps(out, indent=2))
    return 0 if out.get("ok") else 1


def _cmd_intel_officer_sweep(args: argparse.Namespace) -> int:
    from rust_engine_mcp import intel_officer

    body = intel_officer.intel_officer_sweep(
        queue_filter=args.queue_filter or "",
        include_witness_scan=not bool(args.no_witness_scan),
        compression_level=int(args.compress),
    )
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_intel_officer_apply(args: argparse.Namespace) -> int:
    from rust_engine_mcp import intel_officer

    ids = [x.strip() for x in args.ids.split(",") if x.strip()]
    body = intel_officer.intel_officer_apply(
        ids=ids,
        dry_run=not bool(args.apply),
        action=args.action,
        note=args.note or "",
    )
    print(json.dumps(body, indent=2))
    return 0 if body.get("ok") else 1


def _cmd_intel_officer_sweep_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp import intel_officer

    body = intel_officer.refresh_intel_officer_sweep_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_queue_integrity_reconcile_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.validators.queue_integrity import refresh_queue_integrity_reconcile_witness

    body = refresh_queue_integrity_reconcile_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_mcp_witness_integrity_ops_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.witness_honesty_lib import refresh_mcp_witness_integrity_ops_witness

    body = refresh_mcp_witness_integrity_ops_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("written") else 1


def _cmd_agent_queue_board(args: argparse.Namespace) -> int:
    print(json.dumps(agent_queue.agent_queue_board(queue=args.queue, agent=args.agent or ""), indent=2))
    return 0


def _cmd_witness_brief(args: argparse.Namespace) -> int:
    profile = getattr(args, "profile", None) or None
    print(json.dumps(agent_queue.witness_brief(args.path, profile=profile), indent=2))
    return 0


def _cmd_review_order_brief(_: argparse.Namespace) -> int:
    from rust_engine_mcp import ops_intelligence

    print(json.dumps(ops_intelligence.review_order_brief(), indent=2))
    return 0


def _cmd_slice_exec_brief(args: argparse.Namespace) -> int:
    queue = args.queue or None
    body = agent_queue.slice_exec_brief(args.slice_id, queue=queue)
    print(json.dumps(body, indent=2))
    return 0 if body.get("ok") else 1


def _cmd_mcp_phase4_queue_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp import ops_intelligence

    body = ops_intelligence.write_mcp_phase4_queue_live_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_mcp_valid_construction_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp import ops_intelligence

    body = ops_intelligence.write_mcp_valid_construction_live_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_handoff_brief(_args: argparse.Namespace) -> int:
    print(json.dumps(agent_queue.handoff_brief(), indent=2))
    return 0


def _cmd_file_digest(args: argparse.Namespace) -> int:
    print(json.dumps(agent_queue.file_digest(args.path, max_lines=int(args.max_lines)), indent=2))
    return 0


def _cmd_orchestrator_brief(args: argparse.Namespace) -> int:
    print(json.dumps(agent_queue.orchestrator_brief(use_cached=not args.fresh), indent=2))
    return 0


def _cmd_token_savings_guide(_args: argparse.Namespace) -> int:
    print(json.dumps(agent_queue.token_savings_guide(), indent=2))
    return 0


def _cmd_pipeline_preflight(args: argparse.Namespace) -> int:
    from rust_engine_mcp import mcp_productivity_p0

    body = mcp_productivity_p0.pipeline_preflight(
        queue=args.queue,
        check_build_set=bool(getattr(args, "check_build_set", False)),
    )
    print(json.dumps(body, indent=2))
    return 0 if body.get("ok") else 1


def _cmd_ops_get_project_brief(_: argparse.Namespace) -> int:
    from rust_engine_mcp import ops_intelligence

    print(json.dumps(ops_intelligence.ops_get_project_brief(), indent=2))
    return 0


def _cmd_ops_get_retry_guidance(args: argparse.Namespace) -> int:
    from rust_engine_mcp import ops_intelligence

    body = ops_intelligence.ops_get_retry_guidance(args.task_id)
    print(json.dumps(body, indent=2))
    return 0 if body.get("ok") else 1


def _cmd_ops_get_active_blockers(_: argparse.Namespace) -> int:
    from rust_engine_mcp import ops_intelligence

    body = ops_intelligence.ops_get_active_blockers()
    print(json.dumps(body, indent=2))
    return 0 if body.get("ok") else 1


def _cmd_ops_dashboard_refresh(args: argparse.Namespace) -> int:
    from rust_engine_mcp.ops_telemetry import write_ops_dashboard_witness

    hours = int(getattr(args, "window_hours", 168) or 168)
    body = write_ops_dashboard_witness(window_hours=hours)
    print(json.dumps(body, indent=2))
    return 0 if body.get("ok") else 1


def _cmd_ops_process_scan(_: argparse.Namespace) -> int:
    from rust_engine_mcp.ops_telemetry import scan_processes

    print(json.dumps(scan_processes(), indent=2))
    return 0


def _cmd_ops_drift_scan(_: argparse.Namespace) -> int:
    from rust_engine_mcp.ops_telemetry import scan_drift_instances

    print(json.dumps(scan_drift_instances(), indent=2))
    return 0


def _cmd_ops_run_events_rollup(args: argparse.Namespace) -> int:
    from rust_engine_mcp.ops_telemetry import scan_run_events

    hours = int(getattr(args, "window_hours", 168) or 168)
    print(json.dumps(scan_run_events(window_hours=hours), indent=2))
    return 0


def _cmd_ops_crash_scan(_: argparse.Namespace) -> int:
    from rust_engine_mcp.ops_crash_exporter import run_crash_scan, write_prometheus_metrics

    body = run_crash_scan(record_events=True)
    body["prometheus_written"] = write_prometheus_metrics(body)
    print(json.dumps(body, indent=2))
    return 0 if body.get("ok") else 1


def _cmd_ops_triage_refresh(args: argparse.Namespace) -> int:
    from rust_engine_mcp.ops_crash_exporter import write_triage_witness

    hours = int(getattr(args, "window_hours", 168) or 168)
    body = write_triage_witness(window_hours=hours)
    print(json.dumps(body, indent=2))
    return 0 if body.get("ok") else 1


def _cmd_ops_crash_daemon(args: argparse.Namespace) -> int:
    from rust_engine_mcp.ops_crash_exporter import daemon_loop

    interval = int(getattr(args, "interval_sec", 30) or 30)
    max_cycles = getattr(args, "max_cycles", None)
    if max_cycles is not None:
        max_cycles = int(max_cycles)
    daemon_loop(interval_sec=interval, max_cycles=max_cycles)
    return 0


def _cmd_ops_mcp_function_layer_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp import ops_intelligence

    body = ops_intelligence.refresh_ops_mcp_function_layer_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_build_read_grammar_v0_002_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp import arch_build_grammar

    body = arch_build_grammar.write_build_read_grammar_v0_002_witness()
    arch_build_grammar.write_aps_dna_consumer_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_aps_dna_consumer_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp import arch_build_grammar

    body = arch_build_grammar.write_aps_dna_consumer_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_arch_dna_consumer_contract(_: argparse.Namespace) -> int:
    from rust_engine_mcp import arch_build_grammar

    print(json.dumps(arch_build_grammar.consumer_contract(), indent=2))
    return 0


def _cmd_arch_dna_snapshot_brief(args: argparse.Namespace) -> int:
    from rust_engine_mcp import arch_build_grammar

    body = arch_build_grammar.arch_dna_snapshot_brief(args.path)
    print(json.dumps(body, indent=2))
    return 0 if body.get("ok") else 1


def _cmd_grammar_set_brief(args: argparse.Namespace) -> int:
    from rust_engine_mcp import grammar_build_set

    if getattr(args, "write_witness", False):
        body = grammar_build_set.write_grammar_set_brief_witness()
    else:
        body = grammar_build_set.grammar_set_brief(set_id=args.set_id or None)
    print(json.dumps(body, indent=2))
    return 0 if body.get("ok") else 1


def _cmd_grammar_preset_pair_validate(args: argparse.Namespace) -> int:
    from rust_engine_mcp import grammar_build_set

    body = grammar_build_set.grammar_preset_pair_validate(
        preset_id=args.preset_id or None,
        path=args.path or None,
    )
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_grammar_eval_sweep(args: argparse.Namespace) -> int:
    from rust_engine_mcp import grammar_build_set

    seeds = [int(x) for x in args.seeds.split(",")] if args.seeds else None
    body = grammar_build_set.grammar_eval_sweep(
        archetype_id=args.archetype,
        district_style=args.district,
        seeds=seeds,
    )
    print(json.dumps(body, indent=2))
    return 0 if body.get("ok") else 1


def _cmd_grammar_pilot_parity(_: argparse.Namespace) -> int:
    from rust_engine_mcp import grammar_build_set

    body = grammar_build_set.grammar_pilot_parity()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_grammar_set_tier(args: argparse.Namespace) -> int:
    from rust_engine_mcp import grammar_build_set

    if getattr(args, "write_witness", False):
        rel = getattr(args, "witness_path", "") or None
        body = grammar_build_set.write_grammar_set_tier_witness(rel_path=rel or None)
    else:
        body = grammar_build_set.grammar_set_tier()
    print(json.dumps(body, indent=2))
    return 0


def _cmd_grammar_facility_brief(args: argparse.Namespace) -> int:
    from rust_engine_mcp import grammar_facility_brief

    gid = getattr(args, "grammar_id", "") or None
    if getattr(args, "write_witness", False):
        body = grammar_facility_brief.write_grammar_facility_brief_witness(grammar_id=gid)
    else:
        body = grammar_facility_brief.grammar_facility_brief(grammar_id=gid)
    print(json.dumps(body, indent=2))
    return 0 if body.get("ok") else 1


def _cmd_site_zone_validate_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.validators.site_zone_grid import write_site_zone_validate_witness

    body = write_site_zone_validate_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_grammar_sweep_process_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp import grammar_build_set

    body = grammar_build_set.write_grammar_sweep_process_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_designer_grammar_quality_loop(args: argparse.Namespace) -> int:
    from rust_engine_mcp.designer_grammar_quality_loop import run_designer_grammar_quality_loop

    mode = "full" if getattr(args, "full", False) else "fast"
    body = run_designer_grammar_quality_loop(
        mode=mode,
        sweep_seeds=int(getattr(args, "sweep_seeds", 24) or 24),
        write_witness=bool(getattr(args, "write_witness", False)),
    )
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_building_set_coverage(_: argparse.Namespace) -> int:
    from rust_engine_mcp import grammar_build_set

    body = grammar_build_set.building_set_coverage_report()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_building_set_coverage_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp import grammar_build_set

    body = grammar_build_set.write_building_set_coverage_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_aps_guard_brief_parity_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.grammar_build_set import write_aps_guard_brief_parity_witness

    body = write_aps_guard_brief_parity_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_aps_grammar_tier_gates_live_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.grammar_build_set import write_aps_grammar_tier_gates_live_witness

    body = write_aps_grammar_tier_gates_live_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_aps_session_presence_dump(args: argparse.Namespace) -> int:
    from rust_engine_mcp.grammar_build_set import aps_session_presence_dump, write_aps_session_presence_witness

    if getattr(args, "write_witness", False):
        body = write_aps_session_presence_witness()
    else:
        body = aps_session_presence_dump()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_landscape_grammar_presets_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp import landscape_grammar_presets

    batch = landscape_grammar_presets.write_landscape_grammar_presets_witness()
    sign = landscape_grammar_presets.refresh_mcp_landscape_grammar_sign_witness()
    green = bool(batch.get("green")) and bool(sign.get("green"))
    print(json.dumps({"green": green, "batch": batch.get("written"), "sign": sign.get("written")}, indent=2))
    return 0 if green else 1


def _cmd_landscape_sign_atlas_witness(args: argparse.Namespace) -> int:
    from rust_engine_mcp import landscape_sign_atlas

    body = landscape_sign_atlas.run_landscape_sign_atlas_refresh(
        refresh_atlas=bool(getattr(args, "refresh_atlas", False)),
    )
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_kit_production_002_g2_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp import kit_production_002

    body = kit_production_002.refresh_kit_production_002_g2_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_kit_production_002_g2_run(args: argparse.Namespace) -> int:
    from rust_engine_mcp import kit_production_002

    body = kit_production_002.run_kit_production_002_g2_full(promote=not args.no_promote)
    print(json.dumps(body, indent=2))
    return 0 if body.get("ok") else 1


def _cmd_kit_production_002_g3_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp import kit_production_002

    body = kit_production_002.refresh_kit_production_002_g3_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_kit_production_002_g3_validate(_: argparse.Namespace) -> int:
    from rust_engine_mcp import kit_production_002

    body = kit_production_002.validate_kit_production_002_g3_batch()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_kit_production_002_g4_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp import kit_production_002

    body = kit_production_002.refresh_kit_production_002_g4_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_kit_production_002_g4_evaluate(_: argparse.Namespace) -> int:
    from rust_engine_mcp import kit_production_002

    body = kit_production_002.evaluate_kit_production_002_g4()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_dmcp_e3_vegetation_catalog_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.vegetation_variant_catalog import refresh_dmcp_e3_witness

    body = refresh_dmcp_e3_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_veg_catalog_burn_rows_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.veg_catalog_loader import refresh_veg_catalog_burn_rows_witness

    body = refresh_veg_catalog_burn_rows_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_dmcp_e4_matrix_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.dmcp_designer_signoff import refresh_dmcp_e4_matrix_witness

    body = refresh_dmcp_e4_matrix_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_dmcp_e0_artist_reverdict_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.dmcp_designer_signoff import refresh_dmcp_e0_artist_reverdict_witness

    body = refresh_dmcp_e0_artist_reverdict_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_dmcp_ovr_g0_audit_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.aps_uiux_g0_audit import refresh_dmcp_ovr_g0_audit_witness

    body = refresh_dmcp_ovr_g0_audit_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("audit_complete") else 1


def _cmd_dmcp_ovr_p2_impl_audit_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.aps_uiux_p2_impl_audit import refresh_dmcp_ovr_p2_impl_audit_witness

    body = refresh_dmcp_ovr_p2_impl_audit_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_dmcp_ovr_p3_accept_rubric_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.aps_uiux_p3_accept_rubric import refresh_dmcp_ovr_p3_accept_rubric_witness

    body = refresh_dmcp_ovr_p3_accept_rubric_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_dmcp_veg_wave_tail_reconcile_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.dmcp_veg_wave_tail_reconcile import refresh_dmcp_veg_wave_tail_reconcile_witness

    body = refresh_dmcp_veg_wave_tail_reconcile_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_dmcp_parallel_lane_bkkf_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.dmcp_parallel_lane_bkkf import refresh_dmcp_parallel_lane_bkkf_witness

    body = refresh_dmcp_parallel_lane_bkkf_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_dmcp_facility_binding_lane_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.dmcp_facility_binding_lane import refresh_dmcp_facility_binding_lane_witness

    body = refresh_dmcp_facility_binding_lane_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_dmcp_nuclear_pwr_spec_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.dmcp_nuclear_pwr_spec import refresh_dmcp_nuclear_pwr_spec_witness

    body = refresh_dmcp_nuclear_pwr_spec_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_dmcp_transformer_pad_spec_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.dmcp_transformer_pad_spec import refresh_dmcp_transformer_pad_spec_witness

    body = refresh_dmcp_transformer_pad_spec_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_dmcp_art_spine_hub_wave_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.dmcp_art_spine_hub_wave import refresh_dmcp_art_spine_hub_wave_witness

    body = refresh_dmcp_art_spine_hub_wave_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_dmcp_designer_mcp_open_lane_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.dmcp_designer_mcp_open_lane import refresh_dmcp_designer_mcp_open_lane_witness

    body = refresh_dmcp_designer_mcp_open_lane_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_dmcp_style_landscape_riparian_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.dmcp_style_landscape_riparian import refresh_dmcp_style_landscape_riparian_witness

    body = refresh_dmcp_style_landscape_riparian_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_dmcp_reaction_territory_events_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.dmcp_reaction_territory_events import refresh_dmcp_reaction_territory_events_witness

    body = refresh_dmcp_reaction_territory_events_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_reaction_territory_resolve(args: argparse.Namespace) -> int:
    from rust_engine_mcp.reaction_territory import resolve_reaction_territory_variant

    body = resolve_reaction_territory_variant(args.event_id, args.domain)
    print(json.dumps(body, indent=2))
    return 0


def _cmd_reaction_territory_sessions_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.reaction_territory import refresh_reaction_territory_witness

    body = refresh_reaction_territory_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_pwr_utility_manifest_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_utility import refresh_pwr_utility_manifest_witness

    body = refresh_pwr_utility_manifest_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_pwr_substation_batch_run(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_utility import refresh_substation_batch_witness, run_substation_batch

    result = run_substation_batch()
    witness = refresh_substation_batch_witness()
    print(json.dumps({"batch": result, "witness": witness}, indent=2))
    return 0 if witness.get("green") else 1


def _cmd_pwr_transformer_batch_run(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_utility import refresh_transformer_batch_witness, run_transformer_batch

    result = run_transformer_batch()
    witness = refresh_transformer_batch_witness()
    print(json.dumps({"batch": result, "witness": witness}, indent=2))
    return 0 if witness.get("green") else 1


def _cmd_pwr_substation_promote(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_utility import promote_substation, refresh_substation_promote_witness

    result = promote_substation()
    witness = refresh_substation_promote_witness()
    print(json.dumps({"promote": result, "witness": witness}, indent=2))
    return 0 if witness.get("green") else 1


def _cmd_pwr_transformer_promote(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_utility import promote_transformer, refresh_transformer_promote_witness

    result = promote_transformer()
    witness = refresh_transformer_promote_witness()
    print(json.dumps({"promote": result, "witness": witness}, indent=2))
    return 0 if witness.get("green") else 1


def _cmd_pwr_substation_batch_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_utility import refresh_substation_batch_witness

    body = refresh_substation_batch_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_pwr_transformer_batch_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_utility import refresh_transformer_batch_witness

    body = refresh_transformer_batch_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_pwr_substation_promote_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_utility import refresh_substation_promote_witness

    body = refresh_substation_promote_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_pwr_transformer_promote_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_utility import refresh_transformer_promote_witness

    body = refresh_transformer_promote_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_pwr_downstream_close_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_utility import (
        refresh_power_grid_art_downstream_close_witness,
        refresh_substation_qc_witness,
        refresh_transformer_qc_witness,
        sync_power_grid_queue_statuses,
    )

    refresh_substation_qc_witness()
    refresh_transformer_qc_witness()
    body = refresh_power_grid_art_downstream_close_witness()
    queue = sync_power_grid_queue_statuses()
    body["queue_sync"] = queue
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_pwr_industrial_activation_utility_refresh(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_utility import refresh_industrial_activation_utility_art

    body = refresh_industrial_activation_utility_art()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_pwr_nuclear_manifest_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_nuclear import refresh_nuclear_manifest_witness

    body = refresh_nuclear_manifest_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_pwr_nuclear_batch_run(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_nuclear import refresh_nuclear_batch_witness, run_nuclear_batch

    result = run_nuclear_batch()
    witness = refresh_nuclear_batch_witness()
    print(json.dumps({"batch": result, "witness": witness}, indent=2))
    return 0 if witness.get("green") else 1


def _cmd_pwr_nuclear_promote(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_nuclear import promote_nuclear, refresh_nuclear_promote_witness

    result = promote_nuclear()
    witness = refresh_nuclear_promote_witness()
    print(json.dumps({"promote": result, "witness": witness}, indent=2))
    return 0 if witness.get("green") else 1


def _cmd_pwr_nuclear_batch_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_nuclear import refresh_nuclear_batch_witness

    body = refresh_nuclear_batch_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_pwr_nuclear_promote_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_nuclear import refresh_nuclear_promote_witness

    body = refresh_nuclear_promote_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_pilot_hardcode_lint(_: argparse.Namespace) -> int:
    from rust_engine_mcp.pilot_hardcode_lint import pilot_hardcode_lint

    body = pilot_hardcode_lint()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_pilot_hardcode_lint_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.pilot_hardcode_lint import write_pilot_hardcode_lint_witness

    body = write_pilot_hardcode_lint_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_example_teachable_audit(_: argparse.Namespace) -> int:
    from rust_engine_mcp.build_set_guards import example_teachable_audit

    body = example_teachable_audit()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_example_teachable_audit_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.build_set_guards import write_example_teachable_audit_witness

    body = write_example_teachable_audit_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_single_archetype_ratio_guard(_: argparse.Namespace) -> int:
    from rust_engine_mcp.build_set_guards import single_archetype_ratio_guard

    body = single_archetype_ratio_guard()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_warehouse_track_guard(_: argparse.Namespace) -> int:
    from rust_engine_mcp.build_set_guards import warehouse_track_guard

    body = warehouse_track_guard()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_build_set_guards_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.build_set_guards import write_build_set_guards_witnesses

    body = write_build_set_guards_witnesses()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_grammar_integration_validate(args: argparse.Namespace) -> int:
    from rust_engine_mcp.grammar_integration import grammar_integration_validate, write_grammar_integration_witness

    if getattr(args, "write_witness", False):
        body = write_grammar_integration_witness(args.path or None)
    else:
        body = grammar_integration_validate(args.path)
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_mcp_witness_honesty_validator_witness(_: argparse.Namespace) -> int:
    from rust_engine_mcp.validators.witness_honesty import refresh_mcp_witness_honesty_validator_witness

    body = refresh_mcp_witness_honesty_validator_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_validate_report(args: argparse.Namespace) -> int:
    if args.validator == "witness_honesty" and bool(getattr(args, "scan", False)):
        from rust_engine_mcp.validators.witness_honesty import validate_witness_honesty_scan

        report = validate_witness_honesty_scan(
            args.target or "debug_runs",
            compression_level=int(args.compress),
        )
    else:
        report = run_validator(
            args.validator,
            target=args.target or None,
            package=args.package or None,
            compression_level=int(args.compress),
            use_cached=bool(args.cached),
        )
    print(json.dumps(report.to_dict()))
    return 0 if report.status == "passed" else 1


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="rust_engine_mcp.cli")
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("ping").set_defaults(func=_cmd_ping)
    sub.add_parser("locate-blender").set_defaults(func=_cmd_locate_blender)
    sub.add_parser("list-staging").set_defaults(func=_cmd_list_staging)

    p = sub.add_parser("validate-spec")
    p.add_argument("path")
    p.set_defaults(func=_cmd_validate_spec)

    p = sub.add_parser("write-spec")
    p.add_argument("path")
    p.set_defaults(func=_cmd_write_spec)

    p = sub.add_parser("run-geometry")
    p.add_argument("path")
    p.set_defaults(func=_cmd_run_geometry)

    p = sub.add_parser("job-status")
    p.add_argument("job_id")
    p.set_defaults(func=_cmd_job_status)

    p = sub.add_parser("validate-glb")
    p.add_argument("path")
    p.set_defaults(func=_cmd_validate_glb_cmd)

    p = sub.add_parser("promote")
    p.add_argument("job_id")
    p.add_argument("--force", action="store_true")
    p.add_argument("--no-register", action="store_true")
    p.add_argument("--allow-smoke", action="store_true", help="Allow explicit smoke-tier harness promote")
    p.set_defaults(func=_cmd_promote)

    p = sub.add_parser("library-register")
    p.add_argument("job_id", nargs="?", default="")
    p.add_argument("--rebuild-all", action="store_true")
    p.set_defaults(func=_cmd_library_register)

    p = sub.add_parser("library-search")
    p.add_argument("--tags", default="")
    p.add_argument("--archetype", default="")
    p.add_argument("--style-pack", default="")
    p.add_argument("--category", default="")
    p.add_argument("--batch-id", default="")
    p.set_defaults(func=_cmd_library_search)

    p = sub.add_parser("write-witness")
    p.add_argument("batch_id")
    p.set_defaults(func=_cmd_write_witness)

    p = sub.add_parser("generate-material-textures")
    p.add_argument("--profile", default="")
    p.add_argument("--all-pilot", action="store_true")
    p.set_defaults(func=_cmd_generate_material_textures)

    p = sub.add_parser("tile-atlas-pack")
    p.add_argument("folder", help="Folder of PNG stills from Blender keyframe render")
    p.add_argument("-pk", "--keyframe-rename", action="store_true", help="Rename keyframe prefixes before pack")
    p.set_defaults(func=_cmd_tile_atlas_pack)

    p = sub.add_parser("lod0-batch-run")
    p.add_argument("--batch", dest="batch_id", default="kit_lod0_003")
    p.add_argument(
        "--phase",
        default="full",
        choices=["g0g1", "geometry", "promote", "full", "all"],
    )
    p.set_defaults(func=_cmd_lod0_batch_run)

    p = sub.add_parser("tile-batch-run")
    p.add_argument("path", nargs="?", default="", help="tile_batch_v1 JSON path")
    p.set_defaults(func=_cmd_tile_batch_run)

    p = sub.add_parser("assembly-snapshot-generate")
    p.add_argument("--style-pack", default="style_victorian")
    p.add_argument("--footprint", default="4x3", help="WxD e.g. 4x3")
    p.add_argument("--floors", type=int, default=2)
    p.add_argument("--seed", type=int, default=42)
    p.set_defaults(func=_cmd_assembly_snapshot_generate)

    p = sub.add_parser("build-iso-rig")
    p.add_argument(
        "--procedural-only",
        action="store_true",
        help="Skip legacy Light_keysshotsetup extraction; build minimal sun/fill/camera rig",
    )
    p.set_defaults(func=_cmd_build_iso_rig)

    p = sub.add_parser("assembly-build-run")
    p.add_argument("path", help="assembly_snapshot_v1 JSON path")
    p.add_argument(
        "--ensure-materials",
        action="store_true",
        help="BUILD-WORKER-001: generate PBR textures from snapshot material_profile then build blend",
    )
    p.add_argument("--write-witness", action="store_true", help="Write debug_runs/build_worker_001_live.json")
    p.add_argument(
        "--render-still",
        action="store_true",
        help="After blend build, render preview still (keyframe headless or trimesh fallback)",
    )
    p.add_argument(
        "--building-definition",
        help="building_definition JSON for keyframe render leg (default warehouse pilot)",
    )
    p.set_defaults(func=_cmd_assembly_build_run)

    p = sub.add_parser("pg-module-audit-002-run")
    p.add_argument(
        "--phase",
        default="full",
        choices=["sync", "specs", "geometry", "promote", "full", "all"],
        help="Pipeline phase (default full = specs+geometry+promote)",
    )
    p.add_argument("--priorities", default="P0,P1", help="Comma-separated gap priorities")
    p.add_argument("--no-blender", action="store_true", help="Bootstrap GLB from lod0 without Blender")
    p.set_defaults(func=_cmd_pg_module_audit_002_run)

    p = sub.add_parser("keyframe-matrix-export")
    p.add_argument(
        "path",
        help="building_definition JSON (warehouse pilot)",
        nargs="?",
        default="tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json",
    )
    p.add_argument("--plan-only", action="store_true", help="Write compile plan JSON only (24 cells)")
    p.add_argument("--force", action="store_true", help="Re-bake existing PNG cells")
    p.add_argument("--dry-run", action="store_true", help="Skip Blender requirement (plan/validate only)")
    p.set_defaults(func=_cmd_keyframe_matrix_export)

    p = sub.add_parser("preview-assembly")
    p.add_argument(
        "path",
        help="assembly_snapshot_v1 JSON (e.g. schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json)",
    )
    p.add_argument("--out", help="PNG output path (default debug_runs/preview_jobs/<assembly_id>_thumb.png)")
    p.add_argument("--no-browser", action="store_true", help="Do not open three.js preview tab")
    p.add_argument(
        "--serve-seconds",
        type=float,
        default=0,
        help="Keep preview HTTP server alive N seconds (default 120 open-browser, 300 --no-browser)",
    )
    p.add_argument("--skip-bevy", action="store_true", help="Skip Bevy worker attempt")
    p.add_argument("--write-witness", action="store_true", help="Write debug_runs/aps_preview_002_live.json")
    p.set_defaults(func=_cmd_preview_assembly)

    p = sub.add_parser("tile-batch-status")
    p.add_argument("batch_id")
    p.set_defaults(func=_cmd_tile_batch_status)

    p = sub.add_parser("variant-set-validate")
    p.add_argument("path", help="variant_set_v1 .json or .ron path")
    p.set_defaults(func=_cmd_variant_set_validate)

    p = sub.add_parser("variant-set-patch")
    p.add_argument("path", help="variant_set document path")
    p.add_argument("patch_json", help="RFC6902 patch JSON file or {patch:[...]} wrapper")
    p.add_argument("--dry-run", action="store_true", help="Apply in memory only")
    p.set_defaults(func=_cmd_variant_set_patch)

    p = sub.add_parser("variant-bake")
    p.add_argument("path", help="variant_set_v1 path")
    p.add_argument("variant_key")
    p.add_argument("--seed", type=int, default=0, help="0 = use variant_set seed")
    p.set_defaults(func=_cmd_variant_bake)

    p = sub.add_parser("variant-agent-request")
    p.add_argument("request_json", help="Agent callback contract JSON")
    p.add_argument("--no-write", action="store_true", help="Skip debug_runs write")
    p.set_defaults(func=_cmd_variant_agent_request)

    p = sub.add_parser("tile-atlas-register")
    p.add_argument("batch_id", help="tile_batch batch_id e.g. tile_rowhouse_victorian_pilot_v1")
    p.add_argument("--batch-json", default="", help="Optional tile_batch_v1.json path")
    p.set_defaults(func=_cmd_tile_atlas_register)

    p = sub.add_parser("rail-warehouse-pilot-batch-run")
    p.add_argument(
        "--no-headless",
        action="store_true",
        help="Skip headless keyframe export (expects PNGs in staging)",
    )
    p.set_defaults(func=_cmd_rail_warehouse_pilot_batch_run)

    p = sub.add_parser("tile-keyframe-export")
    p.add_argument("path", help="tile_batch_v1.json (keyframe_pack / ship production)")
    p.set_defaults(func=_cmd_tile_keyframe_export)

    p = sub.add_parser("variant-matrix-expand")
    p.add_argument("path", help="variant_matrix_* v1 YAML path")
    p.add_argument("--minimum-only", action="store_true", help="Ship-minimum keys only")
    p.add_argument("--no-fire-row", action="store_true", help="Omit burning_00..07")
    p.add_argument("--write-batch", action="store_true", help="Include tile_batch + variant_set rows")
    p.set_defaults(func=_cmd_variant_matrix_expand)

    sub.add_parser("write-procedural-tiles-production-bake-witness").set_defaults(
        func=_cmd_write_procedural_tiles_production_bake_witness
    )

    p = sub.add_parser("validate-report")
    p.add_argument(
        "validator",
        choices=[
            "cargo",
            "bevy",
            "mcp_spec",
            "mcp_job",
            "asset_glb",
            "tile_batch",
            "atlas_meta_v2",
            "visual_config",
            "tile_promotion",
            "assembly_grammar",
            "assembly_p0",
            "assembly_production",
            "building_set_coverage",
            "pilot_hardcode_lint",
            "landscape_grammar",
            "landscape_grammar_presets",
            "arch_build_grammar",
            "site_zone_grid",
            "construction",
            "witness_honesty",
            "queue_integrity",
        ],
        help="Validator id",
    )
    p.add_argument("target", nargs="?", default="", help="Path for file validators")
    p.add_argument("--scan", action="store_true", help="Scan *_live.json under target dir (witness_honesty)")
    p.add_argument("-p", "--package", default="")
    p.add_argument("--compress", default="3", help="Compression level 1-4")
    p.add_argument("--cached", action="store_true", help="Use orchestrator last_run.json for cargo")
    p.set_defaults(func=_cmd_validate_report)

    p = sub.add_parser("write-tile-fix-10-witness")
    p.add_argument(
        "--building",
        default="tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json",
    )
    p.set_defaults(func=_cmd_write_tile_fix_10_witness)

    p = sub.add_parser("write-tile-fix-designer-g4-witness")
    p.add_argument(
        "--building",
        default="tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json",
    )
    p.add_argument(
        "--allow-headless",
        action="store_true",
        help="Schema-only (debug); default requires keyframe_manual art_quality for proceed_ship",
    )
    p.set_defaults(func=_cmd_write_tile_fix_designer_g4_witness)

    p = sub.add_parser("agent-queue-next")
    p.add_argument(
        "agent",
        help="planner | coder | designer | coder-mcp | designer-mcp | coder_a | operator | …",
    )
    p.add_argument("--queue", default="auto", choices=["auto", *sorted(agent_queue.QUEUE_REGISTRY)])
    p.add_argument("--mark-in-progress", action="store_true")
    p.set_defaults(func=_cmd_agent_queue_next)

    p = sub.add_parser("get-que", help="BLANG:Q+ multi-parallel — next slice + drain board (alias: get que)")
    p.add_argument("agent")
    p.add_argument("--track", default="", help="T1..T8 or track_id filter")
    p.add_argument("--demand", action="store_true", help="Attach hour-scale demand todo list")
    p.add_argument("--minutes", default="60", help="Demand budget when --demand (default 60)")
    p.add_argument("--mark-in-progress", action="store_true")
    p.set_defaults(func=_cmd_get_que)

    p = sub.add_parser("agent-queue-demand", help="Build ordered session todo list without picking first slice")
    p.add_argument("agent")
    p.add_argument("--minutes", default="60")
    p.add_argument("--max-slices", default="8")
    p.add_argument("--track", default="")
    p.set_defaults(func=_cmd_agent_queue_demand)

    p = sub.add_parser("agent-queue-update")
    p.add_argument("slice_id")
    p.add_argument("status")
    p.add_argument("--note", default="")
    p.add_argument(
        "--queue",
        default="auto",
        choices=["auto", "designer_active", *sorted(agent_queue.QUEUE_REGISTRY)],
    )
    p.add_argument(
        "--enforce",
        action="store_true",
        help="Block done if exit_predicate missing or witness fails WIT-EXIT-PREDICATE",
    )
    p.set_defaults(func=_cmd_agent_queue_update)

    p = sub.add_parser("agent-queue-board")
    p.add_argument("--agent", default="")
    p.add_argument("--queue", default="grammar", choices=sorted(agent_queue.QUEUE_REGISTRY))
    p.set_defaults(func=_cmd_agent_queue_board)

    p = sub.add_parser("witness-brief")
    p.add_argument("path")
    p.add_argument("--profile", default="", help="construction | map_pick | fire_product | honesty")
    p.set_defaults(func=_cmd_witness_brief)

    sub.add_parser("review-order-brief").set_defaults(func=_cmd_review_order_brief)
    p = sub.add_parser("slice-exec-brief")
    p.add_argument("slice_id")
    p.add_argument("--queue", default="", help="phase4 | grammar | … (default: search all)")
    p.set_defaults(func=_cmd_slice_exec_brief)
    sub.add_parser("mcp-phase4-queue-witness").set_defaults(func=_cmd_mcp_phase4_queue_witness)
    sub.add_parser("mcp-valid-construction-witness").set_defaults(func=_cmd_mcp_valid_construction_witness)

    p = sub.add_parser("handoff-brief")
    p.set_defaults(func=_cmd_handoff_brief)

    p = sub.add_parser("file-digest")
    p.add_argument("path")
    p.add_argument("--max-lines", default="40")
    p.set_defaults(func=_cmd_file_digest)

    p = sub.add_parser("orchestrator-brief")
    p.add_argument("--fresh", action="store_true", help="Skip last_run.json cache")
    p.set_defaults(func=_cmd_orchestrator_brief)

    sub.add_parser("token-savings-guide").set_defaults(func=_cmd_token_savings_guide)

    p = sub.add_parser("pipeline-preflight", help="MCP-PREFLIGHT-001 environment check")
    p.add_argument("--queue", default="grammar")
    p.add_argument("--check-build-set", action="store_true", help="MCP-PREFLIGHT-BUILD-SET-001")
    p.set_defaults(func=_cmd_pipeline_preflight)

    sub.add_parser("ops-get-project-brief").set_defaults(func=_cmd_ops_get_project_brief)
    sub.add_parser("build-read-grammar-v0-002-witness").set_defaults(func=_cmd_build_read_grammar_v0_002_witness)
    sub.add_parser("aps-dna-consumer-witness").set_defaults(func=_cmd_aps_dna_consumer_witness)
    sub.add_parser("arch-dna-consumer-contract").set_defaults(func=_cmd_arch_dna_consumer_contract)
    p = sub.add_parser("arch-dna-snapshot-brief")
    p.add_argument("path")
    p.set_defaults(func=_cmd_arch_dna_snapshot_brief)
    p = sub.add_parser("ops-get-retry-guidance")
    p.add_argument("task_id")
    p.set_defaults(func=_cmd_ops_get_retry_guidance)

    p = sub.add_parser("ops-get-active-blockers")
    p.set_defaults(func=_cmd_ops_get_active_blockers)

    p = sub.add_parser("ops-dashboard-refresh")
    p.add_argument("--window-hours", type=int, default=168)
    p.set_defaults(func=_cmd_ops_dashboard_refresh)

    sub.add_parser("ops-process-scan").set_defaults(func=_cmd_ops_process_scan)
    sub.add_parser("ops-drift-scan").set_defaults(func=_cmd_ops_drift_scan)

    p = sub.add_parser("ops-run-events-rollup")
    p.add_argument("--window-hours", type=int, default=168)
    p.set_defaults(func=_cmd_ops_run_events_rollup)

    sub.add_parser("ops-crash-scan").set_defaults(func=_cmd_ops_crash_scan)

    p = sub.add_parser("ops-triage-refresh")
    p.add_argument("--window-hours", type=int, default=168)
    p.set_defaults(func=_cmd_ops_triage_refresh)

    p = sub.add_parser("ops-crash-daemon")
    p.add_argument("--interval-sec", type=int, default=30)
    p.add_argument("--max-cycles", type=int, default=None)
    p.set_defaults(func=_cmd_ops_crash_daemon)

    p = sub.add_parser("ops-mcp-function-layer-witness")
    p.set_defaults(func=_cmd_ops_mcp_function_layer_witness)

    p = sub.add_parser("grammar-set-brief")
    p.add_argument("--set-id", default="")
    p.add_argument("--write-witness", action="store_true")
    p.set_defaults(func=_cmd_grammar_set_brief)

    p = sub.add_parser("grammar-preset-pair-validate")
    p.add_argument("--preset-id", default="")
    p.add_argument("--path", default="")
    p.set_defaults(func=_cmd_grammar_preset_pair_validate)

    p = sub.add_parser("grammar-eval-sweep")
    p.add_argument("--archetype", default="IndustrialWarehouse")
    p.add_argument("--district", default="industrial_west")
    p.add_argument("--seeds", default="", help="Comma-separated seed list")
    p.set_defaults(func=_cmd_grammar_eval_sweep)

    sub.add_parser("grammar-pilot-parity").set_defaults(func=_cmd_grammar_pilot_parity)

    p = sub.add_parser("grammar-set-tier")
    p.add_argument("--write-witness", action="store_true")
    p.add_argument("--witness-path", default="", help="Relative path under repo root")
    p.set_defaults(func=_cmd_grammar_set_tier)

    p = sub.add_parser("grammar-facility-brief")
    p.add_argument("--grammar-id", default="", help="Single grammar_id (default: all grammars)")
    p.add_argument("--write-witness", action="store_true")
    p.set_defaults(func=_cmd_grammar_facility_brief)

    sub.add_parser("site-zone-validate-witness").set_defaults(func=_cmd_site_zone_validate_witness)
    sub.add_parser("grammar-sweep-process-witness").set_defaults(func=_cmd_grammar_sweep_process_witness)

    p = sub.add_parser("designer-grammar-quality-loop")
    p.add_argument("--full", action="store_true", help="Run grammar_eval_sweep per archetype")
    p.add_argument("--sweep-seeds", type=int, default=24)
    p.add_argument("--write-witness", action="store_true")
    p.set_defaults(func=_cmd_designer_grammar_quality_loop)

    sub.add_parser("building-set-coverage").set_defaults(func=_cmd_building_set_coverage)
    sub.add_parser("building-set-coverage-witness").set_defaults(func=_cmd_building_set_coverage_witness)
    sub.add_parser("aps-guard-brief-parity-witness").set_defaults(func=_cmd_aps_guard_brief_parity_witness)
    sub.add_parser("aps-grammar-tier-gates-live-witness").set_defaults(
        func=_cmd_aps_grammar_tier_gates_live_witness
    )
    p = sub.add_parser("aps-session-presence-dump")
    p.add_argument("--write-witness", action="store_true")
    p.set_defaults(func=_cmd_aps_session_presence_dump)
    sub.add_parser("landscape-grammar-presets-witness").set_defaults(func=_cmd_landscape_grammar_presets_witness)

    p = sub.add_parser("landscape-sign-atlas-witness")
    p.add_argument("--refresh-atlas", action="store_true")
    p.set_defaults(func=_cmd_landscape_sign_atlas_witness)

    p = sub.add_parser("kit-production-002-g2-run")
    p.add_argument("--no-promote", action="store_true")
    p.set_defaults(func=_cmd_kit_production_002_g2_run)

    sub.add_parser("kit-production-002-g2-witness").set_defaults(func=_cmd_kit_production_002_g2_witness)

    sub.add_parser("kit-production-002-g3-validate").set_defaults(func=_cmd_kit_production_002_g3_validate)
    sub.add_parser("kit-production-002-g3-witness").set_defaults(func=_cmd_kit_production_002_g3_witness)
    sub.add_parser("kit-production-002-g4-evaluate").set_defaults(func=_cmd_kit_production_002_g4_evaluate)
    sub.add_parser("kit-production-002-g4-witness").set_defaults(func=_cmd_kit_production_002_g4_witness)
    sub.add_parser("dmcp-e3-vegetation-catalog-witness").set_defaults(func=_cmd_dmcp_e3_vegetation_catalog_witness)
    sub.add_parser("veg-catalog-burn-rows-witness").set_defaults(func=_cmd_veg_catalog_burn_rows_witness)
    sub.add_parser("dmcp-e4-matrix-charter-witness").set_defaults(func=_cmd_dmcp_e4_matrix_witness)
    sub.add_parser("dmcp-e0-artist-reverdict-witness").set_defaults(func=_cmd_dmcp_e0_artist_reverdict_witness)
    sub.add_parser("dmcp-ovr-g0-audit-witness").set_defaults(func=_cmd_dmcp_ovr_g0_audit_witness)
    sub.add_parser("dmcp-ovr-p2-impl-audit-witness").set_defaults(func=_cmd_dmcp_ovr_p2_impl_audit_witness)
    sub.add_parser("dmcp-ovr-p3-accept-rubric-witness").set_defaults(func=_cmd_dmcp_ovr_p3_accept_rubric_witness)
    sub.add_parser("dmcp-veg-wave-tail-reconcile-witness").set_defaults(func=_cmd_dmcp_veg_wave_tail_reconcile_witness)
    sub.add_parser("dmcp-parallel-lane-bkkf-witness").set_defaults(func=_cmd_dmcp_parallel_lane_bkkf_witness)
    sub.add_parser("dmcp-facility-binding-lane-witness").set_defaults(func=_cmd_dmcp_facility_binding_lane_witness)
    sub.add_parser("dmcp-nuclear-pwr-spec-witness").set_defaults(func=_cmd_dmcp_nuclear_pwr_spec_witness)
    sub.add_parser("dmcp-transformer-pad-spec-witness").set_defaults(func=_cmd_dmcp_transformer_pad_spec_witness)
    sub.add_parser("dmcp-art-spine-hub-wave-witness").set_defaults(func=_cmd_dmcp_art_spine_hub_wave_witness)
    sub.add_parser("dmcp-designer-mcp-open-lane-witness").set_defaults(
        func=_cmd_dmcp_designer_mcp_open_lane_witness
    )
    sub.add_parser("dmcp-reaction-territory-events-witness").set_defaults(
        func=_cmd_dmcp_reaction_territory_events_witness
    )
    p = sub.add_parser("reaction-territory-resolve")
    p.add_argument("event_id")
    p.add_argument("domain")
    p.set_defaults(func=_cmd_reaction_territory_resolve)
    sub.add_parser("reaction-territory-sessions-witness").set_defaults(
        func=_cmd_reaction_territory_sessions_witness
    )
    sub.add_parser("pwr-utility-manifest-witness").set_defaults(func=_cmd_pwr_utility_manifest_witness)
    sub.add_parser("pwr-substation-batch-run").set_defaults(func=_cmd_pwr_substation_batch_run)
    sub.add_parser("pwr-transformer-batch-run").set_defaults(func=_cmd_pwr_transformer_batch_run)
    sub.add_parser("pwr-substation-promote").set_defaults(func=_cmd_pwr_substation_promote)
    sub.add_parser("pwr-transformer-promote").set_defaults(func=_cmd_pwr_transformer_promote)
    sub.add_parser("pwr-substation-batch-witness").set_defaults(func=_cmd_pwr_substation_batch_witness)
    sub.add_parser("pwr-transformer-batch-witness").set_defaults(func=_cmd_pwr_transformer_batch_witness)
    sub.add_parser("pwr-substation-promote-witness").set_defaults(func=_cmd_pwr_substation_promote_witness)
    sub.add_parser("pwr-transformer-promote-witness").set_defaults(func=_cmd_pwr_transformer_promote_witness)
    sub.add_parser("pwr-downstream-close-witness").set_defaults(func=_cmd_pwr_downstream_close_witness)
    sub.add_parser("pwr-industrial-activation-utility-refresh").set_defaults(
        func=_cmd_pwr_industrial_activation_utility_refresh
    )
    sub.add_parser("pwr-nuclear-manifest-witness").set_defaults(func=_cmd_pwr_nuclear_manifest_witness)
    sub.add_parser("pwr-nuclear-batch-run").set_defaults(func=_cmd_pwr_nuclear_batch_run)
    sub.add_parser("pwr-nuclear-promote").set_defaults(func=_cmd_pwr_nuclear_promote)
    sub.add_parser("pwr-nuclear-batch-witness").set_defaults(func=_cmd_pwr_nuclear_batch_witness)
    sub.add_parser("pwr-nuclear-promote-witness").set_defaults(func=_cmd_pwr_nuclear_promote_witness)
    sub.add_parser("mcp-witness-honesty-validator-witness").set_defaults(
        func=_cmd_mcp_witness_honesty_validator_witness
    )
    p = sub.add_parser("intel-officer-sweep", help="False-positive surveillance — done/green cull candidates")
    p.add_argument("--queue-filter", default="", help="Limit queue_integrity scan to one queue_id")
    p.add_argument("--no-witness-scan", action="store_true", help="Skip debug_runs *_live.json green scan")
    p.add_argument("--compress", default="3", choices=["1", "2", "3", "4"])
    p.set_defaults(func=_cmd_intel_officer_sweep)

    p = sub.add_parser("intel-officer-apply", help="Supervised reopen/demote from sweep findings")
    p.add_argument("--ids", required=True, help="Comma-separated slice_id or finding id")
    p.add_argument("--apply", action="store_true", help="Execute (default is dry-run preview)")
    p.add_argument("--action", default="reopen", choices=["reopen", "blocked"])
    p.add_argument("--note", default="")
    p.set_defaults(func=_cmd_intel_officer_apply)

    sub.add_parser("intel-officer-sweep-witness").set_defaults(func=_cmd_intel_officer_sweep_witness)

    sub.add_parser("queue-integrity-reconcile-witness").set_defaults(func=_cmd_queue_integrity_reconcile_witness)
    sub.add_parser("mcp-witness-integrity-ops-witness").set_defaults(func=_cmd_mcp_witness_integrity_ops_witness)
    sub.add_parser("pilot-hardcode-lint").set_defaults(func=_cmd_pilot_hardcode_lint)
    sub.add_parser("pilot-hardcode-lint-witness").set_defaults(func=_cmd_pilot_hardcode_lint_witness)
    sub.add_parser("example-teachable-audit").set_defaults(func=_cmd_example_teachable_audit)
    sub.add_parser("example-teachable-audit-witness").set_defaults(func=_cmd_example_teachable_audit_witness)
    sub.add_parser("single-archetype-ratio-guard").set_defaults(func=_cmd_single_archetype_ratio_guard)
    sub.add_parser("warehouse-track-guard").set_defaults(func=_cmd_warehouse_track_guard)
    sub.add_parser("build-set-guards-witness").set_defaults(func=_cmd_build_set_guards_witness)
    p = sub.add_parser("grammar-integration-validate")
    p.add_argument(
        "path",
        nargs="?",
        default="tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json",
    )
    p.add_argument("--write-witness", action="store_true")
    p.set_defaults(func=_cmd_grammar_integration_validate)

    args = parser.parse_args(argv)
    try:
        return args.func(args)
    except Exception as exc:  # noqa: BLE001 — CLI surface
        print(json.dumps({"error": str(exc)}), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
