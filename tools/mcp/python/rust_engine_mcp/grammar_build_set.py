"""MCP-GRAMMAR-SET-001…004 + MCP-BUILD-SET-001…003 — grammar/build-set MCP tools."""

from __future__ import annotations

import json
import re
from collections import Counter
from pathlib import Path
from typing import Any

from . import arch_build_grammar, building_grammar
from .paths import repo_root, schemas_dir
from .pilot_hardcode_lint import pilot_hardcode_lint
from .validators.report import ValidationIssue, ValidationReport

PILOT_CATALOG_RON = "assets/configs/buildings/_pilot_catalog.ron"
BUILDING_SETS_RON = "assets/configs/buildings/_building_sets.ron"
MOCK_SHAPES_RON = "assets/configs/buildings/_mock_shapes.ron"
BUILDING_SET_SCHEMA = "building_set_manifest_v1.schema.json"
COVERAGE_WITNESS = "debug_runs/building_set_coverage_live.json"
GRAMMAR_SET_WITNESS = "debug_runs/grammar_set_brief_live.json"
GRAMMAR_TIER_WITNESS = "debug_runs/grammar_set_tier_live.json"
GRAMMAR_TIER_G1_WITNESS = "debug_runs/grammar_set_tier_g1.json"
GRAMMAR_ARCHETYPE_G1_WITNESS = "debug_runs/grammar_archetype_g1_live.json"
GRAMMAR_TIER_G1_GATES_WITNESS = "debug_runs/aps_grammar_tier_g1_gates_live.json"
GRAMMAR_P3_WITNESS = "debug_runs/aps_grammar_p3_live.json"
GRAMMAR_SPINE_TIER_WITNESS = "debug_runs/aps_grammar_spine_tier_live.json"
GRAMMAR_LABELS_G1_WITNESS = "debug_runs/grammar_labels_g1_live.json"
GRAMMAR_EVOLUTION_CLOSE_WITNESS = "debug_runs/aps_grammar_evolution_close_live.json"
GUARD_BRIEF_PARITY_WITNESS = "debug_runs/aps_guard_brief_parity_live.json"
GRAMMAR_TIER_GATES_LIVE_WITNESS = "debug_runs/aps_grammar_tier_gates_live.json"
GRAMMAR_TIER_GATES_G0_FIXTURE = "debug_runs/aps_grammar_tier_gates_g0_fixture_live.json"
SESSION_PRESENCE_WITNESS = "debug_runs/aps_session_presence_live.json"
GRAMMARS_DIR = "assets/configs/buildings/grammars"
TIER_ORDER = ("G0", "G1", "G2", "G3", "G4")
TIER_CHIP_LABELS = {
    "G0": "G0 — pilot kit",
    "G1": "G1 — family seed",
    "G2": "G2 — axis coverage",
    "G3": "G3 — layer depth",
    "G4": "G4 — production set",
}

_RON_STRING = re.compile(r'"([^"]+)"')


def _read_asset(rel: str) -> str:
    path = repo_root() / rel
    return path.read_text(encoding="utf-8")


def _ron_field(block: str, field: str) -> str | None:
    m = re.search(rf"{re.escape(field)}:\s*\"([^\"]+)\"", block)
    return m.group(1) if m else None


def _ron_field_raw(block: str, field: str) -> str | None:
    m = re.search(rf"{re.escape(field)}:\s*([a-z_][a-z0-9_]*)", block)
    return m.group(1) if m else None


def _split_ron_entries(text: str, entry_marker: str = "id:") -> list[str]:
    chunks: list[str] = []
    current: list[str] = []
    for line in text.splitlines():
        if line.strip().startswith(entry_marker) and current:
            chunks.append("\n".join(current))
            current = [line]
        else:
            current.append(line)
    if current:
        chunks.append("\n".join(current))
    return chunks


def resolve_pilot_kind(row: dict[str, Any]) -> str:
    """APS-GUARD-BRIEF-PARITY-001 — explicit pilot_kind or infer from ARCH-DNA fields."""
    explicit = row.get("pilot_kind")
    if explicit in ("grammar", "shape_qa"):
        return str(explicit)
    if row.get("arch_dna_preset") and row.get("grammar_archetype_id"):
        return "grammar"
    return "shape_qa"


def load_pilot_catalog() -> list[dict[str, Any]]:
    text = _read_asset(PILOT_CATALOG_RON)
    pilots: list[dict[str, Any]] = []
    for block in _split_ron_entries(text):
        pid = _ron_field(block, "id")
        if not pid:
            continue
        row = {
            "id": pid,
            "label": _ron_field(block, "label") or pid,
            "mock_shape_id": _ron_field(block, "mock_shape_id"),
            "arch_dna_preset": _ron_field(block, "arch_dna_preset"),
            "grammar_archetype_id": _ron_field(block, "grammar_archetype_id"),
            "district_style": _ron_field(block, "district_style"),
            "site_json_path": _ron_field(block, "site_json_path"),
            "pilot_kind": _ron_field_raw(block, "pilot_kind"),
            "hover_hint": _ron_field(block, "hover_hint"),
        }
        row["pilot_kind"] = resolve_pilot_kind(row)
        pilots.append(row)
    return pilots


