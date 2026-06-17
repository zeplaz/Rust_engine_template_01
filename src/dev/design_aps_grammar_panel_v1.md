# DES-APS-GRAMMAR-PANEL-UX-001 — Landscape Grammar tab wireframe `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-GRAMMAR-PANEL-UX-001** |
| **Blocks** | **APS-E2-GRAMMAR-PANEL-001** |
| **Territory** | `landscape_grammar_panel.py` — **not** `assembly_panel.py` / footprint |
| **Date** | 2026-06-16 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |

---

## Hard rule

**No footprint grid.** Grammar tab edits **topology_graph** (Network / Corridor / Ring / Patch / Cluster / Fringe) — graph-not-grid per Option D IA.

Building `FootprintCanvas` and `module_placements` UI **must not** appear in Landscape lane.

---

## Wireframe (≥1100px — 3-pane)

```text
┌─ Grammar — topology graph ship truth ──────────────────────────────────────────────┐
│ ENGINE_READ_PATH (landscape) — collapsible ▼                                       │
├──────────────────────────┬──────────────────────────┬──────────────────────────────┤
│ TOPOLOGY TREE            │ GRAPH PREVIEW            │ SELECTED NODE                │
│ (parent/child)           │ (schematic, not geo map) │                              │
│                          │                          │ Kind: Patch                  │
│ ▼ fire_recovery_v0       │   [net]──[patch]         │ ID: patch_burn_01          │
│   ├─ network net_fire_01 │        \                 │ Scale: M                   │
│   ├─ patch patch_burn_01 │         [corr]──[clus]   │ Parent: net_fire_01        │
│   ├─ corridor corr_…     │                          │                              │
│   └─ cluster clus_…      │  ○▒▒ glyph plan strip    │ Operator stack:            │
│                          │                          │  1. seed_patch             │
│ [Expand all] [Collapse]  │  click node ↔ tree sync  │  2. clear_suppress         │
│                          │                          │                              │
│                          │                          │ [Apply stack] (scaffold)   │
├──────────────────────────┴──────────────────────────┴──────────────────────────────┤
│ Preset: fire_recovery_v0 · 4 nodes · land_dna: upland / fire_return                │
│ [Generate grammar] [Save preset] [Validate schema]    ○ Grammar pending            │
└────────────────────────────────────────────────────────────────────────────────────┘
```

## Wireframe @ 960×600 (2-pane — [`design_aps_assembly_density_v1.md`](design_aps_assembly_density_v1.md))

```text
┌─ Tree + preview (stacked 55%) ─┐┌─ Node inspector (45%) ─┐
│ Tree (top half)               ││ Selected node fields     │
│ Graph preview (bottom half)   ││ Operator stack           │
└───────────────────────────────┘└──────────────────────────┘
```

---

## Pane responsibilities

| Pane | Authority | Widgets |
|:---|:---|:---|
| **Topology tree** | `topology_graph[]` hierarchy | `ttk.Treeview` `show=tree`; parent_id links |
| **Graph preview** | Read-only schematic | `tk.Canvas` — nodes as labeled boxes + edges; **not** lat/long |
| **Selected node** | One graph node | Kind, id, scale_band, anchor_ref, operator_stack list |
| **Glyph plan** | `metadata.glyph_planning` | Read-only strip e.g. `○▒▒` with word legend |

---

## Tree row format

```text
{kind_word} · {node_id}     e.g.  patch · patch_burn_01
```

Kind words (plain): network · corridor · ring · patch · cluster · fringe.

---

## Interaction model

| Action | Behavior |
|:---|:---|
| Select tree row | Highlight canvas node; fill inspector |
| Click canvas node | Select tree row; fill inspector |
| `Generate grammar` | MCP iterate operators on preset (job strip `⟳`) |
| `Save preset` | Write `landscape_grammar_v0` JSON; set `landscape_grammar_saved` |
| `Validate schema` | `validate-report landscape_grammar` → inline PASS/FAIL |
| No preset selected | Tree empty; banner `Select a preset on Presets tab` |

### Keyboard

| Key | Action |
|:---|:---|
| `↑`/`↓` | Move tree selection |
| `Enter` | Focus inspector |
| `Ctrl+S` | Save preset (when dirty) |

---

## Status line (inline — not modal)

| State | Copy |
|:---|:---|
| No preset | `○ Grammar pending — no preset` |
| Loaded | `◐ Grammar loaded · N topology nodes` |
| Saved | `◐ Grammar saved (QC not run)` |
| Valid | `✓ Grammar valid` |
| Fail | `✗ Grammar schema FAIL` |

---

## Explicitly forbidden

- Footprint W×D entry, placement list, material library, slot previews
- Reuse `AssemblyPanel` or `FootprintCanvas` in landscape notebook
- Building grammar archetype/district combos (those stay on Buildings Assembly)

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-16 |

```text
DES-APS-GRAMMAR-PANEL-UX-001 Q✓ — unblocks APS-E2-GRAMMAR-PANEL-001
```
