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


def load_pilot_catalog() -> list[dict[str, Any]]:
    text = _read_asset(PILOT_CATALOG_RON)
    pilots: list[dict[str, Any]] = []
    for block in _split_ron_entries(text):
        pid = _ron_field(block, "id")
        if not pid:
            continue
        pilots.append(
            {
                "id": pid,
                "label": _ron_field(block, "label") or pid,
                "mock_shape_id": _ron_field(block, "mock_shape_id"),
                "arch_dna_preset": _ron_field(block, "arch_dna_preset"),
                "grammar_archetype_id": _ron_field(block, "grammar_archetype_id"),
                "district_style": _ron_field(block, "district_style"),
                "site_json_path": _ron_field(block, "site_json_path"),
                "pilot_kind": _ron_field_raw(block, "pilot_kind") or "shape_qa",
                "hover_hint": _ron_field(block, "hover_hint"),
            }
        )
    return pilots


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
    pilots = load_pilot_catalog()
    presets = arch_build_grammar.list_preset_ids()
    grammar_pilots = [p for p in pilots if p.get("pilot_kind") == "grammar"]
    shape_pilots = [p for p in pilots if p.get("pilot_kind") == "shape_qa"]
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
        "lines": hist_lines,
        "text": "\n".join(hist_lines[:12]),
    }
    return body


def grammar_pilot_parity() -> dict[str, Any]:
    """MCP-GRAMMAR-SET-004 — MCP wrap of catalog parity checks."""
    errors: list[str] = []
    pilots = load_pilot_catalog()
    if len(pilots) < 8:
        errors.append(f"pilot_count={len(pilots)} expected ≥8")
    grammar = [p for p in pilots if p.get("pilot_kind") == "grammar"]
    shape = [p for p in pilots if p.get("pilot_kind") == "shape_qa"]
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
    pilots = {p["id"]: p for p in load_pilot_catalog()}
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
        "pilot_hardcode_green": hardcode.get("green"),
    }
    return body


def building_set_health_brief() -> dict[str, Any]:
    """MCP-BUILD-SET-003 / OPS-BUILD-SET-001 — rollup for OPS + APS."""
    brief = grammar_set_brief()
    coverage = building_set_coverage_report()
    parity = grammar_pilot_parity()
    hardcode = pilot_hardcode_lint()
    return {
        "green": brief.get("green") and coverage.get("green") and parity.get("green"),
        "grammar_set_brief": brief.get("text"),
        "coverage_green": coverage.get("green"),
        "parity_green": parity.get("green"),
        "hardcode_green": hardcode.get("green"),
        "grammar_pilot_count": brief.get("counts", {}).get("grammar_pilots"),
        "preset_count": brief.get("counts", {}).get("arch_dna_presets"),
        "gaps": brief.get("gaps") or [],
        "coverage_errors": coverage.get("errors") or [],
    }


def write_building_set_coverage_witness() -> dict[str, Any]:
    body = building_set_coverage_report()
    out = repo_root() / COVERAGE_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body


def write_grammar_set_brief_witness() -> dict[str, Any]:
    body = grammar_set_brief()
    out = repo_root() / GRAMMAR_SET_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body


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
