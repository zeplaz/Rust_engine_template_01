# Agent doc digest cache (MCP-DOC-READ-004)

Populated by `agent_doc_promote_hot_reads()` when paths in `debug_runs/agent_ops/doc_reads.jsonl` exceed repeat thresholds.

Consumers: `agent_doc_digest_cached(path)` before `agent_doc_touch()`.

Rollup witness: `debug_runs/agent_ops/doc_reads_brief_latest.json`
