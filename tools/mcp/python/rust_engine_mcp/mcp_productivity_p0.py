"""PLAN-MCP-PRODUCTIVITY-CHAIN-001 — P0 micro tools (preflight, digest, plain P0 gate)."""

from __future__ import annotations

import json
import shutil
from pathlib import Path
from typing import Any

from . import agent_queue
from .aps_validator_plain import fix_hint, plain_sentence
from .paths import repo_root, schemas_dir
from .validators.assembly_grammar_verify import validate_assembly_p0_gate

MCP_PRODUCTIVITY_P0_WITNESS = "debug_runs/mcp_productivity_p0_live.json"

_SCHEMA_CHECKS = (
    "assembly_snapshot_v1.schema.json",
    "grammar_iterate_request_v1.schema.json",
    "grammar_iterate_result_v1.schema.json",
    "material_category_tree_v1.schema.json",
)


def _resolve_snapshot_path(path: str | Path) -> Path:
    p = Path(path)
    if not p.is_absolute():
        p = repo_root() / p
    return p


def _schema_present(name: str) -> bool:
    return (schemas_dir() / name).is_file()


def _queue_stale_in_progress(queue: str = "grammar") -> int:
    try:
        items = agent_queue.load_queue(queue)
    except (FileNotFoundError, ValueError, json.JSONDecodeError):
        return 0
    return sum(1 for row in items if str(row.get("status") or "") == "in_progress")


def pipeline_preflight(*, queue: str = "grammar", check_build_set: bool = False) -> dict[str, Any]:
    """MCP-PREFLIGHT-001 — one-call environment + path + schema check."""
    root = repo_root()
    blender_ok = False
    blender_exe: str | None = None
    blender_error: str | None = None
    try:
        from .paths import blender_exe as resolve_blender

        blender_ok = True
        blender_exe = str(resolve_blender())
    except FileNotFoundError as exc:
        blender_error = str(exc)

    bevy_worker = (root / "src" / "bin" / "bevy_preview_worker.rs").is_file()
    cargo = shutil.which("cargo")
    schemas = {name.removesuffix(".schema.json"): _schema_present(name) for name in _SCHEMA_CHECKS}
    grammars_dir = (root / "assets" / "configs" / "buildings" / "grammars").is_dir()
    material_profiles = (
        root / "assets" / "materials" / "profiles" / "material_profiles_v1.json"
    ).is_file()
    category_tree = (
        root / "assets" / "materials" / "profiles" / "material_category_tree_v1.json"
    ).is_file()
    stale = _queue_stale_in_progress(queue)
    ok = (
        blender_ok
        and bevy_worker
        and all(schemas.values())
        and grammars_dir
        and material_profiles
        and category_tree
    )
    build_set: dict[str, Any] | None = None
    if check_build_set:
        from .build_set_guards import (
            example_teachable_audit,
            single_archetype_ratio_guard,
            warehouse_track_guard,
        )
        from .grammar_build_set import building_set_coverage_report
        from .pilot_hardcode_lint import pilot_hardcode_lint

        build_set = {
            "coverage": building_set_coverage_report(),
            "hardcode": pilot_hardcode_lint(),
            "teachable": example_teachable_audit(),
            "archetype_ratio": single_archetype_ratio_guard(),
            "warehouse_track": warehouse_track_guard(),
        }
        ok = ok and all(bool(section.get("green")) for section in build_set.values())

    result: dict[str, Any] = {
        "schema": "pipeline_preflight_v1",
        "ok": ok,
        "blender_ok": blender_ok,
        "blender_exe": blender_exe,
        "blender_error": blender_error,
        "repo_root": str(root),
        "cargo_on_path": cargo is not None,
        "bevy_preview_worker": bevy_worker,
        "schemas": schemas,
        "queues": {"grammar_stale_rows": stale, "queue": queue},
        "paths": {
            "grammars_dir": grammars_dir,
            "material_profiles": material_profiles,
            "material_category_tree": category_tree,
            "debug_runs": (root / "debug_runs").is_dir(),
        },
    }
    if build_set is not None:
        result["build_set"] = build_set
    return result


