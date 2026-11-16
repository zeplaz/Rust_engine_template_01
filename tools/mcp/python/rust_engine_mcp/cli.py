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
import subprocess
import sys
from pathlib import Path

from . import blender_runner, library, material_textures, paths, promote, schemas, validate_glb
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


def _cmd_studio_task_enqueue(args: argparse.Namespace) -> int:
    from rust_engine_mcp import studio_task

    task = studio_task.enqueue(
        kind=args.kind,
        seed=int(args.seed),
        payload_path=args.payload,
        task_id=args.task_id,
    )
    if args.run and args.kind in (
        "geometry",
        "massing_export",
        "assembly_build",
        "param_frame",
    ):
        task = studio_task.run_queued(str(task["task_id"]))
    print(json.dumps(task))
    return 0 if task.get("status") in ("queued", "done") else 1


def _cmd_studio_task_status(args: argparse.Namespace) -> int:
    from rust_engine_mcp import studio_task

    st = studio_task.read_task(args.task_id)
    if st is None:
        print(json.dumps({"error": "not found", "task_id": args.task_id}))
        return 1
    print(json.dumps(st))
    return 0


def _cmd_studio_task_list(args: argparse.Namespace) -> int:
    from rust_engine_mcp import studio_task

    print(json.dumps({"tasks": studio_task.list_tasks(limit=int(args.limit))}))
    return 0


def _cmd_studio_task_handoff(args: argparse.Namespace) -> int:
    from rust_engine_mcp import studio_task

    marker = studio_task.blender_handoff_marker(args.task_id)
    print(json.dumps({"handoff": str(marker), "task_id": args.task_id}))
    return 0


def _cmd_studio_task_escalate(args: argparse.Namespace) -> int:
    from rust_engine_mcp import studio_task

    try:
        result = studio_task.escalate_from_param_frame(
            task_id=args.task_id,
            param_frame_path=args.param_frame,
            run=bool(args.run),
            blender_session=args.blender_session,
        )
    except (FileNotFoundError, ValueError) as exc:
        print(json.dumps({"error": str(exc)}))
        return 1
    print(json.dumps(result))
    failed = [c for c in result.get("children", []) if c.get("status") == "failed"]
    return 1 if failed else 0


def _cmd_studio_dash_render(args: argparse.Namespace) -> int:
    import sys as _sys

    dash = paths.repo_root() / "tools" / "studio_dash"
    if str(dash.parent) not in _sys.path:
        _sys.path.insert(0, str(dash.parent))
    from studio_dash.render_report import write_report

    data = json.loads(Path(args.json_path).read_text(encoding="utf-8"))
    out = Path(args.out)
    write_report(data, out, title=args.title)
    print(json.dumps({"written": str(out)}))
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


def _cmd_effect_pack(args: argparse.Namespace) -> int:
    from rust_engine_mcp import effect_promote as ep

    body = ep.pack_effect_spec(args.spec_path, force=args.force)
    print(json.dumps(body, indent=2))
    return 0


def _cmd_effect_promote(args: argparse.Namespace) -> int:
    from rust_engine_mcp import effect_promote as ep

    body = ep.effect_promote(
        spec_path=args.spec_path or "",
        batch_id=args.batch_id or "",
        phase=args.phase,
        force=args.force,
        write_witness=False,
    )
    print(json.dumps(body, indent=2))
    return 0


def _cmd_effect_preview_capture(args: argparse.Namespace) -> int:
    from rust_engine_mcp import effect_preview_capture as epc

    body = epc.effect_preview_capture(
        spec_path=args.spec_path or "",
        batch_id=args.batch_id or "",
        pack_first=not args.no_pack,
        force=args.force,
        write_witness=False,
    )
    print(json.dumps(body, indent=2))
    return 0


def _cmd_effect_chain_smoke(args: argparse.Namespace) -> int:
    from rust_engine_mcp.effect_chain_smoke import run_effect_chain_smoke

    body = run_effect_chain_smoke(write_witness=False)
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


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
        source_tier=str(getattr(args, "source_tier", None) or "production"),
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
        )
    else:
        result = assembly_build_run(args.path)
    print(json.dumps(result, indent=2))
    ok = result.get("ok")
    if ok is None:
        ok = result.get("status") == "done"
    return 0 if ok else 1


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


