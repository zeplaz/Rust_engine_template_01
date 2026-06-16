# DESIGN-PROC-ART-ACCEPTANCE-001 — Procedural module art acceptance `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-PROC-ART-ACCEPTANCE-001** |
| **Baseline** | [`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md) (**PASS**) |
| **Version** | `1.0.0` |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **Status** | **HOLD** — full 50-module sign-off deferred |
| **Active pilot** | [`mcp_fleet_production_pilot_rowhouse_v1.md`](mcp_fleet_production_pilot_rowhouse_v1.md) — rowhouse production G4 only |
| **Unblocks** | PG-2 **textured** LOD fleet-wide (not this pilot) |
| **No Rust** | Acceptance checklist only |

---

## Purpose

Greybox (**PG-1/PG-2**) uses flat modules per kit PASS. This doc gates **textured** modules against the 50-ID manifest and 7 style packs.

---

## Acceptance per category (10 each)

| Category | Check |
|:---|:---|
| Walls | 10 meshes; 1u/2u snaps; style tags match RON |
| Windows | 10 variants; fit wall bays |
| Doors | 10 variants; ground-floor scale |
| Roofs | 10 variants; align footprint **R** grid |
| Corner/prop | 10 variants; no gameplay collision volume |

---

## Style pack completeness

| Pack | Min modules populated | Status |
|:---|:---:|:---|
| Victorian | 80% of referenced IDs | ☐ |
| Modern | 80% | ☐ |
| Industrial Western | 80% | ☐ |
| Industrial Soviet | 60% | ☐ |
| Military | 60% | ☐ |
| Rural | 60% | ☐ |
| Colonial | 60% | ☐ |

---

## Greybox vs texture

| LOD | Art | Blocks |
|:---|:---|:---|
| LOD0 greybox | flat color | **PG-2** coder milestone |
| LOD1 textured | final materials | **this** acceptance |

---

## Sign-off (when art ready)

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | ☐ **PASS** | — |
| Artist | ☐ manifest match | — |
