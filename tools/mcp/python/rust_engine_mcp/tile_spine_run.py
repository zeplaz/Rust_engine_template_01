"""MCP-SPINE-CHAIN-001 — WRK→ATL one-call chain with per-step witness."""

from __future__ import annotations

import json
import os
import time
from pathlib import Path
from typing import Any

from .mcp_productivity_p0 import snapshot_digest, validate_p0_gate_plain
from .paths import repo_root
from .tile_pipeline import assembly_build_run, tile_atlas_pack, tile_batch_run
from .tile_promotion_honest import tile_promotion_honest_check as _tile_promotion_honest_check
from .validators import run_validator

TILE_SPINE_RUN_WITNESS = "debug_runs/tile_spine_run_001_live.json"

DEFAULT_STEPS: tuple[str, ...] = (
    "p0_gate",
    "snapshot_digest",
    "preview",
    "assembly_build",
    "tile_batch",
    "atlas_pack",
    "atlas_validate",
)

EXAMPLE_SNAPSHOT = (
    "tools/mcp/schemas/examples/assembly_snapshot_rowhouse_victorian_production_v1.json"
)
EXAMPLE_BATCH_ID = "tile_rowhouse_victorian_production_v1"


def _resolve_path(path: str | Path) -> Path:
    p = Path(path)
    if not p.is_absolute():
        p = repo_root() / p
    return p.resolve()


def _rel(path: Path) -> str:
    try:
        return str(path.relative_to(repo_root())).replace("\\", "/")
    except ValueError:
        return str(path)


def _tile_batch_path(batch_id: str, explicit: str | None) -> Path:
    if explicit:
        return _resolve_path(explicit)
    stem = batch_id.removeprefix("tile_")
    examples = repo_root() / "tools" / "mcp" / "schemas" / "examples"
    candidates = [
        examples / f"tile_batch_{batch_id}.json",
        examples / f"tile_batch_{stem}.json",
        examples / f"{batch_id}.json",
    ]
    for candidate in candidates:
        if candidate.is_file():
            return candidate
    return candidates[1]


def _staging_folder(batch_id: str) -> Path:
    return repo_root() / "assets" / "staging" / "tiles" / batch_id


def _step_result(
    step: str,
    *,
    ok: bool,
    started: float,
    witness_path: str | None = None,
    artist_message: str | None = None,
    detail: dict[str, Any] | None = None,
    warn: str | None = None,
) -> dict[str, Any]:
    row: dict[str, Any] = {
        "step": step,
        "ok": ok,
        "duration_ms": int((time.perf_counter() - started) * 1000),
        "witness_path": witness_path,
    }
    if artist_message:
        row["artist_message"] = artist_message
    if warn:
        row["warn"] = warn
    if detail:
        row["detail"] = detail
    return row


def _run_step(
    step: str,
    ctx: dict[str, Any],
) -> dict[str, Any]:
    started = time.perf_counter()
    snapshot = ctx["snapshot_path"]
    batch_id = ctx["batch_id"]
    batch_path = ctx["batch_path"]
    staging = ctx["staging"]
    ship = bool(ctx.get("ship"))

    if step == "p0_gate":
        plain = validate_p0_gate_plain(snapshot, ship=ship)
        ok = plain.get("status") == "passed"
        msg = None
        if not ok and plain.get("artist_messages"):
            msg = plain["artist_messages"][0].get("sentence")
        return _step_result("p0_gate", ok=ok, started=started, artist_message=msg, detail={"status": plain.get("status")})

    if step == "snapshot_digest":
        dig = snapshot_digest(snapshot)
        ok = bool(dig.get("ok"))
        return _step_result(
            "snapshot_digest",
            ok=ok,
            started=started,
            artist_message=None if ok else dig.get("error"),
            detail={"footprint": dig.get("footprint"), "placements": dig.get("placements")},
        )

    if step == "preview":
        if os.environ.get("PREVIEW_SKIP", "").strip() in ("1", "true", "TRUE"):
            return _step_result("preview", ok=True, started=started, warn="skipped PREVIEW_SKIP=1")
        from . import assembly_preview

        result = assembly_preview.preview_assembly(
            str(snapshot),
            open_browser=False,
            try_bevy=False,
        )
        ok = bool(result.get("ok"))
        msg = result.get("error") if not ok else None
        return _step_result("preview", ok=ok, started=started, artist_message=msg)

    if step == "assembly_build":
        result = assembly_build_run(snapshot)
        ok = bool(result.get("ok"))
        witness = "debug_runs/build_worker_001_live.json" if ok else None
        msg = result.get("error") if not ok else None
        return _step_result(
            "assembly_build",
            ok=ok,
            started=started,
            witness_path=witness,
            artist_message=msg,
            detail={"status": result.get("status"), "job_id": result.get("job_id")},
        )

    if step == "tile_batch":
        result = tile_batch_run(batch_path)
        ok = bool(result.get("ok"))
        msg = result.get("error") or result.get("status") if not ok else None
        return _step_result("tile_batch", ok=ok, started=started, artist_message=msg, detail={"status": result.get("status")})

    if step == "atlas_pack":
        if not staging.is_dir():
            return _step_result(
                "atlas_pack",
                ok=False,
                started=started,
                artist_message=f"Staging folder missing: {_rel(staging)}",
            )
        pack = tile_atlas_pack(staging, keyframe_rename=True)
        ok = bool(pack.get("ok"))
        msg = pack.get("error") if not ok else None
        return _step_result(
            "atlas_pack",
            ok=ok,
            started=started,
            artist_message=msg,
            detail={"atlas_path": pack.get("atlas_path")},
        )

    if step == "atlas_validate":
        report = run_validator("tile_batch", str(batch_path))
        honest = _tile_promotion_honest_check(
            batch_path=batch_path,
            staging=staging,
            ship=ship,
            honest_bake=bool(ctx.get("honest_bake")),
        )
        meta_report = None
        meta_path = staging / "atlas_meta.json"
        if meta_path.is_file():
            meta_report = run_validator("atlas_meta_v2", str(meta_path))
        ok = report.status == "passed" and honest.get("ok", True)
        if meta_report is not None and meta_report.status == "failed":
            ok = False
        msg = None
        if not ok:
            if report.status != "passed" and report.errors:
                msg = report.errors[0].hint or report.summary
            elif honest.get("artist_message"):
                msg = honest["artist_message"]
            elif meta_report and meta_report.errors:
                msg = meta_report.errors[0].hint
        warn = None
        if honest.get("warnings"):
            warn = "; ".join(honest["warnings"][:2])
        return _step_result(
            "atlas_validate",
            ok=ok,
            started=started,
            artist_message=msg,
            warn=warn,
            detail={
                "tile_batch_status": report.status,
                "atlas_meta_status": meta_report.status if meta_report else None,
            },
        )

    return _step_result(step, ok=False, started=started, artist_message=f"Unknown step: {step}")


