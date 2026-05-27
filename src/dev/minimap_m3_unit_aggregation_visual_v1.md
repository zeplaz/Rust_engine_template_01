# DESIGN-M3-DEPTH-001 — minimap M3 unit aggregation depth `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-M3-DEPTH-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **DEFER** (depth UX spec; witness does not yet expose “depth” slice flags in current proof) |
| **Unblocks** | `UI-P3-M3-DEPTH-001` (coder B3/B4) |
| **Witness (base)** | `debug_runs/minimap_compositor_live.json` → `/ui_p3_m3_units_001_green` and `/ui_p3_m3_green` |
| **Do not break** | `/ui_p3_m3_units_001_green`, `/ui_p3_m3_replay_001_green` |

---
## Purpose
Extend M3 unit reader UX from **aggregated strategic markers** toward **depth-aware readout**.

Rules:
- Do NOT reopen/modify the existing M3 witness rows for units (`DESIGN-M3-UNITS-001`).
- This spec defines how the **same aggregation channel** reads by zoom band (cluster vs individual markers).

---
## Depth-aware unit reader UX (cluster vs individual by zoom)
Zoom bands (conceptual):
| Zoom band | Minimap markers | Reader UX |
|:---|:---|:---|
| **High (strategic operational)** | Clusters | “Where is mass?” (centroids; sparse) |
| **Lower (closer operational)** | Cluster + selective individuals | “Where is detail?” (only when density is manageable) |
| **Tactical (if enabled by coder)** | Individuals only | “Where are units” (no aggregation) |

---
## Cursor/overlay policy
| Item | Spec |
|:---|:---|
| Cluster rendering | Must use the existing M3 unit aggregation compositor channel |
| Individual markers | Must not add new extract passes; only change the presentation threshold |

---
## Acceptance checklist (designer)
1. At strategic zoom, unit markers appear as clusters (not per-entity sprites).
2. As zoom approaches operational detail, cluster readability remains (no marker overlap explosion).
3. At the deepest zoom band (if enabled), individuals may appear without changing the M3 extraction authority.

