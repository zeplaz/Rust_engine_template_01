# Agent queue + token-savings MCP tools

| Field | Value |
|:---|:---|
| **Status** | **SHIPPED** |
| **Module** | `tools/mcp/python/rust_engine_mcp/agent_queue.py` |
| **Queue files** | `grammar_continuation_queue.json`, `continuation_queue.json` |

---

## Problem

Agents idle with “waiting on planner,” re-read full plans/logs/witness JSON every turn → **~10× token burn**.

## Pattern: queue-drain (not poll)

```text
Session start → agent_queue_next("coder")
  → action=work + slice + drain_reason
  → implement
  → agent_queue_update(slice_id, "done", note="witness path")
  → agent_queue_next again until action=idle
```

No periodic “check other agents.” Dependencies are **`depends_on` + `status: done`** in JSON; blocked slices list `blocked_by` and optional `fallback_when_blocked`.

---

## MCP tools (use these first)

| Tool | Replaces |
|:---|:---|
| `agent_queue_next` | Re-reading plan todo tables / “what should I do?” |
| `agent_queue_update` | Manual queue JSON edits |
| `agent_queue_board` | Loading full queue file into chat |
| `witness_brief` | `Read` on multi-KB `debug_runs/*.json` |
| `handoff_brief` | Full `HANDOFF.md` |
| `file_digest` | Full source file read for orientation |
| `orchestrator_brief` | Parsing `last_run.json` + raw cargo |
| `token_savings_guide` | Ad-hoc reminders |
| `validate_*_report(compress=4)` | Raw `cargo check` / blender logs |

## CLI parity

```powershell
cd tools\mcp\python
python -m rust_engine_mcp.cli agent-queue-next planner
python -m rust_engine_mcp.cli agent-queue-update PLAN-APS-TAGS-001 done --note "schema shipped"
python -m rust_engine_mcp.cli witness-brief debug_runs/grammar_diversity_witness.json
python -m rust_engine_mcp.cli token-savings-guide
```

## Agent session ritual (paste once per chat)

```text
Before any work: token_savings_guide() then agent_queue_next("<your-agent>").
After shipping: agent_queue_update("<slice_id>", "done|blocked", note="...").
Build/test: validate_cargo_report(compress=4, use_cached=true) only.
```

## Extend queues

Add rows to `tools/orchestrator/queues/*.json` with: `id`, `agent`, `priority`, `status`, `depends_on`, optional `stop_point`, `fallback_when_blocked`.

Register new queue files in `QUEUE_REGISTRY` inside `agent_queue.py`.
