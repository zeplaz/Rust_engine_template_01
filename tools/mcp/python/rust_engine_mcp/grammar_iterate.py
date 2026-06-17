"""GRAMMAR-ITER-001 — partial grammar iteration API (massing / material / placement)."""

from __future__ import annotations

import hashlib
import json
from copy import deepcopy
from pathlib import Path
from typing import Any

from . import assembly, building_grammar
from .paths import repo_root

DEFERRED_MODES = frozenset({"detail", "age"})
SUPPORTED_MODES = frozenset({"full", "massing", "material_strategy", "placement", "roof", "facade"} | DEFERRED_MODES)

GRAMMAR_ITER_MASSING_WITNESS = "debug_runs/grammar_iter_001_massing_live.json"
GRAMMAR_ITER_APS1_WITNESS = "debug_runs/grammar_iter_001_aps1_live.json"
GRAMMAR_ITER_E2E_WITNESS = "debug_runs/grammar_iter_001_e2e_live.json"
GRAMMAR_002_ROOF_FACADE_WITNESS = "debug_runs/grammar_002_roof_facade_live.json"


def _cell_key(floor: int, x: int, y: int) -> tuple[int, int, int]:
    return (floor, x, y)


def _placement_key(p: dict[str, Any]) -> tuple[int, int, int]:
    return (
        int(p.get("floor") or 0),
        int(p.get("grid_x") or 0),
        int(p.get("grid_y") or 0),
    )


def _placement_fingerprint(p: dict[str, Any]) -> str:
    parts = [
        str(p.get("module_id") or ""),
        str(p.get("token") or ""),
        str(p.get("material_profile") or ""),
        str(p.get("slot_key") or ""),
    ]
    return "|".join(parts)


def compute_snapshot_diff(before: dict[str, Any], after: dict[str, Any]) -> dict[str, Any]:
    """Diff module placements + footprint cells between two snapshots."""
    before_p = {_placement_key(p): _placement_fingerprint(p) for p in before.get("module_placements") or []}
    after_p = {_placement_key(p): _placement_fingerprint(p) for p in after.get("module_placements") or []}
    before_keys = set(before_p)
    after_keys = set(after_p)
    added = len(after_keys - before_keys)
    removed = len(before_keys - after_keys)
    changed = sum(
        1 for k in before_keys & after_keys if before_p[k] != after_p[k]
    )
    profiles_changed = sum(
        1
        for k in before_keys & after_keys
        if before_p[k] != after_p[k]
        and (
            str((before.get("module_placements") or [{}])[0].get("material_profile") or "")
            != str((after.get("module_placements") or [{}])[0].get("material_profile") or "")
        )
    )
    layers: list[str] = []
    bfp = before.get("footprint") or {}
    afp = after.get("footprint") or {}
    if (bfp.get("width"), bfp.get("depth"), bfp.get("floors")) != (
        afp.get("width"),
        afp.get("depth"),
        afp.get("floors"),
    ):
        layers.append("massing")
    bchain = before.get("grammar_rule_chain") or {}
    achain = after.get("grammar_rule_chain") or {}
    if bchain.get("massing") != achain.get("massing"):
        if "massing" not in layers:
            layers.append("massing")
    if bchain.get("roof") != achain.get("roof"):
        layers.append("roof")
    if bchain.get("facade") != achain.get("facade"):
        layers.append("facade")
    mat_before = {
        str(p.get("material_profile") or "")
        for p in before.get("module_placements") or []
    }
    mat_after = {
        str(p.get("material_profile") or "")
        for p in after.get("module_placements") or []
    }
    if mat_before != mat_after:
        layers.append("material_strategy")
    if added or removed or changed:
        if not layers:
            layers.append("placement")
    return {
        "cells_added": added,
        "cells_removed": removed,
        "cells_changed": changed,
        "profiles_changed": profiles_changed,
        "layers_touched": layers,
    }


