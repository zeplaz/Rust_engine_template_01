"""VSS-T4-005 — write scenario trigger RON markers bound to EffectSpecs.

Assign is hard-gated on ``honest_gate == honest``. Never invent a parallel
fire_vfx / Bevy emit path — files only under ``assets/scenarios/triggers/``.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from rust_engine_mcp.effect_registry_browse import EffectBrowseEntry, get_effect_entry
from rust_engine_mcp.paths import repo_root

TRIGGERS_REL = "assets/scenarios/triggers"


class AssignBlockedError(ValueError):
    """Raised when assign is attempted without honest_gate=honest."""


def _cells_from_entry(entry: EffectBrowseEntry) -> list[dict[str, Any]]:
    cfg = entry.spawn_hook_config or {}
    if entry.spawn_hook == "world_xy" and isinstance(cfg.get("world_xy"), dict):
        wx = cfg["world_xy"]
        tile_x = int(wx.get("tile_x") or 0)
        tile_y = int(wx.get("tile_y") or 0)
        # Map tile → coarse chunk (4×4) for trigger sketch parity with demo_ignite.
        return [
            {
                "chunk_x": tile_x // 4,
                "chunk_y": tile_y // 4,
                "cell": (tile_y % 4) * 4 + (tile_x % 4),
                "spark": 0.35,
            }
        ]
    return [{"chunk_x": 0, "chunk_y": 0, "cell": 0, "spark": 0.35}]


def format_trigger_ron(
    *,
    marker_id: str,
    cells: list[dict[str, Any]],
    cause_id: str | None = None,
    when_tick_min: int | None = 0,
    source: str = "scenario_script",
) -> str:
    """Serialize TriggerSpecV1 matching ``demo_ignite.trigger.ron`` shape."""
    cause = cause_id or f"CAUSE-trigger-{marker_id}"
    tick = f"Some({when_tick_min})" if when_tick_min is not None else "None"
    cell_lines: list[str] = []
    for c in cells:
        cell_lines.append(
            "        ("
            f"chunk_x: {int(c.get('chunk_x', 0))}, "
            f"chunk_y: {int(c.get('chunk_y', 0))}, "
            f"cell: {int(c.get('cell', 0))}, "
            f"spark: {float(c.get('spark', 0.35))}"
            "),"
        )
    cells_block = "\n".join(cell_lines) if cell_lines else ""
    return (
        "(\n"
        "    schema_version: 1,\n"
        f'    id: "{marker_id}",\n'
        f"    when_tick_min: {tick},\n"
        f'    source: "{source}",\n'
        f'    cause_id: "{cause}",\n'
        "    parent_effect_id: None,\n"
        "    cells: [\n"
        f"{cells_block}\n"
        "    ],\n"
        ")\n"
    )


def can_assign(entry: EffectBrowseEntry | None) -> bool:
    return bool(entry and entry.honest_gate == "honest")


def assign_effect_to_marker(
    effect_id: str,
    marker_id: str | None = None,
    *,
    repo: Path | None = None,
    cells: list[dict[str, Any]] | None = None,
    force: bool = False,
    entry: EffectBrowseEntry | None = None,
) -> dict[str, Any]:
    """Write ``assets/scenarios/triggers/{marker_id}.trigger.ron`` when honest.

    ``force`` is test-only — production UI never passes it.
    """
    root = repo or repo_root()
    row = entry or get_effect_entry(effect_id, repo=root)
    if row is None:
        raise FileNotFoundError(f"effect_id not found: {effect_id}")
    if not force and not can_assign(row):
        raise AssignBlockedError(
            f"⊘ Assign blocked — preview honesty is {row.honest_gate}. "
            "Capture frames before binding to a scenario."
        )
    mid = (marker_id or row.effect_id).strip()
    if not mid:
        raise ValueError("marker_id empty")
    out_dir = root / TRIGGERS_REL
    out_dir.mkdir(parents=True, exist_ok=True)
    out_path = out_dir / f"{mid}.trigger.ron"
    cell_rows = cells if cells is not None else _cells_from_entry(row)
    text = format_trigger_ron(marker_id=mid, cells=cell_rows)
    out_path.write_text(text, encoding="utf-8")
    rel = str(out_path.relative_to(root).as_posix())
    return {
        "ok": True,
        "effect_id": row.effect_id,
        "marker_id": mid,
        "path": rel,
        "honest_gate": row.honest_gate,
        "display_name": row.display_name,
    }