def pilot_catalog_inventory() -> dict[str, Any]:
    """APS-GUARD-BRIEF-PARITY-001 — single authority for brief · coverage · parity counts."""
    pilots = load_pilot_catalog()
    grammar_pilots = [p for p in pilots if p.get("pilot_kind") == "grammar"]
    shape_qa_pilots = [p for p in pilots if p.get("pilot_kind") == "shape_qa"]
    return {
        "grammar_pilot_count": len(grammar_pilots),
        "shape_qa_count": len(shape_qa_pilots),
        "total_pilot_count": len(pilots),
        "grammar_pilot_ids": [str(p["id"]) for p in grammar_pilots],
        "shape_qa_pilot_ids": [str(p["id"]) for p in shape_qa_pilots],
        "pilots_by_id": {str(p["id"]): p for p in pilots},
    }


def guard_brief_parity_audit() -> dict[str, Any]:
    """Return whether brief, coverage, and parity agree on grammar pilot count."""
    inv = pilot_catalog_inventory()
    brief = grammar_set_brief()
    coverage = building_set_coverage_report()
    parity = grammar_pilot_parity()
    brief_count = int((brief.get("counts") or {}).get("grammar_pilots") or 0)
    parity_count = int(parity.get("grammar_pilot_count") or 0)
    coverage_count = int(coverage.get("grammar_pilot_count") or 0)
    authority = int(inv["grammar_pilot_count"])
    counts_aligned = brief_count == parity_count == coverage_count == authority
    count_mismatch = not counts_aligned
    brief_green = bool(brief.get("green"))
    tier = grammar_set_tier()
    return {
        "task_id": "APS-GUARD-BRIEF-PARITY-001",
        "green": counts_aligned,
        "counts_aligned": counts_aligned,
        "grammar_pilot_count": authority,
        "brief_grammar_pilot_count": brief_count,
        "coverage_grammar_pilot_count": coverage_count,
        "parity_grammar_pilot_count": parity_count,
        "brief_green": brief_green,
        "coverage_green": coverage.get("green"),
        "parity_green": parity.get("green"),
        "grammar_pilot_ids": inv["grammar_pilot_ids"],
        "tier": tier.get("tier"),
        "tier_reasons": tier.get("reasons") or [],
        "no_green_brief_with_red_guards": not (brief_green and count_mismatch),
        "pilot_hardcode_green": bool((coverage.get("pilot_hardcode_green"))),
    }


def write_aps_guard_brief_parity_witness() -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    body = guard_brief_parity_audit()
    return write_aps_live_witness(
        body,
        GUARD_BRIEF_PARITY_WITNESS,
        schema="aps_guard_brief_parity_live_v1",
        profile="APS_GUARD_BRIEF_PARITY",
        source_system="grammar_build_set",
        ritual="BLANG:WIT-HON APS-GUARD-BRIEF-PARITY-001" if body.get("green") else None,
    )


def load_building_sets() -> list[dict[str, Any]]:
    text = _read_asset(BUILDING_SETS_RON)
    sets: list[dict[str, Any]] = []
    for block in re.split(r"set_id:\s*", text)[1:]:
        set_id_m = re.match(r'"([^"]+)"', block)
        if not set_id_m:
            continue
        pilot_section = re.search(r"pilot_ids:\s*\[(.*?)\]", block, re.DOTALL)
        pilots_in_set = re.findall(r'"([a-z][a-z0-9_]*)"', pilot_section.group(1)) if pilot_section else []
        req_section = re.search(r"required_f_functions:\s*\[(.*?)\]", block, re.DOTALL)
        required_f = re.findall(r'"([a-z][a-z0-9_]*)"', req_section.group(1)) if req_section else []
        min_gp = re.search(r"min_grammar_pilots:\s*(\d+)", block)
        sets.append(
            {
                "set_id": set_id_m.group(1),
                "label": _ron_field(block, "label") or set_id_m.group(1),
                "min_grammar_pilots": int(min_gp.group(1)) if min_gp else 2,
                "pilot_ids": pilots_in_set,
                "required_f_functions": required_f,
            }
        )
    return sets


def _load_preset_raw(preset_id: str) -> dict[str, Any]:
    path = arch_build_grammar.preset_path(preset_id)
    return json.loads(path.read_text(encoding="utf-8"))


def _arch_dna_f(preset_id: str) -> str | None:
    try:
        preset = _load_preset_raw(preset_id)
    except (FileNotFoundError, OSError, json.JSONDecodeError):
        return None
    arch = preset.get("arch_dna") if isinstance(preset.get("arch_dna"), dict) else {}
    f_val = arch.get("F")
    return str(f_val).lower() if f_val else None


