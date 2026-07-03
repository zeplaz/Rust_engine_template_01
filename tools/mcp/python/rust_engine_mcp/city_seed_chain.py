"""CITY-G1-C4-001 — Python parity for settlement seed chain (Rust: seed_chain.rs)."""

from __future__ import annotations

import hashlib
import json
from typing import Any

from . import assembly, building_grammar

DEFAULT_WORLD_SEED = 99_001
DEFAULT_TOWN_ID = "portland"
CITY_G1_C4_WIT_BLOCK = "industrial_west_b01"
CITY_G1_C4_WIT_LOT_IDX = 7
CITY_G1_C4_WIT_ARCHETYPE = "IndustrialWarehouse"
CITY_G1_C4_WIT_DISTRICT = "industrial_west"
CITY_G1_C4_LIVE_JSON = "debug_runs/city_g1_c4_001_live.json"


def mix_u64(parent: int, label: str, key: str) -> int:
    raw = f"{parent}:{label}:{key}".encode()
    digest = hashlib.sha256(raw).digest()
    return int.from_bytes(digest[:8], "little", signed=False)


def town_seed(world_seed: int, town_id: str) -> int:
    return mix_u64(world_seed, "town", town_id)


def block_seed(parent_town_seed: int, block_id: str) -> int:
    return mix_u64(parent_town_seed, "block", block_id)


def lot_seed(parent_block_seed: int, lot_idx: int) -> int:
    return mix_u64(parent_block_seed, "lot", str(lot_idx))


def building_grammar_seed(parent_lot_seed: int, archetype_id: str) -> int:
    return mix_u64(parent_lot_seed, "building_grammar", archetype_id)


def building_grammar_seed_chain(
    world_seed: int,
    town_id: str,
    block_id: str,
    lot_idx: int,
    archetype_id: str,
) -> int:
    ts = town_seed(world_seed, town_id)
    bs = block_seed(ts, block_id)
    ls = lot_seed(bs, lot_idx)
    return building_grammar_seed(ls, archetype_id)


def lot_idx_from_site_id(site_id: int) -> int:
    return site_id & 0xFFFF_FFFF


def block_id_for_site(site_id: int) -> str:
    return f"site_block_{site_id}"


def building_grammar_seed_for_site(
    world_seed: int, site_id: int, archetype_id: str
) -> int:
    return building_grammar_seed_chain(
        world_seed,
        DEFAULT_TOWN_ID,
        block_id_for_site(site_id),
        lot_idx_from_site_id(site_id),
        archetype_id,
    )


def witness_context() -> dict[str, Any]:
    return {
        "world_seed": DEFAULT_WORLD_SEED,
        "town_id": DEFAULT_TOWN_ID,
        "block_id": CITY_G1_C4_WIT_BLOCK,
        "lot_idx": CITY_G1_C4_WIT_LOT_IDX,
        "archetype_id": CITY_G1_C4_WIT_ARCHETYPE,
        "district_style": CITY_G1_C4_WIT_DISTRICT,
    }


def _assembly_snapshot_stable_hash(snapshot: dict[str, Any]) -> str:
    payload = json.dumps(snapshot, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(payload).hexdigest()


def build_city_g1_c4_001_witness_body() -> dict[str, Any]:
    ctx = witness_context()
    grammar_seed = building_grammar_seed_chain(
        ctx["world_seed"],
        ctx["town_id"],
        ctx["block_id"],
        ctx["lot_idx"],
        ctx["archetype_id"],
    )
    ts = town_seed(ctx["world_seed"], ctx["town_id"])
    bs = block_seed(ts, ctx["block_id"])
    ls = lot_seed(bs, ctx["lot_idx"])

    run_hashes: list[str] = []
    contract_ok = False
    for _ in range(3):
        grammar = building_grammar.generate(
            ctx["archetype_id"],
            ctx["district_style"],
            grammar_seed,
        )
        snap = assembly.generate_assembly_snapshot(
            archetype_id=ctx["archetype_id"],
            district_style=ctx["district_style"],
            seed=grammar_seed,
            grammar_result=grammar,
            write=False,
        )
        contract_ok = bool(snap.get("module_placements")) and bool(snap.get("assembly_id"))
        run_hashes.append(_assembly_snapshot_stable_hash(snap))

    three_run_stable = len(run_hashes) == 3 and len(set(run_hashes)) == 1
    green = three_run_stable and contract_ok

    return {
        "gate": "CMCP-CITY-G1-C4-001",
        "issue": "CITY-C4",
        "green": green,
        "three_run_stable": three_run_stable,
        "auto_001_contract": contract_ok,
        "chain_layers": {
            "town_seed": f"{ts:#018x}",
            "block_seed": f"{bs:#018x}",
            "lot_seed": f"{ls:#018x}",
            "building_grammar_seed": f"{grammar_seed:#018x}",
        },
        "stable_hash": run_hashes[0] if run_hashes else None,
        "run_hashes": run_hashes,
        "witness_context": ctx,
        "rules_check": {
            "passed": green,
            "blocked_by": [] if green else ["deterministic_output"],
            "seed": str(grammar_seed),
        },
    }


def write_city_g1_c4_001_witness() -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    body = build_city_g1_c4_001_witness_body()
    return write_aps_live_witness(
        body,
        CITY_G1_C4_LIVE_JSON,
        schema="city_g1_c4_seed_chain_live_v1",
        profile="CMCP_CITY_G1_C4",
        source_system="city_seed_chain",
        ritual="BLANG:WIT-HON CMCP-CITY-G1-C4-001" if body.get("green") else None,
    )
