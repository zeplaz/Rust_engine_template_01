# Stage 5.5 — open lane (post–Stage 5 operational)

**Prerequisites:** [`stage5_operational_signoff.md`](stage5_operational_signoff.md) — Stage 5 operational gate **CLOSED**.

**Status (2026-05-23):** All tracks **DONE** — [`stage5_5_active_todos.md`](stage5_5_active_todos.md). **Open next:** [`stage6_plan_open.md`](stage6_plan_open.md).

Stage 5.5 is **infrastructure hardening + product spine extension**, not a repeat of the FULL_APP exit gate. New features must still attach to existing contracts (no parallel representation stacks).

---

## Entry criteria (pick one primary track per cycle)

| Track | Goal | Primary docs | Proof style |
|-------|------|--------------|-------------|
| **5.5-A View runtime** | Land `view_runtime` module; VM-06…11 checklist in code | [`view_runtime_architecture_v1.md`](view_runtime_architecture_v1.md) §15 | `infrastructure_view_isolation_live.json` + new unit tests |
| **5.5-B Render / GPU** | Instanced tile path authoritative; retire CPU gizmo fallback when policy says so | [`base_finsh_5.md`](../../prompts/guides/base_finsh_5.md) §2, `gpu_tile_debug` | Visual + render witness |
| **5.5-C Perf shell** | p95 frame budget; logging cost attribution | [`operational_readiness_vs_infrastructure_perf_v1.md`](../../prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md) | `perf_attribution_60s.md` |
| **5.5-D Wave S (product)** | Blueprint / shell RON round-trip; BQ-128/130/133 | [`backlog_serialization_preview_streaming_runbook_v1.md`](../../prompts/guides/backlog_serialization_preview_streaming_runbook_v1.md), designer briefs | Editor + fixture JSON |
| **5.5-E Fire sim depth** | Streaming, LOD tiers, ecology F1 follow-up | [`fire_ecology_f1_todos.md`](fire_ecology_f1_todos.md), triage T3 | `fire_ecology_live.json` (not FULL_APP gate) |

**Default recommendation:** **5.5-A** (view runtime) — unblocks multiview correctness for everything else.

---

## Explicitly out of scope for 5.5-A (other lanes)

| Lane | Where |
|------|--------|
| Construction gameplay expansion | `construction_*` boards — already operational green |
| Stage 7 behavioral AI | `stage7_*` planning docs |
| Re-opening FULL_APP gate | Only if regression in `stage5_readiness_passes` |

---

## Witnesses to extend (not replace Stage 5)

| File | Use |
|------|-----|
| `debug_runs/infrastructure_view_isolation_live.json` | Per-view isolation regression |
| `debug_runs/replay_editor_parity_live.json` | Replay ring + editor parity stamp |
| `debug_runs/stage5_full_app_live.json` | Re-run after major spine changes only |

---

## First actions (suggested order)

1. `cargo test -p proc_A_dine01 --lib` (baseline)
2. Read [`view_runtime_architecture_v1.md`](view_runtime_architecture_v1.md) §15 + implement `view_runtime/{ids,surface,authority,trace}.rs` (types + trace only is OK for slice 1)
3. VM-A2 / VM-A3 from same doc (writer tags + minimap focus)
4. Pick Wave S **or** perf track from product priority — not both in one closure cycle

---

## Agent routing

| Work | Agent |
|------|--------|
| View runtime / VM | sim-steward, debug-intelligence, planner |
| GPU tile / WGSL | coder |
| Wave S / UX | designer + coder |
| Fire sim depth | coder + sim-steward |

---

## Sign-off for Stage 5.5

Stage 5.5 closes when the **chosen track** has a runbook row marked Done with on-disk proof JSON and no FULL_APP regression.
