"""BUILD-READ-GRAMMAR-v0 — ARCH-DNA presets + β pressure field for APS snapshots."""

from __future__ import annotations

import json
from copy import deepcopy
from pathlib import Path
from typing import Any

from .paths import repo_root, schemas_dir
from .schemas import validate_arch_build_grammar

BETA_KEYS: tuple[str, ...] = (
    "beta_sym",
    "beta_irr",
    "beta_yard",
    "beta_svc",
    "beta_mod",
    "beta_exp",
    "beta_vert",
    "beta_roof",
)

BETA_LABELS: dict[str, str] = {
    "beta_sym": "Symmetry (βsym)",
    "beta_irr": "Irregularity (βirr)",
    "beta_yard": "Yard pressure (βyard)",
    "beta_svc": "Service (βsvc)",
    "beta_mod": "Module density (βmod)",
    "beta_exp": "Expansion (βexp)",
    "beta_vert": "Verticality (βvert)",
    "beta_roof": "Roof articulation (βroof)",
}

DNA_KEYS: tuple[str, ...] = ("F", "L", "C", "D", "W", "I", "S", "P", "M", "A")

DNA_LABELS: dict[str, str] = {
    "F": "Function",
    "L": "Lineage",
    "C": "Climate",
    "D": "Density",
    "W": "Wealth",
    "I": "Infrastructure",
    "S": "Security",
    "P": "Philosophy",
    "M": "Material",
    "A": "Age",
}

DEFAULT_PRESET_ID = "logistics_rail_warehouse_v0"
WITNESS_PATH = "debug_runs/aps_build_read_grammar_v0_002_live.json"
CONSUMER_WITNESS_PATH = "debug_runs/aps_dna_consumer_contract_live.json"

# @coder consumer — read these snapshot keys (BUILD-READ-CONSUMER-MCP-001)
SNAPSHOT_FIELD_PRESET = "arch_build_grammar_preset_id"
SNAPSHOT_FIELD_GRAMMAR = "arch_build_grammar_id"
SNAPSHOT_FIELD_DNA = "arch_dna"
SNAPSHOT_FIELD_PRESSURE = "pressure_field"
SNAPSHOT_CONSUMER_FIELDS = (
    SNAPSHOT_FIELD_PRESET,
    SNAPSHOT_FIELD_GRAMMAR,
    SNAPSHOT_FIELD_DNA,
    SNAPSHOT_FIELD_PRESSURE,
)
RUST_CONSUMER_HINT = "src/construction/procedural/arch_build_grammar_v0.rs::load_arch_dna_preset"


def presets_dir() -> Path:
    return schemas_dir() / "examples"


def preset_path(preset_id: str) -> Path:
    pid = str(preset_id or "").strip()
    if not pid:
        raise ValueError("preset_id required")
    direct = presets_dir() / f"arch_dna_{pid}.json"
    if direct.is_file():
        return direct
    for path in sorted(presets_dir().glob("arch_dna_*.json")):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        if str(data.get("preset_id") or "") == pid:
            return path
    raise FileNotFoundError(f"arch_build_grammar preset not found: {preset_id}")


def list_preset_ids(*, catalog_only: bool = True) -> list[str]:
    if catalog_only:
        try:
            from .grammar_build_set import load_pilot_catalog

            ids: list[str] = []
            for row in load_pilot_catalog():
                pid = str(row.get("arch_dna_preset") or "").strip()
                if pid and pid not in ids:
                    ids.append(pid)
            if ids:
                return ids
        except (ImportError, OSError, ValueError, KeyError):
            pass
    out: list[str] = []
    for path in sorted(presets_dir().glob("arch_dna_*.json")):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        pid = str(data.get("preset_id") or path.stem.removeprefix("arch_dna_"))
        if pid and pid not in out:
            out.append(pid)
    return out or [DEFAULT_PRESET_ID]


def load_preset(preset_id: str) -> dict[str, Any]:
    path = preset_path(preset_id)
    raw = json.loads(path.read_text(encoding="utf-8"))
    data = {k: v for k, v in raw.items() if k != "_meta"}
    validate_arch_build_grammar(data)
    return data


