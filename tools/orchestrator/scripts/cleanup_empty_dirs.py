#!/usr/bin/env python3
"""Remove empty directories under repo (excludes target*, .git, node_modules)."""

from __future__ import annotations

import argparse
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
SKIP_PARTS = {".git", "node_modules", "target"}
SKIP_PREFIXES = ("target_",)


def should_skip(path: Path) -> bool:
    return any(p in SKIP_PARTS or p.startswith(SKIP_PREFIXES) for p in path.parts)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    removed: list[str] = []
    # Bottom-up: deepest dirs first
    dirs = sorted(
        (p for p in REPO.rglob("*") if p.is_dir() and not should_skip(p)),
        key=lambda p: len(p.parts),
        reverse=True,
    )
    for d in dirs:
        try:
            if any(d.iterdir()):
                continue
        except OSError:
            continue
        removed.append(d.relative_to(REPO).as_posix())
        if not args.dry_run:
            try:
                d.rmdir()
            except OSError:
                pass

    print(f"{'would remove' if args.dry_run else 'removed'} {len(removed)} empty dirs")
    for rel in removed[:60]:
        print(f"  {rel}")
    if len(removed) > 60:
        print(f"  ... +{len(removed) - 60} more")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
