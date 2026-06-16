# UI overhaul — Phase 3 M2 construction/ecology closure `v1` (UI-OH-M3-001)

| Field | Value |
|:---|:---|
| **Lane ID** | **UI-OH-M3-001** |
| **Planner queue** | **PLAN-UI-P3-M3-001** |
| **Coder queue (alias)** | **UI-P3-M3-001** · **UI-P3-M2-CODER-A** (same slice) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — **UI-P3-M3-001** **CLOSED** · optional tails **PASS** |
| **Naming authority** | [`ui_phase3_minimap_track_naming_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase3_minimap_track_naming_v1.md) |
| **M2 impl rollup** | [`ui_phase3_minimap_m2_impl_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase3_minimap_m2_impl_plan_v1.md) (**PLAN-UI-P3-M2-IMPL-001**) |
| **M2 impl full** | [`ui_phase3_minimap_m2_impl_full_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase3_minimap_m2_impl_full_plan_v1.md) |
| **Compositor rollup** | [`ui_phase3_minimap_compositor_full_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase3_minimap_compositor_full_plan_v1.md) (**PLAN-UI-P3-COMPOSITOR-001**) |
| **Master lane** | [`ui_overhaul_plan.md`](ui_overhaul_plan.md) |
| **Phase 2+3 closure** | [`ui_overhaul_phase23_closure_plan_v1.md`](ui_overhaul_phase23_closure_plan_v1.md) |
| **Live rollup** | [`witness_status_live_v1.md`](witness_status_live_v1.md) |
| **Witness** | `debug_runs/minimap_compositor_live.json` |

**No Rust in this deliverable.** Maps **PLAN-UI-P3-M3-001** into the UI-OH lane ID **UI-OH-M3-001**.

---

## Naming guard (mandatory)

| Term | Meaning |
|:---|:---|
| **UI-P3-M3-001** | **Design M2** construction + ecology GPU channels — witness `ui_p3_m3_green` |
| **UI-OH-M3-001** | UI-OH rollup alias for the same slice (this plan) |
| **Design M3** | Fog-of-war + EW + units — **UI-P3-M4-001** / **D-MINIMAP-M3** — **out of scope** |
| **`ui_p3_m3_green`** | M2 construction + ecology only — **not** FoW green |