def _cmd_arch_dna_snapshot_brief(args: argparse.Namespace) -> int:
    from rust_engine_mcp import arch_build_grammar

    body = arch_build_grammar.arch_dna_snapshot_brief(args.path)
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


def _cmd_heatmap_ramp_export(args: argparse.Namespace) -> int:
    from rust_engine_mcp import perceptual_ramp

    out_dir = Path(args.out_dir) if args.out_dir else None
    body = perceptual_ramp.export_heatmap_ramp_from_path(
        Path(args.spec),
        out_dir=out_dir,
        write_files=not args.dry_run,
    )
    print(json.dumps(body, indent=2, sort_keys=True))
    return 0 if body.get("green") else 1


def _cmd_heatmap_ramp_qc(args: argparse.Namespace) -> int:
    from rust_engine_mcp import perceptual_ramp

    body = perceptual_ramp.qc_lut_file(Path(args.lut))
    print(json.dumps(body, indent=2, sort_keys=True))
    return 0 if body.get("green") else 1


def _cmd_generate_and_test_report(args: argparse.Namespace) -> int:
    from rust_engine_mcp import building_quality_qc

    if args.massing_seeds:
        from rust_engine_mcp import grammar_iterate

        seeds = [int(x) for x in args.massing_seeds.split(",")]
        body = grammar_iterate.run_massing_iterate_generate_and_test(
            seeds=seeds,
            archetype_id=args.archetype,
            district_style=args.district,
            top_k=int(args.top_k),
        )
    else:
        body = building_quality_qc.refresh_pcg08_golden_report(
            top_k=int(args.top_k),
            write_witness=not args.no_witness,
        )
    text = building_quality_qc.format_generate_and_test_cli(body)
    if args.json:
        print(json.dumps(body, indent=2, sort_keys=True))
    else:
        print(text)
    return 0


def _cmd_building_set_coverage(_: argparse.Namespace) -> int:
    from rust_engine_mcp import grammar_build_set

    body = grammar_build_set.building_set_coverage_report()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_grammar_rule_coverage_report(args: argparse.Namespace) -> int:
    from rust_engine_mcp import kit_coverage_audit

    if args.write:
        body = kit_coverage_audit.write_pcg06_grammar_rule_coverage_report()
    else:
        body = kit_coverage_audit.run_grammar_rule_coverage_report()
    if args.markdown:
        print(body.get("designer_table_markdown") or kit_coverage_audit.format_pcg06_designer_table_markdown(body))
    elif args.json:
        print(json.dumps(body, indent=2, sort_keys=True))
    else:
        table = body.get("designer_action_table") or []
        print(f"{kit_coverage_audit.PCG_06_TASK_ID} rows={len(table)} written={body.get('written', '')}")
        for row in table[:40]:
            rid = row.get("rule_id") or row.get("slot_key") or ""
            print(f"  {row.get('category')}: {row.get('grammar_id')} {row.get('layer', '')}:{rid}")
    return 0


