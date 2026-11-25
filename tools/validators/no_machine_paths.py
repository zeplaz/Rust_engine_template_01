#!/usr/bin/env python3
"""Fail if a tracked text file contains a machine-specific absolute path.

Catches Windows drive paths into user profiles / dev folders / Program Files (plain, `/` or
JSON-escaped) and relative paths that climb out of the repo into a user profile. Commit paths
repo-relative instead; per-machine settings go in gitignored files (e.g. tools/mcp/config.local.json).
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

_REPO = Path(__file__).resolve().parents[2]
_SEP = r"(?:\\\\|\\|/)"
_BAD = [
    re.compile(r"(?<![A-Za-z0-9_])[A-Za-z]:" + _SEP + r"(?:Users|dev|dev_world|Program Files|ProgramData)" + _SEP, re.I),
    re.compile(r"(?:\.\." + _SEP + r")+Users" + _SEP, re.I),
]
_SKIP = {"tools/validators/no_machine_paths.py"}


def main() -> int:
    files = subprocess.run(["git", "ls-files", "-z"], cwd=_REPO, capture_output=True, check=True).stdout
    hits: list[str] = []
    for rel in filter(None, files.decode("utf-8").split("\0")):
        if rel in _SKIP:
            continue
        path = _REPO / rel
        try:
            raw = path.read_bytes()
        except OSError:
            continue
        if b"\0" in raw[:8000]:
            continue
        text = raw.decode("utf-8", "replace")
        for n, line in enumerate(text.splitlines(), 1):
            if any(rx.search(line) for rx in _BAD):
                hits.append(f"{rel}:{n}")
    print(f"no_machine_paths: {len(hits)} hit(s)")
    for h in hits[:50]:
        print("  " + h)
    return 1 if hits else 0


if __name__ == "__main__":
    raise SystemExit(main())
