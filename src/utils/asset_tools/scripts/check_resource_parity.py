#!/usr/bin/env python3
"""Fail if Rust `ResourceType` variants are missing from `asset_config.RESOURCE_TYPES`."""

from __future__ import annotations

import re
import sys
from pathlib import Path

# `scripts/` -> `asset_tools/` -> add `src` for `config` + `repo_paths`
_ASSET_TOOLS = Path(__file__).resolve().parent.parent
_SRC = _ASSET_TOOLS / "src"
if str(_SRC) not in sys.path:
    sys.path.insert(0, str(_SRC))

from config.asset_config import RESOURCE_TYPES  # noqa: E402
from repo_paths import REPO_ROOT  # noqa: E402


def rust_resource_variants() -> set[str]:
    path = REPO_ROOT / "src" / "entities" / "types" / "p_enumz.rs"
    text = path.read_text(encoding="utf-8")
    m = re.search(r"pub enum ResourceType\s*\{([^}]*)\}", text, re.DOTALL)
    if not m:
        print("check_resource_parity: could not parse ResourceType enum", file=sys.stderr)
        raise SystemExit(2)
    body = m.group(1)
    return {x for x in re.findall(r"\b([A-Z][a-zA-Z0-9]*)\b", body) if x not in ("Self",)}


def main() -> int:
    rust = rust_resource_variants()
    py = set(RESOURCE_TYPES)
    missing = sorted(rust - py)
    if missing:
        print(
            "check_resource_parity: Rust ResourceType variants missing from RESOURCE_TYPES:",
            ", ".join(missing),
            file=sys.stderr,
        )
        return 1
    extra = sorted(py - rust)
    if extra:
        print(
            "check_resource_parity: OK (Python has authoring-only extras: "
            + f"{len(extra)} entries not in Rust enum)"
        )
    else:
        print("check_resource_parity: OK — Python RESOURCE_TYPES matches Rust enum exactly.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
