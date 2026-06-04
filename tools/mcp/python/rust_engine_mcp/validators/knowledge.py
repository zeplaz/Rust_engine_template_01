"""Error signature knowledge base — match issues to known fixes."""

from __future__ import annotations

import json
from functools import lru_cache
from pathlib import Path
from typing import Any

from .report import KnownFix, ValidationIssue


def _knowledge_path() -> Path:
    return Path(__file__).resolve().parents[4] / "tools" / "validators" / "knowledge" / "error_signatures.json"


@lru_cache(maxsize=1)
def load_entries() -> list[dict[str, Any]]:
    path = _knowledge_path()
    if not path.is_file():
        return []
    data = json.loads(path.read_text(encoding="utf-8"))
    return list(data.get("entries") or [])


def _issue_matches(issue: ValidationIssue, match: dict[str, Any]) -> bool:
    for key, expected in match.items():
        got = getattr(issue, key, None) or issue.to_dict().get(key)
        if str(got) != str(expected):
            return False
    return True


def lookup_fixes(issues: list[ValidationIssue]) -> list[KnownFix]:
    fixes: list[KnownFix] = []
    seen: set[str] = set()
    for issue in issues:
        sig = issue.signature or issue.rustc_code or issue.kind
        for entry in load_entries():
            match = entry.get("match") or {}
            if _issue_matches(issue, match):
                signature = str(entry.get("signature") or sig)
                if signature in seen:
                    continue
                seen.add(signature)
                fixes.append(
                    KnownFix(
                        signature=signature,
                        fix=str(entry.get("fix") or ""),
                        confidence=float(entry.get("confidence") or 0.0),
                    )
                )
    return fixes