**Do not** reopen **UI-OH-M3-001** for FoW/EW/units — use **UI-P3-M4-001** ([`minimap_m3_operational_overlay_spec_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/minimap_m3_operational_overlay_spec_v1.md)).

---

## Executive summary

| Track | Verdict |
|:---|:---|
| **M2-02** construction heat | **PASS** — `construction_rows: 18` |
| **M2-03** ecology heat | **PASS** — `ecology_rows: 100` |
| **UI-P3-M3-001** / **UI-P3-M2-CODER-A** | **CLOSED** — `ui_p3_m3_green: true` |
| **UI-P3-M3-UNITS-001** | **PASS** (optional tail) — `ui_p3_m3_units_001_green` |
| **UI-P3-M3-REPLAY-001** | **PASS** (optional tail) — `ui_p3_m3_replay_001_green` |
| **UI-OH-M3-001 rollup** | **PASS** — mirrors `ui_p3_m3_green` |

**Depends on:** **M1** spine green · **UI-P3-M2-001** logistics (for full `ui_p3_m2_green`, not for M3 slice alone).

**Does not block:** Stage 5 spine · **IND-E01** · **UI-OH-P4/P5** lanes (disjoint witnesses).

---

## Gate chain (PLAN-UI-P3-M3-001)

```text
D-MINIMAP-M1 (M1 spine)                    ☑
D-MINIMAP-M2 (M2 overlay sign-off)           ☑
PLAN-UI-P3-M2-IMPL-001 (impl full plan)      ☑ SIGNED 2026-05-25
        │
        ▼
UI-P3-M2-001 (M2-01 logistics)             ☑ logistics_rows > 0
        │
        ▼
UI-P3-M2-CODER-A / UI-P3-M3-001              ☑ construction + ecology
        │   seed_minimap_m2_overlay_witness
        ▼
ui_p3_m3_green                               ☑
        │
        ├─► UI-P3-M3-UNITS-001 (optional)     ☑
        ├─► UI-P3-M3-REPLAY-001 (optional)    ☑
        └─► UI-OH-M3-001 rollup               ☑ (this plan)
```

**Forbidden:** Implement FoW/EW in **UI-P3-M3-001**; second minimap extract; gameplay mutation from preview chrome.

---

## PASS gate — UI-P3-M3-001 (coder slice)

| # | Criterion | Witness path | Required | 2026-05-25 |
|:---:|:---|:---|:---:|:---:|
| M3-1 | Construction channel on | `construction_heat_enabled` | `true` | ☑ |
| M3-2 | Ecology channel on | `ecology_heat_enabled` | `true` | ☑ |
| M3-3 | Row depth | `construction_rows` **or** `ecology_rows` | `> 0` (either) | ☑ 18 / 100 |
| M3-4 | Acceptance rollup | `ui_p3_m3_green` | `true` | ☑ |
| M3-5 | Code predicate | `ui_p3_m3_minimap_acceptance_green` | pass | ☑ lib |

**Code anchor:** `src/render/minimap_compositor/live_proof.rs` — `ui_p3_m3_minimap_acceptance_green`.

**Test seed:** `seed_minimap_m2_overlay_witness` in `visual_domain_snapshots.rs` (M2-02/03, not design M3).

**Lib anchor:**

```powershell
cargo test -p proc_A_dine01 --lib ui_p3_m3
cargo test -p proc_A_dine01 --lib minimap_compositor
```

---

## PASS gate — optional tails (non-blocking)

| # | Queue ID | Witness path | 2026-05-25 |
|:---:|:---|:---|:---:|
| T1 | **UI-P3-M3-UNITS-001** | `ui_p3_m3_units_001_green` | ☑ |
| T2 | **UI-P3-M3-REPLAY-001** | `ui_p3_m3_replay_001_green` | ☑ |

Forward polish only — do **not** reopen **UI-OH-M3-001** if tails regress while M3-1…M3-4 stay green.

---

## UI-OH-M3-001 rollup (witness block)

**Coder slice (optional):** `ui_oh_m3_001` block in `minimap_compositor_live.json` (parity with `ui_oh_m2_001`).

| Path | Green when |
|:---|:---|
| `ui_oh_m3_001.gate` | `"UI-OH-M3-001"` |
| `ui_oh_m3_001.green` | `ui_p3_m3_green` **true** |
| `ui_oh_m3_001.construction_rows` | mirrors compositor |
| `ui_oh_m3_001.ecology_rows` | mirrors compositor |
| `ui_oh_m3_001.construction_heat_enabled` | `true` |
| `ui_oh_m3_001.ecology_heat_enabled` | `true` |

**Formula (planner):**

```text
ui_oh_m3_001.green :=
  construction_heat_enabled
  AND ecology_heat_enabled
  AND (construction_rows > 0 OR ecology_rows > 0)
-- equivalent to ui_p3_m3_green / ui_p3_m3_minimap_acceptance_green
```

After compositor proof refresh, **`ui_oh_m3_001.green`** mirrors **`ui_p3_m3_green`**.

**Distinct from UI-OH-M2-001:** **UI-OH-M2-001** = logistics + construction only; **UI-OH-M3-001** = construction + ecology (no logistics requirement).

---

## Witness field table

| Phase | File | Field | Role |
|:---|:---|:---|:---|
| M2-02/03 | `minimap_compositor_live.json` | `construction_rows` | Corridor construction heat |
| M2-02/03 | `minimap_compositor_live.json` | `ecology_rows` | Ecology heat |
| Coder | `minimap_compositor_live.json` | `ui_p3_m3_green` | **UI-P3-M3-001** acceptance |
| M2 full | `minimap_compositor_live.json` | `ui_p3_m2_green` | Logistics + M3 + tray (rollup) |
| OH M2 | `minimap_compositor_live.json` | `ui_oh_m2_001.green` | Logistics + construction OH gate |
| OH M3 | `minimap_compositor_live.json` | `ui_oh_m3_001.green` | **UI-OH-M3-001** rollup (proof writer) |
| Design M3 | `minimap_compositor_live.json` | `ui_p3_m4_green` | **UI-P3-M4-001** — separate lane |

---

## Relationship to other lanes

| Lane | Relationship |
|:---|:---|
| **UI-OH-M2-001** | Subset (logistics + construction); **does not** require ecology |
| **UI-P3-M2-001** | M2-01 logistics — prerequisite for `ui_p3_m2_green`, not `ui_p3_m3_green` alone |
| **UI-P3-M4-001** | Design M3 FoW/EW — **downstream**, already green on disk |
| **PLAN-UI-P3-COMPOSITOR-001** | Parent rollup — M1+M2+M3+M4 witness spine |
| **UI-OH-GATE-001** | Steward bundle includes compositor refresh |

---

## Regression (maintain only)

```powershell
cargo test -p proc_A_dine01 --lib ui_p3_m3 minimap_compositor
cargo test -p proc_A_dine01 --lib steward_ui_oh_gate_001_lib_bundle
cargo orchestrate --skip-cargo
```

**Visual (optional):** `MINIMAP_GPU_COMPOSITOR=1 cargo run -p proc_A_dine01 --release -- --test visual` — refreshes compositor JSON under envelope.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | **PLAN-UI-P3-M3-001** → **UI-OH-M3-001** — M2 construction/ecology CLOSED on disk |
