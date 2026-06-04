# PLAN-BQ128-APPLY-EXEC-001 — BQ-128 preset apply → ghost `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-BQ128-APPLY-EXEC-001** |
| **Coder lane** | **BQ-128-APPLY-001** |
| **Parent** | [`bq128_editor_path_plan_v1.md`](bq128_editor_path_plan_v1.md) § BQ-128-APPLY-001 (**PLAN-UX-BQ128-001** SIGNED) |
| **Design** | [`bq128_editor_path_design_note_v1.md`](bq128_editor_path_design_note_v1.md) · [`bq128_apply_ghost_ux_review_v1.md`](bq128_apply_ghost_ux_review_v1.md) (**DESIGN-BQ128-APPLY-UX-001** PASS) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@planner` |
| **Status** | **READY (planner finalized)** |

**Planner sign-off:** PASS (2026-05-27). MVP RON/hydrate **CLOSED** — apply ghost is the open slice.

---

## Scope

Wire **preset picker → Apply → ghost placement only**. Commit remains Enter / existing execute funnel per [`construction_invariants.md`](construction_invariants.md).

---

## Authority map

| Resource | Writer | Rule |
|:---|:---|:---|
| `WaveSImportedBlueprints` | Wave S hydrate | read-only for picker |
| `ConstructionQueueIntent` | construction panel | emit `ApplyPreset { index }` only |
| Ghost / preview | `BuildPlacementPreview` | Apply updates ghost — **no** commit |
| `CommitConstructionSiteEvent` | execute funnel only | Enter / confirm unchanged |
| Witness | lib test + optional `wave_s` JSON extension | no hand-edit |

---

## Task list (≤3 files per PR)

### BA-1 — Intent + panel UI

| File | Change |
|:---|:---|
| `src/construction/construction_queue_intent.rs` | `ApplyPreset { index: usize }` |
| `src/construction/pending_construction_panel.rs` | preset list + Apply button |
| `src/construction/mod.rs` | register intent routing |

### BA-2 — Ghost consumer

| File | Change |
|:---|:---|
| `src/construction/build_interaction.rs` or placement module | map preset → ghost footprint/origin/archetype |
| Reuse validation | `allows_commit` unchanged until Enter |

### BA-3 — Tests + witness

| File | Change |
|:---|:---|
| `src/construction/integration_tests.rs` | Apply sets ghost; no site spawn until commit |
| Optional witness field in `wave_s_blueprint_roundtrip.json` writer | `bq128_apply_ghost_wired: true` |

---

## Witness / acceptance

| Source | Field | Required |
|:---|:---|:---|
| `wave_s_blueprint_roundtrip.json` | `roundtrip_ok: true` | prereq ( **CLOSED** ) |
| `wave_s_hydrate_live.json` | `blueprint_count > 0` | prereq |
| Apply slice | `bq128_apply_ghost_wired` or lib-only proof | **true** after BA-3 |

**Green predicate:**

```text
roundtrip_ok == true
AND apply_sets_ghost_without_commit == true
AND enter_commit_path_unchanged == true
```

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib wave_s construction
```

Manual: Import preset → Apply → ghost visible → Enter commits once.

---

## Anti-patterns

- Apply → instant site spawn
- Logic outside `src/construction/`
- WorldGen / map editor scope
- Reopening UX-E02-BQ128 design sign-off

---

## Coder handoff

| Field | Value |
|:---|:---|
| **Unblocks** | `BQ-128-APPLY-001` |
| **Depends on** | DESIGN-BQ128-APPLY-UX-001 PASS; `roundtrip_ok` green |
| **Acceptance** | Apply→ghost wired; commit funnel unchanged; construction + wave_s lib green |
