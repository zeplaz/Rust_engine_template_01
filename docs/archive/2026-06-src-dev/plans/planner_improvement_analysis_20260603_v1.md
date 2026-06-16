# Planner improvement analysis — June 2026 `v1`

| Field | Value |
|:---|:---|
| **ID** | **PLANNER-IMPROVE-ANALYSIS-001** |
| **Date** | 2026-06-03 |
| **Purpose** | Second-pass fleet analysis → prioritized **@planner** work that reduces coder thrash, doc drift, and gate ambiguity |
| **Inputs** | Queue v5.5.0 · audit v18 · construction drain · DSM Track A · infra epic closure · HANDOFF |

**Rule:** Planner owns **exec slices, audits, gate definitions, queue hygiene** — not Rust, not Tk/MCP implementation.

---

## 1. Executive summary — what improved since audit v18

| Domain | v18 (2026-06-02) | Now (2026-06-03) | Planner action |
|:---|:---|:---|:---|
| **Construction P1–P6** | CON-P2/P3 open | **Closed** — 144/144 lib; scaling + parametric + organic green | Unblock **PLAN-AUDIT-019** |
| **Infra E0–E3** | E0–E1 partial | **Closed** — `transport_network_live.json` green | Sign tail exec **E4–E6** |
| **Infra E4–E6** | Not started | **Mostly closed (B)** — E4-001/003/004, E5-001/003, E6-003; **A tail open** | Thin exec for E4-002, E5-002, E6-001/002/004 |
| **PHASE-STABLE P2** | Many open tails | **Mostly closed** (ENG/RENDER/CONTAIN/STAB-CI) | Refresh gate rollup in v19 |
| **G-PLAY-01** | OPEN | Still **OPEN** (operator runbook exists; no sign-off row) | **PLAN-G-PLAY-CLOSE-001** |
| **APS Track A** | Phase 0 gate | Phase 0 **closed**; coder-mcp on 2–4; WRK/ATL **QC UX gap** | **PLAN-DSM-WRK-ATL-001** |
| **Designer** | Long-run active | **Drained** — on-call only | No planner load unless SIGNOFF |
| **Coder B** | 12 rows | **1 row** (ECON-OG-SAVE) + growth execute gap | **PLAN-ORG-GROWTH-EXEC-002** |
| **Queue metadata** | Dual-track v18 | **Stale** — HANDOFF, `coder_b_next`, planner `next_phase` | **PLAN-QUEUE-SYNC-002** |

**Bottom line:** Coders are execution-bound on **thin tails** (infra A, organic save/approve, APS MCP). Planner value now is **(a) audit v19**, **(b) unblocking P7 logistics + organic execute**, **(c) DSM closure criteria**, **(d) queue/doc sync**.

---

## 2. Gap analysis (five friction classes)

### A — Witness green ≠ product ready (still #1)

- `play_scenario_live.json` green but **G-PLAY-01** needs operator §1–8.
- `stage7_play_live.json` may still lag `ind_e02_green` on disk refresh cadence.
- **Planner fix:** v19 matrix adds **Operator sign-off** column; **PLAN-G-PLAY-CLOSE-001** checklist ties runbook → audit row.

### B — Program handoffs missing exec (coders invent scope)

| Handoff | Blocker | Missing plan |
|:---|:---|:---|
| **G-INFRA-07 → CON-P7** | E1-004 **done** | No `plan_construction_p7_logistics_exec_*` |
| **Approve → ConstructionPlanQueue** | Inspector wired, execute stub | No approve/policy exec slice |
| **E5-002 graph-only paths** | Coder A row ready | No acceptance table vs logistics witness |
| **WRK○ / ATL○** | DSM routing | No witness keys for worker QC + atlas validate UX |

### C — Doc / queue drift (orchestrator tax)

| Artifact | Drift |
|:---|:---|
| `planner_active_queue.json` `_meta.date` | 2026-05-27; `next_phase` still CON-P2+INFRA-E0 |
| `PLAN-AUDIT-019` `blocked_by` | Lists CON-P3 — **done on disk** |
| `infrastructure_program.coder_b_next` | Lists **done** E4/E5 slices |
| `designer_planner_parallel_wave` | Shows designer idle; HANDOFF still had audit as active |
| Audit v18 § Active work | Still lists CON-P2, CONTAIN-MINIMAP as open — **closed** |

