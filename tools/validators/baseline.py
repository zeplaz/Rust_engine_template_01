"""Committed-baseline gate shared by the report validators (CQ-78).

A finding's key has no line number (file + rule + code text), so moving code does not churn the
baseline. ``--fail`` fails only on keys not in the baseline; ``--write-baseline`` rewrites it.
Keys are counted, so a second copy of an already-baselined line is still new.
"""

from __future__ import annotations

from collections import Counter
from pathlib import Path


def load(path: Path) -> Counter[str]:
    if not path.is_file():
        return Counter()
    return Counter(
        line for line in path.read_text(encoding="utf-8").splitlines() if line and not line.startswith("#")
    )


def write(path: Path, keys: list[str], header: str) -> None:
    body = "\n".join(sorted(keys))
    path.write_text(f"# {header}\n# Regenerate: python tools/validators/<script>.py --write-baseline\n{body}\n", encoding="utf-8")


def new_keys(keys: list[str], baseline: Counter[str]) -> list[str]:
    """Keys beyond what the baseline allows (multiset difference)."""
    return sorted((Counter(keys) - baseline).elements())


def gate(argv: list[str], keys: list[str], path: Path, header: str) -> int:
    """Apply ``--write-baseline`` / ``--fail``; print new findings; return the exit code."""
    if "--write-baseline" in argv:
        write(path, keys, header)
        print(f"baseline written: {path.name} ({len(keys)} entries)")
        return 0
    base = load(path)
    fresh = new_keys(keys, base)
    fixed = sum((base - Counter(keys)).values())
    print(f"baseline {path.name}: {sum(base.values())} known, {len(fresh)} new, {fixed} fixed since")
    for k in fresh:
        print(f"  NEW {k}")
    if fixed:
        print("  (fixed entries can be dropped with --write-baseline)")
    return 1 if ("--fail" in argv and fresh) else 0
