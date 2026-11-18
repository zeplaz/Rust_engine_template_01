#!/usr/bin/env python3
"""E1 / CONC-03 — report ResMut<Name> / &mut Name outside the owner prefix (report-only)."""

from __future__ import annotations

import re
import sys
from pathlib import Path

_REPO = Path(__file__).resolve().parents[2]
_TSV = Path(__file__).resolve().parent / "knowledge" / "sole_writers.tsv"
_SRC = _REPO / "src"

_RESMUT = re.compile(r"ResMut\s*<\s*([A-Za-z0-9_:]+)\s*>")
_MUT_NAME = re.compile(r"&mut\s+([A-Za-z0-9_]+)\b")


def load_owners() -> list[tuple[str, str]]:
    rows = []
    for line in _TSV.read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.startswith("Resource"):
            continue
        name, prefix = line.split("\t", 1)
        rows.append((name.strip(), prefix.strip().replace("\\", "/")))
    return rows


def strip_cfg_test(text: str) -> str:
    """Remove #[cfg(test)] … mod tests { … } blocks (best-effort brace match)."""
    out = []
    i = 0
    while i < len(text):
        m = re.search(r"#\[cfg\(test\)\]", text[i:])
        if not m:
            out.append(text[i:])
            break
        out.append(text[i : i + m.start()])
        j = i + m.end()
        # skip whitespace / attributes / pub / mod name
        bm = re.search(r"\bmod\s+\w+\s*\{", text[j:])
        if not bm:
            out.append(text[i + m.start() :])
            break
        start = j + bm.end() - 1  # on '{'
        depth = 0
        k = start
        while k < len(text):
            if text[k] == "{":
                depth += 1
            elif text[k] == "}":
                depth -= 1
                if depth == 0:
                    k += 1
                    break
            k += 1
        i = k
    return "".join(out)


def scan_file(path: Path, owners: list[tuple[str, str]]) -> list[str]:
    rel = path.relative_to(_REPO).as_posix()
    if "/tests/" in f"/{rel}/" or rel.endswith("/tests.rs"):
        return []
    try:
        raw = path.read_text(encoding="utf-8")
    except OSError:
        return []
    text = strip_cfg_test(raw)
    lines = text.splitlines()
    # Map cleaned line index is lossy; re-scan raw with simple cfg skip by line state
    violations = []
    in_test = False
    brace_depth = 0
    test_depth = None
    raw_lines = raw.splitlines()
    for lineno, line in enumerate(raw_lines, 1):
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
        names_hit = set(_RESMUT.findall(line))
        # &mut Name only when Name is a tracked resource (avoid &mut commands noise)
        for m in _MUT_NAME.finditer(line):
            names_hit.add(m.group(1))
        for name, prefix in owners:
            short = name.split("::")[-1]
            if short not in names_hit and name not in names_hit:
                continue
            if "ResMut" not in line and f"&mut {short}" not in line and f"&mut {name}" not in line:
                continue
            owner_ok = rel == prefix or rel.startswith(prefix.rstrip("/") + "/") or rel.startswith(prefix)
            # prefix may be a file
            if prefix.endswith(".rs"):
                owner_ok = rel == prefix
            if not owner_ok:
                violations.append(f"{rel}:{lineno}  ResMut/&mut {short} outside {prefix}")
    return violations


def main() -> int:
    # CQ-78: default report-only; `--fail` exits non-zero when violations exist.
    fail = "--fail" in sys.argv
    owners = load_owners()
    all_v: list[str] = []
    for path in sorted(_SRC.rglob("*.rs")):
        all_v.extend(scan_file(path, owners))
    print(f"{len(owners)} resources, {len(all_v)} violations")
    for v in all_v:
        print(v)
    if fail and all_v:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