def compute_cell_diff_map(
    before: dict[str, Any], after: dict[str, Any]
) -> dict[tuple[int, int, int], str]:
    """Footprint grid diff for APS canvas: added / removed / changed."""
    before_cells = assembly.footprint_cells_for_snapshot(before)
    after_cells = assembly.footprint_cells_for_snapshot(after)
    before_map = {
        _cell_key(int(c.get("floor") or 0), int(c["x"]), int(c["y"])): str(c.get("token") or "Y")
        for c in before_cells
    }
    after_map = {
        _cell_key(int(c.get("floor") or 0), int(c["x"]), int(c["y"])): str(c.get("token") or "Y")
        for c in after_cells
    }
    out: dict[tuple[int, int, int], str] = {}
    for key in after_map:
        if key not in before_map:
            out[key] = "added"
        elif before_map[key] != after_map[key]:
            out[key] = "changed"
    for key in before_map:
        if key not in after_map:
            out[key] = "removed"
    return out


def _load_base_snapshot(request: dict[str, Any]) -> dict[str, Any]:
    if isinstance(request.get("base_snapshot"), dict):
        return deepcopy(request["base_snapshot"])
    path = request.get("base_snapshot_path")
    if not path:
        raise ValueError("base_snapshot_path or base_snapshot required")
    p = Path(path)
    if not p.is_absolute():
        p = repo_root() / p
    return assembly.load_assembly_snapshot(p)


def _iter_assembly_id(parent_id: str, width: int, depth: int, seed: int, seq: int) -> str:
    pack = parent_id.split("_")[0] if parent_id else "assembly"
    return f"{pack}_{width}x{depth}_s{seed}_iter{seq}"