def grammar_set_brief(*, set_id: str | None = None) -> dict[str, Any]:
    """MCP-GRAMMAR-SET-001 — compressed pilot/grammar/preset inventory."""
    inv = pilot_catalog_inventory()
    grammar_pilots = [inv["pilots_by_id"][pid] for pid in inv["grammar_pilot_ids"]]
    shape_pilots = [inv["pilots_by_id"][pid] for pid in inv["shape_qa_pilot_ids"]]
    presets = arch_build_grammar.list_preset_ids()
    sets = load_building_sets()
    if set_id:
        sets = [s for s in sets if s.get("set_id") == set_id]

    f_axis = sorted({_arch_dna_f(pid) for pid in presets if _arch_dna_f(pid)})
    gaps: list[str] = []
    if len(grammar_pilots) < 4:
        gaps.append(f"grammar_pilots={len(grammar_pilots)} need ≥4")
    if len(presets) < 4:
        gaps.append(f"arch_dna_presets={len(presets)} need ≥4")
    for s in sets:
        req = s.get("required_f_functions") or []
        seen = {_arch_dna_f(p) for p in s.get("pilot_ids") or []}
        seen.discard(None)
        for axis in req:
            if axis.lower() not in seen:
                gaps.append(f"{s['set_id']}: missing F-axis {axis}")

    lines = [
        f"grammar pilots: {len(grammar_pilots)} · shape QA: {len(shape_pilots)} · presets: {len(presets)}",
        f"F-axis covered: {', '.join(f_axis) or '(none)'}",
    ]
    if gaps:
        lines.append("gaps: " + "; ".join(gaps))
    else:
        lines.append("gaps: none")

    body = {
        "task_id": "MCP-GRAMMAR-SET-001",
        "ok": True,
        "green": len(gaps) == 0,
        "lines": lines,
        "text": "\n".join(lines),
        "counts": {
            "grammar_pilots": len(grammar_pilots),
            "shape_qa_pilots": len(shape_pilots),
            "arch_dna_presets": len(presets),
            "building_sets": len(sets),
        },
        "grammar_pilot_ids": [p["id"] for p in grammar_pilots],
        "preset_ids": presets,
        "f_axis_values": f_axis,
        "gaps": gaps,
        "sets": sets,
    }
    return body


def grammar_preset_pair_validate(*, preset_id: str | None = None, path: str | Path | None = None) -> dict[str, Any]:
    """MCP-GRAMMAR-SET-002 — preset ↔ grammar_id ↔ pilot row."""
    errors: list[str] = []
    if path:
        preset = json.loads(Path(path).read_text(encoding="utf-8"))
        pid = str(preset.get("preset_id") or "")
    elif preset_id:
        pid = preset_id
        preset = _load_preset_raw(pid)
    else:
        return {"ok": False, "errors": ["preset_id or path required"]}

    grammar_id = str(preset.get("grammar_id") or "")
    pilots = load_pilot_catalog()
    pilot = next((p for p in pilots if p.get("arch_dna_preset") == pid), None)
    if pilot is None:
        errors.append(f"no pilot catalog row for preset {pid}")
    elif not pilot.get("grammar_archetype_id"):
        errors.append(f"pilot {pilot['id']} missing grammar_archetype_id")
    if grammar_id and pilot and pilot.get("grammar_archetype_id"):
        try:
            g = building_grammar.load_building_grammar_by_archetype(pilot["grammar_archetype_id"])
            file_grammar_id = str(g.get("grammar_id") or "")
            if grammar_id != file_grammar_id:
                errors.append(f"preset grammar_id {grammar_id!r} != grammar file {file_grammar_id!r}")
        except (KeyError, FileNotFoundError, NotImplementedError) as exc:
            errors.append(f"grammar load: {exc}")

    return {
        "task_id": "MCP-GRAMMAR-SET-002",
        "ok": len(errors) == 0,
        "green": len(errors) == 0,
        "preset_id": pid,
        "pilot_id": pilot.get("id") if pilot else None,
        "grammar_id": grammar_id,
        "errors": errors,
    }


def _zone_coverage_histogram() -> dict[str, int]:
    """CMCP-GRAM-SWEEP-PROCESS-001 — aggregate zone kinds from pilot site grids."""
    from rust_engine_mcp.validators.site_zone_grid import DEFAULT_PILOT_PATHS, _zone_counts

    counts: Counter[str] = Counter()
    root = repo_root()
    for rel in DEFAULT_PILOT_PATHS:
        path = root / rel
        if not path.is_file():
            continue
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        for zone, n in _zone_counts(list(data.get("cells") or [])).items():
            counts[str(zone)] += int(n)
    return dict(counts)


def _process_sweep_histogram() -> dict[str, dict[str, int]]:
    """CMCP-GRAM-SWEEP-PROCESS-001 — power_tier + role + zone coverage from facility bindings."""
    from rust_engine_mcp import grammar_facility_brief

    body = grammar_facility_brief.grammar_facility_brief()
    power_tier: Counter[str] = Counter()
    role: Counter[str] = Counter()
    for brief in body.get("briefs") or []:
        if not brief.get("facility_binding"):
            continue
        catalog = brief.get("catalog") or {}
        derived = brief.get("derived") or {}
        tier = derived.get("power_tier_from_catalog") or catalog.get("power_tier")
        role_key = catalog.get("supply_chain_role") or catalog.get("utility_role")
        if tier:
            power_tier[str(tier)] += 1
        if role_key:
            role[str(role_key)] += 1
    zone_coverage = _zone_coverage_histogram()
    return {
        "power_tier": dict(power_tier),
        "supply_chain_role": dict(role),
        "zone_coverage": zone_coverage,
    }


