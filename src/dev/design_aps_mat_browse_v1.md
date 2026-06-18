# APS Materials tab browse IA `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-MAT-BROWSE-001** |
| **Program** | APS materials · Track B3 |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Authority** | [`plan_material_studio_phase_v1.md`](plan_material_studio_phase_v1.md) · [`aps_sweep_layout_20260616_v1.md`](aps_sweep_layout_20260616_v1.md) MAT-1–3 |
| **A11y** | [`design_aps_color_a11y_audit_v1.md`](design_aps_color_a11y_audit_v1.md) |
| **Handoff** | APS-MAT-002 scale polish · `material_library_widget.py` |
| **Verdict** | **PASS** |

```text
DES-APS-MAT-BROWSE-001 Q✓
300+ profiles — tree + search + recents + unsorted bucket
```

---

## 0. Problem

Combobox + card grid **does not scale** past ~50 profiles. Artists need **library navigation** without losing assignment flow (Materials → Assembly).

---

## 1. Layout (flatten MAT-1)

**One horizontal paned** — no paned-in-paned starve.

```text
┌ Materials ─────────────────────────────────────────────────────────┐
│ [Search…………………]  Category [All ▼]     [⋯] [Use in Assembly →]   │
├──────────────┬─────────────────────┬───────────────────────────────┤
│ Categories   │ Profiles            │ Preview + status              │
│ (tree)       │ (list)              │ (sphere / wall / section)     │
│ min 160      │ min 220             │ min 280                       │
│              │                     │                               │
│ ▼ Industrial │ steel_panel_01  ✓   │  ┌─────────────────┐        │
│   Steel      │ steel_rust_02   ◐   │  │                 │        │
│   Concrete   │ …                 │  └─────────────────┘        │
│ ▼ Recent     │                   │  ✓ ready — steel_panel_01     │
│ ▼ Unsorted   │                   │                               │
└──────────────┴─────────────────────┴───────────────────────────────┘
```

**Total min width:** 160 + 220 + 280 + sashes = **~670** — fits APS MIN after flatten.

---

## 2. Category tree

**Source:** `material_category_tree.tree_roots()` — designer-facing labels via `category_label()`.

| Node | Behavior |
|:---|:---|
| `All` | Root default — entire library |
| Category leaves | Filter profile list |
| **Recent** | Last **12** opened/edited profile ids (session + persisted) |
| **Unsorted** | Profiles where `infer_category(id) == "unsorted"` |

**Tree rules:**

- Single select · no multi-check
- Expand/collapse persists per session
- Count badge: `Steel (42)` on leaves when ≥1 profile

---

## 3. Search

| Rule | Spec |
|:---|:---|
| Scope | `profile_id` · `label` · `tags[]` |
| Match | case-insensitive substring |
| Debounce | **200ms** |
| Clear | `Esc` in search field clears query |
| Empty result | `○ No profiles match "{query}"` |

**Search + tree:** search **overrides** tree filter; clearing search restores tree selection.

---

## 4. Profile list (not card grid at scale)

| Column | Content |
|:---|:---|
| Id | mono `profile_id` |
| Status | `format_material_texture_status` — `✓ ready` / `◐ partial` / `○ missing` |
| Category | muted caption when viewing All |

**Selection:** single row · Enter opens preview · double-click → **Use in Assembly** if slot context exists.

**Sort:** within category — `profile_id` alpha; Recent bucket — LRU order.

---

## 5. Unsorted bucket

| Rule | Spec |
|:---|:---|
| Definition | Profiles failing category inference or missing `category` field |
| Label | **Unsorted** (not "unknown") |
| Banner | `◐ {n} profiles need category — edit metadata or drag to category` |
| Action | Select row → **Edit metadata** focuses category field |

**Goal:** drive `unsorted` count → 0 over time; never hide profiles.

---

## 6. Recents

| Rule | Spec |
|:---|:---|
| Push | on select · on save · on generate complete |
| Cap | 12 ids |
| Dedup | move-to-front |
| Persist | `~/.rust_engine/aps_mat_recents.json` (coder path) |

---

## 7. Toolbar overflow (MAT-3)

**Always visible:** Add · Search · Category combo · Use in Assembly

**⋯ menu:** Generate · Open folder · Reload registry · Import PNGs

---

## 8. Preview strip (bottom or right third)

Unchanged modes: sphere · wall strip · building section — per [`material_preview_modes.py`](../../tools/mcp/art_pipeline_suite/material_preview_modes.py).

Preview **does not** replace list selection — list remains primary navigation.

---

## 9. Acceptance

| # | Check |
|:---:|:---|
| M1 | 300-profile fixture loads list < 500ms (coder perf) |
| M2 | Tree + search + Recent + Unsorted all reachable in ≤2 clicks |
| M3 | No horizontal tab scroll at MIN width after flatten |
| M4 | Status uses glyph+word — no `●` / `GEN` / `ERR` |
| M5 | Witness `debug_runs/aps_mat_browse_ia_live.json` |

---

## 10. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |
