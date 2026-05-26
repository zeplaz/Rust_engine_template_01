# UI Phase 3 M2 — implementation full plan `v1` (PLAN-UI-P3-M2-IMPL-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-UI-P3-M2-IMPL-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — unblocks **UI-P3-M2-CODER-A** (construction + ecology) |
| **Rollup** | [`ui_phase3_minimap_m2_impl_plan_v1.md`](ui_phase3_minimap_m2_impl_plan_v1.md) |
| **Overlay plan** | [`ui_phase3_m2_minimap_overlay_plan_v1.md`](ui_phase3_m2_minimap_overlay_plan_v1.md) (**UI-P3-M2-PLAN**) |
| **Naming** | [`ui_phase3_minimap_track_naming_v1.md`](ui_phase3_minimap_track_naming_v1.md) |
| **Design** | [`minimap_d_m2_signoff_v1.md`](../../../src/dev/minimap_d_m2_signoff_v1.md) (**D-MINIMAP-M2**) |
| **Witness** | [`debug_runs/minimap_compositor_live.json`](../../../debug_runs/minimap_compositor_live.json) |
| **Coder queue** | [`ui_phase3_coder_queue_v1.md`](ui_phase3_coder_queue_v1.md) |

**No new Rust in this doc.** Full planner gate chain for M2 GPU minimap overlays — **coder A** = **UI-P3-M2-CODER-A** / queue alias **UI-P3-M3-001** (M2-02 + M2-03 only).

---

## What this plan unblocks

| Blocked work | Unblocked when |
|:---|:---|
| **UI-P3-M2-CODER-A** (coder A — construction + ecology) | This plan **SIGNED** + **D-MINIMAP-M2** + **M1 green** + **UI-P3-M2-001** logistics landed (or in same session after M2-01) |
| **UI-P3-M3-001** (queue alias — same slice as coder A) | Same as coder A — **not** design M3 |
| **UI-P3-M2-TRAY-OPT** | After coder A witness fields exist (M2-05 mask bits) |
| **UI-P3-M4-001** (design M3 fog/EW/units) | **Separate** — **not** this plan |

**Naming rule:** `ui_p3_m3_green` = **M2 construction + ecology**, not fog-of-war. Design M3 → **UI-P3-M4-001**.

---

## Gate chain (strict)

```text
D-MINIMAP-M1 (M1 spine)              ☑ SIGNED
        │
        ▼
UI-P3-M2-PLAN (overlay + legend)     ☑ SIGNED
D-MINIMAP-M2 (M2-01…M2-06)           ☑ SIGNED
PLAN-UI-P3-M2-IMPL-001 (this plan)    ☑ SIGNED 2026-05-25
        │
        ▼
UI-P3-M2-001 (M2-01 logistics)       ☑ DONE → logistics_rows > 0
        │
        ▼
UI-P3-M2-CODER-A / UI-P3-M3-001      ☑ DONE → construction_rows, ecology_rows, ui_p3_m3_green
        │   (coder A — max 3 files)
        ▼
UI-P3-M2-TRAY-OPT (M2-06)            ☑ DONE → ui_p3_m2_tray_opt_green
        │
        ▼
M2 rollup                             ☑ ui_p3_m2_green: true
```

**Forbidden:** Coder A before M1 `composite_ok`; second minimap extract; implementing FoW/EW in **UI-P3-M3-001**; editing `world_preview/window.rs` in same session.

---

## Coder lane map

| Lane | Queue ID | M2 item | Owner | Blocks rollup? |
|:---|:---|:---|:---|:---:|
| Logistics | **UI-P3-M2-001** | M2-01 | @coder (any) | Yes — `ui_p3_m2_green` needs rows |
| **Coder A** | **UI-P3-M2-CODER-A** / **UI-P3-M3-001** | M2-02, M2-03 | @coder A | Yes — `ui_p3_m3_green` |
| Tray | **UI-P3-M2-TRAY-OPT** | M2-06 | @coder (HUD) | Optional for `ui_p3_m2_green` if tray omitted |

### Coder A — unblock checklist

| # | Prerequisite | Met (2026-05-25) |
|:---:|:---|:---:|
| 1 | **PLAN-UI-P3-M2-IMPL-001** full plan **SIGNED** | ☑ |
| 2 | **D-MINIMAP-M2** **SIGNED** | ☑ |
| 3 | M1: `composite_ok`, `composite_path: GpuCompute`, `ui_p3_001_green` | ☑ |
| 4 | **UI-P3-M2-001** or witness shows `logistics_rows > 0` | ☑ |
| 5 | `MinimapOverlayMask` has `construction_heat`, `ecology_heat` | ☑ |
| 6 | Read naming authority — **UI-P3-M3-001 ≠ design M3** | ☑ |

---

## Coder A — implementation contract

### Scope (M2-02 + M2-03 only)

| Channel | Snapshot source | Mask bit | WGSL |
|:---|:---|:---|:---|
| Construction | `CorridorConstructionBook` | `construction_heat` | `minimap_composite.wgsl` construction channel |
| Ecology | `EcologyVisualSnapshot` | `ecology_heat` | ecology macro band |

### Primary files (max 3 per slice)

| File | Change |
|:---|:---|
| [`src/render/visual_domain_snapshots.rs`](../../../src/render/visual_domain_snapshots.rs) | `seed_minimap_m2_overlay_witness` |
| [`src/render/minimap_compositor/composite.rs`](../../../src/render/minimap_compositor/composite.rs) | row counts + uniform bits |
| [`src/render/minimap_compositor/live_proof.rs`](../../../src/render/minimap_compositor/live_proof.rs) | `ui_p3_m3_minimap_acceptance_green` |

**Also touched (session):** [`pass.rs`](../../../src/render/minimap_compositor/pass.rs), [`simulation_session.rs`](../../../src/gui/hud/simulation_session.rs) — `OnEnter(Simulation)` seed when rows low.

### Data flow

```text
FireSimulationSnapshot + CorridorConstructionBook + ClimateBand + EcologyVisualSnapshot
        │
        ▼
seed_minimap_m2_overlay_witness()     ← lib test + OnEnter(Simulation)
        │
        ▼
MinimapCompositorPlugin (GpuCompute)
  composite.rs / pass.rs
  minimap_composite.wgsl
        │
        ▼
live_proof → minimap_compositor_live.json
  construction_rows, ecology_rows, ui_p3_m3_green, ui_p3_m2_green
```

### Witness gates (coder A exit)

| Function | Pass when |
|:---|:---|
| `ui_p3_m3_minimap_acceptance_green` | `construction_heat_enabled` + `ecology_heat_enabled` + rows > 0 |
| `ui_p3_m2_minimap_acceptance_green` | M1 ok + logistics + **m3 green** + tray parity (if tray present) |

**JSON fields:**

```json
"construction_rows": 18,
"ecology_rows": 100,
"ui_p3_m3_green": true,
"ui_p3_m2_green": true
```

### Verify commands

```powershell
cargo test -p proc_A_dine01 --lib minimap_compositor
cargo test -p proc_A_dine01 --lib stage5
$env:MINIMAP_GPU_COMPOSITOR = "1"
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Coder A — copy-paste (active reference)

```
Lane: UI-P3-M2-CODER-A (alias UI-P3-M3-001 — M2 construction + ecology ONLY)
Read: prompts/guides/ui/ui_phase3_minimap_m2_impl_full_plan_v1.md
      src/dev/minimap_d_m2_signoff_v1.md M2-02/M2-03
      prompts/guides/ui/ui_phase3_minimap_track_naming_v1.md
First: seed_minimap_m2_overlay_witness + compositor construction/ecology channels
Max files: 3 — visual_domain_snapshots.rs, composite.rs|pass.rs, live_proof.rs
Do NOT: MinimapOnlyExtract; fog/EW/units (that's UI-P3-M4-001); world_preview chrome
Verify: cargo test -p proc_A_dine01 --lib minimap_compositor stage5
Witness: minimap_compositor_live.json → construction_rows, ecology_rows, ui_p3_m3_green
```

---

## Architecture invariants (all M2 coders)

| Invariant | Enforcement |
|:---|:---|
| Single compositor surface | `MinimapGpuImageNode` only |
| Snapshot readers only | `LogisticsVisualSnapshot`, book, ecology — no parallel LOD extract |
| Stage 5 spine | M2 edits must not break FULL_APP — fix witness, not readiness predicates |
| Sim defaults | [`simulation_minimap_overlay_defaults`](../../../src/gui/minimap_shell.rs) — four heats on in sim |
| World Preview disjoint | No `world_preview/window.rs` in M2 session |

---

## Witness bundle (planner + coder)

| File | Required fields |
|:---|:---|
| `minimap_compositor_live.json` | `composite_ok`, `logistics_rows`, `construction_rows`, `ecology_rows` |
| | `ui_p3_001_green`, `ui_p3_m3_green`, `ui_p3_m2_green`, `ui_p3_m2_tray_opt_green` |
| | `dual_minimap_present: false` |
| `stage5_full_app_live.json` | `ui_p3_m2_green` rollup (harness reads compositor proof) |

**Fleet snapshot (2026-05-25):** all greens **true**; logistics `2`, construction `18`, ecology `100`.

---

## Acceptance — PLAN-UI-P3-M2-IMPL-001

| # | Criterion | Met |
|:---:|:---|:---:|
| F1 | Gate chain documents **coder A** prerequisites | ☑ |
| F2 | Naming authority linked — no M3 fog confusion | ☑ |
| F3 | File + witness contract for M2-02/03 | ☑ |
| F4 | Logistics (M2-01) sequenced before or with coder A | ☑ |
| F5 | Tray + rollup closure criteria | ☑ |
| F6 | Design M3 explicitly out of scope → **UI-P3-M4-001** | ☑ |

**M2 implementation:** **CLOSED** (2026-05-24–25). This full plan is the authoritative unblock doc for audits and future coder sessions.

---

## Out of scope (forward)

| Item | Track | Plan |
|:---|:---|:---|
| Fog-of-war, EW, unit markers | Design **M3** | **UI-P3-M4-001** · [`minimap_m3_operational_overlay_spec_v1.md`](minimap_m3_operational_overlay_spec_v1.md) |
| Overlay legend PNG | Designer optional | **D-MINIMAP-M2-LEGEND** |
| CPU duplicate upload | Polish | `ux_gpu_minimap_design_v1.md` §8 |

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — PLAN-UI-P3-M2-IMPL-001 full plan |
| Designer | 2026-05-24 | **D-MINIMAP-M2 SIGNED** |
| Coder A | 2026-05-24–25 | **UI-P3-M2-CODER-A** / **UI-P3-M3-001** **DONE** |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Full plan — unblocks UI-P3-M2-CODER-A |
