# Dev docs archive index `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **DEV-ARCHIVE-INDEX-001** |
| **Archive root** | [`docs/archive/2026-06-fleet-drain/`](../../docs/archive/2026-06-fleet-drain/) · [`2026-06-src-dev/`](../../docs/archive/2026-06-src-dev/) |
| **Script** | `archive_stale_dev_docs.py` · `archive_prompts_phase2.py` · **`archive_src_dev_phase3.py`** |
| **Moved log** | [`docs/archive/2026-06-fleet-drain/MOVED_LOG.json`](../../docs/archive/2026-06-fleet-drain/MOVED_LOG.json) |

---

## What stays in `src/dev/` (active)

- **Planning hub:** [`development_plan_index.md`](development_plan_index.md)
- **Post-drain / phase exec:** `post_drain_dispatch_program_v1.md`, `plan_phase2_exec_001_v1.md`, `plan_phase3_exec_001_v1.md`
- **Planner ledger (current):** `planner_status_audit_v16.md` … `planner_status_audit_v19.md`
- **Fleet (current):** `fleet_snapshot_20260602_v3.md`, `fleet_longrun_prompts_20260602_v1.md`, active `mcp_fleet_*_orders_v1.md`
- **Stage signoffs / triage:** `stage5_*`, `visual_run_blockers.md`, `stage5_triage_backlog.md`
- **Rust proof modules:** `src/dev/*.rs` referenced from `mod.rs` — never archive without code cleanup
- **mod.rs policy:** [`mod_rs_spectrum_audit_v1.md`](mod_rs_spectrum_audit_v1.md)

---

## What was archived (2026-06 drain)

| Bucket | Examples |
|:---|:---|
| `planner_audits/` | `planner_status_audit_v5` … `v15`, ledger checklists 010/015 |
| `fleet_closed/` | Wave 3–7 dispatches, May 20260527–28 snapshots, signoffs |
| `dev_dispatch/` | `orchestrator_*_dispatch_*`, `snapshot_drain_review_*` |
| `prompts_drafts/` | MCP drafts, `base_visual_dev01_plan_status` |
| `prompts_rough_agents/` | Pre-skill agent drafts (canonical: `.cursor/skills/`) |

## Phase 2 (prompts guides drain)

| Bucket | Location |
|:---|:---|
| UI phase specs | `docs/archive/2026-06-prompts-guides/ui_phases/` |
| Runbooks | `docs/archive/2026-06-prompts-guides/runbooks/guides/` |
| Matrix runbooks | `docs/archive/2026-06-prompts-guides/matrix/` |
| User/outside material | `docs/reference/` (not mixed with workfiles) |

**Active prompts spine:** `prompts/llm_agent_brief.md` + 7 files under `prompts/guides/` — see [`prompts/README.md`](../../prompts/README.md).

Script: `python tools/orchestrator/scripts/archive_src_dev_phase3.py`

**Result:** ~51 active `.md` + all `.rs` proof modules remain in `src/dev/`.

---

## Related

- Asset archive (tiles): [`assets/archive/README.md`](../../assets/archive/README.md)
- Cleanup policy: `.cursor/skills/cleanup-completion-intelligence/SKILL.md`
