#!/usr/bin/env python3
"""E4 — report-only determinism lint in sim dirs (MEM-08, LOOP-02, MATH-02)."""

from __future__ import annotations

import re
import sys
from pathlib import Path

_REPO = Path(__file__).resolve().parents[2]
_SIM_DIRS = [
    _REPO / "src" / "combat",
    _REPO / "src" / "economy",
    _REPO / "src" / "systems",
    _REPO / "src" / "strategic",
    _REPO / "src" / "ai",
    _REPO / "src" / "construction",
]

_RULES = [
    ("rng", re.compile(r"thread_rng|rand::random\b")),
    ("wall_clock", re.compile(r"\bInstant::now\b|\bSystemTime\b")),
    ("real_time", re.compile(r"Res\s*<\s*Time\s*>|Time\s*<\s*Real\s*>|\.delta_secs\s*\(")),
    ("hash_iter", re.compile(r"\bfor\s+[^\n]+in\s+[^\n]*(?:HashMap|HashSet)")),
]


def strip_test_lines(raw: str) -> list[tuple[int, str]]:
    out = []
    in_test = False
    brace_depth = 0
    test_depth = None
    for lineno, line in enumerate(raw.splitlines(), 1):
        if re.search(r"#\[cfg\(test\)\]", line):
            in_test = True
            test_depth = None
        if in_test:
            brace_depth += line.count("{") - line.count("}")
            if "{" in line and test_depth is None:
                test_depth = brace_depth
            if test_depth is not None and brace_depth < test_depth:
                in_test = False
                test_depth = None
            continue
        out.append((lineno, line))
    return out


def main() -> int:
    hits: list[tuple[str, str, int, str]] = []
    counts = {r: 0 for r, _ in _RULES}
    for root in _SIM_DIRS:
        if not root.is_dir():
            continue
        for path in sorted(root.rglob("*.rs")):
            rel = path.relative_to(_REPO).as_posix()
            if "/tests/" in f"/{rel}/":
                continue
            try:
                raw = path.read_text(encoding="utf-8")
            except OSError:
                continue
            # HashMap binding heuristic: if file declares HashMap/HashSet names, flag .iter/.values/.keys
            map_names = set(re.findall(r"\b(?:HashMap|HashSet)\s*<[^>]+>\s*(\w+)", raw))
            map_names |= set(re.findall(r"\b(\w+)\s*:\s*(?:HashMap|HashSet)\s*<", raw))
            for lineno, line in strip_test_lines(raw):
                for rule, rx in _RULES:
                    if rule == "hash_iter":
                        if rx.search(line):
                            hits.append((rule, rel, lineno, line.strip()[:120]))
                            counts[rule] += 1
                        elif map_names and re.search(r"\.(?:iter|values|keys)\s*\(", line):
                            # only if a known map name appears on the line
                            if any(n in line for n in map_names):
                                hits.append((rule, rel, lineno, line.strip()[:120]))
                                counts[rule] += 1
                        continue
                    if rx.search(line):
                        hits.append((rule, rel, lineno, line.strip()[:120]))
                        counts[rule] += 1
    print("counts per rule:")
    for rule, _ in _RULES:
        print(f"  {rule}: {counts[rule]}")
    hits.sort(key=lambda x: (x[0], x[1], x[2]))
    print("top 15:")
    for rule, rel, lineno, snip in hits[:15]:
        print(f"  {rule} {rel}:{lineno}  {snip}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