def _cmd_pwr_substation_batch_run(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_utility import run_substation_batch

    result = run_substation_batch()
    print(json.dumps(result, indent=2))
    return 0 if (result.get("ok", True) if isinstance(result, dict) else True) else 1


def _cmd_pwr_transformer_batch_run(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_utility import run_transformer_batch

    result = run_transformer_batch()
    print(json.dumps(result, indent=2))
    return 0 if (result.get("ok", True) if isinstance(result, dict) else True) else 1


def _cmd_pwr_substation_promote(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_utility import promote_substation

    result = promote_substation()
    print(json.dumps(result, indent=2))
    return 0 if (result.get("ok", True) if isinstance(result, dict) else True) else 1


def _cmd_pwr_transformer_promote(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_utility import promote_transformer

    result = promote_transformer()
    print(json.dumps(result, indent=2))
    return 0 if (result.get("ok", True) if isinstance(result, dict) else True) else 1


def _cmd_pwr_nuclear_batch_run(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_nuclear import run_nuclear_batch

    result = run_nuclear_batch()
    print(json.dumps(result, indent=2))
    return 0 if (result.get("ok", True) if isinstance(result, dict) else True) else 1


def _cmd_pwr_nuclear_promote(_: argparse.Namespace) -> int:
    from rust_engine_mcp.mcp_pwr_nuclear import promote_nuclear

    result = promote_nuclear()
    print(json.dumps(result, indent=2))
    return 0 if (result.get("ok", True) if isinstance(result, dict) else True) else 1


def _cmd_grammar_integration_validate(args: argparse.Namespace) -> int:
    from rust_engine_mcp.grammar_integration import grammar_integration_validate

    body = grammar_integration_validate(args.path)
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


def _cmd_validate_report(args: argparse.Namespace) -> int:
    report = run_validator(
        args.validator,
        target=args.target or None,
        package=args.package or None,
        compression_level=int(args.compress),
        use_cached=bool(args.cached),
    )
    print(json.dumps(report.to_dict()))
    return 0 if report.status == "passed" else 1


def _cmd_validate_sound_cues(args: argparse.Namespace) -> int:
    """AUD-03 S6 — validate sound_cue_groups_v1 manifest (variants, peaks, paths)."""
    from rust_engine_mcp.validators.sound_cue import validate_sound_cue_path

    report = validate_sound_cue_path(Path(args.path), compression_level=int(args.compress))
    print(json.dumps(report.to_dict()))
    return 0 if report.status == "passed" else 1


def _cmd_pixel_conclude(args: argparse.Namespace) -> int:
    from .pixel_pipeline.cli import cli_main

    argv = ["--image", args.image, "--kind", args.kind]
    if args.out:
        argv.extend(["--out", args.out])
    return cli_main(argv)


def _cmd_ref_lookup(args: argparse.Namespace) -> int:
    """Forward argv to src/dev/ref_books_index/lookup.py (PLAN-REF-BOOKS-UPGRADE-v2 T4)."""
    lookup = paths.repo_root() / "src" / "dev" / "ref_books_index" / "lookup.py"
    forwarded = list(args.argv or [])
    if forwarded and forwarded[0] == "--":
        forwarded = forwarded[1:]
    cmd = [sys.executable, str(lookup), *forwarded]
    r = subprocess.run(cmd, cwd=str(paths.repo_root()), capture_output=True, text=True)
    if r.stdout:
        sys.stdout.write(r.stdout)
        if not r.stdout.endswith("\n"):
            sys.stdout.write("\n")
    if r.stderr:
        sys.stderr.write(r.stderr)
    return int(r.returncode)


def build_parser() -> argparse.ArgumentParser:
    """Full CLI parser — single source for ``main``, ``commands`` and MCP ``micro_tool_help``."""
    parser = argparse.ArgumentParser(
        prog="rust_engine_mcp.cli",
        description="Rust engine MCP micro CLI (same code paths as the MCP server tools).",
        epilog="List commands by group: `commands` (or `commands --grep tile`). "
        "Per-command help: `help <command>` or `<command> --help`.",
    )
    sub = parser.add_subparsers(dest="command", required=True, metavar="<command>")

    p = sub.add_parser("commands", help="List CLI commands grouped by lane")
    p.add_argument("--grep", default="", help="Substring filter on command name or help")
    p.add_argument("--group", default="", help="Only this group id (see output)")
    p.add_argument("--json", action="store_true", help="Machine-readable catalog")
    p.set_defaults(func=_cmd_commands)

    p = sub.add_parser("help", help="Show help for one command")
    p.add_argument("name", nargs="?", default="")
    p.set_defaults(func=_cmd_help)

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

    p = sub.add_parser(
        "studio-task-enqueue",
        help="Enqueue a StudioTask on the shared APS/Blender job bus",
    )
    p.add_argument(
        "--kind",
        required=True,
        choices=[
            "geometry",
            "assembly_build",
            "tile_bake",
            "report",
            "massing_export",
            "param_frame",
            "open_staging",
        ],
    )
    p.add_argument("--seed", type=int, required=True)
    p.add_argument("--payload", default=None, help="Path to underlying job/spec JSON")
    p.add_argument("--task-id", default=None)
    p.add_argument(
        "--run",
        action="store_true",
        help="Run queued worker immediately (geometry, massing_export, assembly_build, param_frame)",
    )
    p.set_defaults(func=_cmd_studio_task_enqueue)

    p = sub.add_parser(
        "studio-task-escalate",
        help="Enqueue StudioTasks from ParamFrame escalate hints (IO-09 / pipeline_app_layer)",
    )
    p.add_argument("--task-id", default=None, help="Existing kind=param_frame StudioTask id")
    p.add_argument(
        "--param-frame",
        default=None,
        help="Repo-relative ParamFrame JSON path (alternative to --task-id)",
    )
    p.add_argument("--run", action="store_true", help="Run each enqueued child task")
    p.add_argument(
        "--blender-session",
        default=None,
        help="Optional blender_session marker (e.g. addon)",
    )
    p.set_defaults(func=_cmd_studio_task_escalate)

    p = sub.add_parser("studio-task-status", help="Read a StudioTask status JSON")
    p.add_argument("task_id")
    p.set_defaults(func=_cmd_studio_task_status)

    p = sub.add_parser("studio-task-list", help="List recent StudioTasks")
    p.add_argument("--limit", type=int, default=50)
    p.set_defaults(func=_cmd_studio_task_list)

    p = sub.add_parser("studio-task-handoff", help="Write Blender interactive handoff marker")
    p.add_argument("task_id")
    p.set_defaults(func=_cmd_studio_task_handoff)

    p = sub.add_parser("studio-dash-render", help="Render studio HTML dashboard from JSON")
    p.add_argument("json_path")
    p.add_argument("-o", "--out", required=True)
    p.add_argument("--title", default=None)
    p.set_defaults(func=_cmd_studio_dash_render)

    p = sub.add_parser("validate-glb")
    p.add_argument("path")
    p.set_defaults(func=_cmd_validate_glb_cmd)

    p = sub.add_parser("promote")
    p.add_argument("job_id")
    p.add_argument("--force", action="store_true")
    p.add_argument("--no-register", action="store_true")
    p.add_argument("--allow-smoke", action="store_true", help="Allow explicit smoke-tier harness promote")
    p.set_defaults(func=_cmd_promote)

    p = sub.add_parser("effect-pack", help="VSS-T4-003 — wgsl_pack hash copy to assets/staging/<effect_id>/")
    p.add_argument("spec_path", help="EffectSpec JSON path")
    p.add_argument("--force", action="store_true", help="Overwrite staging shaders on hash drift")
    p.set_defaults(func=_cmd_effect_pack)

    p = sub.add_parser("effect-promote", help="VSS-T4-003 — pack and/or promote EffectSpec to effects/registry/")
    p.add_argument("spec_path", nargs="?", default="", help="EffectSpec JSON (omit with --batch-id)")
    p.add_argument("--batch-id", default="", help=f"Reference batch (default: vss_t4_reference_effects_v1)")
    p.add_argument(
        "--phase",
        default="full",
        choices=["pack", "promote", "full"],
        help="pack=staging only; promote=registry (expects pack); full=both",
    )
    p.add_argument(
        "--force",
        action="store_true",
        help=(
            "Overwrite staging shaders on hash drift. PCI-28 operator override: "
            "production tier may promote while honest_gate is pending. "
            "Does not override dishonest_gate."
        ),
    )
    p.set_defaults(func=_cmd_effect_promote)

    p = sub.add_parser(
        "effect-preview-capture",
        help="VSS-T4 residual — deterministic staging_pack_digest frames → honest_gate=honest",
    )
    p.add_argument("spec_path", nargs="?", default="", help="EffectSpec JSON (omit with --batch-id)")
    p.add_argument("--batch-id", default="", help="Reference batch (default: vss_t4_reference_effects_v1)")
    p.add_argument("--no-pack", action="store_true", help="Skip pack step (expect existing staging)")
    p.add_argument("--force", action="store_true", help="Force pack overwrite on hash drift")
    p.set_defaults(func=_cmd_effect_preview_capture)

    p = sub.add_parser(
        "effect-chain-smoke",
        help="PCI-21 — isolated spec→validate→preview-capture→honest_gate→promote (no cron)",
    )
    p.set_defaults(func=_cmd_effect_chain_smoke)

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
    p.add_argument(
        "--source-tier",
        default="production",
        choices=("production", "lod0"),
        help="Module tier preference (default production — BQ-ASSEMBLE-PREF-PROD-001)",
    )
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

    p = sub.add_parser("tile-keyframe-export")
    p.add_argument("path", help="tile_batch_v1.json (keyframe_pack / ship production)")
    p.set_defaults(func=_cmd_tile_keyframe_export)

    p = sub.add_parser("variant-matrix-expand")
    p.add_argument("path", help="variant_matrix_* v1 YAML path")
    p.add_argument("--minimum-only", action="store_true", help="Ship-minimum keys only")
    p.add_argument("--no-fire-row", action="store_true", help="Omit burning_00..07")
    p.add_argument("--write-batch", action="store_true", help="Include tile_batch + variant_set rows")
    p.set_defaults(func=_cmd_variant_matrix_expand)

    p = sub.add_parser(
        "validate-sound-cues",
        help="AUD-03 — validate sound_cue_groups_v1 (≥2 variants, peak spread ≤1.5 dB, unique paths)",
    )
    p.add_argument("path", help="sound_cue_groups_v1 JSON path (repo-relative or absolute)")
    p.add_argument("--compress", default="3", help="ValidationReport compression level 1-4")
    p.set_defaults(func=_cmd_validate_sound_cues)

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
            "effect_spec",
            "sound_cue",
            "tile_promotion",
            "assembly_grammar",
            "assembly_p0",
            "assembly_production",
            "building_set_coverage",
            "landscape_grammar",
            "landscape_grammar_presets",
            "arch_build_grammar",
            "site_zone_grid",
            "render_variant",
            "build_graph",
            "tile_promotion_honest",
            "param_frame",
            "massing_preview",
            "massing",
            "building_grammar",
        ],
        help="Validator id",
    )
    p.add_argument("target", nargs="?", default="", help="Path for file validators")
    p.add_argument("-p", "--package", default="")
    p.add_argument("--compress", default="3", help="Compression level 1-4")
    p.add_argument("--cached", action="store_true", help="Use orchestrator last_run.json for cargo")
    p.set_defaults(func=_cmd_validate_report)

    p = sub.add_parser("arch-dna-snapshot-brief")
    p.add_argument("path")
    p.set_defaults(func=_cmd_arch_dna_snapshot_brief)
    p = sub.add_parser("grammar-preset-pair-validate")
    p.add_argument("--preset-id", default="")
    p.add_argument("--path", default="")
    p.set_defaults(func=_cmd_grammar_preset_pair_validate)

    p = sub.add_parser("grammar-eval-sweep")
    p.add_argument("--archetype", default="IndustrialWarehouse")
    p.add_argument("--district", default="industrial_west")
    p.add_argument("--seeds", default="", help="Comma-separated seed list")
    p.set_defaults(func=_cmd_grammar_eval_sweep)

    p = sub.add_parser(
        "heatmap-ramp-export",
        help="RND-20 — export lightness-monotonic heatmap ramp LUT + RON (staging)",
    )
    p.add_argument(
        "spec",
        nargs="?",
        default="tools/mcp/schemas/examples/heatmap_ramp_default_overlay_v1.json",
        help="heatmap_ramp_v1 JSON spec",
    )
    p.add_argument(
        "--out-dir",
        default="",
        help="Output directory (default: assets/staging/ramps)",
    )
    p.add_argument("--dry-run", action="store_true", help="QC + LUT bytes only; do not write files")
    p.set_defaults(func=_cmd_heatmap_ramp_export)

    p = sub.add_parser(
        "heatmap-ramp-qc",
        help="RND-20 — QC existing .lut for strictly increasing Rec.709 lightness",
    )
    p.add_argument("lut", help="Path to RND20LUT binary")
    p.set_defaults(func=_cmd_heatmap_ramp_qc)

    p = sub.add_parser(
        "generate-and-test-report",
        help="PCG-08 — seeded generate-and-test histogram (golden set default)",
    )
    p.add_argument("--top-k", default="3", help="Top-k candidates to list")
    p.add_argument("--json", action="store_true", help="Emit JSON instead of deterministic CLI text")
    p.add_argument("--no-witness", action="store_true", help="Skip writing debug_runs/pcg_08_generate_test_live.json")
    p.add_argument("--archetype", default="IndustrialWarehouse", help="Massing iterate archetype")
    p.add_argument("--district", default="industrial_west", help="Massing iterate district")
    p.add_argument(
        "--massing-seeds",
        default="",
        help="Optional comma-separated seeds for massing iterate sweep (default: golden BQ-Q3 set)",
    )
    p.set_defaults(func=_cmd_generate_and_test_report)

    p = sub.add_parser(
        "grammar-rule-coverage-report",
        help="PCG-06 — golden-seed grammar rule coverage (report only, staging JSON)",
    )
    p.add_argument("--json", action="store_true", help="Emit full JSON report")
    p.add_argument("--markdown", action="store_true", help="Emit designer markdown table")
    p.add_argument(
        "--write",
        action="store_true",
        help="Write assets/staging/reports/pcg_06_grammar_rule_coverage.json",
    )
    p.set_defaults(func=_cmd_grammar_rule_coverage_report)

    sub.add_parser("building-set-coverage").set_defaults(func=_cmd_building_set_coverage)
    sub.add_parser("pwr-substation-batch-run").set_defaults(func=_cmd_pwr_substation_batch_run)
    sub.add_parser("pwr-transformer-batch-run").set_defaults(func=_cmd_pwr_transformer_batch_run)
    sub.add_parser("pwr-substation-promote").set_defaults(func=_cmd_pwr_substation_promote)
    sub.add_parser("pwr-transformer-promote").set_defaults(func=_cmd_pwr_transformer_promote)
    sub.add_parser("pwr-nuclear-batch-run").set_defaults(func=_cmd_pwr_nuclear_batch_run)
    sub.add_parser("pwr-nuclear-promote").set_defaults(func=_cmd_pwr_nuclear_promote)
    p = sub.add_parser("grammar-integration-validate")
    p.add_argument(
        "path",
        nargs="?",
        default="tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json",
    )
    p.set_defaults(func=_cmd_grammar_integration_validate)

    p = sub.add_parser(
        "pixel-conclude",
        help="Deterministic image conclusion: k-means, mean-shift, matrix, verdict",
    )
    p.add_argument("--image", required=True)
    p.add_argument("--kind", required=True, choices=["gui", "ui", "world", "art"])
    p.add_argument("--out", default=None)
    p.set_defaults(func=_cmd_pixel_conclude)

    p = sub.add_parser(
        "ref-lookup",
        help="Forward to src/dev/ref_books_index/lookup.py (book index)",
    )
    p.add_argument(
        "argv",
        nargs=argparse.REMAINDER,
        help="Args passed through (e.g. --check, --brief path, --open NAV)",
    )
    p.set_defaults(func=_cmd_ref_lookup)

    return parser


def _cmd_commands(args: argparse.Namespace) -> int:
    from .cli_catalog import catalog, render_text

    rows = catalog(
        build_parser(),
        grep=args.grep,
        group=args.group,
    )
    if args.json:
        print(json.dumps(rows, indent=2))
    else:
        print(render_text(rows))
    return 0


def _cmd_help(args: argparse.Namespace) -> int:
    from .cli_catalog import subparser_map

    parser = build_parser()
    if not args.name:
        parser.print_help()
        return 0
    sp = subparser_map(parser).get(args.name)
    if sp is None:
        print(json.dumps({"error": f"unknown command: {args.name}", "hint": "run `commands --grep <text>`"}))
        return 2
    sp.print_help()
    return 0


def _ensure_utf8_stdio() -> None:
    """Symbolic output (arrows, ●◐○) must not crash on a cp1252 pipe / redirect."""
    for stream in (sys.stdout, sys.stderr):
        enc = (getattr(stream, "encoding", None) or "").lower().replace("-", "")
        if enc == "utf8" or not hasattr(stream, "reconfigure"):
            continue
        try:
            if stream.isatty():
                stream.reconfigure(errors="replace")
            else:
                stream.reconfigure(encoding="utf-8", errors="replace")
        except (OSError, ValueError):
            pass


def main(argv: list[str] | None = None) -> int:
    _ensure_utf8_stdio()
    # ref-lookup must forward flags like `--check` unchanged; argparse would reject them.
    raw = list(sys.argv[1:] if argv is None else argv)
    if raw and raw[0] == "ref-lookup":
        try:
            return _cmd_ref_lookup(argparse.Namespace(argv=raw[1:]))
        except Exception as exc:  # noqa: BLE001 — CLI surface
            print(json.dumps({"error": str(exc)}), file=sys.stderr)
            return 2
    args = build_parser().parse_args(argv)
    try:
        return args.func(args)
    except Exception as exc:  # noqa: BLE001 — CLI surface
        print(json.dumps({"error": str(exc)}), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
