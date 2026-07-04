# Coder remaining work — post Bevy 0.19 `v1`

| Field | Value |
|:---|:---|
| **Date** | 2026-07-03 |
| **Bevy** | **0.19** on `master` — `debug_runs/mig_bevy_019/mig_v1_gate.json` → `gate_pass: true` |
| **Rule** | Keep **non-0.19 ecosystem features OFF** unless explicitly upgraded (see § Feature gates) |
| **Companion** | [`coder_todos_v1.md`](coder_todos_v1.md) (authoritative pick board) · [`coder_non_migration_todos_v1.md`](coder_non_migration_todos_v1.md) (stream detail) |

---

## Feature gates — keep OFF until upgraded

| Feature / crate | Status | Why OFF |
|:---|:---|:---|
| **`bevy_tilemap_adapter`** | **OFF** (not in `default`) | `bevy_ecs_tilemap` **0.18.1** — no stable 0.19 line; use `tile_world_fallback` raster |
| **`hanabi_l3`** | optional | Hanabi embellishment; opt-in only |
| **`research_lmodels`** | optional | linfa stack stub |
| **`legacy_engine`** | optional | historical stub |
| **`legacy_transport_ecs_stubs`** | optional | INFRA-E0-003 legacy |
| **`engine_deep_debug`** | optional | intrusive witnesses |
| **`tracy`** | optional | profiling |

**Do not** add these to `[features] default`. Re-enable `bevy_tilemap_adapter` only after a 0.19-compatible `bevy_ecs_tilemap` release + compat witness.

---

## Stream A — CITY / BSN product (coder) — **CLOSED**

| ☑ | **CITY-C6-BSN-001** | `city_c6_bsn_001_live.json` |
| ☑ | **CITY-P1/P2** | `city_p1_001_live.json` · `city_p2_001_live.json` |
| — | **DR-MIG-TILEMAP** | steward only — `bevy_tilemap_adapter` OFF |

---

## Stream B — PERF / VFX (coder_b)

| ☐ | ID | Goal |
|:---:|:---|:---|
| ☐ | **PERF-INSTR-VFX-002** | lib green · display acceptance pending |

---

## Stream C — BQ + APSR (coder / coder-mcp)

| ☐ | ID | Owner |
|:---:|:---|:---|
| ☑ | BQ-C1–C4 | coder-mcp + coder |
| ☑ | BQ-A1 | coder — `edge_adjacency.rs` |
| ☑ | BQ-A2 | coder — `building_quality_live.json` |
| ☑ | APSR-S1–S2 | coder-mcp |

---

## Stream D — BQ tail + APSR P/D/Q (unblocked)

| ☐ | Phase | Slices |
|:---:|:---|:---|
| ☑ | BQ-H1/H2/H3 | facade by_massing · street-facing · v0 freeze shim |
| ☑ | BQ-Q1 | APS QC strip · `apsr_a4_q1_001_live.json` |
| ☐ | BQ-K | kit charters · slot audit · style-pack fill |
| ☐ | BQ-Q2/Q3 | screenshot QC · golden-seed done bar |
| ☐ | APSR-P/D/Q | panel split · design lint · QC surfaces (Q1 strip ☑) |

---

## Stream E — POST-MIG perf (NOT migration — program closed)

MIG-A11/A13/A17 “deep” work lives under **PERF-GPU-TERRAIN** / fire perf, not migration.

| ☑ | ID | Note |
|:---:|:---|:---|
| ☑ | MIG-PROGRAM | **CLOSED** — `mig_a_program_close.json` |
| ☐ | GPU-P0C-PRIME | Instanced terrain + depth prepass (plan_gpu_terrain) |

---

## Session pick order (recommended)

1. **SCH-W1-T1-001** (@coder)  
2. **CLN-P0-*** (@coder_a/b) parallel  
3. **PERF-INSTR-VFX-002** display run (operator)  
4. **BQ-K** kit charters (designer-mcp) · **BQ-Q3** golden seeds  
5. **GPU-P0C-PRIME** · **DR-MIG-TILEMAP** steward only
