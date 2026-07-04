"""APSR-A0-T1-001 — direct SuiteState mutation inventory must not grow."""

from __future__ import annotations

from pathlib import Path

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.suite_state_mutation_inventory import (
    ALLOWLIST_REL,
    load_mutation_allowlist,
    scan_suite_state_mutations,
    suite_state_mutation_inventory,
)


def test_suite_state_mutation_inventory_matches_allowlist() -> None:
    body = suite_state_mutation_inventory()
    assert body["live_count"] == body["allowlist_count"], (
        f"unexpected={body['unexpected_sites']!r} removed={body['removed_sites']!r}"
    )
    assert body["green"] is True
    assert body["unexpected_sites"] == []
    assert body["removed_sites"] == []


def test_suite_state_mutation_allowlist_on_disk() -> None:
    path = repo_root() / ALLOWLIST_REL
    assert path.is_file(), f"missing allowlist: {path}"
    data = load_mutation_allowlist(path)
    assert data.get("gate") == "APSR-A0-T1-001"
    assert len(data.get("sites", [])) >= 33


def test_new_direct_suite_state_write_fails_inventory(tmp_path: Path) -> None:
    suite = tmp_path / "art_pipeline_suite"
    suite.mkdir()
    bad = suite / "evil_panel.py"
    bad.write_text("class Evil:\n    def go(self):\n        self.state.evil_field = 1\n", encoding="utf-8")
    body = suite_state_mutation_inventory(root=suite)
    assert body["green"] is False
    assert len(body["unexpected_sites"]) == 1
    assert body["live_count"] == 1


def test_scan_excludes_state_dataclass_file(tmp_path: Path) -> None:
    suite = tmp_path / "art_pipeline_suite"
    suite.mkdir()
    (suite / "state.py").write_text("class SuiteState:\n    art_domain: str = 'buildings'\n", encoding="utf-8")
    (suite / "panel.py").write_text("self.state.art_domain = 'landscape'\n", encoding="utf-8")
    sites = scan_suite_state_mutations(root=suite)
    assert len(sites) == 1
    assert sites[0].field == "art_domain"
