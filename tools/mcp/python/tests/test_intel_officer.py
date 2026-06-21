"""Intel officer sweep + supervised apply."""

from __future__ import annotations

import json

import pytest

from rust_engine_mcp import intel_officer


def test_deliverable_stub_detects_short_md(tmp_path) -> None:
    p = tmp_path / "stub.md"
    p.write_text("# Outline\n\nTODO fill in.\n", encoding="utf-8")
    assert intel_officer._deliverable_stub_reason(p) is not None


def test_intel_sweep_returns_schema() -> None:
    body = intel_officer.intel_officer_sweep(include_witness_scan=False, compression_level=3)
    assert body["schema"] == "intel_officer_sweep_v1"
    assert "cull_candidates" in body
    assert "by_signature" in body


def test_intel_apply_dry_run_unknown_id() -> None:
    body = intel_officer.intel_officer_apply(ids=["NONEXISTENT-SLICE-999"], dry_run=True)
    assert body["ok"] is True
    assert body["dry_run"] is True
    assert body["applied"]
    assert body["applied"][0]["would"] == "reopen"


def test_intel_apply_reopen_queue_row(tmp_path, monkeypatch) -> None:
    qpath = tmp_path / "q.json"
    qpath.write_text(
        json.dumps(
            {
                "_meta": {"v": 1},
                "drain": [
                    {
                        "id": "TEST-FALSE-DONE",
                        "status": "done",
                        "owner": "coder",
                        "witness": "debug_runs/missing_witness.json",
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    monkeypatch.setitem(
        intel_officer.agent_queue.QUEUE_REGISTRY,
        "multi_parallel",
        "test/q.json",
    )

    def _qpath(q: str):
        if q == "multi_parallel":
            return qpath
        raise KeyError(q)

    monkeypatch.setattr(intel_officer.agent_queue, "queue_path", _qpath)
    monkeypatch.setattr(intel_officer, "repo_root", lambda: tmp_path)

    sweep = {
        "cull_candidates": [
            {
                "id": "queue:TEST-FALSE-DONE:no-witness",
                "slice_id": "TEST-FALSE-DONE",
                "signature": "INTEL-DONE-NO-WITNESS",
                "recommended_action": "reopen",
                "reason": "test",
            }
        ]
    }
    out = intel_officer.intel_officer_apply(
        ids=["TEST-FALSE-DONE"], dry_run=False, sweep=sweep
    )
    assert out["applied"]
    rows = json.loads(qpath.read_text())["drain"]
    assert rows[0]["status"] == "reopened"