def default_preset_id() -> str:
    ids = list_preset_ids()
    if DEFAULT_PRESET_ID in ids:
        return DEFAULT_PRESET_ID
    return ids[0]


def clamp_beta(value: float) -> float:
    return max(0.0, min(1.0, round(float(value), 4)))


def normalize_pressure_field(raw: dict[str, Any] | None) -> dict[str, float]:
    src = raw if isinstance(raw, dict) else {}
    return {key: clamp_beta(src.get(key, 0.0)) for key in BETA_KEYS}


def extract_from_snapshot(snapshot: dict[str, Any]) -> dict[str, Any]:
    snap = snapshot if isinstance(snapshot, dict) else {}
    preset_id = str(
        snap.get("arch_build_grammar_preset_id")
        or snap.get("arch_dna_preset_id")
        or ""
    ).strip()
    arch_dna = snap.get("arch_dna") if isinstance(snap.get("arch_dna"), dict) else {}
    pressure = normalize_pressure_field(
        snap.get("pressure_field") if isinstance(snap.get("pressure_field"), dict) else None
    )
    grammar_id = str(snap.get("arch_build_grammar_id") or "").strip()
    if not preset_id and arch_dna and pressure:
        preset_id = default_preset_id()
    return {
        "preset_id": preset_id or default_preset_id(),
        "arch_dna": dict(arch_dna),
        "pressure_field": pressure,
        "grammar_id": grammar_id,
        "enabled": bool(arch_dna and any(arch_dna.values())),
    }


def apply_to_snapshot(
    snapshot: dict[str, Any],
    *,
    preset_id: str,
    pressure_field: dict[str, float] | None = None,
    arch_dna: dict[str, str] | None = None,
    include: bool = True,
) -> dict[str, Any]:
    out = deepcopy(snapshot)
    if not include:
        for key in (
            "arch_build_grammar_preset_id",
            "arch_dna_preset_id",
            "arch_build_grammar_id",
            "arch_dna",
            "pressure_field",
        ):
            out.pop(key, None)
        return out
    preset = load_preset(preset_id)
    out["arch_build_grammar_preset_id"] = str(preset["preset_id"])
    out["arch_dna"] = deepcopy(arch_dna or preset["arch_dna"])
    out["pressure_field"] = normalize_pressure_field(pressure_field or preset["pressure_field"])
    grammar_id = preset.get("grammar_id")
    if grammar_id:
        out["arch_build_grammar_id"] = str(grammar_id)
    return out


def consumer_contract() -> dict[str, Any]:
    """BUILD-READ-CONSUMER-MCP-001 — @coder wiring contract for APS snapshot DNA+β."""
    return {
        "schema": "aps_dna_consumer_contract_v1",
        "task_id": "BUILD-READ-CONSUMER-MCP-001",
        "producer": "BUILD-READ-GRAMMAR-v0-002",
        "snapshot_fields": list(SNAPSHOT_CONSUMER_FIELDS),
        "beta_keys": list(BETA_KEYS),
        "dna_keys": list(DNA_KEYS),
        "rust_consumer": RUST_CONSUMER_HINT,
        "aps_ui": {
            "panel": "tools/mcp/art_pipeline_suite/grammar_dna_panel.py",
            "apply_hook": "assembly_panel._apply_grammar_dna_from_ui",
        },
        "mcp_tools": [
            "arch_dna_snapshot_brief(path)",
            "snapshot_digest(path) — includes arch_dna block",
            "apply_to_snapshot(snapshot, preset_id=..., pressure_field=...)",
        ],
        "preset_ids": list_preset_ids(),
    }


