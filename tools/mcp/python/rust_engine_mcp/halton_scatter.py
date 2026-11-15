"""MATH-03 Halton scatter — engine parity with `render/world_map_vegetation_mesh.rs`."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

MATH03_HALTON_FIXTURE_REL = "assets/test_fixtures/math03_halton_seed1_uv16.json"

_MIX64_ADD = 0x9E37_79B9_7F4A_7C15
_MIX64_MUL1 = 0xBF58_476D_1CE4_E5B9
_MIX64_MUL2 = 0x94D0_49BB_1331_11EB
_CHUNK_HALTON_XOR = 0x4A98_9B4C_D3E2_F107
_SITE_INDEX_SALT = 0x9E37_79B1
_U64_MASK = 0xFFFFFFFFFFFFFFFF


def _i32_as_u64(value: int) -> int:
    """Match Rust `coord.x as i32 as u64` (sign-extended)."""
    bits = value & 0xFFFFFFFF
    if bits & 0x80000000:
        return bits | (~0 << 32) & _U64_MASK
    return bits


def mix64(x: int) -> int:
    """Same 64-bit mix as engine `mix64` (unsigned wrap)."""
    x = (x + _MIX64_ADD) & 0xFFFFFFFFFFFFFFFF
    x = ((x ^ (x >> 30)) * _MIX64_MUL1) & 0xFFFFFFFFFFFFFFFF
    x = ((x ^ (x >> 27)) * _MIX64_MUL2) & 0xFFFFFFFFFFFFFFFF
    return (x ^ (x >> 31)) & 0xFFFFFFFFFFFFFFFF


def chunk_halton_seed(chunk_x: int, chunk_y: int) -> int:
    """Per-chunk salt for Halton scatter; same coord → same seed (engine parity)."""
    coord_bits = (_i32_as_u64(chunk_x) + (_i32_as_u64(chunk_y) << 32)) & _U64_MASK
    return mix64(coord_bits ^ _CHUNK_HALTON_XOR) & 0xFFFFFFFF


def halton_radical_inverse(index: int, base: int) -> float:
    """Radical inverse in base `base`; digit work is integer-only (engine parity)."""
    if base < 2:
        raise ValueError("base must be >= 2")
    acc = 0
    denom = 1
    while index > 0:
        denom *= base
        acc = acc * base + (index % base)
        index //= base
    if denom <= 1:
        return 0.0
    return acc / denom


def halton_scatter_uv(seed: int, site_index: int) -> tuple[float, float]:
    """Halton (2, 3) pair for one site inside a chunk."""
    n = (seed + site_index) & 0xFFFFFFFF
    return halton_radical_inverse(n, 2), halton_radical_inverse(n, 3)


def scatter_jitter_in_cell(
    chunk_x: int, chunk_y: int, site_ix: int, site_iy: int
) -> tuple[float, float]:
    """Jitter in [0.2, 0.8] tile fractions inside a placement cell."""
    u, v = halton_scatter_uv(
        chunk_halton_seed(chunk_x, chunk_y),
        (site_ix * _SITE_INDEX_SALT + site_iy) & 0xFFFFFFFF,
    )
    return 0.2 + 0.6 * u, 0.2 + 0.6 * v


def math03_halton_fixture_path(*, repo: Path | None = None) -> Path:
    return (repo or repo_root()) / MATH03_HALTON_FIXTURE_REL


def load_math03_halton_fixture(*, repo: Path | None = None) -> dict[str, Any]:
    path = math03_halton_fixture_path(repo=repo)
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError(f"{path}: expected object root")
    return data