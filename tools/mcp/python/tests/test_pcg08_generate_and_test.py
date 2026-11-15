"""PCG-08 — generate-and-test report (golden seed determinism)."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

from rust_engine_mcp import building_quality_qc, golden_seed_review
from rust_engine_mcp.paths import repo_root


def test_pcg08_golden_report_deterministic_cli() -> None:
    report = golden_seed_review.run_golden_generate_and_test_report(top_k=3, repo=repo_root())
    text_a = building_quality_qc.format_generate_and_test_cli(report)
    text_b = building_quality_qc.format_generate_and_test_cli(report)
    assert text_a == text_b
    assert "rejection_histogram:" in text_a
    hist = report.get("rejection_histogram") or {}
    assert isinstance(hist, dict) and hist
    for _reason, count in hist.items():
        assert int(count) >= 1
    assert report.get("candidate_count") == len(golden_seed_review.load_golden_seeds(repo=repo_root()))


def test_pcg08_cli_subprocess_matches_format_twice() -> None:
    py = Path(sys.executable)
    root = repo_root()
    cmd = [str(py), "-m", "rust_engine_mcp.cli", "generate-and-test-report", "--no-witness"]
    r1 = subprocess.run(cmd, cwd=str(root / "tools/mcp/python"), capture_output=True, text=True, check=True)
    r2 = subprocess.run(cmd, cwd=str(root / "tools/mcp/python"), capture_output=True, text=True, check=True)
    assert r1.stdout == r2.stdout
    assert "rejection_histogram:" in r1.stdout
    for line in r1.stdout.splitlines():
        if line.startswith("  ") and ":" in line:
            _reason, count = line.strip().split(": ", 1)
            assert int(count) >= 1


def test_pcg08_strip_suffix_after_witness_refresh() -> None:
    building_quality_qc.refresh_pcg08_golden_report(top_k=3, write_witness=True, repo=repo_root())
    suffix = building_quality_qc.format_pcg08_histogram_strip_suffix(repo=repo_root())
    assert "PCG-08" in suffix
    assert "reject" in suffix