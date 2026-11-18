"""S1 — ParamFrame Studio DCC: browse grammars/modifiers/connections without bpy.

Refs: PCG-06 · PCG-08 · DES-04 + LEGACY pcg · shape_grammars (dual-cite).
"""

from __future__ import annotations

import json
from copy import deepcopy
from pathlib import Path
from typing import Any

from . import arch_build_grammar, building_grammar, building_quality_qc
from . import grammar_facility_brief, landscape_preset_browse, studio_task
from .paths import repo_root, schemas_dir
from .schemas import validate_param_frame

S1_DUAL_CITE: dict[str, list[str]] = {
    "new_rows": ["PCG-06", "PCG-08", "DES-04"],
    "legacy_keywords": ["pcg", "shape_grammars"],
}

STUDIO_MODIFIER_IDS: tuple[str, ...] = (
    "district_style",
    "facility_binding",
    "arch_dna_pressure",
    "massing_floor_cap",
    "material_strategy",
    "age_band",
)

GRAMMAR_STEP_ORDER: tuple[str, ...] = (
    "archetype",
    "massing",
    "roof",
    "facade",
    "detail",
    "age",
    "landscape",
    "district",
)


def default_example_path() -> Path:
    return schemas_dir() / "examples" / "param_frame_rail_warehouse_v1.json"


def load_param_frame(path: Path | str | None = None) -> dict[str, Any]:
    p = Path(path) if path else default_example_path()
    if not p.is_file():
        raise FileNotFoundError(p)
    data = json.loads(p.read_text(encoding="utf-8"))
    validate_param_frame(data)
    return data


def merge_ref_books_dual_cite(frame: dict[str, Any]) -> dict[str, Any]:
    """Ensure S1 orient rows are recorded on the frame (non-destructive merge)."""
    out = deepcopy(frame)
    cite = dict(out.get("ref_books_dual_cite") or {})
    new_rows = list(cite.get("new_rows") or [])
    legacy = list(cite.get("legacy_keywords") or [])
    for row in S1_DUAL_CITE["new_rows"]:
        if row not in new_rows:
            new_rows.append(row)
    for kw in S1_DUAL_CITE["legacy_keywords"]:
        if kw not in legacy:
            legacy.append(kw)
    cite["new_rows"] = new_rows
    cite["legacy_keywords"] = legacy
    out["ref_books_dual_cite"] = cite
    return out


def browse_grammar_catalog(*, repo: Path | None = None) -> list[dict[str, Any]]:
    """Disk-only grammar browse — no generate / no bpy."""
    _ = repo
    rows: list[dict[str, Any]] = []
    for archetype in building_grammar.list_archetype_ids():
        try:
            grammar = building_grammar.load_building_grammar_by_archetype(archetype)
            gid = str(grammar.get("grammar_id") or "")
        except (OSError, KeyError, ValueError, FileNotFoundError):
            gid = ""
        if gid:
            rows.append(
                {
                    "grammar_ref": building_grammar.GRAMMAR_RULES_VERSION,
                    "grammar_id": gid,
                    "step": "archetype",
                    "label": f"{archetype} ({gid})",
                    "archetype_id": archetype,
                }
            )
    for gid in grammar_facility_brief.list_grammar_ids():
        if any(r.get("grammar_id") == gid for r in rows):
            continue
        rows.append(
            {
                "grammar_ref": building_grammar.GRAMMAR_RULES_VERSION,
                "grammar_id": gid,
                "step": "massing",
                "label": gid,
            }
        )
    for pid in arch_build_grammar.list_preset_ids():
        rows.append(
            {
                "grammar_ref": "arch_build_grammar_v0",
                "grammar_id": pid,
                "step": "district",
                "rule_key": "pressure_field",
                "label": f"arch_dna:{pid}",
            }
        )
    landscape = landscape_preset_browse.list_landscape_presets()
    for pid in landscape.get("ship_presets") or []:
        rows.append(
            {
                "grammar_ref": "landscape_grammar_v0",
                "grammar_id": str(pid),
                "step": "landscape",
                "label": f"landscape:{pid}",
            }
        )
    rows.sort(key=lambda r: (str(r.get("grammar_ref")), str(r.get("grammar_id"))))
    return rows


def browse_modifier_catalog() -> list[dict[str, Any]]:
    return [{"modifier_id": mid, "label": mid.replace("_", " ")} for mid in STUDIO_MODIFIER_IDS]


def browse_connection_graph_ids(*, repo: Path | None = None) -> list[dict[str, Any]]:
    root = repo or repo_root()
    examples = root / "tools" / "mcp" / "schemas" / "examples"
    rows: list[dict[str, Any]] = []
    if not examples.is_dir():
        return rows
    for path in sorted(examples.glob("assembly_snapshot_*.json")):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        aid = str(data.get("assembly_id") or path.stem)
        rows.append(
            {
                "connection_graph_id": _sanitize_graph_id(aid),
                "source_file": path.name,
                "assembly_id": aid,
            }
        )
    # stable pilot ids from existing param frame example
    for graph_id in ("rail_warehouse_pilot",):
        if not any(r["connection_graph_id"] == graph_id for r in rows):
            rows.append(
                {
                    "connection_graph_id": graph_id,
                    "source_file": "param_frame_rail_warehouse_v1.json",
                    "assembly_id": graph_id,
                }
            )
    rows.sort(key=lambda r: r["connection_graph_id"])
    return rows


