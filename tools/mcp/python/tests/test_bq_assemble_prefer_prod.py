"""BQ-ASSEMBLE-PREF-PROD-001 — assemble defaults to production."""

from __future__ import annotations

import json
import subprocess
import sys

from rust_engine_mcp.bq_assemble_prefer_prod import (
    WITNESS_REL,
    write_bq_assemble_prefer_prod_witness,
)
from rust_engine_mcp.paths import repo_root


def test_bq_assemble_prefer_prod_witness():
    body = write_bq_assemble_prefer_prod_witness()
    assert body.get("gate") == "BQ-ASSEMBLE-PREF-PROD-001"
    assert body.get("green") is True, body.get("summary") or body.get("focus_packs")
    assert body.get("default_source_tier") == "production"
    assert body.get("index_prefer_tier_default") == "production"
    path = repo_root() / WITNESS_REL
    assert path.is_file()
    disk = json.loads(path.read_text(encoding="utf-8"))
    assert disk.get("green") is True
    assert disk.get("_agent_meta", {}).get("task_id") == "BQ-ASSEMBLE-PREF-PROD-001"


def test_cli_bq_assemble_prefer_prod():
    proc = subprocess.run(
        [sys.executable, "-m", "rust_engine_mcp.cli", "bq-assemble-prefer-prod"],
        cwd=repo_root() / "tools/mcp/python",
        capture_output=True,
        text=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr or proc.stdout
    body = json.loads(proc.stdout)
    assert body.get("ok") is True
