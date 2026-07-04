"""APSR-A0-T1-001 — inventory of direct ``SuiteState`` field writes in Art Pipeline Suite."""

from __future__ import annotations

import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

ALLOWLIST_REL = "tools/mcp/schemas/aps_suite_state_mutation_allowlist_v1.json"
APS_SUITE_DIR = repo_root() / "tools" / "mcp" / "art_pipeline_suite"

# Assignment only — exclude ``==`` / ``!=`` comparisons on state fields.
_MUTATION_RE = re.compile(r"(?:self\.)?state\.([a-z_][a-z0-9_]*)\s*=(?!=)")


@dataclass(frozen=True)
class SuiteStateMutationSite:
    file: str
    line: int
    field: str

    @property
    def site_id(self) -> str:
        return f"{self.file}:{self.line}"


def scan_suite_state_mutations(root: Path | None = None) -> list[SuiteStateMutationSite]:
    suite = root or APS_SUITE_DIR
    sites: list[SuiteStateMutationSite] = []
    for path in sorted(suite.rglob("*.py")):
        if path.name == "state.py":
            continue
        if root is None or suite.resolve() == APS_SUITE_DIR.resolve():
            rel = path.relative_to(repo_root()).as_posix()
        else:
            rel = str(path.relative_to(suite.parent).as_posix()).replace("\\", "/")
        for line_no, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            stripped = line.strip()
            if stripped.startswith("#"):
                continue
            match = _MUTATION_RE.search(line)
            if match is None:
                continue
            sites.append(
                SuiteStateMutationSite(
                    file=rel,
                    line=line_no,
                    field=match.group(1),
                )
            )
    return sites


def load_mutation_allowlist(path: Path | None = None) -> dict[str, Any]:
    allowlist_path = path or (repo_root() / ALLOWLIST_REL)
    return json.loads(allowlist_path.read_text(encoding="utf-8"))


def allowed_site_ids(allowlist: dict[str, Any] | None = None) -> set[str]:
    data = allowlist if allowlist is not None else load_mutation_allowlist()
    return {str(entry["id"]) for entry in data.get("sites", [])}


def sync_mutation_allowlist_from_scan(*, root: Path | None = None) -> dict[str, Any]:
    """Rewrite allowlist ``sites`` from live scan (APSR-S1 maintenance)."""
    sites = scan_suite_state_mutations(root=root)
    payload = {
        "schema_version": 1,
        "gate": "APSR-A0-T1-001",
        "sites": [
            {"id": site.site_id, "file": site.file, "line": site.line, "field": site.field}
            for site in sites
        ],
    }
    path = repo_root() / ALLOWLIST_REL
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return {"written": str(path), "site_count": len(sites)}


def suite_state_mutation_inventory(*, root: Path | None = None) -> dict[str, Any]:
    sites = scan_suite_state_mutations(root=root)
    allowlist = load_mutation_allowlist()
    allowed = allowed_site_ids(allowlist)
    live_ids = {site.site_id for site in sites}
    unexpected = sorted(live_ids - allowed)
    removed = sorted(allowed - live_ids)
    ok = not unexpected and not removed
    return {
        "gate": "APSR-A0-T1-001",
        "green": ok,
        "ok": ok,
        "live_count": len(sites),
        "allowlist_count": len(allowed),
        "unexpected_sites": unexpected,
        "removed_sites": removed,
        "sites": [
            {"id": site.site_id, "file": site.file, "line": site.line, "field": site.field}
            for site in sites
        ],
    }