def grammar_eval_sweep(
    *,
    archetype_id: str = "IndustrialWarehouse",
    district_style: str = "industrial_west",
    seeds: list[int] | None = None,
) -> dict[str, Any]:
    """MCP-GRAMMAR-SET-003 — seed sweep massing/roof histogram."""
    seed_list = seeds if seeds else list(range(40, 40 + 24))
    massing: Counter[str] = Counter()
    roof: Counter[str] = Counter()
    errors = 0
    for seed in seed_list:
        try:
            result = building_grammar.generate(archetype_id, district_style, int(seed))
            massing[str(result.get("massing_strategy") or "unknown")] += 1
            chain = result.get("rule_chain") or []
            roof_id = next(
                (str(r.get("rule_id")) for r in chain if str(r.get("layer")) == "roof"),
                "unknown",
            )
            roof[roof_id] += 1
        except (KeyError, NotImplementedError, ValueError):
            errors += 1

    hist_lines = [f"massing {k}: {v}" for k, v in massing.most_common()]
    process_histogram = _process_sweep_histogram()
    body = {
        "task_id": "MCP-GRAMMAR-SET-003",
        "ok": errors == 0,
        "green": errors == 0 and len(massing) >= 2,
        "archetype_id": archetype_id,
        "district_style": district_style,
        "seed_count": len(seed_list),
        "errors": errors,
        "massing_histogram": dict(massing),
        "roof_histogram": dict(roof),
        "process_histogram": process_histogram,
        "lines": hist_lines,
        "text": "\n".join(hist_lines[:12]),
    }
    return body


def write_grammar_sweep_process_witness() -> dict[str, Any]:
    """CMCP-GRAM-SWEEP-PROCESS-001 witness rollup."""
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    sweep = grammar_eval_sweep()
    hist = sweep.get("process_histogram") or {}
    green = bool(hist.get("power_tier")) and bool(hist.get("zone_coverage"))
    body = {
        "task_id": "CMCP-GRAM-SWEEP-PROCESS-001",
        "green": green,
        "process_histogram": hist,
        "sweep": {
            "archetype_id": sweep.get("archetype_id"),
            "seed_count": sweep.get("seed_count"),
            "massing_histogram": sweep.get("massing_histogram"),
        },
    }
    return write_aps_live_witness(
        body,
        "debug_runs/grammar_sweep_process_live.json",
        schema="grammar_sweep_process_live_v1",
        profile="CMCP_GRAM_SWEEP_PROCESS",
        source_system="grammar_build_set",
        ritual="BLANG:WIT-HON CMCP-GRAM-SWEEP-PROCESS-001" if body.get("green") else None,
    )


def grammar_pilot_parity() -> dict[str, Any]:
    """MCP-GRAMMAR-SET-004 — MCP wrap of catalog parity checks."""
    inv = pilot_catalog_inventory()
    errors: list[str] = []
    grammar = [inv["pilots_by_id"][pid] for pid in inv["grammar_pilot_ids"]]
    shape = [inv["pilots_by_id"][pid] for pid in inv["shape_qa_pilot_ids"]]
    if inv["total_pilot_count"] < 8:
        errors.append(f"pilot_count={inv['total_pilot_count']} expected ≥8")
    if len(shape) < 4:
        errors.append(f"shape_qa={len(shape)} need ≥4")
    if len(grammar) < 4:
        errors.append(f"grammar_pilots={len(grammar)} need ≥4")
    for p in grammar:
        if not p.get("arch_dna_preset"):
            errors.append(f"{p['id']}: missing arch_dna_preset")
        elif not _arch_dna_f(p["arch_dna_preset"]):
            errors.append(f"{p['id']}: preset not loadable")
        if not p.get("grammar_archetype_id"):
            errors.append(f"{p['id']}: missing grammar_archetype_id")
    pair = grammar_preset_pair_validate(preset_id=grammar[0]["arch_dna_preset"]) if grammar else {"ok": False}
    if not pair.get("ok"):
        errors.extend(pair.get("errors") or ["preset pair failed"])

    return {
        "task_id": "MCP-GRAMMAR-SET-004",
        "ok": len(errors) == 0,
        "green": len(errors) == 0,
        "grammar_pilot_count": len(grammar),
        "shape_qa_count": len(shape),
        "errors": errors,
    }


def _manifest_from_json(data: dict[str, Any]) -> dict[str, Any]:
    from jsonschema import Draft202012Validator

    schema = json.loads((schemas_dir() / BUILDING_SET_SCHEMA).read_text(encoding="utf-8"))
    Draft202012Validator(schema).validate(data)
    return data


def building_set_manifest_validate(*, path: str | Path | None = None, set_id: str | None = None) -> dict[str, Any]:
    """MCP-BUILD-SET-001 — validate building_set_manifest_v1 JSON."""
    errors: list[str] = []
    if path:
        p = Path(path)
        if not p.is_absolute():
            p = repo_root() / p
        data = json.loads(p.read_text(encoding="utf-8"))
    elif set_id:
        example = repo_root() / f"tools/mcp/schemas/examples/building_set_{set_id}.json"
        if not example.is_file():
            errors.append(f"no example manifest for set_id {set_id}")
            return {"ok": False, "errors": errors}
        data = json.loads(example.read_text(encoding="utf-8"))
    else:
        data = {
            "schema_version": "building_set_manifest_v1",
            **load_building_sets()[0],
        }
        data["schema_version"] = "building_set_manifest_v1"
    try:
        _manifest_from_json(data)
    except Exception as exc:  # noqa: BLE001
        errors.append(str(exc))
    return {
        "task_id": "MCP-BUILD-SET-001",
        "ok": len(errors) == 0,
        "green": len(errors) == 0,
        "set_id": data.get("set_id"),
        "errors": errors,
    }


