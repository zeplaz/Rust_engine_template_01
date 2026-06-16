"""MCP-P2-RUN-EVENT-001 — agent_run_append OPS Phase 1 telemetry witness."""

from __future__ import annotations

import json
from typing import Any

from . import agent_doc_read
from .paths import repo_root

MCP_P2_RUN_EVENT_WITNESS = "debug_runs/mcp_p2_run_event_001_live.json"


def write_mcp_p2_run_event_001_witness() -> dict[str, Any]:
    append = agent_doc_read.agent_run_append(
        {
            "slice_id": "MCP-P2-RUN-EVENT-001",
            "tools_called": ["agent_run_append", "pipeline_preflight", "coder_mcp_drain_brief"],
            "witness": MCP_P2_RUN_EVENT_WITNESS,
        },
        agent="coder-mcp",
    )
    ledger = repo_root() / agent_doc_read.RUN_EVENTS_LEDGER
    green = bool(append.get("ok") and ledger.is_file())
    body: dict[str, Any] = {
        "gate_id": "MCP-P2-RUN-EVENT-001",
        "ok": green,
        "green": green,
        "append_ok": append.get("ok"),
        "ledger_path": agent_doc_read.RUN_EVENTS_LEDGER,
        "mcp_tool": "agent_run_append",
        "cli": "agent-run-append",
        "prior_witness": agent_doc_read.AGENT_RUN_APPEND_WITNESS,
    }
    out = repo_root() / MCP_P2_RUN_EVENT_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body
