# BQ-128 editor path — plan `v1` (PLAN-UX-BQ128-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-UX-BQ128-001** |
| **Designer lane** | **UX-E02** · **UX-E02-BQ128-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — design **CLOSED** · apply slice **OPEN** |
| **Design authority** | [`bq128_editor_path_design_note_v1.md`](bq128_editor_path_design_note_v1.md) **SIGNED** |
| **Wave S** | [`wave_s_open.md`](wave_s_open.md) |
| **Invariants** | [`construction_invariants.md`](construction_invariants.md) |
| **Witness** | [`debug_runs/wave_s_blueprint_roundtrip.json`](../../debug_runs/wave_s_blueprint_roundtrip.json) · [`debug_runs/wave_s_hydrate_live.json`](../../debug_runs/wave_s_hydrate_live.json) |

**No Rust in this doc.** Planner rollup for **BQ-128** blueprint preset editor path: **UX-E02** designer sign-off + coder phase-2 slices.

---

## Track map

| Track | ID | Owner | Status |
|:---|:---|:---|:---:|
| **Design** | **UX-E02-BQ128-001** | `@designer` | **CLOSED** — design note **SIGNED** |
| **MVP IO** | **WS-A01…A04** (Wave S) | `@coder` | **DONE** — RON round-trip + import/export buffer |
| **Phase 2a** | **BQ-128-APPLY-001** | `@coder` | **DONE** — preset picker → ghost |
| **Phase 2b** | **BQ-128-APPLY-002** | `@coder` | **DONE** — merge vs replace on import |
| **Phase 2c** | **BQ-128-EXT-001** | docs | **DEFERRED** — offline `presets.ron` edit |

**Does not block:** Stage 5 FULL_APP, minimap compositor, VM-09, industrial activation.

---

## Master gate chain

```text
UX-E02-BQ128-001 (designer path)                 ☑ SIGNED 2026-05-25
        │
        ▼
WS-A03 / wave_s blueprint RON + hydrate          ☑ roundtrip_ok
        │
        ▼
BQ-128-APPLY-001 (preset picker → ghost)         ☑ DONE
        │
        ├─► BQ-128-APPLY-002 (merge/replace)       ☑ DONE
        └─► BQ-128-EXT-001 (external editor)       ☐ deferred
```

---

## UX-E02 — designer deliverable (**CLOSED**)

**Queue:** **UX-E02-BQ128-001** (registry alias **BQ-128**, **UX-E02**)

| # | Designer locked | Doc § |
|:---:|:---|:---|
| 1 | `blueprints/presets.ron` under save bundle | On-disk contract |
| 2 | **Simulation-only** primary path | Session matrix |
| 3 | Import / export / capture flows | Editor path MVP |
| 4 | No second commit authority | Authority boundaries |
| 5 | Phase-2 picker scope for coder | § phase 2 |
| 6 | WorldGen / map editor out of scope | Session matrix |

**Verdict:** ☑ **SIGNED** — full detail in [`bq128_editor_path_design_note_v1.md`](bq128_editor_path_design_note_v1.md).

**Briefs:** [`experience_layer_ux_hud_designer_brief_v1.md`](../docs/archive/2026-06-prompts-guides/runbooks/guides/experience_layer_ux_hud_designer_brief_v1.md) §4 · [`rulebook_backlog_designer_brief_v1.md`](../docs/archive/2026-06-prompts-guides/runbooks/guides/rulebook_backlog_designer_brief_v1.md) **BQ-128**.

---

## MVP landed (do not re-implement)

| Step | Module | Witness |
|:---|:---|:---|
| `WAVE_S_BLUEPRINT_PRESETS_REL_PATH` | `io/save/wave_s_artifacts.rs` | `artifact_path` in roundtrip JSON |
| Hydrate `WaveSImportedBlueprints` | `hydrate_wave_s_artifacts_from_bundle` | `wave_s_hydrate_live.json` |
| Panel import/export RON buffer | `construction/pending_construction_panel.rs` | manual + lib `wave_s` |
| Capture to bundle | `gui/hud/dock_shell.rs` | explicit user gesture |
| DTO | `construction/blueprint_preset.rs` | `roundtrip_ok: true` |