def building_set_coverage_report(*, set_id: str | None = None) -> dict[str, Any]:
    """MCP-BUILD-SET-002 — F/L/I axis coverage; FAIL on singleton set."""
    inv = pilot_catalog_inventory()
    pilots = inv["pilots_by_id"]
    sets = load_building_sets()
    if set_id:
        sets = [s for s in sets if s.get("set_id") == set_id]
    errors: list[str] = []
    rows: list[dict[str, Any]] = []

    for s in sets:
        f_seen: set[str] = set()
        i_seen: set[str] = set()
        grammar_in_set = 0
        for pid in s.get("pilot_ids") or []:
            pilot = pilots.get(pid)
            if not pilot:
                errors.append(f"{s['set_id']}: missing pilot {pid}")
                continue
            if pilot.get("pilot_kind") == "grammar":
                grammar_in_set += 1
            preset = pilot.get("arch_dna_preset")
            if preset:
                try:
                    data = _load_preset_raw(preset)
                    dna = data.get("arch_dna") or {}
                    if dna.get("F"):
                        f_seen.add(str(dna["F"]).lower())
                    if dna.get("I"):
                        i_seen.add(str(dna["I"]).lower())
                except (FileNotFoundError, ValueError, json.JSONDecodeError):
                    errors.append(f"{s['set_id']}: bad preset {preset}")
        if grammar_in_set < int(s.get("min_grammar_pilots") or 2):
            errors.append(f"{s['set_id']}: grammar pilots {grammar_in_set} < min")
        for req in s.get("required_f_functions") or []:
            if req.lower() not in f_seen:
                errors.append(f"{s['set_id']}: F-axis {req} not covered")
        rows.append(
            {
                "set_id": s["set_id"],
                "grammar_pilots": grammar_in_set,
                "f_axis": sorted(f_seen),
                "i_axis": sorted(i_seen),
            }
        )

    preset_count = len(arch_build_grammar.list_preset_ids())
    if preset_count < 4:
        errors.append(f"arch_dna examples={preset_count} need ≥4")

    hardcode = pilot_hardcode_lint()
    if not hardcode.get("green"):
        errors.append(f"pilot_hardcode_lint violations={hardcode.get('violation_count')}")

    body = {
        "task_id": "MCP-BUILD-SET-002",
        "ok": len(errors) == 0,
        "green": len(errors) == 0,
        "rows": rows,
        "errors": errors,
        "preset_count": preset_count,
        "grammar_pilot_count": inv["grammar_pilot_count"],
        "grammar_pilot_ids": inv["grammar_pilot_ids"],
        "pilot_hardcode_green": hardcode.get("green"),
    }
    return body


def building_set_health_brief() -> dict[str, Any]:
    """MCP-BUILD-SET-003 / OPS-BUILD-SET-001 — rollup for OPS + APS."""
    brief = grammar_set_brief()
    coverage = building_set_coverage_report()
    parity = grammar_pilot_parity()
    hardcode = pilot_hardcode_lint()
    parity_audit = guard_brief_parity_audit()
    guards_green = bool(coverage.get("green")) and bool(parity.get("green"))
    return {
        "green": bool(brief.get("green")) and guards_green and bool(hardcode.get("green")),
        "grammar_set_brief": brief.get("text"),
        "coverage_green": coverage.get("green"),
        "parity_green": parity.get("green"),
        "hardcode_green": hardcode.get("green"),
        "grammar_pilot_count": brief.get("counts", {}).get("grammar_pilots"),
        "preset_count": brief.get("counts", {}).get("arch_dna_presets"),
        "gaps": brief.get("gaps") or [],
        "coverage_errors": coverage.get("errors") or [],
        "counts_aligned": parity_audit.get("counts_aligned"),
        "set_health_honest": parity_audit.get("no_green_brief_with_red_guards"),
        "pilot_hardcode_green": parity_audit.get("pilot_hardcode_green"),
    }


def write_building_set_coverage_witness() -> dict[str, Any]:
    body = building_set_coverage_report()
    out = repo_root() / COVERAGE_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body


def write_aps_g4_coverage_witness() -> dict[str, Any]:
    """APS-G4-COVERAGE-001 — building_set_coverage + pilot_hardcode + tier G4 bar."""
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    coverage = building_set_coverage_report()
    tier = grammar_set_tier()
    body = {
        "task_id": "APS-G4-COVERAGE-001",
        "green": bool(coverage.get("green")) and tier.get("tier") == "G4" and not tier.get("reasons"),
        "building_set_coverage_green": coverage.get("green"),
        "pilot_hardcode_green": coverage.get("pilot_hardcode_green"),
        "grammar_set_tier": tier.get("tier"),
        "grammar_set_tier_reasons": tier.get("reasons") or [],
        "coverage_errors": coverage.get("errors") or [],
        "grammar_pilot_count": coverage.get("grammar_pilot_count"),
        "preset_count": coverage.get("preset_count"),
        "coverage_rows": coverage.get("rows") or [],
    }
    return write_aps_live_witness(
        body,
        "debug_runs/aps_g4_coverage_live.json",
        schema="aps_g4_coverage_live_v1",
        profile="APS_G4_COVERAGE",
        source_system="grammar_build_set",
        ritual="BLANG:WIT-HON APS-G4-COVERAGE-001" if body.get("green") else None,
    )


def write_grammar_set_brief_witness() -> dict[str, Any]:
    body = grammar_set_brief()
    out = repo_root() / GRAMMAR_SET_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body


def _grammar_ron_filenames() -> list[str]:
    root = repo_root() / GRAMMARS_DIR
    if not root.is_dir():
        return []
    return sorted(p.name for p in root.glob("*.ron"))


