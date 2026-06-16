# DESIGN-PARAM-STAGING-POLISH-002 — Staged placements UX polish `v2`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-PARAM-STAGING-POLISH-002** |
| **Coder lane** | **B-C4** · **CONSTRUCTION-PARAM-CODER-004** (P3-A) |
| **Baseline** | [`construction_parametric_staged_panel_v1.md`](construction_parametric_staged_panel_v1.md) (v1 columns + footer) |
| **Tray host** | [`construction_parametric_tray_mock_v1.md`](construction_parametric_tray_mock_v1.md) |
| **Product spec** | [`construction_parametric_placement_spec_v1.md`](construction_parametric_placement_spec_v1.md) § Stage placements ON |
| **Version** | `2.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Unblocks** | `staging_toggle_wired`, `build_approved_drains_staged` witness flags |
| **No Rust** | Interaction polish + copy + states only |

---

## Purpose

v1 defines **columns, footer labels, and empty/invalid mocks**. v2 adds **interaction polish** so B-C4 can ship a staging panel that feels intentional in PLAY-01 — not a debug list.

**Non-goals:** new commit funnel, save-game persistence of staged rows, drag-reorder priority, map multi-select.

---

## Toggle + header chrome

| Element | Spec |
|:---|:---|
| **Toggle label** | `Stage placements` (unchanged) |
| **Count badge** | When `staged_count > 0`, show `(n)` on panel title: `Staged placements (3)` |
| **Session hint** | First time toggle ON per session: one-line toast `Staged ghosts are snapshots — edit active ghost, then LMB to add.` (dismiss, do not repeat) |
| **Toggle OFF with rows** | Panel stays visible (v1 rule); title appends muted `— turn on to add more` |

---

## List interaction (polish)

### Scroll + density

| Property | Value |
|:---|:---|
| Max visible rows before scroll | **6** |
| Row height | **22px** (28px when error sub-line visible) |
| Scrollbar | Thin tray scrollbar; wheel over panel scrolls list, not map |
| Virtualization | Not required v1 — cap staged at **32** rows; at cap LMB add shows toast `Staged list full (32). Build or remove rows.` |

### Row hover + focus

| State | Treatment |
|:---|:---|
| Default | `row_bg` transparent |
| Hover | `row_bg_hover` `#FFFFFF` @ 6% |
| Focused (keyboard) | 1px `accent` left rail |
| Approved + valid | Checkbox filled `accent` |
| Approved + invalid | Checkbox filled but row `Validity == Bad` — commit skips with inline note |

### Keyboard (tray focused)

| Key | Action |
|:---|:---|
| **↑ / ↓** | Move row focus |
| **Space** | Toggle Approved on focused row |
| **Delete** | Remove focused row (no commit) |
| **Enter** | **Build approved** (same as footer primary) |
| **Ctrl+A** | Approve all **valid** rows only (invalid stay unchecked) |

Map camera keys unchanged when map viewport has focus.

### Bulk actions (footer extensions)

| Control | Label | Behavior |
|:---|:---|:---|
| Link (11px) | **Approve all valid** | Sets ☑ on every row with `allows_commit` |
| Link (11px) | **Unapprove all** | Clears all ☑; does not remove rows |

Keep primary footer buttons from v1 unchanged.

---

## Validity + error polish

| Validity | Badge | Row treatment |
|:---|:---|:---|
| **OK** | `OK` · `badge_ok` | Normal |
| **Warn** | `Warn` · `badge_warn` | Tooltip: first warning string |
| **Bad** | `Bad` · `badge_err` | Error sub-line (v1); map red weights |

**Overlap sub-line format (exact):**

```text
Σw > 1.0 at ({x}, {z})
```

**Stale snapshot:** If catalog row removed since add, badge `Stale` · row dimmed; **Build** skips; remove encouraged.

---

## Commit feedback (player read)

| Event | Feedback |
|:---|:---|
| **Build approved** success | Toast `Placed {n} building(s).` · clear approved rows from list |
| **Build approved** partial | Toast `Placed {n} of {m} — {k} skipped (invalid).` |
| **Build all valid** | Toast `Placed {n} building(s).` · list empty |
| **Zero commits** | Toast `Nothing to place — approve valid rows first.` |
| Active ghost still valid after batch | Active ghost **retained** (not cleared) unless player RMB/Esc |

No modal confirm v1 — batch is explicit via footer.

---

## Staging ON — LMB add polish

| Rule | Spec |
|:---|:---|
| Valid ghost + LMB | Push snapshot; brief flash on map ghost bound (100ms `accent` @ 20%) |
| Invalid ghost + LMB | Toast `Cannot stage — fix placement first.` |
| Duplicate origin+scale+rot+mirror+catalog | Toast `Already staged — remove row or move ghost.` |
| After add | Active ghost **unchanged**; new row **unchecked** |

---

## Layout integration (R4 legend)

| Rule | Spec |
|:---|:---|
| Panel expansion | Staging block grows tray body; **R4 48+52 legend stays pinned to tray bottom** |
| Min tray height (staging ON, 0 rows) | **200px** total body (toggle + empty list + footer + legend slot) |
| Corridor legend | Visible only per [`construction_r4_tray_legend_v1.md`](construction_r4_tray_legend_v1.md) — staging panel must not cover swatches |

---

## ASCII — keyboard focus row

```text
┌ Staged placements (2) ─────────────────────────────────────────┐
│ ▌☑  Solar Array      1.24×   90°   OK    ✕    ← focus rail       │
│  ☐  Fuel Depot       1.80×    0°   Bad   ✕                       │
│      Σw > 1.0 at (12, 44)                                       │
├──────────────────────────────────────────────────────────────┤
│ [ Build approved ]  [ Build all valid ]  [ Clear unapproved ]  │
│  Approve all valid · Unapprove all                               │
└──────────────────────────────────────────────────────────────┘
```

---

## Witness alignment (construction_parametric_placement_001)

| Flag | Designer criterion |
|:---|:---|
| `staging_toggle_wired` | Toggle + panel visibility rules |
| `build_approved_drains_staged` | Footer + Enter drain only approved valid rows; feedback toast optional in sim |

---

## Acceptance (designer)

1. Count badge, scroll cap 32, and keyboard table implemented or explicitly deferred in coder notes.
2. Footer labels remain **Build approved** / **Build all valid** / **Clear unapproved** (v1).
3. Overlap error uses `Σw > 1.0 at (x, z)` format.
4. R4 legend remains visible at tray bottom when corridor rules apply.
5. Product acceptance §2 (stage ≥3, check 2, Build approved → 2 commits) satisfied without new columns.

---

## Coder mapping

| File | Owner |
|:---|:---|
| `src/construction/staged_ghost_panel.rs` | CODER-004 |
| `src/construction/pending_construction.rs` | snapshot + duplicate detect |
| `src/construction/build_interaction.rs` | Enter → Build approved when staging ON |

---

## Sign-off

| Role | Status | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-05-26 |
| `@coder` | **Unblocked** for B-C4 polish slice | — |