### D — Parallel programs without territory refresh

Six lanes advance simultaneously:

1. Construction drain (B: save/approve)
2. Infra A tail (E4/E5/E6)
3. APS Track A (coder-mcp 2–4)
4. Weather (coder C)
5. Grammar Track C / planner-mcp schemas
6. Bevy SIM-HUD slices (@coder)

**Risk:** same-week edits to `play_scenario.rs`, `economy/activation`, `construction/` from infra + construction + play truth.

**Planner fix:** **PLAN-TERRITORY-MATRIX-002** — one table: file prefix → owning program → merge window.

### E — Deferred work re-surfaces

- Track B **MCP-PILOT-GRAMMAR-001** manual keyframe — must stay **DEFER** in index + queue.
- APS Phase 9 E2E — defer until WRK/ATL ○ → ★.

**Planner fix:** **PLAN-DEFER-REGISTRY-001** — single signed defer list referenced by orchestrator.

---

## 3. Recommended @planner work queue (priority order)

### P0 — Reconcile truth (this week)

| P | ID | Deliverable | Unblocks | Est |
|:---:|:---|:---|:---|:---:|
| 1 | **PLAN-AUDIT-019** | [`planner_status_audit_v19.md`](planner_status_audit_v19.md) + checklist 019 | Queue sync; operator knows real open tails | 4h |
| 2 | **PLAN-QUEUE-SYNC-002** | Update `planner_active_queue`, `coder_active_queue` `_meta`, HANDOFF, `infrastructure_program.coder_*_next` | Stops wrong picks | 2h |
| 3 | **PLAN-G-PLAY-CLOSE-001** | Operator closure checklist → v19 **G-PLAY-01** row | Product sign-off | 1h |

**PLAN-AUDIT-019 must include:**

- New columns: **Operator**, **DSM node** (where applicable)
- Mark **closed**: CON-P2/P3, CONTAIN-MINIMAP, STAB-CI, infra E0–E3, most E4–E6 B slices
- **Open**: G-PLAY-01, ECON-OG-SAVE/APPROVE, infra A tail, APS WRK/ATL, weather WITNESS
- Sign **PLAN-LEDGER-REFRESH-019**

### P1 — Unblock coders (next)

| P | ID | Deliverable | Unblocks | Owner hint |
|:---:|:---|:---|:---|:---|
| 4 | **PLAN-CON-P7-LOGISTICS-001** | Exec: graph-only freight paths; witness keys in `logistics_throughput_live.json` | **INFRA-E5-002**, **CON-P7** | Coder A |
| 5 | **PLAN-ORG-GROWTH-EXEC-002** | Approve→`ConstructionPlanQueue`; `AutoBuildPolicy` tick rules; witness `execute_via_pipeline` | **PROC-OG-APPROVE-001**, **PROC-OG-POLICY-001** | Coder B |
| 6 | **PLAN-INFRA-TAIL-001** | Thin exec: E4-002 utility flow, E6-001 material tags, E6-002 nav — acceptance one-liners | Coder A active[] | Coder A |
| 7 | **PLAN-DSM-WRK-ATL-001** | Closure criteria: WRK○ → BUILD-WORKER witness; ATL○ → atlas validate UX witness | coder-mcp + operator | Coder-mcp |

### P2 — Art / grammar coordination (planner-mcp handoff)

| P | ID | Deliverable | Notes |
|:---:|:---|:---|:---|
| 8 | **APS-VALIDATOR-PLAIN-001** | Sign [`aps_validator_plain_language_v1.md`](aps_validator_plain_language_v1.md) or extend | Already drafted — planner **sign + wire to Phase 7** |
| 9 | **GRAMMAR-ITER-SNAPSHOT-001** | Schema extend | Delegate **@planner-mcp** per parallel wave |
| 10 | **PLAN-DEFER-REGISTRY-001** | Track B keyframe + APS E2E defer | Prevents re-queue noise |

