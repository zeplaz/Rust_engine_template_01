# BQ-128 apply ghost — UX review `v1` (DESIGN-BQ128-APPLY-UX-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-BQ128-APPLY-UX-001** |
| **Coder lane** | **UX-E02-APPLY-POLISH-001** (Coder B **#9**) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Status** | **SIGNED** |
| **Verdict** | **PASS (qualified)** |
| **Prereq** | [`bq128_editor_path_design_note_v1.md`](bq128_editor_path_design_note_v1.md) (**UX-E02-BQ128-001**) |
| **Code** | [`pending_construction_panel.rs`](../construction/pending_construction_panel.rs) · [`blueprint_preset.rs`](../construction/blueprint_preset.rs) |
| **Witness** | [`debug_runs/wave_s_blueprint_roundtrip.json`](../debug_runs/wave_s_blueprint_roundtrip.json) (`roundtrip_ok: true`) |

**No Rust.** Review of landed **Apply ghost** affordance; coders may close **UX-E02-APPLY-POLISH-001** on witness + this record.

---

## Executive summary

| Area | Verdict |
|:---|:---:|
| **Authority** — ghost only, Enter to commit | **PASS** |
| **Discoverability** — helper + per-row Apply | **PASS** |
| **Copy** — commit path clear | **PASS** |
| **Visual parity** — ghost tokens vs multiview spec | **PASS (qualified)** |
| **Merge vs replace import** | **DEFERRED** — **BQ-128-APPLY-002** |

---

## Current UX (as shipped)

| Element | Observed | Spec match |
|:---|:---|:---:|
| Section hint | `Wave S presets — Apply loads ghost only (Enter to commit)` | ☑ |
| Row label | `{label} @ ({x},{z}) {w}×{d}` | ☑ |
| Primary action | Button **Apply ghost** | ☑ |
| Intent | `ConstructionQueueIntent::ApplyImportedPreset { preset_index }` | ☑ |
| Ghost write | `apply_blueprint_preset_to_build_ghost` — origin, footprint, rotation, tool | ☑ |

**Session:** Simulation only — World Preview / WorldGen **out of scope** per BQ-128 design note.

---

## Readability checklist

| # | Criterion | Result |
|:---:|:---|:---:|
| 1 | Operator understands Apply does **not** queue or commit | **PASS** — weak helper text sufficient |
| 2 | Valid / invalid footprint colors unchanged after apply | **PASS** — uses [`ghost_visual.rs`](../construction/ghost_visual.rs) tokens |
| 3 | Apply switches build tool to preset archetype | **PASS** — `BuildTool::Building` + strip sync |
| 4 | No instant-build on click | **PASS** — CSTR-2 path |
| 5 | Import still separate from Apply | **PASS** — Import Wave S presets (N) above list |

---

## Polish recommendations (optional — not blocking)

| ID | Change | Priority |
|:---|:---|:---:|
| P2-A | After Apply, flash build-strip selection to matching archetype icon | Low |
| P2-B | Disabled **Apply ghost** when preset footprint invalid on terrain | Medium — coder |
| P2-C | Single-line toast on ops strip: `Blueprint loaded — Enter to place` | Low — pairs with IND-E03 ops vocabulary |
| P2-D | **BQ-128-APPLY-002** — Append vs Replace on import | Plan only |

---

## Witness evidence

| Artifact | Field | Value (2026-05-26) |
|:---|:---|:---|
| `wave_s_blueprint_roundtrip.json` | `roundtrip_ok` | `true` |
| | `preset_count` | `1` |
| Lib | `apply_blueprint_preset_sets_ghost_origin_and_footprint` | unit test in `blueprint_preset.rs` |

---

## Coder exit — UX-E02-APPLY-POLISH-001

```
Lane: UX-E02-APPLY-POLISH-001
Prereq: DESIGN-BQ128-APPLY-UX-001 SIGNED (this doc)
Read: bq128_apply_ghost_ux_review_v1.md · pending_construction_panel.rs
Exit: roundtrip_ok + manual sim Apply → ghost moves · Enter commits
Verify: cargo test -p proc_A_dine01 --lib wave_s
```

---

## Sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-26 | **PASS (qualified)** |
| Coder B | — | May close on witness + optional P2-A…D |

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **DESIGN-BQ128-APPLY-UX-001** |
