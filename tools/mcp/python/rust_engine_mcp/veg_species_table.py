"""VEG-SHAPE-001 — parametric species table (Studio / APS read check).

Engine authority: `VegSpeciesTable` + `VegExtractRow.species` (extract cache).
Dual-cite: PCG-05 stage-specific rules · LEGACY `l_systems` (aig_e3 p701).
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

VEG_SPECIES_REL = "assets/configs/ecology/veg_species_v1.ron"
_ALLOWED_PROFILES = frozenset({"Round", "Cone", "Column", "Spread"})
_ID_RE = re.compile(r'\bid:\s*"([^"]+)"')
_PROFILE_RE = re.compile(r"\bprofile:\s*(\w+)")
_TIERS_RE = re.compile(r"\btiers:\s*(\d+)")
_SEED_RE = re.compile(r"\bseed:\s*(\d+)")


def load_veg_species_ron_text(
    *, repo: Path | None = None, path: Path | None = None
) -> str:
    if path is not None:
        resolved = Path(path)
        if not resolved.is_file():
            raise FileNotFoundError(str(resolved))
        return resolved.read_text(encoding="utf-8")
    root = repo or repo_root()
    default = root / VEG_SPECIES_REL
    if not default.is_file():
        raise FileNotFoundError(VEG_SPECIES_REL)
    return default.read_text(encoding="utf-8")


def brief_veg_species_table(
    *, repo: Path | None = None, path: Path | None = None
) -> dict[str, Any]:
    """Lightweight structural QC — mirrors Rust `VegSpeciesTable::validate` bounds."""
    resolved = Path(path) if path is not None else (repo or repo_root()) / VEG_SPECIES_REL
    text = load_veg_species_ron_text(repo=repo, path=path)
    ids = _ID_RE.findall(text)
    profiles = _PROFILE_RE.findall(text)
    tiers = [int(x) for x in _TIERS_RE.findall(text)]
    seed_m = _SEED_RE.search(text)
    issues: list[str] = []
    if len(ids) < 1:
        issues.append("no species rows")
    if len(ids) != len(set(ids)):
        issues.append("duplicate species id")
    if len(ids) > 64:
        issues.append("table exceeds 64 species")
    bad_profiles = sorted(set(profiles) - _ALLOWED_PROFILES)
    if bad_profiles:
        issues.append(f"unknown profile: {', '.join(bad_profiles)}")
    if len(profiles) != len(ids):
        issues.append("profile count != species count")
    if any(t < 1 or t > 5 for t in tiers):
        issues.append("tiers must be 1..=5")
    if len(tiers) != len(ids):
        issues.append("tiers count != species count")
    green = not issues
    try:
        rel = str(resolved.relative_to(repo or repo_root()))
    except ValueError:
        rel = str(resolved)
    return {
        "schema": "veg_species_table_brief_v1",
        "path": rel.replace("\\", "/"),
        "seed": int(seed_m.group(1)) if seed_m else None,
        "species_count": len(ids),
        "species_ids": ids,
        "profiles": profiles,
        "ref_books_dual_cite": {
            "new_rows": ["PCG-05"],
            "legacy_keywords": ["l_systems"],
        },
        "green": green,
        "issues": issues,
    }