# Stage 7 Play — industrial / logistics / construction E2E `v1`

| Field | Value |
|:---|:---|
| **Track ID** | `S7-PLAY` |
| **Version** | `1.0.0` |
| **Status** | **ACTIVE** — coder **DONE** · designer **OPEN** |
| **Exit milestone** | **Stage 7 Play SIGNED** — one sim session: build chain, see flow, industrial witness green |
| **Sign-off** | [`../stage_tracks_signoff_ledger_v1.md`](../stage_tracks_signoff_ledger_v1.md) § S7-PLAY |
| **Index** | [`../stage_tracks_execution_index_v1.md`](../stage_tracks_execution_index_v1.md) |

**Does not reopen** Stage 5/6 gates ([`operational_readiness_vs_infrastructure_perf_v1.md`](../../prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md)).

---

## North star

Player can **place → build → commission → run** a concrete supply chain and see **logistics + power + production** reflected in witnesses — not fixture-only `production_green`.

---

## Witness bundle (exit)

| File | Required fields |
|:---|:---|
| `debug_runs/industrial_activation_live.json` | `activation_green: true`, `concrete_chain_e2e.production_green: true`, `open_todos: 0` |
| `debug_runs/construction_stage_live.json` | operational + Round 3 witnesses current |
| `debug_runs/logistics_throughput_live.json` | throughput > 0 in seeded scenario |
| `debug_runs/minimap_compositor_live.json` | `logistics_rows > 0` when corridors active |

---

## @designer instructions

### S7P-DESIGN-001 — Play scenario script (required before operator sign-off)

**Deliver:** [`src/dev/stage7_play_scenario_v1.md`](stage7_play_scenario_v1.md) (create on sign-off) with:

| Step | Player action | Expected read |
|:---:|:---|:---|
| 1 | Enter Simulation after world gen | HUD collapsed; build rail visible |
| 2 | Place aggregate mine → kiln → mixer (construction) | Sites advance phases |
| 3 | Wait for Operational | Production tick / HUD indicator |
| 4 | Open logistics overlay / minimap | Corridor heat visible |
| 5 | Optional: place smelter load | Grid stress message or witness flag |

**Reference:** [`ui_construction_playtest_v1.md`](../ui_construction_playtest_v1.md), [`concrete_industry_sim_runbook_v1.md`](../../prompts/guides/concrete_industry_sim_runbook_v1.md)

**Sign-off:** Designer marks **SIGNED** in scenario doc header when steps reproducible in `--test visual` or manual sim.

### S7P-DESIGN-002 — Grid overload UX note (optional)

Read [`power_damage_ui_persistence_v1.md`](../../prompts/designer_questions/production_economy/power_damage_ui_persistence_v1.md) — one paragraph: what player sees when smelter overloads (toast vs tray vs diagnostics). No Rust.

---

## @coder instructions

### Slice map

| ID | Goal | Status | Evidence |
|:---|:---|:---|:---|
| **S7P-IND-001** | `activation_green` + board sync | **DONE** | `activation_green: true` 2026-05-24 |
| **S7P-CON-001** | P9 catalog in sim | **DONE** | `con_e01_p9_green: true` |

| ID | Goal | Files (≤3/step) | Prereq |
|:---|:---|:---|:---|
| **S7P-IND-002** | Power plant + transformer placeable activation | `economy/activation/`, `entities/production/` | S7P-IND-001 |
| **S7P-LOG-001** | Scenario seed → throughput witness | transport seed, `logistics_throughput_live.json` writer | LOG-E01 code |
| **S7P-GRID-001** | Smelter load → grid witness field | `PowerRuntimePlugin`, activation bridge | S7P-IND-001 |

**Authority:** [`industrial_activation_pipeline.md`](../industrial_activation_pipeline.md) — activation on `Operational` only; no second construction execute path.

### Copy-paste — S7P-IND-001 (primary)

```
Track: S7-PLAY — S7P-IND-001
Read: src/dev/stages/stage7_play_plan_v1.md
      src/dev/industrial_activation_pipeline.md § Exit gate
First: run cargo test -p proc_A_dine01 --lib industrial_activation
      fix open_todos until activation_green in live_proof
Do NOT: reopen Stage 5 spine; bypass ConstructionSite phase funnel
Verify: cargo test -p proc_A_dine01 --lib industrial_activation stage5
        cargo run -p proc_A_dine01 --release -- --test visual
Witness: debug_runs/industrial_activation_live.json → activation_green: true
```

### Copy-paste — S7P-CON-001

```
Track: S7-PLAY — S7P-CON-001
Read: src/dev/construction_recovery_todos.md (P9 rows)
First: reconcile PHASE2-BUILD-16…20 in sim vs static board
Do NOT: new placement authority outside src/construction/
Verify: cargo test -p proc_A_dine01 --lib construction
Witness: debug_runs/construction_stage_live.json refreshed
```

### Acceptance — Stage 7 Play exit

| # | Criterion |
|:---:|:---|
| E1 | `activation_green: true` in `industrial_activation_live.json` |
| E2 | Designer scenario **SIGNED** |
| E3 | `cargo test -p proc_A_dine01 --lib construction industrial_activation stage5` green |
| E4 | Operator ran `--test visual` once after last code merge |
| E5 | Minimap or logistics witness shows flow when scenario run |

---

## @operator instructions

1. `cargo run -p proc_A_dine01 --release -- --test visual`
2. Follow designer scenario doc steps 1–4
3. Confirm JSON timestamps updated under `debug_runs/`
4. If `activation_green` false, paste `open_todos` array into handoff — do not guess fixes

---

## Parallel lanes (disjoint files)

| Lane | Can run parallel to S7P-IND-001? |
|:---|:---|
| VFX Phase 2 | ✅ |
| UI Phase 4 | ✅ |
| Infra 5.5+ | ✅ small slices only |
| Wave C | ⚠️ avoid same session as heavy construction test |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | Initial Stage 7 Play plan |