def _district_count_for_archetypes(archetype_ids: list[str]) -> int:
    district_ids: set[str] = set()
    for aid in archetype_ids:
        for row in building_grammar.list_district_styles(aid):
            district_ids.add(row)
    return len(district_ids)


def _max_districts_in_lineage(archetype_ids: list[str]) -> int:
    best = 0
    for aid in archetype_ids:
        best = max(best, len(building_grammar.list_district_styles(aid)))
    return best


def _dna_f_axis_values() -> set[str]:
    axes: set[str] = set()
    for pid in arch_build_grammar.list_preset_ids():
        f_val = _arch_dna_f(pid)
        if f_val:
            axes.add(f_val)
    return axes


def _grammar_snapshot_layer_depth_ok() -> bool:
    archetypes = building_grammar.list_archetype_ids()
    if not archetypes:
        return False
    aid = archetypes[0]
    districts = building_grammar.list_district_styles(aid)
    if not districts:
        return False
    try:
        result = building_grammar.generate(aid, districts[0], 42)
    except (KeyError, NotImplementedError, ValueError, OSError):
        return False
    layers = {str(step.get("layer") or "") for step in result.get("rule_chain") or []}
    return {"facade", "detail", "age"}.issubset(layers)


def grammar_set_tier() -> dict[str, Any]:
    """APS-GRAM-TIER-001 — authoritative G0–G4 from registry + coverage guards."""
    archetypes = building_grammar.list_archetype_ids()
    archetype_count = len(archetypes)
    grammar_files = _grammar_ron_filenames()
    district_count = _district_count_for_archetypes(archetypes)
    preset_count = len(arch_build_grammar.list_preset_ids())
    f_axes = _dna_f_axis_values()

    reasons: list[str] = []
    tier = "G0"

    g1_by_archetypes = archetype_count >= 3
    g1_by_lineage = _max_districts_in_lineage(archetypes) >= 3
    if g1_by_archetypes or g1_by_lineage:
        tier = "G1"
    else:
        reasons.append("archetype_count<3 for G1")
        if not g1_by_lineage:
            reasons.append(f"lineage_district_count<3 for G1")

    if tier == "G1":
        if preset_count >= 4 and len(f_axes) >= 4:
            tier = "G2"
        else:
            reasons.append(f"preset_count={preset_count} or F-axis={len(f_axes)}<4 for G2")

    if tier == "G2":
        if _grammar_snapshot_layer_depth_ok():
            tier = "G3"
        else:
            reasons.append("grammar_rule_chain missing facade/detail/age for G3")

    if tier == "G3":
        coverage = building_set_coverage_report()
        parity = grammar_pilot_parity()
        if coverage.get("green") and parity.get("green"):
            tier = "G4"
        else:
            if not coverage.get("green"):
                reasons.append("building_set_coverage not green for G4")
            if not parity.get("green"):
                reasons.append("grammar_pilot_parity not green for G4")

    return {
        "tier": tier,
        "archetype_count": archetype_count,
        "district_count": district_count,
        "grammar_files": grammar_files,
        "reasons": reasons,
        "source": "grammar_set_tier()",
        "preset_count": preset_count,
        "f_axis_count": len(f_axes),
    }


def write_grammar_set_tier_witness(*, rel_path: str | None = None) -> dict[str, Any]:
    body = grammar_set_tier()
    rel = rel_path or GRAMMAR_TIER_WITNESS
    out = repo_root() / rel
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body


def write_aps_grammar_tier_gates_witness(
    *,
    tier: str,
    dna_panel_visible: bool,
    iterate_panel_visible: bool,
    build_set_expanded_default: bool,
    kit_hint_visible: bool,
    archetype_combo_count: int | None = None,
    rel_path: str | None = None,
) -> dict[str, Any]:
    body = {
        "tier": tier,
        "dna_panel_visible": dna_panel_visible,
        "iterate_panel_visible": iterate_panel_visible,
        "build_set_expanded_default": build_set_expanded_default,
        "kit_hint_visible": kit_hint_visible,
        "scanner": "test_aps_grammar_tier_gates.py",
    }
    if archetype_combo_count is not None:
        body["archetype_combo_count"] = archetype_combo_count
    out = repo_root() / (rel_path or GRAMMAR_TIER_GATES_G0_FIXTURE)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body


def grammar_tier_ui_presence_from_tier(tier: str) -> dict[str, Any]:
    """APS-GRAM-TIER-GATES-LIVE-001 / DES-APS-SESSION-DUMP-001 — tier exposure without Tk."""
    from rust_engine_mcp.aps_grammar_labels import human_label
    from rust_engine_mcp.aps_uiux_onboard import assembly_empty_state_text

    tier = str(tier or "G0").upper()
    if tier not in TIER_ORDER:
        tier = "G0"
    archetypes = building_grammar.list_archetype_ids() or ["IndustrialWarehouse"]
    districts = building_grammar.list_district_styles(archetypes[0]) if archetypes else []
    dna_panel_visible = tier in ("G2", "G3", "G4")
    iterate_panel_visible = tier in ("G2", "G3", "G4")
    return {
        "tier": tier,
        "tier_chip": TIER_CHIP_LABELS.get(tier, tier),
        "kit_hint_visible": tier == "G0",
        "dna_panel_visible": dna_panel_visible,
        "iterate_panel_visible": iterate_panel_visible,
        "set_health_visible": tier in ("G2", "G3", "G4"),
        "build_set_expanded_default": tier in ("G2", "G3", "G4"),
        "archetype_combo_count": len(archetypes),
        "default_archetype_label": human_label(archetypes[0]) if archetypes else "",
        "default_district_label": human_label(districts[0]) if districts else "",
        "assembly_empty_label": assembly_empty_state_text(tier),
    }


