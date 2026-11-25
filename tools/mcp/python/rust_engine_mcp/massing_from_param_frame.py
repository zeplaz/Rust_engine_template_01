"""ParamFrame massing_intent → massing_preview.json (no Blender).

PLAN-REF-BOOKS-UPGRADE-v3 S2 — Studio DCC path; presentation-only (DR-VOXEL-OCCUPATION).
Dual-cite: PCG-06 · RND-09 · LGT-04 + geometry_visual_effects.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .massing_from_pilot import build_massing_preview_from_pilot
from .schemas import validate_massing_preview, validate_param_frame

def _origin_from_intent(intent: dict[str, Any]) -> tuple[int, int]:
    hints = intent.get("footprint_hint_tiles")
    if isinstance(hints, list) and hints:
        first = hints[0]
        if isinstance(first, (list, tuple)) and len(first) >= 2:
            return int(first[0]), int(first[1])
    return 0, 0


def _grammar_max_floors(frame: dict[str, Any]) -> int | None:
    for step in frame.get("grammar_chain") or []:
        if not isinstance(step, dict):
            continue
        if step.get("step") == "massing" or step.get("rule_key") == "massing":
            params = step.get("params")
            if isinstance(params, dict) and "max_floors" in params:
                return int(params["max_floors"])
    return None


def _clamp_column_floors(
    columns: list[dict[str, Any]],
    *,
    grammar_max: int | None,
    target_min: int | None,
    target_max: int | None,
) -> list[dict[str, Any]]:
    lo = 1
    hi = 4
    if target_min is not None:
        lo = max(1, min(4, int(target_min)))
    if target_max is not None:
        hi = max(lo, min(4, int(target_max)))
    if grammar_max is not None:
        cap = max(1, min(4, int(grammar_max)))
        hi = min(hi, cap)
    out: list[dict[str, Any]] = []
    for col in columns:
        row = dict(col)
        try:
            f = int(row.get("floors", 1))
        except (TypeError, ValueError):
            f = 1
        row["floors"] = min(hi, max(lo, f))
        out.append(row)
    return out




def build_massing_preview_from_param_frame(frame: dict[str, Any]) -> dict[str, Any]:
    """Resolve massing_preview from a validated ParamFrame (pilot + intent; no bpy)."""
    validate_param_frame(frame)
    intent = frame["massing_intent"]
    if not isinstance(intent, dict):
        raise ValueError("massing_intent must be an object")
    pilot_id = str(intent["pilot_id"])
    seed = max(0, int(frame.get("seed", 0)))
    origin = _origin_from_intent(intent)

    payload = build_massing_preview_from_pilot(pilot_id, seed=seed, origin=origin)

    payload["param_frame_id"] = str(frame["frame_id"])
    if frame.get("title"):
        payload["title"] = f"massing_preview:{frame['frame_id']}"
    material = intent.get("material_strategy")
    if material:
        payload["material_strategy"] = str(material)
    district = intent.get("district_style_id")
    if district and not payload.get("arch_build_grammar_preset_id"):
        payload["arch_build_grammar_preset_id"] = str(district)

    grammar_max = _grammar_max_floors(frame)
    tmin = intent.get("target_floors_min")
    tmax = intent.get("target_floors_max")
    payload["massing_columns"] = _clamp_column_floors(
        list(payload.get("massing_columns") or []),
        grammar_max=grammar_max,
        target_min=int(tmin) if tmin is not None else None,
        target_max=int(tmax) if tmax is not None else None,
    )

    payload["note"] = (
        "VOXEL-MASSING-PRES-001 — ParamFrame→preview (S2); never commit to occupation"
    )
    validate_massing_preview(payload)
    return payload


def export_massing_preview_from_param_frame(
    massing_path: Path,
    frame_path: Path,
    *,
    frame: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Load ParamFrame JSON, build massing_preview, validate, write."""
    frame_path = Path(frame_path)
    if frame is None:
        if not frame_path.is_file():
            raise FileNotFoundError(frame_path)
        frame = json.loads(frame_path.read_text(encoding="utf-8"))
    payload = build_massing_preview_from_param_frame(frame)
    massing_path = Path(massing_path)
    massing_path.parent.mkdir(parents=True, exist_ok=True)
    massing_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    return payload


def load_param_frame(path: Path) -> dict[str, Any]:
    data = json.loads(Path(path).read_text(encoding="utf-8"))
    validate_param_frame(data)
    return data