```powershell
cargo test -p proc_A_dine01 --lib wave_s construction::
```

**Fleet truth:** `wave_s_blueprint_roundtrip.json` → `preset_count: 1`, `roundtrip_ok: true`.

**Import button today:** copies bundle collection into **monospace buffer** — **not** a substitute for **BQ-128-APPLY-001** picker.

---

## BQ-128-APPLY-001 — preset picker → ghost (**DONE**)

**Unblocked by:** **UX-E02** sign-off (this plan).

### Goal

One-click **Apply** on a listed preset from `WaveSImportedBlueprints` → **ghost placement only** (origin, footprint, archetype, rotation/mirror). **Commit** still via Enter / existing confirm funnel.

### Coder contract

| Rule | Requirement |
|:---|:---|
| Authority | Add `ConstructionQueueIntent::ApplyPreset { index }` (or equivalent) — panel **emits intent only** |
| Ghost | Reuse `BuildPlacementPreview` / existing placement validation — **no** `CommitConstructionSiteEvent` from Apply |
| Data | Read `BlueprintPresetEntryR8` fields — map `archetype_tag` → `SiteArchetype` / catalog id per existing queue row shape |
| UI | Row under Import: combo or list of `label` + **Apply** per entry |
| Max files | **3** — `pending_construction_panel.rs`, `construction_queue_intent.rs`, one consumer in `construction/` |

### Copy-paste — BQ-128-APPLY-001

```
Lane: BQ-128-APPLY-001 — preset picker → apply-to-ghost
Read: docs/archive/2026-06-src-dev/plans/bq128_editor_path_plan_v1.md § BQ-128-APPLY-001
      docs/archive/2026-06-src-dev/plans/bq128_editor_path_design_note_v1.md
      src/dev/construction_invariants.md
Prereq: UX-E02-BQ128-001 SIGNED; wave_s blueprint roundtrip green
First: ConstructionQueueIntent::ApplyPreset + panel picker UI
Do NOT: instant-build; commit outside execute funnel; logic outside src/construction/
Verify: cargo test -p proc_A_dine01 --lib wave_s construction::
Witness: manual — Apply sets ghost; Enter still required to commit
```

### Acceptance

| # | Criterion |
|:---:|:---|
| A1 | Picker lists imported preset labels when `WaveSImportedBlueprints` non-empty |
| A2 | Apply updates ghost at preset origin/footprint without queue commit |
| A3 | Enter / confirm path unchanged |
| A4 | `wave_s` + `construction` lib tests green |

---

## BQ-128-APPLY-002 — merge vs replace (**DONE**)

| UX | Append imported presets to queue vs replace with confirm |
|:---|:---|
| **When** | After APPLY-001 or bundled in same PR if ≤3 files exceeded |

---

## Authority boundaries (all tracks)

| Forbidden | Reason |
|:---|:---|
| Preset click → instant site spawn | CSTR-2 / construction invariants |
| Shell writing gameplay state directly | Intent → queue → execute only |
| BQ-128 in WorldGen / map editor | UX-E02 session matrix |
| Second blueprint extract parallel to construction | Wave S data path only |

---

## Witness bundle

| File | Use |
|:---|:---|
| `wave_s_blueprint_roundtrip.json` | RON schema + `roundtrip_ok` |
| `wave_s_hydrate_live.json` | Bundle hydrate `blueprint_count` |
| `wave_s_shell_roundtrip.json` | Shell sibling (BQ-130) |

**Post-APPLY (optional):** extend `wave_s_hydrate_live.json` or construction live proof with `bq128_apply_green` when implemented.

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — PLAN-UX-BQ128-001 |
| Designer UX-E02 | 2026-05-25 | **SIGNED** — UX-E02-BQ128-001 |
| Coder APPLY-001 | 2026-05-27 | **DONE** |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | UX-E02 closed; BQ-128-APPLY-001 handoff |
