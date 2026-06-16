# APS-MAT-IA-001 — Materials tab information architecture `v1`

| Field | Value |
|:---|:---|
| **Program** | APS-MAT-IA-001 |
| **Owner** | `@designer` (IA) · `@coder-mcp` (studio_tree + navigation) |
| **Related** | APS-MAT-002/003 · ARCH-MAT-001 · APS-MAT-AUTH-UI-001 |
| **Date** | 2026-06-03 |
| **Verdict** | **APPROVED** |

---

## Mission

Materials tab is **browse + preview + registry edit** — not ship authority. Artists must understand:

1. How to find a profile among **300+** entries
2. How **Use in Assembly** connects to assign workflow
3. Why **Apply** only exists on Assembly (with cell selected)

---

## Authority model (on-tab, always visible)

```text
Materials tab     → browse · generate maps · edit registry
Assembly tab      → assign material_profile onto placement (SHIP)
Blender           → never assigns ship materials in APS workflow
```

Intro line (lock copy):

```text
Material Studio — browse, generate, and edit profiles. Drop authored PNGs into each
profile folder, then Reload preview. Assign on the Assembly tab.
```

Metadata flow panel (`context=materials`) — collapsed by default after first visit polish.

---

## Layout wireframe

```text
┌─ Materials tab ─────────────────────────────────────────────────────────────┐
│ [intro line]                                                                │
│ [Metadata → engine — materials]                                             │
│ ┌─ Toolbar ───────────────────────────────────────────────────────────────┐ │
│ │ Add profile │ Gen selected │ Gen all missing │ Open folder │ Registry   │ │
│ │                                    [Use in Assembly]  ← primary CTA     │ │
│ └─────────────────────────────────────────────────────────────────────────┘ │
│ Search [________]   Category [all ▼]  (combobox legacy; tree is primary)    │
│ ┌─ Paned (horizontal) ────────────────────────────────────────────────────┐ │
│ │ ┌ Categories ────┐ ┌ Profiles ──────────────┐ ┌ Preview & maps ──────┐ │ │
│ │ │ ▼ All (312)    │ │ [thumb] steel_panel_01 │ │ [large preview]      │ │ │
│ │ │ ▼ Industrial   │ │     Ready              │ │ profile meta         │ │ │
│ │ │   · Steel (48) │ │ [thumb] steel_door_…   │ │ Ready · Partial maps │ │ │
│ │ │   · Corrugated │ │ … scroll …             │ │ [Reload preview]     │ │ │
│ │ │ ▼ Residential  │ │                        │ │ (no Apply here)      │ │ │
│ │ │   · Brick      │ │                        │ │                      │ │ │
│ │ └────────────────┘ └────────────────────────┘ └──────────────────────┘ │ │
│ └─────────────────────────────────────────────────────────────────────────┘ │
│ ┌─ Preview modes (below or right per pane split) ────────────────────────┐ │
│ │ Sphere │ Wall strip │ Building section │ [Refresh]                       │ │
│ └─────────────────────────────────────────────────────────────────────────┘ │
│ Status: 48 shown · cache 72px · 312 total                                 │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Pane weights:** Categories 1 · Profiles 2 · Preview strip 2 (matches current `studio_tree` panedwindow).

---

## Category tree (APS-MAT-003)

| Level | Example | Tree label |
|:---|:---|:---|
| Root | — | `All ({total})` |
| Parent | `industrial` | `Industrial ({count})` |
| Leaf | `industrial/steel` | `Steel ({count})` |
| Leaf | `industrial/corrugated` | `Corrugated ({count})` |

**Rules:**

- Parent select → show all profiles whose category starts with `parent/` or equals parent
- Leaf select → exact category match
- Search box filters **within** tree selection
- Flat categories without `/` → under parent node matching first segment

**Do not** show raw slash paths in profile rows — display `profile_id` + status text.

---

## Profile row (list, not card grid)

Each row in Profiles pane:

```text
[72px thumb]  steel_panel_01
              Ready · industrial/steel
```

| Element | Spec |
|:---|:---|
| Status | Text **Ready** / **Partial** / **Missing** beside optional ●◐○ (APS-UX-POLISH: text required) |
| Selection | Single select; drives preview strip + preview modes |
| Double-click | Open texture folder (keep) |

---

## Use in Assembly — primary CTA

| Step | Behavior |
|:---|:---|
| 1 | Artist selects profile in Materials tab |
| 2 | Clicks **Use in Assembly** |
| 3 | App switches to **Assembly** tab |
| 4 | Material browser highlights same `profile_id` |
| 5 | On-screen callout (existing next-step label): **Select a footprint cell, then Apply** |

**If no snapshot loaded:** Still switch tabs; Assembly shows “Generate or load snapshot first.”

**Tooltip (`mat_use_in_assembly`):** Switch to Assembly tab with this profile highlighted — select a footprint cell, then Apply.

**Do not** auto-apply without selected cell — preserves ARCH-MAT-001 explicit assign.

---

## Assign vs browse modes

| Surface | Mode | Apply button |
|:---|:---|:---:|
| Materials tab | `studio` / browse | **Hidden** — Use in Assembly only |
| Assembly · material browser | `assign` | **Apply to selected slot** visible |

Cross-link:

- Assembly → “Open in Materials” (if present) switches tab + highlights profile
- Materials → **Use in Assembly** (primary)

---

## Preview modes panel

| Mode | When to use |
|:---|:---|
| Sphere | Quick albedo/normal read |
| Wall strip | Facade-scale |
| Building section | Before assigning to wall/roof slots |

Refresh on profile select; share same `profile_id` as library selection.

---

## Empty / error states

| State | Copy |
|:---|:---|
| No profile selected | Preview: “(select profile)” |
| Catalog load fail | Status bar: “Could not load material registry” |
| Generate fail | Toast/log with profile id |

---

## Out of scope (this IA)

- Nested tree drag-reorder
- Inline registry JSON editor in panel (Open registry JSON stays advanced)
- Material assignment from Variants tab (variant material override is separate layer)

---

## Acceptance (@coder-mcp)

| # | Criterion |
|:---:|:---|
| 1 | Tree shows parent → leaf hierarchy for slash categories |
| 2 | **Use in Assembly** visible in Materials toolbar; switches tab + highlights profile |
| 3 | No **Apply** button on Materials tab studio mode |
| 4 | 50+ profiles browsable without typing search (tree + scroll) |
| 5 | Status text on every profile row (not glyph-only) |

---

## Sign-off

```text
APS-MAT-IA-001 complete
Primary CTA: Use in Assembly → Assembly tab → cell → Apply
Tree: Industrial→Steel / Residential→Brick pattern
Sign-off: APPROVED
```