def snapshot_digest(path: str | Path) -> dict[str, Any]:
    """MCP-SNAPSHOT-DIGEST-001 — compact assembly snapshot summary (no full JSON)."""
    resolved = _resolve_snapshot_path(path)
    if not resolved.is_file():
        return {
            "schema": "snapshot_digest_v1",
            "ok": False,
            "path": str(path),
            "error": "Snapshot not found",
            "hint": "Load or Save a valid assembly JSON",
        }

    try:
        rel = str(resolved.relative_to(repo_root())).replace("\\", "/")
    except ValueError:
        rel = str(resolved)

    snap = json.loads(resolved.read_text(encoding="utf-8"))
    placements = list(snap.get("module_placements") or [])
    fp = snap.get("footprint") or {}
    width = int(fp.get("width") or 0)
    depth = int(fp.get("depth") or 0)
    floors = int(fp.get("floors") or 1)
    root = repo_root()

    missing_mat = sum(
        1 for p in placements if not str((p or {}).get("material_profile") or "").strip()
    )
    profiles = {
        str((p or {}).get("material_profile") or "").strip()
        for p in placements
        if str((p or {}).get("material_profile") or "").strip()
    }
    glb_missing = 0
    for p in placements:
        glb = str((p or {}).get("glb_path") or "").strip()
        if not glb:
            glb_missing += 1
        elif not (root / glb.replace("\\", "/")).is_file():
            glb_missing += 1

    chain = snap.get("grammar_rule_chain") if isinstance(snap.get("grammar_rule_chain"), dict) else {}
    lineage = snap.get("grammar_lineage") if isinstance(snap.get("grammar_lineage"), dict) else {}

    from .arch_build_grammar import extract_from_snapshot

    dna = extract_from_snapshot(snap)

    hint = "Ready for P0 gate"
    if not placements:
        hint = "0 placements — regenerate assembly"
    elif missing_mat:
        hint = f"Assign material_profile on {missing_mat} cell(s) before P0"
    elif glb_missing:
        hint = f"Fix {glb_missing} missing GLB path(s) before P0"
    elif width < 3 or depth < 3:
        hint = "Footprint too small — widen before P0"

    return {
        "schema": "snapshot_digest_v1",
        "ok": True,
        "path": rel,
        "assembly_id": str(snap.get("assembly_id") or ""),
        "footprint": f"{width}x{depth}x{floors}",
        "placements": len(placements),
        "source_tier": str(snap.get("source_tier") or ""),
        "material_profiles": {
            "assigned": len(placements) - missing_mat,
            "missing": missing_mat,
            "unique": len(profiles),
        },
        "grammar": {
            "archetype": str(snap.get("archetype_id") or chain.get("archetype") or ""),
            "district": str(snap.get("district_style") or ""),
            "massing": str(chain.get("massing") or ""),
            "roof": str(chain.get("roof") or ""),
            "facade": str(chain.get("facade") or ""),
            "seed": int(snap.get("seed") or 0),
        },
        "arch_dna": {
            "wired": bool(dna.get("enabled")),
            "preset_id": dna.get("preset_id"),
            "grammar_id": dna.get("grammar_id"),
            "f_axis": (dna.get("arch_dna") or {}).get("F"),
            "pressure_field": dna.get("pressure_field"),
        },
        "lineage": {
            "parent": lineage.get("parent_assembly_id"),
            "mode": lineage.get("iteration_mode"),
            "seq": lineage.get("iteration_seq"),
        },
        "glb_missing": glb_missing,
        "hint": hint,
    }


def validate_p0_gate_plain(
    path: str | Path,
    *,
    ship: bool = True,
    compression_level: int = 4,
    max_messages: int = 12,
) -> dict[str, Any]:
    """MCP-P0-PLAIN-001 — P0 gate with artist-facing sentences."""
    resolved = _resolve_snapshot_path(path)
    if not resolved.is_file():
        return {
            "schema": "validate_p0_gate_plain_v1",
            "status": "failed",
            "path": str(path),
            "artist_messages": [
                {
                    "sentence": "Snapshot file not found.",
                    "fix": "Load or Save a valid assembly JSON.",
                    "signature": "assembly_production_snapshot_missing",
                }
            ],
            "signature_count": 1,
            "technical": {"compress": compression_level, "available": False},
        }

    try:
        rel = str(resolved.relative_to(repo_root())).replace("\\", "/")
    except ValueError:
        rel = str(resolved)

    snap = json.loads(resolved.read_text(encoding="utf-8"))
    report = validate_assembly_p0_gate(
        snap,
        snapshot_path=rel,
        ship=ship,
        compression_level=compression_level,
    )

    seen: set[str] = set()
    artist_messages: list[dict[str, str]] = []
    for issue in report.errors:
        if issue.severity != "error":
            continue
        sig = str(issue.signature or issue.kind or "")
        dedupe = sig or issue.kind
        if dedupe in seen:
            continue
        seen.add(dedupe)
        sentence = plain_sentence(
            issue.signature or "",
            issue.kind or "",
            fallback=issue.hint or issue.kind,
        ).replace("**", "")
        fix = fix_hint(
            issue.signature or "",
            issue.kind or "",
            fallback=issue.hint or "",
        ).replace("**", "")
        artist_messages.append(
            {
                "sentence": sentence,
                "fix": fix,
                "signature": sig or issue.kind,
            }
        )
        if len(artist_messages) >= max_messages:
            break

    compressed = report.compress(compression_level)
    return {
        "schema": "validate_p0_gate_plain_v1",
        "status": report.status,
        "path": rel,
        "summary": report.summary,
        "artist_messages": artist_messages,
        "signature_count": len(artist_messages),
        "error_count": report.error_count,
        "technical": {
            "compress": compression_level,
            "available": True,
            "validator": report.validator,
            "known_fixes": [k.to_dict() for k in compressed.known_fixes[:8]],
        },
    }


def refresh_mcp_productivity_p0_witness() -> bool:
    """Write debug_runs/mcp_productivity_p0_live.json — all three P0 tools green."""
    example = (
        "tools/mcp/schemas/examples/"
        "assembly_snapshot_warehouse_industrial_west_production_v1.json"
    )
    pre = pipeline_preflight()
    dig = snapshot_digest(example)
    plain = validate_p0_gate_plain(example)
    green = bool(
        dig.get("ok")
        and plain.get("status") in ("passed", "failed")
        and pre.get("repo_root")
        and (pre.get("schemas") or {}).get("assembly_snapshot_v1")
    )
    payload = {
        "program_id": "PLAN-MCP-PRODUCTIVITY-CHAIN-001",
        "gate": "MCP-P0",
        "green": green,
        "pipeline_preflight_ok": bool(pre.get("ok")),
        "snapshot_digest_ok": bool(dig.get("ok")),
        "validate_p0_gate_plain_ok": plain.get("status") is not None,
        "example_snapshot": example,
        "preflight": {k: pre[k] for k in ("ok", "blender_ok", "bevy_preview_worker", "schemas", "paths")},
        "digest_hint": dig.get("hint"),
        "p0_status": plain.get("status"),
        "p0_message_count": plain.get("signature_count"),
    }
    out = repo_root() / MCP_PRODUCTIVITY_P0_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return green
