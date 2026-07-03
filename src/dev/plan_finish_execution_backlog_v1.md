# Plan finish execution backlog `v1`

```text
⟦SYMLANG⟧⟐v1  ◈BACKLOG
⟨ID⟩ PLAN-FINISH-EXEC-001
Date: 2026-06-21
Status: **ACTIVE**
Owner: @orchestrator routes · @coder / @coder-mcp implement
```

**Why plans feel stuck:** SIGNED plans and queue JSON are not closure. A program closes only when **witness JSON is green** + **exit_predicate passes** + **operator row executed** where required. Recent sessions skewed heavily toward planning; implementation picks were not run in the same thread.

---

## Root causes (honest)

| Cause | Symptom | Fix |
|:---|:---|:---|
| **Plan-only sessions** | Many `plan_*.md` SIGNED, no `@coder` pick | Each plan must seed **machine queue row** + **one immediate impl slice** |
| **Intel officer bulk cull (2026-06-20)** | Rows marked `done`/`blocked` **without witness** | Reopen via `exit_predicate`; do not trust status alone |
| **Stale plan copy** | e.g. designer backlog says **G0**, repo boots **G3** | `PLAN-APS-PRESENCE-PLAN-EDIT-001` + stop citing outdated §0 tables |
| **HANDOFF drift** | Active programs table omits open queues | Sync HANDOFF when seeding new queue |
| **Operator gates** | G-PLAY-OPERATOR-01, APS rubric walks | Machine green ≠ product sign-off |
| **G4 content bar** | ~~Tier stuck G3~~ | **Done** 2026-07-03 — `aps_g4_coverage_live.json` · tier **G4** |

**Rule:** No new SIGNED plans until the **top open row** in this backlog has a witness path or explicit defer in `defer_registry.json`.

---

## Tier 0 — Unblock truth (P0, ~1 session @coder-mcp)

| ☐ | ID | Owner | Exit witness | Notes |
|:---|:---|:---|:---|:---|
| ☑ | APS-GUARD-BRIEF-PARITY-001 | coder-mcp | `debug_runs/aps_guard_brief_parity_live.json` | **Done** — counts aligned; G4 still blocked on coverage/hardcode |
| ☑ | APS-GRAM-TIER-GATES-LIVE-001 | coder-mcp | `aps_grammar_tier_gates_live.json` tier == G3 | **Done** — live tier G3 |
| ☑ | DES-APS-SESSION-DUMP-001 | coder-mcp | `aps_session_presence_live.json` | **Done** — CLI + WIT-HON green |
| ☑ | PLAN-APS-PRESENCE-PLAN-EDIT-001 | planner | plan docs amended | **Done** 2026-06-21 |
| ☑ | OVR-APS-PRESENCE-OPERATOR-001 | operator | HANDOFF + attestation witness | **Done** 2026-07-03 · `aps_presence_operator_attestation_live.json` |
| ☑ | APS-G4-COVERAGE-001 | coder-mcp | `debug_runs/aps_g4_coverage_live.json` | **Done** 2026-07-03 — tier **G4** · pilot_hardcode green |

**Queue:** `tools/orchestrator/queues/aps_presence_correction_queue.json`

---

## Tier 1 — Shipped but needs close witness (P1)

| Program | Machine state | Still open |
|:---|:---|:---|
| **Power grid UX** | Track A–D rows **done** in queue; code on disk (`BuildTool::PowerLine`, overlay, damage, hover) | Re-run `cargo test -p proc_A_dine01 --lib power_` + refresh close witness if intel cull stale |
| **Power grid art** | Downstream queue **done**; style doc exists | MCP module GLBs: substation yard, transformer pad (`DMCP-SPEC-*`) |
| **Sim HUD Phase 2** | Design specs largely on disk | Coder wire: build picker cohesion, popup tier migration |

---

## Tier 2 — Plan-only (needs first impl slice)

| Program | Plan | First coder pick | Blocker |
|:---|:---|:---|:---|
| **Nuclear failure** | `plan_nuclear_power_failure_meltdown_v1.md` | COD-NUCLEAR-GRID-LINK-001 (LOOP/SCRAM) | No sim state machine yet |
| **Industrial facility grammar** | `plan_industrial_facility_grammar_suite_v1.md` | CMCP-GRAM-FACILITY-BRIEF-001 | Facility binding schema + site pilots |
| **Designer Track F/G** | Power + nuclear art | DMCP-SPEC-SUBSTATION-YARD-001 | Style bible signed → MCP job |

---

## Tier 3 — Deferred / operator / product

| ID | Gate |
|:---|:---|
| G-PLAY-OPERATOR-01 | Human playtest checklist |
| PLAN-AUDIT-020 | Blocked on G-PLAY operator |
| Iso utility tiles (Lane G) | Product confirms iso-first read |

---

## Recommended pick order (next 2 weeks)

```text
Week 1  @coder_a/@coder_b  CITY-G0-S11-001 / CITY-G0-S1C-001 (plan_city_grammar G0)

Week 1  @operator   (optional) APS rubric v2 pixel follow-up @ display session

Week 2  @coder      COD-NUCLEAR-GRID-LINK-001 P1 (LOOP/SCRAM only)
        @designer-mcp DMCP-SPEC-SUBSTATION-YARD-001 + TRANSFORMER-PAD-001
        @coder        Sim HUD build picker wire (COD-SIM-HUD-BUILD-PICKER-001)
```

---

## What NOT to do

- Do **not** open another master plan doc for APS tier exposure (already PASS).
- Do **not** mark queue rows done without witness path (intel cull regression).
- Do **not** schedule nuclear meltdown VFX before LOOP/SCRAM sim P1.

---

## Session start ritual

```powershell
python tools/orchestrator/scripts/sync_dispatch_subqueues.py
python tools/orchestrator/scripts/scan_queues_hub.py
# Pick from Tier 0 first — aps_presence_correction_queue.json
```

```text
[/PLAN-FINISH-EXEC-001] ΔWF→@coder-mcp APS-GRAM-TIER-GATES-LIVE-001 · DES-APS-SESSION-DUMP-001
```
