"""MCP-WIT-041 — queue_integrity cross-sync tests."""

from __future__ import annotations

import json

import pytest

from rust_engine_mcp import agent_queue
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators import run_validator
from rust_engine_mcp.validators.queue_integrity import (
    collect_queue_integrity,
    refresh_queue_integrity_reconcile_witness,
    validate_queue_integrity,
)
from rust_engine_mcp.validators.queue_registry import load_queue_registry, validate_queue_registry

_SYNTH_REGISTRY = {
    "status_normalization": {
        "closed": ["done", "lib_done", "signed", "closed"],
        "open": ["blocked", "paused", "deferred", "open", "reopened"],
        "ready": ["ready", "active", "in_progress"],
    },
    "queues": [
        {
            "queue_id": "synth_a",
            "path": "tools/mcp/schemas/examples/witness_honesty_fixtures/_queue_synth_a.json",
            "id_field": "id",
            "rows_path": "rows",
        },
        {
            "queue_id": "synth_b",
            "path": "tools/mcp/schemas/examples/witness_honesty_fixtures/_queue_synth_b.json",
            "id_field": "id",
            "rows_path": "rows",
        },
        {
            "queue_id": "synth_snag",
            "path": "tools/mcp/schemas/examples/witness_honesty_fixtures/_queue_synth_snag.json",
            "id_field": "id",
            "rows_path": "rows",
        },
    ],
}


def test_queue_registry_schema() -> None:
    validate_queue_registry()
    doc = load_queue_registry()
    assert len(doc.get("queues") or []) >= 6


def test_queue_integrity_finds_contradictions() -> None:
    body = collect_queue_integrity()
    assert body.get("contradiction_count", 0) >= 3
    assert body.get("error_count", 0) >= 6
    assert body.get("green") is False
    contradictions = body.get("contradictions") or []
    assert any(c.get("id") == "VEG-F02-MCP-ATLAS-001" for c in contradictions)


def test_synthetic_queue_pair_contradiction() -> None:
    body = collect_queue_integrity(registry=_SYNTH_REGISTRY)
    contradictions = body.get("contradictions") or []
    assert any(c.get("id") == "WIT-SYNTH-CONTRA" for c in contradictions)
    assert any(
        i.signature == "WIT-QUEUE-CONTRADICTION" and i.symbol == "WIT-SYNTH-CONTRA"
        for i in body.get("issues") or []
    )


def test_synthetic_snag_done() -> None:
    body = collect_queue_integrity(registry=_SYNTH_REGISTRY)
    snag_rows = body.get("snag_done") or []
    assert any(r.get("id") == "WIT-SYNTH-SNAG" for r in snag_rows)
    assert any(
        i.signature == "WIT-SNAG-DONE" and i.symbol == "WIT-SYNTH-SNAG"
        for i in body.get("issues") or []
    )


def test_validate_report_queue_integrity() -> None:
    report = validate_queue_integrity(compression_level=3)
    assert report.status == "failed"
    assert report.error_count >= 6
    assert any(e.signature == "WIT-QUEUE-CONTRADICTION" for e in report.errors)


def test_queue_integrity_reconcile_witness() -> None:
    body = refresh_queue_integrity_reconcile_witness()
    assert body.get("gate") == "MCP-WIT-014"
    assert body.get("green") is False
    assert body.get("contradiction_count", 0) >= 3
    assert body.get("error_count", 0) >= 6
    assert len(body.get("stale_ids") or []) > 0
    path = repo_root() / "debug_runs/queue_integrity_reconcile_live.json"
    assert path.is_file()


def test_agent_queue_update_enforce_blocks_bad_done(tmp_path, monkeypatch) -> None:
    fixture = tmp_path / "q.json"
    fixture.write_text(
        json.dumps(
            [
                {
                    "id": "ENFORCE-BAD",
                    "agent": "coder",
                    "priority": 1,
                    "status": "ready",
                    "witness": "tools/mcp/schemas/examples/witness_honesty_fixtures/bad_exit_predicate_live.json",
                }
            ]
        ),
        encoding="utf-8",
    )
    monkeypatch.setitem(agent_queue.QUEUE_REGISTRY, "test", "test/q.json")

    def _qpath(q: str):
        if q == "test":
            return fixture
        return agent_queue.queue_path(q)

    monkeypatch.setattr(agent_queue, "queue_path", _qpath)

    out = agent_queue.agent_queue_update("ENFORCE-BAD", "done", queue="test", enforce=True)
    assert out.get("ok") is False
    assert out.get("enforce") is True
    items = json.loads(fixture.read_text(encoding="utf-8"))
    assert items[0]["status"] == "ready"


def test_agent_queue_update_enforce_allows_good_done(tmp_path, monkeypatch) -> None:
    witness = tmp_path / "good_live.json"
    witness.write_text(
        json.dumps(
            {
                "_agent_meta": {"schema": "witness_honesty_fixture_v1"},
                "green": True,
                "exit_predicate": {"must": [{"path": "green", "eq": True}]},
            }
        ),
        encoding="utf-8",
    )
    fixture = tmp_path / "q.json"
    fixture.write_text(
        json.dumps(
            [
                {
                    "id": "ENFORCE-OK",
                    "agent": "coder",
                    "priority": 1,
                    "status": "ready",
                    "exit_predicate": {"must": [{"path": "green", "eq": True}]},
                    "witness": str(witness),
                }
            ]
        ),
        encoding="utf-8",
    )
    monkeypatch.setitem(agent_queue.QUEUE_REGISTRY, "test", "test/q.json")

    def _qpath(q: str):
        if q == "test":
            return fixture
        return agent_queue.queue_path(q)

    monkeypatch.setattr(agent_queue, "queue_path", _qpath)

    out = agent_queue.agent_queue_update("ENFORCE-OK", "done", queue="test", enforce=True)
    assert out.get("ok") is True
    items = json.loads(fixture.read_text(encoding="utf-8"))
    assert items[0]["status"] == "done"