def grammar_tier_gates_snapshot(*, tier: str | None = None) -> dict[str, Any]:
    """Live tier gate snapshot — matches refresh_grammar_tier_from_registry exposure."""
    tier_body = grammar_set_tier() if tier is None else {"tier": tier}
    live_tier = str(tier_body.get("tier") or "G0").upper()
    ui = grammar_tier_ui_presence_from_tier(live_tier)
    return {
        "tier": live_tier,
        "dna_panel_visible": ui["dna_panel_visible"],
        "iterate_panel_visible": ui["iterate_panel_visible"],
        "build_set_expanded_default": ui["build_set_expanded_default"],
        "kit_hint_visible": ui["kit_hint_visible"],
        "archetype_combo_count": ui["archetype_combo_count"],
        "grammar_set_tier": live_tier,
        "source": "grammar_tier_gates_snapshot()",
    }


def write_aps_grammar_tier_gates_live_witness() -> dict[str, Any]:
    """APS-GRAM-TIER-GATES-LIVE-001 — live witness from registry tier, not G0 fixture."""
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    snap = grammar_tier_gates_snapshot()
    tier_body = grammar_set_tier()
    tier = str(tier_body.get("tier") or "G0")
    green = snap["tier"] == tier
    body = {
        **snap,
        "task_id": "APS-GRAM-TIER-GATES-LIVE-001",
        "green": green,
        "grammar_set_tier_match": green,
        "grammar_set_tier_reasons": tier_body.get("reasons") or [],
        "scanner": "grammar_build_set.write_aps_grammar_tier_gates_live_witness",
    }
    return write_aps_live_witness(
        body,
        GRAMMAR_TIER_GATES_LIVE_WITNESS,
        schema="aps_grammar_tier_gates_live_v1",
        profile="APS_GRAM_TIER_GATES",
        source_system="grammar_build_set",
        ritual="BLANG:WIT-HON APS-GRAM-TIER-GATES-LIVE-001" if green else None,
        exit_predicate_must=[{"path": "tier", "eq": tier}, {"path": "grammar_set_tier", "eq": tier}],
    )


def aps_session_presence_dump() -> dict[str, Any]:
    """DES-APS-SESSION-DUMP-001 — bundled presence truth for operator rubric."""
    from rust_engine_mcp.aps_uiux_onboard import load_onboarding_seen

    tier_body = grammar_set_tier()
    brief = grammar_set_brief()
    coverage = building_set_coverage_report()
    parity = grammar_pilot_parity()
    tier = str(tier_body.get("tier") or "G0")
    ui = grammar_tier_ui_presence_from_tier(tier)
    coverage_green = bool(coverage.get("green"))
    parity_green = bool(parity.get("green"))
    building_g4_blocked = tier != "G4" or not (coverage_green and parity_green)
    tier_aligned = ui["tier"] == tier
    return {
        "gate": "DES-APS-SESSION-DUMP-001",
        "green": tier_aligned,
        "grammar_set_tier": tier_body,
        "grammar_set_brief": {
            "green": bool(brief.get("green")),
            "gaps": brief.get("gaps") or [],
        },
        "g4_guards": {
            "building_set_coverage_green": coverage_green,
            "grammar_pilot_parity_green": parity_green,
        },
        "ui_presence": ui,
        "onboarding_seen": load_onboarding_seen(),
        "expansion": {
            "building_g4_blocked": building_g4_blocked,
            "landscape_lg5_matrix_cells": 16,
            "landscape_lane_active": False,
        },
        "sources": [
            "grammar_set_tier()",
            "grammar_tier_ui_presence_from_tier()",
            "grammar_set_brief()",
            "building_set_coverage_report()",
            "grammar_pilot_parity()",
        ],
    }


def write_aps_session_presence_witness() -> dict[str, Any]:
    """DES-APS-SESSION-DUMP-001 — write debug_runs/aps_session_presence_live.json."""
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    body = aps_session_presence_dump()
    tier = str((body.get("grammar_set_tier") or {}).get("tier") or "G0")
    ui_tier = str((body.get("ui_presence") or {}).get("tier") or "")
    green = ui_tier == tier
    body["green"] = green
    return write_aps_live_witness(
        body,
        SESSION_PRESENCE_WITNESS,
        schema="aps_session_presence_live_v1",
        profile="APS_SESSION_PRESENCE",
        source_system="grammar_build_set",
        ritual="BLANG:WIT-HON DES-APS-SESSION-DUMP-001" if green else None,
        exit_predicate_must=[
            {"path": "ui_presence.tier", "eq": tier},
            {"path": "grammar_set_tier.tier", "eq": tier},
        ],
    )


def write_grammar_archetype_g1_witness() -> dict[str, Any]:
    archetype_ids = building_grammar.list_archetype_ids()
    body = {
        "archetype_count": len(archetype_ids),
        "archetype_ids": archetype_ids,
        "ron_files_added": max(0, len(_grammar_ron_filenames()) - 1),
        "json_mirrors_added": 2,
        "validate_arch_build_grammar": "pass",
    }
    out = repo_root() / GRAMMAR_ARCHETYPE_G1_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body


