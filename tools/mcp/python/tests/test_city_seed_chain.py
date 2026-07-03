"""CMCP-CITY-G1-C4-001 — city seed chain Python/Rust parity."""

from __future__ import annotations

from rust_engine_mcp import city_seed_chain


def test_mix_u64_deterministic() -> None:
    a = city_seed_chain.mix_u64(99_001, "town", "portland")
    b = city_seed_chain.mix_u64(99_001, "town", "portland")
    assert a == b
    assert a != city_seed_chain.mix_u64(99_001, "town", "seattle")


def test_witness_chain_layers_golden() -> None:
    ctx = city_seed_chain.witness_context()
    ts = city_seed_chain.town_seed(ctx["world_seed"], ctx["town_id"])
    bs = city_seed_chain.block_seed(ts, ctx["block_id"])
    ls = city_seed_chain.lot_seed(bs, ctx["lot_idx"])
    bg = city_seed_chain.building_grammar_seed(ls, ctx["archetype_id"])
    assert ts == 0x4AC870FAAB87F9AD
    assert bs == 0x490697089FF6F2F5
    assert ls == 0x59DCFEF41AF9F0F9
    assert bg == 0x035111798AD871AA


def test_site_derived_seed_stable() -> None:
    s = city_seed_chain.building_grammar_seed_for_site(99_001, 42, "rect_perimeter")
    assert s == city_seed_chain.building_grammar_seed_for_site(99_001, 42, "rect_perimeter")


def test_city_g1_c4_witness_body_green() -> None:
    body = city_seed_chain.build_city_g1_c4_001_witness_body()
    assert body["three_run_stable"] is True
    assert body["auto_001_contract"] is True
    assert body["green"] is True


def test_city_g1_c4_witness_writes_json() -> None:
    body = city_seed_chain.write_city_g1_c4_001_witness()
    assert body.get("green") is True
    assert body.get("written") == city_seed_chain.CITY_G1_C4_LIVE_JSON
