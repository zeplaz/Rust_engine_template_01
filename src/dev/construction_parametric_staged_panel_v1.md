# CONSTRUCTION-PARAM-DESIGN-001 — Staged placements panel `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **CONSTRUCTION-PARAM-DESIGN-001** (staged list slice) |
| **Plan** | [`plan_construction_parametric_placement_v1.md`](plan_construction_parametric_placement_v1.md) § UI split · Phase 3 |
| **Tray host** | [`construction_parametric_tray_mock_v1.md`](construction_parametric_tray_mock_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Implementation owner** | `src/construction/staged_ghost_panel.rs` (**CODER-004**) |
| **No Rust** | Column layout + states + button copy only |

---

## Visibility

| Condition | Panel |
|:---|:---|
| `Stage placements` **OFF** and `staged_count == 0` | Hidden |
| `Stage placements` **ON** | Visible (may be empty) |
| `staged_count > 0` | Visible even if toggle turned OFF (reminder until cleared) |

**Min height when visible:** **120px** list + **36px** footer = **156px** inside tray body (extends P2 body beyond default 96px when staging active).

---

## Column layout

| Col | Width | Control | Notes |
|:---:|:---:|:---|:---|
| **Approved** | **28px** | Checkbox ☑ | Row included in **Build approved** |
| **Label** | **flex** (min 88px) | Text | `{catalog_short}` or `Site #{n}` |
| **Scale** | **52px** | `1.24×` | 2 decimal max |
| **Rot** | **36px** | `90°` | `rotation_quarter_turns * 90` |
| **Validity** | **56px** | Badge | `OK` / `Warn` / `Bad` |
| **Remove** | **24px** | `✕` | Removes row from staged book (no commit) |

**Header row** (optional, 10px muted): `Approved · Label · Scale · Rot · Validity`

---

## Footer actions

| Button | Label | Action |
|:---|:---|:---|
| Primary | **Build approved** | Commit all rows with ☑ Approved and `allows_commit` |
| Secondary | **Build all valid** | Approve + commit every row with `allows_commit` (warnings OK) |
| Tertiary | **Clear unapproved** | Remove rows where Approved unchecked OR Invalid |

**Button order (left → right):** Build approved · Build all valid · Clear unapproved

**Disabled rules:**

| Button | Disabled when |
|:---|:---|
| Build approved | No approved rows OR none valid |
| Build all valid | No valid rows |
| Clear unapproved | No unapproved/invalid rows |

---

## State mocks (ASCII)

### Empty (staging ON, 0 rows)

```text
┌ Staged placements (0) ─────────────────────────────────────────┐
│  No staged ghosts — adjust active ghost, then LMB on map.      │
├──────────────────────────────────────────────────────────────┤
│ [ Build approved ]  [ Build all valid ]  [ Clear unapproved ]  │
│      (disabled)          (disabled)            (disabled)       │
└──────────────────────────────────────────────────────────────┘
```

### One row (valid, approved)

```text
┌ Staged placements (1) ─────────────────────────────────────────┐
│ ☑  Solar Array      1.24×   90°   OK    ✕                       │
├──────────────────────────────────────────────────────────────┤
│ [ Build approved ]  [ Build all valid ]  [ Clear unapproved ]  │
│      (enabled)          (enabled)            (disabled)         │
└──────────────────────────────────────────────────────────────┘
```

### Invalid row (overlap)

```text
┌ Staged placements (2) ─────────────────────────────────────────┐
│ ☑  Solar Array      1.24×   90°   OK    ✕                       │
│ ☐  Solar Array      1.80×    0°   Bad   ✕   ← overlap          │
├──────────────────────────────────────────────────────────────┤
│ [ Build approved ]  [ Build all valid ]  [ Clear unapproved ]  │
│      (enabled*)         (enabled*)           (enabled)          │
│  * commits only valid approved row(s); invalid skipped          │
└──────────────────────────────────────────────────────────────┘
```

### Overlap error detail

When row `Validity == Bad`, show sub-line under row (11px err):

```text
  Σ tile weight > 1.0 at (12, 44)
```

Map draw uses ghost visual spec (red weights) — panel does not duplicate map fill.

---

## Row snapshot semantics

| Rule | Spec |
|:---|:---|
| LMB add | Copies active ghost: origin, scale, rotation, mirror, catalog_id, precomputed preview validity |
| Edit active ghost | Does **not** mutate existing staged rows |
| Re-add | Player must LMB again to refresh snapshot |
| Approved default | **unchecked** on add |
| Label | `catalog.display_name` truncated to column; tooltip full name + origin tile |

---

## Enter key equivalence

| Context | Enter |
|:---|:---|
| Staging OFF, valid ghost | Commit single ghost |
| Staging ON, `staged_count > 0` | Same as **Build approved** |
| Staging ON, empty staged | Commit active ghost if valid (same as OFF) |

---

## Interaction with MV-001 / R4

- Panel is **egui/Bevy tray only** — no map mutation.
- Staged map ghosts use **25% desaturated** palette ([`construction_parametric_ghost_visual_v1.md`](construction_parametric_ghost_visual_v1.md)).
- Corridor legend footer unchanged when panel expanded.

---

## Acceptance (designer)

1. Columns match table: Approved, Label, Scale, Rot, Validity, remove.
2. Footer labels exact: **Build approved**, **Build all valid**, **Clear unapproved**.
3. Empty / 1-row / invalid / overlap states documented above are implementable without new columns.
4. Build approved commits only checked valid rows (≥3 staged, 2 checked → exactly 2 commits per product spec § Acceptance).

---

## Coder mapping

| Lane | Deliverable |
|:---|:---|
| **CONSTRUCTION-PARAM-CODER-004** | `staged_ghost_panel.rs` + footer buttons |
| **CONSTRUCTION-PARAM-CODER-003** | Drain approved → `CommitConstructionSiteEvent` |
| **CONSTRUCTION-PARAM-CODER-001** | Validity column from weighted overlap |
