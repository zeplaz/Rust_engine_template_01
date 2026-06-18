"""CMCP-GRAMMAR-FACILITY-BRIEF-001 — join grammar + catalog + supply chain → JSON brief."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from . import building_grammar
from .paths import repo_root
from .power_tier_bands import power_tier_from_units

CHAINS_REL = "assets/configs/industrial_supply_chains.json"
BUILDINGS_DIR = "assets/configs/buildings"
GRAMMARS_DIR = "assets/configs/buildings/grammars"
WITNESS_REL = "debug_runs/grammar_facility_brief_live.json"


def _read_json(rel: str) -> dict[str, Any]:
    path = repo_root() / rel
    return json.loads(path.read_text(encoding="utf-8"))


def load_supply_chains() -> dict[str, Any]:
    body = _read_json(CHAINS_REL)
    return dict(body.get("chains") or {})


def load_catalog(catalog_id: str) -> dict[str, Any]:
    path = repo_root() / BUILDINGS_DIR / f"{catalog_id}.json"
    if not path.is_file():
        raise FileNotFoundError(f"catalog missing: {catalog_id}")
    return json.loads(path.read_text(encoding="utf-8"))


def list_grammar_ids() -> list[str]:
    root = repo_root() / GRAMMARS_DIR
    if not root.is_dir():
        return []
    ids: list[str] = []
    for ron in sorted(root.glob("*.ron")):
        grammar = building_grammar._load_grammar_ron(ron)  # noqa: SLF001 — shared mirror loader
        gid = str(grammar.get("grammar_id") or "")
        if gid:
            ids.append(gid)
    return ids


def load_grammar(grammar_id: str) -> dict[str, Any]:
    for gid in list_grammar_ids():
        if gid == grammar_id:
            root = repo_root() / GRAMMARS_DIR
            for ron in sorted(root.glob("*.ron")):
                grammar = building_grammar._load_grammar_ron(ron)  # noqa: SLF001
                if str(grammar.get("grammar_id")) == grammar_id:
                    return grammar
    raise KeyError(f"unknown grammar_id: {grammar_id}")


def _find_chain_step(
    chains: dict[str, Any],
    *,
    chain_id: str,
    catalog_id: str,
    role: str,
) -> tuple[dict[str, Any] | None, int | None]:
    chain = chains.get(chain_id)
    if not isinstance(chain, dict):
        return None, None
    steps = chain.get("steps") or []
    for idx, step in enumerate(steps):
        if not isinstance(step, dict):
            continue
        if str(step.get("catalog_id")) == catalog_id or str(step.get("role")) == role:
            return step, idx
    return None, None


def _catalog_summary(catalog: dict[str, Any], catalog_id: str) -> dict[str, Any]:
    power = float(catalog.get("power_consumption") or 0)
    gen = float(catalog.get("power_generation") or 0)
    utility = catalog.get("utility_role")
    utility_s = str(utility) if utility else None
    produces = list(catalog.get("produces_resources") or catalog.get("produces") or [])
    consumes = list(catalog.get("consumes_resources") or catalog.get("consumes") or [])
    return {
        "catalog_id": catalog_id,
        "asset_name": catalog.get("asset_name"),
        "supply_chain_role": catalog.get("supply_chain_role"),
        "utility_role": utility_s,
        "power_consumption": power,
        "power_generation": gen,
        "footprint": {
            "w": int(catalog.get("building_size_x") or 0),
            "d": int(catalog.get("building_size_y") or 0),
        },
        "produces": produces,
        "consumes": consumes,
        "produces_top3": produces[:3],
        "consumes_top3": consumes[:3],
        "power_tier": power_tier_from_units(power, utility_role=utility_s, power_generation=gen),
    }


def join_grammar_facility_brief(grammar: dict[str, Any]) -> dict[str, Any]:
    """Join one grammar document to catalog + chain rows."""
    grammar_id = str(grammar.get("grammar_id") or "")
    archetype_id = str((grammar.get("archetype") or {}).get("id") or "")
    binding = grammar.get("facility_binding")
    errors: list[str] = []
    gaps: list[str] = []

    if not isinstance(binding, dict):
        return {
            "grammar_id": grammar_id,
            "archetype_id": archetype_id,
            "ok": False,
            "green": False,
            "facility_binding": None,
            "gaps": ["no facility_binding on grammar"],
            "errors": [],
        }

    catalog_id = str(binding.get("catalog_id") or "")
    chain_id = str(binding.get("chain_id") or "")
    role = str(binding.get("supply_chain_role") or "")
    binding_tier = str(binding.get("power_tier") or "")

    try:
        catalog = load_catalog(catalog_id)
    except FileNotFoundError as exc:
        errors.append(str(exc))
        catalog = {}

    chains = load_supply_chains()
    chain_body = chains.get(chain_id) if chain_id else None
    step, step_index = _find_chain_step(chains, chain_id=chain_id, catalog_id=catalog_id, role=role)

    if chain_id and chain_body is None:
        errors.append(f"unknown chain_id: {chain_id}")
    if catalog and step is None and chain_id:
        errors.append(f"no chain step for catalog_id={catalog_id!r} role={role!r} in {chain_id}")

    catalog_summary = _catalog_summary(catalog, catalog_id) if catalog else {}
    derived_tier = catalog_summary.get("power_tier") if catalog_summary else None

    if catalog:
        cat_role = str(catalog.get("supply_chain_role") or "")
        if role and cat_role and role != cat_role:
            errors.append(f"binding role {role!r} != catalog role {cat_role!r}")
        if step and str(step.get("role")) != role:
            errors.append(f"binding role {role!r} != chain step role {step.get('role')!r}")

    tier_match = derived_tier == binding_tier if derived_tier and binding_tier else False
    if derived_tier and binding_tier and not tier_match:
        errors.append(f"binding power_tier {binding_tier!r} != catalog-derived {derived_tier!r}")

    chain_summary: dict[str, Any] | None = None
    if isinstance(chain_body, dict) and step is not None:
        chain_summary = {
            "chain_id": chain_id,
            "display_name": chain_body.get("display_name"),
            "step_index": step_index,
            "role": step.get("role"),
            "catalog_id": step.get("catalog_id"),
            "power_consumption": step.get("power_consumption"),
            "produces": step.get("produces") or [],
            "consumes": step.get("consumes") or [],
        }

    green = not errors and not gaps and bool(catalog_summary) and bool(chain_summary) and tier_match
    return {
        "grammar_id": grammar_id,
        "archetype_id": archetype_id,
        "ok": not errors,
        "green": green,
        "facility_binding": binding,
        "catalog": catalog_summary or None,
        "chain": chain_summary,
        "derived": {
            "power_tier_from_catalog": derived_tier,
            "power_tier_binding_match": tier_match,
            "site_template_id": binding.get("site_template_id"),
            "program_axes": binding.get("program_axes"),
        },
        "io_summary": {
            "produces_top3": (catalog_summary or {}).get("produces_top3") or [],
            "consumes_top3": (catalog_summary or {}).get("consumes_top3") or [],
        },
        "errors": errors,
        "gaps": gaps,
    }


def grammar_facility_brief(*, grammar_id: str | None = None) -> dict[str, Any]:
    """Build facility brief for one grammar or inventory of all grammars."""
    if grammar_id:
        briefs = [join_grammar_facility_brief(load_grammar(grammar_id))]
    else:
        briefs = [join_grammar_facility_brief(load_grammar(gid)) for gid in list_grammar_ids()]

    bound = [b for b in briefs if b.get("facility_binding")]
    green_bound = [b for b in bound if b.get("green")]
    errors = [e for b in briefs for e in b.get("errors") or []]

    return {
        "task_id": "CMCP-GRAMMAR-FACILITY-BRIEF-001",
        "ok": True,
        "green": bool(bound) and len(green_bound) == len(bound) and not errors,
        "grammar_id": grammar_id,
        "grammar_count": len(briefs),
        "binding_count": len(bound),
        "green_binding_count": len(green_bound),
        "briefs": briefs,
        "brief": briefs[0] if grammar_id and len(briefs) == 1 else None,
        "authority": {
            "chains": CHAINS_REL,
            "catalog_dir": BUILDINGS_DIR,
            "binding_schema": "src/dev/design_facility_binding_schema_v1.md",
            "power_tier_spec": "src/dev/design_power_tier_bands_v1.md",
        },
    }


def write_grammar_facility_brief_witness(*, grammar_id: str | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    body = grammar_facility_brief(grammar_id=grammar_id)
    ritual = "BLANG:WIT-HON CMCP-GRAMMAR-FACILITY-BRIEF-001" if body.get("green") else None
    return write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="grammar_facility_brief_live_v1",
        profile="CMCP_GRAMMAR_FACILITY_BRIEF",
        source_system="grammar_facility_brief",
        ritual=ritual,
    )