def _sanitize_graph_id(raw: str) -> str:
    out: list[str] = []
    for ch in str(raw).lower():
        if ch.isalnum() or ch == "_":
            out.append(ch)
        elif ch in "-.":
            out.append("_")
    s = "".join(out).strip("_")
    if not s or not s[0].isalpha():
        s = f"graph_{s}" if s else "graph_unknown"
    return s[:64]


def upsert_grammar_chain_step(frame: dict[str, Any], entry: dict[str, Any]) -> dict[str, Any]:
    out = deepcopy(frame)
    step = dict(entry)
    step_name = str(step.get("step") or "massing")
    if step_name not in GRAMMAR_STEP_ORDER:
        raise ValueError(f"unsupported grammar step: {step_name}")
    chain = list(out.get("grammar_chain") or [])
    replaced = False
    for idx, row in enumerate(chain):
        if isinstance(row, dict) and str(row.get("step")) == step_name:
            chain[idx] = {
                "step": step_name,
                "grammar_ref": str(step.get("grammar_ref") or "building_grammar_v1"),
                "grammar_id": str(step["grammar_id"]),
                **({"rule_key": str(step["rule_key"])} if step.get("rule_key") else {}),
                **({"params": dict(step["params"])} if isinstance(step.get("params"), dict) else {}),
            }
            replaced = True
            break
    if not replaced:
        chain.append(
            {
                "step": step_name,
                "grammar_ref": str(step.get("grammar_ref") or "building_grammar_v1"),
                "grammar_id": str(step["grammar_id"]),
                **({"rule_key": str(step["rule_key"])} if step.get("rule_key") else {}),
            }
        )
    out["grammar_chain"] = chain
    out = merge_ref_books_dual_cite(out)
    validate_param_frame(out)
    return out


def upsert_modifier(
    frame: dict[str, Any],
    modifier_id: str,
    *,
    params: dict[str, Any] | None = None,
    enabled: bool = True,
) -> dict[str, Any]:
    out = deepcopy(frame)
    mods = list(out.get("modifiers") or [])
    payload = {
        "modifier_id": str(modifier_id),
        "enabled": bool(enabled),
    }
    if params:
        payload["params"] = {k: v for k, v in params.items() if isinstance(v, (str, int, bool))}
    replaced = False
    for idx, row in enumerate(mods):
        if isinstance(row, dict) and str(row.get("modifier_id")) == modifier_id:
            mods[idx] = payload
            replaced = True
            break
    if not replaced:
        mods.append(payload)
    out["modifiers"] = mods
    out = merge_ref_books_dual_cite(out)
    validate_param_frame(out)
    return out


def toggle_connection_graph_id(frame: dict[str, Any], graph_id: str) -> dict[str, Any]:
    out = deepcopy(frame)
    gid = _sanitize_graph_id(graph_id)
    ids = list(out.get("connection_graph_ids") or [])
    if gid in ids:
        ids = [x for x in ids if x != gid]
    else:
        ids.append(gid)
    out["connection_graph_ids"] = ids
    out = merge_ref_books_dual_cite(out)
    validate_param_frame(out)
    return out


def write_param_frame(frame: dict[str, Any], path: Path) -> Path:
    data = merge_ref_books_dual_cite(frame)
    validate_param_frame(data)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    return path


def pcg08_promote_gate_snapshot(*, repo: Path | None = None) -> dict[str, Any]:
    """Optional PCG-08 status for UI chrome (CQ-32: no longer blocks enqueue)."""
    report = building_quality_qc.load_pcg08_report(repo=repo)
    if not report:
        return {
            "ok": False,
            "blocked_by": "PCG-08-missing-report",
            "hint": "Run: python -m rust_engine_mcp.cli generate-and-test-report",
        }
    try:
        pass_rate = float(report.get("pass_rate") or 0.0)
    except (TypeError, ValueError):
        pass_rate = 0.0
    return {
        "ok": pass_rate > 0.0,
        "pass_rate": pass_rate,
        "task_id": report.get("task_id"),
    }


def enqueue_param_frame_task(frame_path: Path, *, task_id: str | None = None, seed: int | None = None) -> dict[str, Any]:
    """Queue StudioTask for a param frame (CQ-32: enqueue is not gated on PCG-08)."""
    data = load_param_frame(frame_path)
    root = repo_root()
    try:
        payload = str(frame_path.resolve().relative_to(root)).replace("\\", "/")
    except ValueError:
        # CQ-23: outside-repo paths (tests / scratch) keep an absolute payload —
        # never write into tools/mcp/schemas/examples/.
        payload = str(frame_path.resolve())
    tid = task_id or f"param_frame_{data['frame_id']}"
    return studio_task.enqueue(
        kind="param_frame",
        seed=int(seed if seed is not None else data.get("seed", 0)),
        payload_path=payload,
        task_id=tid,
    )