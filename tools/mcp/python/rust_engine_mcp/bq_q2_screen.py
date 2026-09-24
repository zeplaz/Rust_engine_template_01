"""BQ-Q2-SCREEN-001 — screenshot QC lane over honest SILH assemblies.

Renders N seeded assemblies per style pack via bevy_preview_worker (trimesh
fallback), attaches PNG paths to the witness, and seeds rubric rows for
"reads as a real building". Machine green never claims a pixel pass without a
usable screenshot path — operator eyes remain BQ-Q3.
"""

from __future__ import annotations

import json
import time
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from rust_engine_mcp import assembly, assembly_preview, building_quality_qc
from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness
from rust_engine_mcp.paths import repo_root

TASK_ID = "BQ-Q2-SCREEN-001"
GATE = "BQ-Q2-SCREEN-001"
WITNESS_REL = "debug_runs/bq_q2_screen_001_live.json"
RUBRIC_ROWS_REL = "debug_runs/bq_q2_screen_rubric_rows.json"
SCREEN_DIR_REL = "debug_runs/bq_q2_screens"
BQ_A2_WITNESS = building_quality_qc.WITNESS_REL
NEXT_RESIDUAL = "BUILDING-LOOK-V2-QC"
PLAN_REF = "src/dev/design_building_visual_concept_v2.md"
RUBRIC_REF = "src/dev/design_building_visual_concept_v2.md#look-gates"
RUBRIC_CRITERION = "reads as a real building"

# Mirror BQ_SILH_SWEEP in assembly_quality_gate.rs — same packs/seeds as honest SILH.
SCREEN_SWEEP: tuple[dict[str, Any], ...] = (
    {"style_pack_id": "style_victorian", "seed": 1, "width": 4, "depth": 2, "floors": 2},
    {"style_pack_id": "style_victorian", "seed": 7, "width": 4, "depth": 2, "floors": 3},
    {"style_pack_id": "style_victorian", "seed": 11, "width": 3, "depth": 3, "floors": 1},
    {"style_pack_id": "style_victorian", "seed": 13, "width": 5, "depth": 3, "floors": 2},
    {"style_pack_id": "style_industrial_west", "seed": 1, "width": 4, "depth": 2, "floors": 2},
    {"style_pack_id": "style_industrial_west", "seed": 5, "width": 6, "depth": 3, "floors": 2},
    {"style_pack_id": "style_industrial_west", "seed": 9, "width": 4, "depth": 2, "floors": 3},
    {"style_pack_id": "style_rural", "seed": 3, "width": 4, "depth": 2, "floors": 1},
    {"style_pack_id": "style_rural", "seed": 3, "width": 5, "depth": 3, "floors": 1},
)

MIN_PACKS = 3
MIN_SEEDS_PER_PACK = 2


def _silh_honest(bq: dict[str, Any] | None) -> dict[str, Any]:
    if not bq:
        return {
            "ok": False,
            "reason": f"missing {BQ_A2_WITNESS}",
            "silh_max_pct": None,
            "bq_silh_honest_001": False,
        }
    silh_max = bq.get("silh_max_pct")
    try:
        silh_max_f = float(silh_max) if silh_max is not None else None
    except (TypeError, ValueError):
        silh_max_f = None
    flag = bool(bq.get("bq_silh_honest_001") or bq.get("silh_honest"))
    green = bool(bq.get("green"))
    ok = green and flag and silh_max_f is not None and silh_max_f <= 100.0
    return {
        "ok": ok,
        "reason": "honest SILH" if ok else "SILH prerequisite not green",
        "silh_max_pct": silh_max_f,
        "bq_silh_honest_001": flag,
        "bq_green": green,
        "assembly_count": len(bq.get("assemblies") or []),
        "style_pack_count": bq.get("style_pack_count"),
    }