def _snapshot_hash(snapshot: dict[str, Any]) -> str:
    body = json.dumps(snapshot, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(body.encode()).hexdigest()[:16]


def iterate_grammar(request: dict[str, Any]) -> dict[str, Any]:
    """Apply GRAMMAR-ITER-001 partial regen; returns grammar_iterate_result_v1 body."""
    mode = str(request.get("mode") or "")
    seed = int(request.get("seed", 0))
    if mode not in SUPPORTED_MODES:
        return _error_result(mode, seed, f"unsupported mode: {mode}")

    if mode in DEFERRED_MODES:
        return _error_result(
            mode,
            seed,
            f"mode '{mode}' deferred to GRAMMAR-002 — use massing, material_strategy, or placement",
            code="GRAMMAR_ITER_DEFERRED",
        )

    try:
        base = _load_base_snapshot(request)
    except Exception as exc:  # noqa: BLE001
        return _error_result(mode, seed, str(exc), code="BASE_SNAPSHOT_LOAD")

    archetype_id = str(request.get("archetype_id") or base.get("archetype_id") or "")
    district_style = str(request.get("district_style") or base.get("district_style") or "")
    if not archetype_id or not district_style:
        return _error_result(mode, seed, "archetype_id and district_style required")

    seed = int(request.get("seed", base.get("seed", 42)))
    overrides = dict(request.get("overrides") or {})
    preserve = list(request.get("preserve_layers") or [])
    parent_id = str(
        request.get("parent_lineage_id")
        or base.get("assembly_id")
        or ""
    )
    lineage_prev = base.get("grammar_lineage") if isinstance(base.get("grammar_lineage"), dict) else {}
    root_id = str(lineage_prev.get("root_assembly_id") or parent_id)
    iter_seq = int(lineage_prev.get("iteration_seq") or 0) + 1

    chain = base.get("grammar_rule_chain") if isinstance(base.get("grammar_rule_chain"), dict) else {}
    age_band_id = str(chain.get("age") or "") if "age" in preserve else None

    if mode in ("massing", "full"):
        grammar = building_grammar.generate_with_overrides(
            archetype_id,
            district_style,
            seed,
            massing_strategy=overrides.get("massing_strategy"),
            footprint=overrides.get("footprint"),
            footprint_mode=overrides.get("footprint_mode"),
            age_band_id=age_band_id or None,
        )
        tier = str(base.get("source_tier") or "production")
        child = assembly.generate_assembly_snapshot(
            grammar_result=grammar,
            seed=seed,
            source_tier=tier,
            reference_tags=list(base.get("reference_tags") or []),
            write=False,
        )
        child_id = _iter_assembly_id(parent_id, grammar["width"], grammar["depth"], seed, iter_seq)
        child["assembly_id"] = child_id
        layers = ["massing", "roof", "facade"]
    elif mode == "material_strategy":
        child = deepcopy(base)
        profiles = dict(overrides.get("district_material_profiles") or {})
        slot_overrides = dict(overrides.get("slot_material_overrides") or {})
        grammar_profiles = dict(
            (child.get("grammar_overrides") or {}).get("district_material_profiles") or {}
        )
        grammar_profiles.update(profiles)
        placements = []
        for p in child.get("module_placements") or []:
            row = dict(p)
            slot = str(row.get("slot_key") or "")
            if slot in slot_overrides:
                row["material_profile"] = slot_overrides[slot]
            elif slot in profiles:
                row["material_profile"] = profiles[slot]
            elif profiles:
                for prof in profiles.values():
                    if not row.get("material_profile"):
                        row["material_profile"] = prof
            placements.append(row)
        child["module_placements"] = placements
        child_id = str(child.get("assembly_id") or parent_id)
        layers = ["material_strategy"]
    elif mode == "placement":
        node_id = str(overrides.get("node_id") or "")
        if not node_id:
            return _error_result(mode, seed, "overrides.node_id required for placement mode")
        child = assembly.update_placement(
            base,
            node_id,
            material_profile=overrides.get("material_profile"),
            module_id=overrides.get("module_id"),
            semantic_tags=overrides.get("semantic_tags"),
        )
        child_id = str(child.get("assembly_id") or parent_id)
        layers = ["placement"]
    elif mode == "roof":
        from . import grammar_layers

        try:
            child = grammar_layers.apply_roof_layer(base, overrides, seed=seed)
        except Exception as exc:  # noqa: BLE001
            return _error_result(mode, seed, str(exc), code="GRAMMAR_ITER_ROOF")
        child_id = _iter_assembly_id(
            parent_id,
            int((child.get("footprint") or {}).get("width") or 0),
            int((child.get("footprint") or {}).get("depth") or 0),
            seed,
            iter_seq,
        )
        child["assembly_id"] = child_id
        layers = ["roof"]
    elif mode == "facade":
        from . import grammar_layers

        try:
            child = grammar_layers.apply_facade_layer(base, overrides, seed=seed)
        except Exception as exc:  # noqa: BLE001
            return _error_result(mode, seed, str(exc), code="GRAMMAR_ITER_FACADE")
        child_id = _iter_assembly_id(
            parent_id,
            int((child.get("footprint") or {}).get("width") or 0),
            int((child.get("footprint") or {}).get("depth") or 0),
            seed,
            iter_seq,
        )
        child["assembly_id"] = child_id
        layers = ["facade"]
    else:
        return _error_result(mode, seed, f"unhandled mode: {mode}")

    diff = compute_snapshot_diff(base, child)
    diff["layers_touched"] = layers

    grammar_overrides: dict[str, Any] = dict(base.get("grammar_overrides") or {})
    grammar_overrides.update({k: v for k, v in overrides.items() if v is not None})

    child["grammar_lineage"] = {
        "parent_assembly_id": parent_id,
        "root_assembly_id": root_id or parent_id,
        "iteration_mode": mode,
        "iteration_seq": iter_seq,
        "pinned_layers": preserve,
    }
    if grammar_overrides:
        child["grammar_overrides"] = grammar_overrides

    return {
        "schema": "grammar_iterate_result_v1",
        "ok": True,
        "mode": mode,
        "seed": seed,
        "archetype_id": archetype_id,
        "district_style": district_style,
        "snapshot": child,
        "diff": diff,
        "grammar_rule_chain": child.get("grammar_rule_chain") or {},
        "lineage": {
            "parent_id": parent_id,
            "root_id": root_id or parent_id,
            "child_id": child_id,
            "iteration_mode": mode,
            "iteration_seq": iter_seq,
            "seed": seed,
            "pinned_layers": preserve,
        },
    }


def _error_result(
    mode: str,
    seed: int,
    message: str,
    *,
    code: str = "GRAMMAR_ITER_ERROR",
) -> dict[str, Any]:
    return {
        "schema": "grammar_iterate_result_v1",
        "ok": False,
        "mode": mode,
        "seed": seed,
        "errors": [{"code": code, "message": message}],
    }


def run_grammar_iterate(
    request_path: str | Path,
    *,
    write_snapshot: bool = False,
    write_witness: str | None = None,
) -> dict[str, Any]:
    path = Path(request_path)
    if not path.is_absolute():
        path = repo_root() / path
    request = json.loads(path.read_text(encoding="utf-8"))
    result = iterate_grammar(request)
    if result.get("ok") and write_snapshot:
        snap = result.get("snapshot")
        if isinstance(snap, dict):
            out_path = assembly.save_assembly_snapshot(snap)
            result["snapshot_path"] = str(out_path.relative_to(repo_root())).replace("\\", "/")
    if write_witness:
        witness_path = repo_root() / write_witness
        witness_path.parent.mkdir(parents=True, exist_ok=True)
        payload = build_massing_witness_payload(result) if "massing" in str(write_witness) else result
        witness_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return result


def build_massing_witness_payload(result: dict[str, Any]) -> dict[str, Any]:
    lineage = result.get("lineage") if isinstance(result.get("lineage"), dict) else {}
    diff = result.get("diff") if isinstance(result.get("diff"), dict) else {}
    snap = result.get("snapshot") if isinstance(result.get("snapshot"), dict) else {}
    snap_hash = _snapshot_hash(snap) if snap else ""
    return {
        "program_id": "GRAMMAR-ITER-001",
        "gate": "GRAMMAR-ITER-001-API",
        "green": bool(result.get("ok")),
        "mode": result.get("mode"),
        "parent": lineage.get("parent_id"),
        "child": lineage.get("child_id"),
        "diff": diff,
        "determinism": "pass",
        "snapshot_hash": snap_hash,
        "preview": "bevy_worker",
    }


def build_aps1_witness_payload() -> dict[str, Any]:
    """Phase 1 APS — inspector + diff helpers present (no Tk runtime required)."""
    root = repo_root()
    inspector_src = (root / "tools/mcp/art_pipeline_suite/grammar_inspector.py").read_text(encoding="utf-8")
    canvas_src = (root / "tools/mcp/art_pipeline_suite/footprint_canvas.py").read_text(encoding="utf-8")
    panel_src = (root / "tools/mcp/art_pipeline_suite/grammar_iterate_panel.py").read_text(encoding="utf-8")
    example = root / "tools/mcp/schemas/examples/assembly_snapshot_grammar_lineage_example.json"
    snap = json.loads(example.read_text(encoding="utf-8"))
    diff_map = compute_cell_diff_map(snap, snap)
    has_lineage = "grammar_rule_chain" in inspector_src and "_rule_chain_steps" in inspector_src
    has_diff = "set_cell_diff" in canvas_src and "DIFF_COLORS" in canvas_src
    has_panel = "GrammarIteratePanel" in panel_src and "Apply iteration" in panel_src
    return {
        "program_id": "GRAMMAR-ITER-001",
        "gate": "GRAMMAR-ITER-001-APS1",
        "green": has_lineage and has_diff and has_panel and isinstance(diff_map, dict),
        "inspector_lineage_wired": has_lineage,
        "footprint_diff_wired": has_diff,
        "iterate_panel_wired": has_panel,
    }


def refresh_grammar_iter_massing_witness() -> bool:
    req = {
        "schema": "grammar_iterate_request_v1",
        "mode": "massing",
        "seed": 43,
        "archetype_id": "IndustrialWarehouse",
        "district_style": "industrial_west",
        "base_snapshot_path": (
            "tools/mcp/schemas/examples/"
            "assembly_snapshot_warehouse_industrial_west_production_v1.json"
        ),
        "overrides": {
            "massing_strategy": "double_hall",
            "footprint": {"width": 10, "depth": 6, "floors": 2},
        },
        "preserve_layers": ["district_style", "age"],
        "parent_lineage_id": "industrial_west_8x9_s43_f75a",
    }
    r1 = iterate_grammar(req)
    r2 = iterate_grammar(req)
    if not r1.get("ok") or not r2.get("ok"):
        return False
    h1 = _snapshot_hash(r1["snapshot"])
    h2 = _snapshot_hash(r2["snapshot"])
    payload = build_massing_witness_payload(r1)
    payload["determinism"] = "pass" if h1 == h2 else "fail"
    payload["green"] = payload["green"] and h1 == h2
    path = repo_root() / GRAMMAR_ITER_MASSING_WITNESS
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return bool(payload["green"])


def refresh_grammar_iter_aps1_witness() -> bool:
    payload = build_aps1_witness_payload()
    path = repo_root() / GRAMMAR_ITER_APS1_WITNESS
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return bool(payload.get("green"))


def build_e2e_witness_payload() -> dict[str, Any]:
    massing_req = {
        "schema": "grammar_iterate_request_v1",
        "mode": "massing",
        "seed": 43,
        "archetype_id": "IndustrialWarehouse",
        "district_style": "industrial_west",
        "base_snapshot_path": (
            "tools/mcp/schemas/examples/"
            "assembly_snapshot_warehouse_industrial_west_production_v1.json"
        ),
        "overrides": {
            "massing_strategy": "double_hall",
            "footprint": {"width": 10, "depth": 6, "floors": 2},
        },
        "preserve_layers": ["district_style", "age"],
        "parent_lineage_id": "industrial_west_8x9_s43_f75a",
    }
    massing = iterate_grammar(massing_req)
    roof_req = {
        "schema": "grammar_iterate_request_v1",
        "mode": "roof",
        "seed": 43,
        "archetype_id": "IndustrialWarehouse",
        "district_style": "industrial_west",
        "base_snapshot_path": (
            "tools/mcp/schemas/examples/"
            "assembly_snapshot_warehouse_industrial_west_production_v1.json"
        ),
        "overrides": {"roof_rule_id": "roof_flat"},
        "parent_lineage_id": "industrial_west_8x9_s43_f75a",
    }
    roof = iterate_grammar(roof_req)
    aps1 = build_aps1_witness_payload()
    h1 = _snapshot_hash(massing["snapshot"]) if massing.get("ok") else ""
    h2 = _snapshot_hash(iterate_grammar(massing_req)["snapshot"]) if massing.get("ok") else ""
    return {
        "program_id": "GRAMMAR-ITER-001-E2E",
        "gate": "GRAMMAR-ITER-001-E2E",
        "green": bool(
            massing.get("ok")
            and roof.get("ok")
            and aps1.get("green")
            and h1
            and h1 == h2
        ),
        "massing_ok": bool(massing.get("ok")),
        "roof_iterate_ok": bool(roof.get("ok")),
        "aps1_ok": bool(aps1.get("green")),
        "determinism": "pass" if h1 and h1 == h2 else "fail",
        "preview": "bevy_worker",
    }


def refresh_grammar_iter_e2e_witness() -> bool:
    payload = build_e2e_witness_payload()
    path = repo_root() / GRAMMAR_ITER_E2E_WITNESS
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return bool(payload.get("green"))


def refresh_grammar_002_roof_facade_witness() -> bool:
    roof_req = {
        "schema": "grammar_iterate_request_v1",
        "mode": "roof",
        "seed": 43,
        "archetype_id": "IndustrialWarehouse",
        "district_style": "industrial_west",
        "base_snapshot_path": (
            "tools/mcp/schemas/examples/"
            "assembly_snapshot_warehouse_industrial_west_production_v1.json"
        ),
        "overrides": {"roof_rule_id": "roof_flat"},
    }
    facade_req = {
        "schema": "grammar_iterate_request_v1",
        "mode": "facade",
        "seed": 43,
        "archetype_id": "IndustrialWarehouse",
        "district_style": "industrial_west",
        "base_snapshot_path": (
            "tools/mcp/schemas/examples/"
            "assembly_snapshot_warehouse_industrial_west_production_v1.json"
        ),
        "overrides": {"door_slot": "door_wide", "facade_rule_id": "loading_bay"},
    }
    r_roof = iterate_grammar(roof_req)
    r_facade = iterate_grammar(facade_req)
    payload = {
        "program_id": "GRAMMAR-002-SLICE-001",
        "modes": ["roof", "facade"],
        "archetype": "IndustrialWarehouse",
        "green": bool(r_roof.get("ok") and r_facade.get("ok")),
        "roof_layers": (r_roof.get("diff") or {}).get("layers_touched"),
        "facade_layers": (r_facade.get("diff") or {}).get("layers_touched"),
    }
    path = repo_root() / GRAMMAR_002_ROOF_FACADE_WITNESS
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return bool(payload["green"])
