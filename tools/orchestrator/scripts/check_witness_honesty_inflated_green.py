"""PCI-06 — repo-wide witness scan that fails closed on inflated green.

WIT-ART-DISHONEST and WIT-PIXELS-DISHONEST are the fail-closed shapes.
Other honesty errors stay in the printed counts. The signed witness-integrity
plan keeps that broader backlog report-only until the reopen program.
"""

from __future__ import annotations

import json
import sys
import time
from pathlib import Path

_REPO = Path(__file__).resolve().parents[3]
_MCP_PY = _REPO / "tools" / "mcp" / "python"
if str(_MCP_PY) not in sys.path:
    sys.path.insert(0, str(_MCP_PY))

from rust_engine_mcp.validators.witness_honesty import (  # noqa: E402
    evaluate_witness_honesty_rules,
    load_witness_integrity_catalog,
)

INFLATED_GREEN = frozenset({"WIT-ART-DISHONEST", "WIT-PIXELS-DISHONEST"})
BUDGET_MS = 120_000
_META = {"_agent_meta": {"schema": "witness_honesty_fixture_v1"}}


def _symbols(data: dict, rel: str, catalog: dict, root: Path) -> set[str]:
    issues = evaluate_witness_honesty_rules(
        data, witness_rel=rel, catalog=catalog, root=root
    )
    return {str(issue.symbol or issue.signature) for issue in issues}


def _self_test(catalog: dict, root: Path) -> None:
    fixture = root / "tools/mcp/schemas/examples/witness_honesty_fixtures/bad_art_dishonest_live.json"
    art = json.loads(fixture.read_text(encoding="utf-8"))
    if "WIT-ART-DISHONEST" not in _symbols(art, fixture.as_posix(), catalog, root):
        raise SystemExit("self-test: bad art fixture did not fail WIT-ART-DISHONEST")

    honest_reject = {
        **_META,
        "green": False,
        "art_quality": "rejected_greybox_ortho",
    }
    if _symbols(honest_reject, "debug_runs/pci06_self_test_live.json", catalog, root) & INFLATED_GREEN:
        raise SystemExit("self-test: green=false rejected art must not be inflated green")

    pixels = {**_META, "green": True, "pixel_regression_green": False}
    if "WIT-PIXELS-DISHONEST" not in _symbols(
        pixels, "debug_runs/product_fire_vfx_live.json", catalog, root
    ):
        raise SystemExit("self-test: false pixel_regression_green did not fail WIT-PIXELS-DISHONEST")

    chrome = {**_META, "green": True, "pixel_regression_green": True, "lod_chrome_leak": True}
    if "WIT-PIXELS-DISHONEST" not in _symbols(
        chrome, "debug_runs/product_fire_vfx_live.json", catalog, root
    ):
        raise SystemExit("self-test: lod_chrome_leak did not fail WIT-PIXELS-DISHONEST")

    clean = {**_META, "green": True, "pixel_regression_green": True, "lod_chrome_leak": False}
    if _symbols(clean, "debug_runs/product_fire_vfx_live.json", catalog, root) & INFLATED_GREEN:
        raise SystemExit("self-test: honest pixel verdict was treated as inflated green")


def _scan(catalog: dict, root: Path) -> dict:
    base = root / "debug_runs"
    started = time.perf_counter()
    scanned = 0
    unreadable = 0
    hits: list[dict[str, str]] = []
    if base.is_dir():
        for path in sorted(base.rglob("*_live.json")):
            scanned += 1
            rel = path.relative_to(root).as_posix()
            try:
                data = json.loads(path.read_text(encoding="utf-8"))
            except (OSError, json.JSONDecodeError):
                unreadable += 1
                continue
            if not isinstance(data, dict):
                unreadable += 1
                continue
            found = sorted(_symbols(data, rel, catalog, root) & INFLATED_GREEN)
            for rule_id in found:
                hits.append({"path": rel, "rule_id": rule_id})
    elapsed_ms = int((time.perf_counter() - started) * 1000)
    return {
        "scanned": scanned,
        "unreadable": unreadable,
        "inflated_green": len(hits),
        "elapsed_ms": elapsed_ms,
        "budget_ms": BUDGET_MS,
        "under_budget": elapsed_ms < BUDGET_MS,
        "hits": hits[:20],
    }


def main() -> int:
    started = time.perf_counter()
    root = _REPO
    catalog = load_witness_integrity_catalog(repo=root)
    _self_test(catalog, root)
    body = _scan(catalog, root)
    command_ms = int((time.perf_counter() - started) * 1000)
    report = {
        "gate": "PCI-06",
        "self_test": "passed",
        "command_ms": command_ms,
        **body,
    }
    print(json.dumps(report, indent=2))
    if command_ms >= BUDGET_MS or not body["under_budget"] or body["inflated_green"]:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