def _quality_row(assembly_id: str, bq: dict[str, Any] | None) -> dict[str, Any] | None:
    if not bq:
        return None
    for row in bq.get("assemblies") or []:
        if isinstance(row, dict) and row.get("assembly_id") == assembly_id:
            return row
    return None


def _screen_png_path(root: Path, assembly_id: str) -> Path:
    return root / SCREEN_DIR_REL / f"{assembly_id}.png"


_SLOT_COLORS: dict[str, tuple[int, int, int]] = {
    "wall_1u": (180, 140, 100),
    "wall_2u": (160, 120, 90),
    "door_default": (90, 70, 50),
    "door_wide": (80, 60, 45),
    "window_1u": (120, 170, 210),
    "window_2u": (100, 150, 200),
    "roof_default": (110, 70, 60),
    "roof_flat": (100, 65, 55),
    "corner_outer": (150, 130, 110),
}


def render_assembly_schematic_png(
    snapshot: dict[str, Any],
    out_png: Path,
    *,
    size: int = 512,
) -> bool:
    """Deterministic top-down schematic when Bevy/trimesh cannot produce a usable PNG.

    Still an assembly screen (placements on grid) — not a photoreal claim. Mode tagged
    ``schematic_pil`` on the screen row so WIT-HON stays honest.
    """
    try:
        from PIL import Image, ImageDraw, ImageFont
    except ImportError:
        return False

    placements = [p for p in (snapshot.get("module_placements") or []) if isinstance(p, dict)]
    if not placements:
        return False

    xs = [int(p.get("grid_x") or 0) for p in placements]
    ys = [int(p.get("grid_y") or 0) for p in placements]
    min_x, max_x = min(xs), max(xs)
    min_y, max_y = min(ys), max(ys)
    span_x = max(1, max_x - min_x + 1)
    span_y = max(1, max_y - min_y + 1)
    margin = 48
    cell = max(8, min((size - 2 * margin) // span_x, (size - 2 * margin) // span_y))

    img = Image.new("RGB", (size, size), (32, 36, 42))
    draw = ImageDraw.Draw(img)
    # Ground plate
    gx0 = margin
    gy0 = margin
    gx1 = margin + span_x * cell
    gy1 = margin + span_y * cell
    draw.rectangle([gx0 - 4, gy0 - 4, gx1 + 4, gy1 + 4], fill=(48, 52, 58), outline=(70, 76, 84))

    for p in placements:
        gx = int(p.get("grid_x") or 0) - min_x
        gy = int(p.get("grid_y") or 0) - min_y
        floor = int(p.get("floor") or 0)
        slot = str(p.get("slot_key") or "")
        base = _SLOT_COLORS.get(slot, (140, 140, 150))
        # Lift upper floors toward lighter so massing reads
        lift = min(40, floor * 18)
        color = (min(255, base[0] + lift), min(255, base[1] + lift), min(255, base[2] + lift))
        x0 = margin + gx * cell + 1
        y0 = margin + gy * cell + 1
        x1 = margin + (gx + 1) * cell - 1
        y1 = margin + (gy + 1) * cell - 1
        draw.rectangle([x0, y0, x1, y1], fill=color, outline=(20, 22, 26))

    aid = str(snapshot.get("assembly_id") or "assembly")
    pack = str(snapshot.get("style_pack_id") or "")
    try:
        font = ImageFont.load_default()
    except OSError:
        font = None
    label = f"{aid} · {pack} · schematic"
    draw.text((12, size - 28), label[:72], fill=(220, 220, 220), font=font)
    draw.text((12, 10), "BQ-Q2 screen (schematic_pil)", fill=(180, 190, 200), font=font)

    out_png.parent.mkdir(parents=True, exist_ok=True)
    img.save(out_png, format="PNG")
    return assembly_preview.png_preview_usable(out_png)


def render_assembly_screen(
    row: dict[str, Any],
    *,
    try_bevy: bool = True,
    repo: Path | None = None,
) -> dict[str, Any]:
    """Generate snapshot + preview PNG for one SILH sweep row."""
    root = repo or repo_root()
    style_pack_id = str(row["style_pack_id"])
    seed = int(row["seed"])
    width = int(row["width"])
    depth = int(row["depth"])
    floors = int(row["floors"])

    snap = assembly.generate_assembly_snapshot(
        style_pack_id=style_pack_id,
        width=width,
        depth=depth,
        floors=floors,
        seed=seed,
        source_tier="production",
        write=True,
    )
    assembly_id = str(snap["assembly_id"])
    snap_rel = str(snap.get("written_path") or "")
    snap_path = root / snap_rel if snap_rel else assembly.default_snapshot_path(assembly_id)
    png_path = _screen_png_path(root, assembly_id)
    png_path.parent.mkdir(parents=True, exist_ok=True)
    # When Bevy is skipped, drop any prior worker PNG so we do not mis-label
    # leftover bevy_worker frames as browser_threejs / skip schematic.
    if not try_bevy and png_path.is_file():
        try:
            png_path.unlink()
        except OSError:
            pass

    preview = assembly_preview.preview_assembly(
        snap_path,
        out_png=png_path,
        open_browser=False,
        try_bevy=try_bevy,
    )
    mode = str(preview.get("mode") or "")
    bevy_status = (preview.get("bevy_status") or {}).get("status")
    png_rel = ""
    usable = assembly_preview.png_preview_usable(png_path)
    if usable:
        png_rel = f"{SCREEN_DIR_REL}/{assembly_id}.png"
    else:
        # Bevy often writes an all-black frame; trimesh needs pyglet. Fall back to schematic.
        placements, _missing = assembly_preview.collect_preview_placements(snap)
        if placements and assembly_preview.try_render_thumbnail_png(
            placements, png_path, variant_state=str(preview.get("preview_variant_state") or "clean")
        ):
            usable = assembly_preview.png_preview_usable(png_path)
            if usable:
                mode = "trimesh_thumbnail"
                png_rel = f"{SCREEN_DIR_REL}/{assembly_id}.png"
        if not usable and render_assembly_schematic_png(snap, png_path):
            usable = True
            mode = "schematic_pil"
            png_rel = f"{SCREEN_DIR_REL}/{assembly_id}.png"
            bevy_status = bevy_status or "blank_rejected"

    return {
        "assembly_id": assembly_id,
        "style_pack_id": style_pack_id,
        "seed": seed,
        "width": width,
        "depth": depth,
        "floors": floors,
        "snapshot_path": snap_rel or str(snap_path.relative_to(root)).replace("\\", "/"),
        "preview_png": png_rel if usable else "",
        "preview_mode": mode,
        "preview_usable": usable,
        "modules_loaded": preview.get("modules_loaded"),
        "missing_glb": preview.get("missing_glb") or [],
        "bevy_status": bevy_status,
        "elapsed_ms": preview.get("elapsed_ms"),
    }


def _rubric_row(screen: dict[str, Any], quality: dict[str, Any] | None) -> dict[str, Any]:
    has_png = bool(screen.get("preview_png"))
    # Machine never passes the pixel criterion — only attaches evidence for operator.
    verdict = "pending_operator" if has_png else "blocked_no_screenshot"
    return {
        "seed_key": (
            f"{screen['style_pack_id']}:{screen['width']}x{screen['depth']}x{screen['floors']}"
            f":s{screen['seed']}"
        ),
        "assembly_id": screen["assembly_id"],
        "style_pack_id": screen["style_pack_id"],
        "seed": screen["seed"],
        "criterion": RUBRIC_CRITERION,
        "verdict": verdict,
        "screen_pass": False,  # exit rule: never pass without operator + screenshot
        "preview_png": screen.get("preview_png") or "",
        "silhouette_continuity_pct": (
            float(quality["silhouette_continuity_pct"])
            if quality and quality.get("silhouette_continuity_pct") is not None
            else None
        ),
        "passes_gate_a2": bool(quality.get("passes_gate")) if quality else None,
        "note": (
            "Screenshot attached — operator must judge 'reads as a real building'."
            if has_png
            else "No usable preview PNG — cannot pass screen gate."
        ),
        "recorded_at": datetime.now(timezone.utc).isoformat(),
        "rubric_ref": RUBRIC_REF,
        "recorded_by": "coder-mcp-machine",
    }


def write_rubric_rows(rows: list[dict[str, Any]], *, repo: Path | None = None) -> str:
    root = repo or repo_root()
    path = root / RUBRIC_ROWS_REL
    path.parent.mkdir(parents=True, exist_ok=True)
    body = {
        "version": 1,
        "task_id": TASK_ID,
        "criterion": RUBRIC_CRITERION,
        "rows": rows,
    }
    path.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return RUBRIC_ROWS_REL


def _coverage(screens: list[dict[str, Any]]) -> dict[str, Any]:
    by_pack: dict[str, list[str]] = defaultdict(list)
    for s in screens:
        if s.get("preview_png"):
            by_pack[str(s["style_pack_id"])].append(str(s["assembly_id"]))
    packs_ok = {
        pack: ids for pack, ids in by_pack.items() if len(ids) >= MIN_SEEDS_PER_PACK
    }
    return {
        "packs_with_preview": {p: len(ids) for p, ids in by_pack.items()},
        "packs_meeting_min": sorted(packs_ok.keys()),
        "pack_count_meeting_min": len(packs_ok),
        "min_packs_required": MIN_PACKS,
        "min_seeds_per_pack": MIN_SEEDS_PER_PACK,
        "coverage_ok": len(packs_ok) >= MIN_PACKS,
    }


def run_q2_screen(
    *,
    try_bevy: bool = True,
    repo: Path | None = None,
    sweep: tuple[dict[str, Any], ...] | None = None,
) -> dict[str, Any]:
    """Render screen pass + evaluate exit predicates (pre-WIT-HON write)."""
    root = repo or repo_root()
    bq = building_quality_qc.load_building_quality_witness(repo=root)
    silh = _silh_honest(bq)
    rows = sweep or SCREEN_SWEEP

    screens: list[dict[str, Any]] = []
    for row in rows:
        screen = render_assembly_screen(row, try_bevy=try_bevy, repo=root)
        quality = _quality_row(str(screen["assembly_id"]), bq)
        if quality:
            screen["silhouette_continuity_pct"] = quality.get("silhouette_continuity_pct")
            screen["passes_gate_a2"] = quality.get("passes_gate")
            screen["overall_score"] = quality.get("overall_score")
        screens.append(screen)

    rubric_rows = [_rubric_row(s, _quality_row(str(s["assembly_id"]), bq)) for s in screens]
    write_rubric_rows(rubric_rows, repo=root)
    cov = _coverage(screens)

    # Exit: no pass without screenshot path
    illicit_pass = [
        r
        for r in rubric_rows
        if (r.get("screen_pass") or r.get("verdict") == "pass") and not r.get("preview_png")
    ]
    no_pass_without_png = len(illicit_pass) == 0
    all_with_png_pending = all(
        (not r.get("preview_png")) or r.get("verdict") == "pending_operator"
        for r in rubric_rows
    )

    green = bool(
        silh["ok"]
        and cov["coverage_ok"]
        and no_pass_without_png
        and all_with_png_pending
        and all(bool(s.get("preview_png")) for s in screens)
    )

    return {
        "task_id": TASK_ID,
        "gate": GATE,
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "honest_gate": "honest_gate",
        "proceed_ship": False,
        "art_quality": "screen_pass_pending_operator",
        "plan_ref": PLAN_REF,
        "silh_prerequisite": silh,
        "building_quality_witness": BQ_A2_WITNESS,
        "coverage": cov,
        "screens": screens,
        "screen_count": len(screens),
        "screens_with_png": sum(1 for s in screens if s.get("preview_png")),
        "rubric_rows_path": RUBRIC_ROWS_REL,
        "rubric_row_count": len(rubric_rows),
        "rubric_criterion": RUBRIC_CRITERION,
        "no_pass_without_screenshot": no_pass_without_png,
        "illicit_pass_count": len(illicit_pass),
        "next_residual": NEXT_RESIDUAL,
        "notes": (
            "Machine screen lane green — PNGs attached (Bevy worker preferred after "
            f"BQ-BEVY-BLANK-FRAME-FIX-001; schematic_pil only if Bevy blank). Pixel "
            f"'{RUBRIC_CRITERION}' awaits operator → {NEXT_RESIDUAL}."
            if green
            else "Screen lane incomplete — fix missing PNGs / SILH prerequisite before operator Q3."
        ),
    }


def format_q2_strip_text(
    assembly_id: str | None = None,
    *,
    repo: Path | None = None,
) -> tuple[str, bool | None]:
    """APS strip helper — screenshot path + pending operator criterion."""
    root = repo or repo_root()
    path = root / WITNESS_REL
    if not path.is_file():
        return (
            "Q2 screen: no witness — run `bq-q2-screen` (coder-mcp).",
            None,
        )
    try:
        body = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return ("Q2 screen: witness unreadable.", False)
    screens = body.get("screens") or []
    if assembly_id:
        for s in screens:
            if isinstance(s, dict) and s.get("assembly_id") == assembly_id:
                png = s.get("preview_png") or ""
                if not png:
                    return (f"Q2 {assembly_id}: no screenshot — cannot pass.", False)
                return (
                    f"Q2 {assembly_id}: {png} · {RUBRIC_CRITERION}: pending_operator",
                    True,
                )
        return (f"Q2 {assembly_id}: not in screen witness.", None)
    n = int(body.get("screens_with_png") or 0)
    total = int(body.get("screen_count") or 0)
    ok = bool(body.get("green"))
    return (
        f"Q2 screen: {'pass' if ok else 'fail'} · {n}/{total} PNG · "
        f"{RUBRIC_CRITERION} → operator ({NEXT_RESIDUAL})",
        ok if total else None,
    )


def write_bq_q2_screen_witness(
    *,
    try_bevy: bool = True,
    repo: Path | None = None,
) -> dict[str, Any]:
    root = repo or repo_root()
    body = run_q2_screen(try_bevy=try_bevy, repo=root)
    body["_agent_meta_extra"] = {
        "track": "PLAN-MCP-APS-TOOLING-FINISH-001",
        "task_id": TASK_ID,
        "proceed_ship": False,
        "art_quality": "screen_pass_pending_operator",
        "agent": "coder-mcp",
    }
    out = write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="bq_q2_screen_001_live_v1",
        profile="BQ_Q2_SCREEN",
        source_system="bq_q2_screen",
        ritual=f"BLANG:WIT-HON→Q✓ {TASK_ID}" if body.get("green") else None,
        exit_predicate_must=[
            {"field": "coverage.coverage_ok", "equals": True},
            {"field": "no_pass_without_screenshot", "equals": True},
            {"field": "silh_prerequisite.ok", "equals": True},
        ],
        repo=root,
    )
    # Preserve agent meta fields expected by OPS spine on top-level _agent_meta.
    meta = dict(out.get("_agent_meta") or {})
    meta.update(body.pop("_agent_meta_extra", {}))
    meta["written_at_epoch_secs"] = int(time.time())
    if out.get("green"):
        meta["ritual"] = f"BLANG:WIT-HON→Q✓ {TASK_ID}"
    out["_agent_meta"] = meta
    (root / WITNESS_REL).write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
    out["written"] = WITNESS_REL
    return out