### P3 — Horizon (after P0–P1)

| ID | When |
|:---|:---|
| **PLAN-WEATHER-EXEC-002** | After WEATHER-WITNESS-001 green |
| **PLAN-TERRITORY-MATRIX-002** | Before next multi-coder week |
| **PLAN-AUDIT-020** | After G-PLAY operator sign-off + organic execute green |

---

## 4. DSM Track A — planner closure model

```
MAT★ → APS★ → SNAP★ → WRK○ → ATL○ → RT○
```

| Node | Planner defines | Witness / exit |
|:---|:---|:---|
| **WRK○** | Artist-readable worker status + material honesty at bake | `debug_runs/build_worker_001_live.json` keys + plain-language panel spec |
| **ATL○** | Validate errors → sentences; UV grid legend (designer done) | `aps_preview_catalog_live.json` / atlas validate block in transport or dedicated APS witness |
| **RT○** | Deferred — registry stamp in engine | After WRK+ATL ★ |

**Cross-link:** [`plan_aps_artist_tool_exec_v1.md`](plan_aps_artist_tool_exec_v1.md) Phases 3, 8, 9 · [`plan_mcp_productivity_chain_v1.md`](plan_mcp_productivity_chain_v1.md).

---

## 5. Construction × infra alignment (planner-only decisions)

| Decision | Recommendation |
|:---|:---|
| **P7 logistics start** | **Now** — E1-004 hydrate green; write **PLAN-CON-P7-LOGISTICS-001** before coder A picks E5-002 |
| **G-TOWN-ONE** | Maintain — settlement save slice (**ECON-OG-SAVE**) before policy auto-build at scale |
| **G-CON-02** | Maintain — approve path must not set Operational; exec-002 must say so explicitly |
| **Infra vs construction file overlap** | `corridor_transport.rs`, `play_scenario.rs` — **one PR per program per week** |

---

## 6. What planner should **not** do now

- Re-open CON-P2/P3/parametric boards (closed on disk).
- Re-plan full infra E0–E3 (done).
- Un-defer Track B keyframe or APS Phase 9 E2E without DSM WRK/ATL ★.
- Write Rust or MCP Python — route to @coder / @coder-mcp.
- Collapse six parallel lanes into a single “primary” — **dual/multi track** remains policy per [`planner_program_alignment_v1.md`](planner_program_alignment_v1.md).

---

## 7. Paste — @planner session (copy-paste)

```text
You are @planner on Rust_engine_template_01 — plans, audits, queue hygiene only.

READ FIRST:
- docs/archive/2026-06-src-dev/plans/planner_improvement_analysis_20260603_v1.md  (this doc)
- src/dev/planner_program_alignment_v1.md
- docs/archive/2026-06-src-dev/plans/construction_coder_drain_order_v1.md
- tools/orchestrator/queues/coder_active_queue.json v5.5.0

SESSION (pick in order):
1) PLAN-AUDIT-019 — planner_status_audit_v19.md
   - Refresh per-witness matrix; close CON-P2/P3/infra E0-E3 tails
   - Add Operator + DSM columns
   - plan_ledger_refresh_019_checklist_v1.md → SIGNED

2) PLAN-QUEUE-SYNC-002 — align HANDOFF, planner_active_queue next_phase,
   infrastructure_program.coder_a_next / coder_b_next with done[]

3) PLAN-CON-P7-LOGISTICS-001 — exec for INFRA-E5-002 + CON-P7 hook
   (G-INFRA-07 now unblocked)

4) PLAN-ORG-GROWTH-EXEC-002 — approve→ConstructionPlanQueue + policy tick
   (unblocks coder B after ECON-OG-SAVE-001)

5) PLAN-DSM-WRK-ATL-001 — WRK/ATL closure criteria for Track A

Deliver: markdown exec plans + queue JSON updates + audit v19 SIGNED.
Do NOT: implement Rust/MCP; do NOT reopen closed construction phases.
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Post-designer-return + infra E1/E3 closure analysis |