def arch_dna_snapshot_brief(path: str | Path) -> dict[str, Any]:
    """Compressed ARCH-DNA + β rollup from assembly snapshot JSON."""
    p = Path(path)
    if not p.is_absolute():
        p = repo_root() / path
    if not p.is_file():
        return {
            "schema": "arch_dna_snapshot_brief_v1",
            "ok": False,
            "path": str(path),
            "error": "Snapshot not found",
        }
    try:
        rel = str(p.relative_to(repo_root())).replace("\\", "/")
    except ValueError:
        rel = str(p)
    snap = json.loads(p.read_text(encoding="utf-8"))
    extracted = extract_from_snapshot(snap)
    pressure = extracted.get("pressure_field") or {}
    arch_dna = extracted.get("arch_dna") or {}
    f_axis = str(arch_dna.get("F") or "")
    wired = bool(arch_dna and any(arch_dna.values()) and any(pressure.values()))
    return {
        "schema": "arch_dna_snapshot_brief_v1",
        "ok": True,
        "path": rel,
        "wired": wired,
        "preset_id": extracted.get("preset_id"),
        "grammar_id": extracted.get("grammar_id"),
        "f_axis": f_axis,
        "arch_dna": {k: arch_dna.get(k) for k in DNA_KEYS if arch_dna.get(k)},
        "pressure_field": pressure,
        "beta_summary": ", ".join(f"{k.removeprefix('beta_')}={pressure.get(k, 0):.2f}" for k in BETA_KEYS[:4]),
        "hint": "Consumer ready" if wired else "Enable DNA panel + Save snapshot in APS",
    }


def write_aps_dna_consumer_witness() -> dict[str, Any]:
    contract = consumer_contract()
    sample = arch_dna_snapshot_brief(
        "tools/mcp/schemas/examples/assembly_snapshot_rail_warehouse_pilot_v1.json"
    )
    all_presets_ok = True
    preset_errors: list[str] = []
    for pid in list_preset_ids():
        try:
            load_preset(pid)
        except (FileNotFoundError, ValueError, OSError) as exc:
            all_presets_ok = False
            preset_errors.append(f"{pid}: {exc}")
    body: dict[str, Any] = {
        "gate_id": "BUILD-READ-CONSUMER-MCP-001",
        "task_id": "BUILD-READ-CONSUMER-MCP-001",
        "depends_on": "BUILD-READ-GRAMMAR-v0-002",
        "ok": all_presets_ok and bool(sample.get("wired")),
        "green": all_presets_ok and bool(sample.get("wired")),
        "consumer_contract": contract,
        "sample_snapshot_brief": sample,
        "preset_load_errors": preset_errors,
        "unblocks": "BUILD-READ-CONSUMER-MCP-001 @coder Rust consumer wiring",
    }
    out = repo_root() / CONSUMER_WITNESS_PATH
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = str(out.relative_to(repo_root())).replace("\\", "/")
    return body


def write_build_read_grammar_v0_002_witness() -> dict[str, Any]:
    preset_id = default_preset_id()
    preset = load_preset(preset_id)
    body: dict[str, Any] = {
        "gate_id": "BUILD-READ-GRAMMAR-v0-002",
        "ops_id": "OPS-006",
        "task_id": "BUILD-READ-GRAMMAR-v0-002",
        "program_id": "art_A",
        "owner": "@coder-mcp",
        "ok": True,
        "green": True,
        "preset_id": preset_id,
        "preset_count": len(list_preset_ids()),
        "consumer_witness": CONSUMER_WITNESS_PATH,
        "consumer_unblocked": True,
        "beta_keys": list(BETA_KEYS),
        "arch_dna_keys": list(DNA_KEYS),
        "ui": {
            "panel": "tools/mcp/art_pipeline_suite/grammar_dna_panel.py",
            "assembly_hook": "tools/mcp/art_pipeline_suite/assembly_panel.py",
            "snapshot_fields": [
                "arch_build_grammar_preset_id",
                "arch_dna",
                "pressure_field",
                "arch_build_grammar_id",
            ],
        },
        "preset_sample": {
            "arch_dna": preset.get("arch_dna"),
            "pressure_field": preset.get("pressure_field"),
        },
    }
    out = repo_root() / WITNESS_PATH
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body
