# DESIGN-WSS-HYDRO-READ-001 — WSS hydrology feature read pass `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-WSS-HYDRO-READ-001** |
| **Baseline** | [`wss_hydrology_player_read_v1.md`](wss_hydrology_player_read_v1.md) (DESIGN-HYDRO-PLAYER-READ-001) |
| **Coder lane** | **FEAT-WSS-HYDRO-READ-001** |
| **Plan** | [`plan_wss_hydro_runtime_exec_001_v1.md`](plan_wss_hydro_runtime_exec_001_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-28 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Witness** | `debug_runs/wss_substrate_live.json` → `wss_hydro_runtime_001.green` |
| **Unblocks** | **FEAT-WSS-HYDRO-READ-001** |
| **No Rust** | Copy + HUD read contract |

---

## Purpose

Extend signed hydrology **band table** with **feature-lane** strings for slab-backed hydro read (hover, status strip, optional F3). Coder wires against this pass — does not reopen ocean/river/lake motion spec.

---

## Player read — one-line status (hover / strip)

| Surface read | Canonical string | When |
|:---|:---|:---|
| **Ocean** | `Open water` | `ocean_mask` dominant |
| **River** | `River — flowing` | `river_mask` + flow visible |
| **Lake** | `Standing water` | standing depth, no centerline streak |
| **Flood** | `Flood — spreading` | depth pulse active (not contamination) |
| **Dry near river** | `Dry riverbed` | `river_mask` + depth ≈ 0 |

**Forbidden:** `HydrologyState`, `slab`, `L1`, `WSS`, drift, dual-write.

---

## Tooltip detail (secondary, ≤ 2 lines)

| Read | Line 1 | Line 2 (optional) |
|:---|:---|:---|
| River | `Flow follows terrain.` | — |
| Lake | `Calm surface.` | — |
| Ocean | `Deep water — strategic view simplifies detail.` | — |
| Flood | `Water level rising.` | `Not a contamination plume.` |

---

## F3 diagnostics (dev / expanded diagnostics only)

Single muted row when WSS section open:

```text
WSS hydro: ocean={ocean_tile_count} rivers={river_segment_count} slab={hydrology_hydrated}
```

Map witness keys: `wss_hydro_runtime_001` block in `wss_substrate_live.json`.

---

## Minimap + tactical (unchanged from v1.0)

- Ribbon α ≤ **0.40** on minimap.
- **No** W2 particles on minimap.
- Tactical: W1/W2 vocabulary per baseline doc — **do not regress** `water_witness_rollup_green`.

---

## Regression guards

| Witness | Row |
|:---|:---|
| `stage5_full_app_live.json` | `water_w1_river_read_green`, `water_w2_foam_001_green`, `water_witness_rollup_green` |
| `wss_substrate_live.json` | `wss_hydro_runtime_001.green` |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-05-28 |

**Unblocks:** **FEAT-WSS-HYDRO-READ-001** (coder B).