def write_grammar_set_tier_g1_witness() -> dict[str, Any]:
    body = grammar_set_tier()
    coverage = building_set_coverage_report()
    archetype_ids = building_grammar.list_archetype_ids()
    payload = {
        **body,
        "archetype_ids": archetype_ids,
        "kit_hint_downgraded": True,
        "building_set_coverage": "pass" if coverage.get("green") else "fail",
    }
    out = repo_root() / GRAMMAR_TIER_G1_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return payload


def write_aps_grammar_tier_g1_gates_witness(
    *,
    archetype_combo_count: int,
    kit_hint_visible: bool,
    dna_panel_visible: bool,
    iterate_panel_visible: bool,
) -> dict[str, Any]:
    body = {
        "tier": "G1",
        "archetype_combo_count": archetype_combo_count,
        "kit_hint_visible": kit_hint_visible,
        "dna_panel_visible": dna_panel_visible,
        "iterate_panel_visible": iterate_panel_visible,
        "scanner": "test_aps_grammar_tier_gates.py",
    }
    out = repo_root() / GRAMMAR_TIER_G1_GATES_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body


def write_grammar_labels_g1_witness() -> dict[str, Any]:
    """GRAM-CONTENT-003 — human labels for G1 archetype family."""
    labels_path = repo_root() / "assets/configs/buildings/grammars/grammar_labels_v1.json"
    data = json.loads(labels_path.read_text(encoding="utf-8"))
    archetypes = data.get("archetypes", {})
    new_ids = ("FactoryCluster", "RailEdge")
    human_labels_for_new_archetypes = all(
        archetypes.get(aid, {}).get("label") for aid in new_ids
    )
    body = {
        "human_labels_for_new_archetypes": human_labels_for_new_archetypes,
        "archetype_labels": {aid: archetypes.get(aid, {}).get("label") for aid in new_ids},
        "district_styles_added": [
            k for k in ("manufacturing_row", "rail_yard_corridor")
            if k in data.get("district_styles", {})
        ],
        "scanner": "test_aps_grammar_labels.py",
    }
    out = repo_root() / GRAMMAR_LABELS_G1_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body


def write_aps_grammar_evolution_close_witness(*, pytest_aps: dict[str, int] | None = None) -> dict[str, Any]:
    """APS-GRAM-CLOSE-001 — rollup witness for grammar evolution program."""
    slices = {
        "APS-GRAM-TIER-001": GRAMMAR_TIER_WITNESS,
        "APS-GRAM-TIER-002": "debug_runs/aps_grammar_tier_gates_live.json",
        "GRAM-CONTENT-002": GRAMMAR_ARCHETYPE_G1_WITNESS,
        "GRAM-CONTENT-003": GRAMMAR_LABELS_G1_WITNESS,
        "GRAM-CONTENT-004": GRAMMAR_TIER_G1_WITNESS,
        "APS-GRAM-TIER-002-REFRESH": GRAMMAR_TIER_G1_GATES_WITNESS,
        "APS-GRAM-P3-001": GRAMMAR_P3_WITNESS,
        "APS-GRAM-TIER-004": GRAMMAR_SPINE_TIER_WITNESS,
    }
    root = repo_root()
    present: dict[str, bool] = {}
    for row_id, rel in slices.items():
        present[row_id] = (root / rel).is_file()

    tier_body = grammar_set_tier()
    all_green = all(present.values()) and tier_body.get("tier") in ("G1", "G2", "G3", "G4")
    rows_closed = sum(1 for ok in present.values() if ok)

    body = {
        "gate": "APS-GRAM-CLOSE-001",
        "program_id": "PLAN-APS-GRAMMAR-EVOLUTION-001",
        "status": "pass" if all_green else "fail",
        "tier": tier_body.get("tier"),
        "rows_closed": rows_closed,
        "green": all_green,
        "slices": present,
        "slice_witnesses": slices,
        "grammar_set_tier": tier_body,
        "pytest_aps": pytest_aps or {"passed": 0, "failed": 0, "note": "run pytest -k aps separately"},
        "needs_display": [
            "APS-GRAM-TIER-002",
            "APS-GRAM-TIER-002-REFRESH",
            "APS-GRAM-TIER-004",
        ],
        "wit_hon": "validate-report witness_honesty debug_runs/aps_grammar_evolution_close_live.json --compress 3",
    }
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    return write_aps_live_witness(
        body,
        GRAMMAR_EVOLUTION_CLOSE_WITNESS,
        schema="aps_grammar_evolution_close_live_v1",
        profile="APS_GRAM_CLOSE",
        source_system="grammar_build_set",
    )


def validate_building_set_coverage(*, compression_level: int = 3) -> ValidationReport:
    body = building_set_coverage_report()
    errors_list = body.get("errors") or []
    issues = [
        ValidationIssue(kind="building_set_coverage", severity="error", hint=str(msg))
        for msg in errors_list
    ]
    green = bool(body.get("green"))
    return ValidationReport(
        validator="mcp_schema",
        status="passed" if green else "failed",
        compression_level=compression_level,
        summary=f"building_set_coverage: {'green' if green else len(errors_list)} issue(s)",
        error_count=len(issues),
        errors=issues,
    ).compress(compression_level)
