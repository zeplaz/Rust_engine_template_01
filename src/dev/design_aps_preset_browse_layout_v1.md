# DES-APS-PRESET-BROWSE-UX-001 — Landscape Presets browse layout `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-PRESET-BROWSE-UX-001** |
| **Program** | APS-E2 |
| **Date** | 2026-06-16 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |

---

## Layout (Landscape · Presets tab)

```text
┌─ Presets ──────────────────────────────────────────────────────────────┐
│ [Filter: district ▼] [Search preset_id________] [Refresh]              │
├──────────────────┬─────────────────────────────────────────────────────┤
│ PRESET LIST      │ SELECTED PRESET                                      │
│ (10 ship rows)   │                                                      │
│                  │ Title: Fire recovery corridor                          │
│ fire_recovery_v0 │ chart_id: NESTED-SUCCESSION-Ψ18                      │
│ old_growth_core… │                                                      │
│ …                │ land_dna summary (plain):                            │
│                  │   Habitat upland · Soil thin ridge · Disturb fire…   │
│                  │                                                      │
│                  │ TOPOLOGY SUMMARY (counts)                            │
│                  │   Network 1 · Patch 2 · Corridor 1 · Cluster 1 …     │
│                  │                                                      │
│                  │ [Validate preset] [Clone…] [Open JSON folder]        │
│                  │ ○ Validation pending / ✓ PASS / ✗ FAIL (inline)      │
├──────────────────┴─────────────────────────────────────────────────────┤
│ pressure_field sliders (λ rows) — plain labels, not lambda_* alone     │
└────────────────────────────────────────────────────────────────────────┘
```

## List row format

`{preset_id}` · `{display_title}` · `{topology_kind_count} kinds`

Display titles: `@designer-mcp` `_display_strings_v1.json` — designer approves pattern here.

## Topology summary (right pane)

| Field | Source |
|:---|:---|
| Kind counts | `topology_graph[].topology_kind` rollup |
| Scale bands | L/M/S chips with words |
| Anchor refs | monospace secondary line |

## Min width @960px

List pane min **220px**; detail pane gets remainder; pressure sliders stack vertically.

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-16 |