def tile_spine_run(request: dict[str, Any] | str | Path) -> dict[str, Any]:
    """Chain WRK→ATL micro-tools — stop on first hard fail."""
    if isinstance(request, (str, Path)):
        req_path = _resolve_path(request)
        if not req_path.is_file():
            return {
                "schema": "tile_spine_run_result_v1",
                "ok": False,
                "error": f"Request not found: {req_path}",
                "steps": [],
            }
        req = json.loads(req_path.read_text(encoding="utf-8"))
    else:
        req = dict(request)

    if req.get("schema") and req.get("schema") != "tile_spine_run_request_v1":
        return {
            "schema": "tile_spine_run_result_v1",
            "ok": False,
            "error": f"Unexpected schema: {req.get('schema')}",
            "steps": [],
        }

    snapshot = _resolve_path(req["snapshot_path"])
    batch_id = str(req["batch_id"])
    batch_path = _tile_batch_path(batch_id, req.get("tile_batch_path"))
    steps = list(req.get("steps") or DEFAULT_STEPS)
    ship = bool(req.get("ship", False))
    write_witness = bool(req.get("write_witness", True))
    honest_bake = bool(req.get("honest_bake", True))

    ctx: dict[str, Any] = {
        "snapshot_path": snapshot,
        "batch_id": batch_id,
        "batch_path": batch_path,
        "staging": _staging_folder(batch_id),
        "ship": ship,
        "honest_bake": honest_bake,
        "parent_lineage_id": req.get("parent_lineage_id"),
    }

    step_rows: list[dict[str, Any]] = []
    stopped_at: str | None = None
    overall_ok = True

    for step in steps:
        row = _run_step(step, ctx)
        step_rows.append(row)
        if not row.get("ok"):
            overall_ok = False
            stopped_at = step
            break

    result: dict[str, Any] = {
        "schema": "tile_spine_run_result_v1",
        "ok": overall_ok,
        "stopped_at": stopped_at,
        "snapshot_path": _rel(snapshot),
        "batch_id": batch_id,
        "tile_batch_path": _rel(batch_path) if batch_path.is_file() else str(batch_path),
        "ship": ship,
        "honest_bake": honest_bake,
        "steps": step_rows,
    }
    if ctx.get("parent_lineage_id"):
        result["parent_lineage_id"] = ctx["parent_lineage_id"]

    if write_witness:
        witness_path = repo_root() / TILE_SPINE_RUN_WITNESS
        witness_path.parent.mkdir(parents=True, exist_ok=True)
        witness_path.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
        result["witness_path"] = TILE_SPINE_RUN_WITNESS

    return result


def refresh_tile_spine_run_witness() -> bool:
    """Lib witness — digest-only spine chain (no Blender)."""
    result = tile_spine_run(
        {
            "schema": "tile_spine_run_request_v1",
            "snapshot_path": EXAMPLE_SNAPSHOT,
            "batch_id": EXAMPLE_BATCH_ID,
            "steps": ["p0_gate", "snapshot_digest"],
            "ship": False,
            "write_witness": True,
            "honest_bake": True,
        }
    )
    green = bool(result.get("ok") and len(result.get("steps") or []) == 2)
    witness = repo_root() / TILE_SPINE_RUN_WITNESS
    if witness.is_file():
        body = json.loads(witness.read_text(encoding="utf-8"))
        body["gate_id"] = "MCP-SPINE-CHAIN-001"
        body["green"] = green
        witness.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return green
