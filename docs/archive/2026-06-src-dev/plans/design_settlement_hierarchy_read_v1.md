# DESIGN-SETTLEMENT-HIERARCHY-READ-001 — Settlement hierarchy player read `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-SETTLEMENT-HIERARCHY-READ-001** |
| **Planner pair** | **PLAN-SETTLEMENT-HIERARCHY-005** (Town book schema — no duplicate UX) |
| **Growth UX** | [`design_organic_growth_ux_v1.md`](design_organic_growth_ux_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Unblocks** | **PROC-OG-4-001**, district picker wiring |
| **No Rust** | Labels + picker model |

---

## Purpose

Players navigate **Town → District → Block** without duplicating planner book schema. Organic growth **district picker** complements **zone tool** — different jobs.

---

## Hierarchy labels (map + inspector)

| Level | Map label | Inspector title | Example |
|:---|:---|:---|:---|
| **Town** | region name on strategic zoom | `Town: Portland Metro` | rollup metrics |
| **District** | boundary + name tag | `District: North Industrial` | growth UX panel |
| **Block** | optional micro-label at tactical | `Block 12` | proposal list grouping |
| **Building** | site name on select | `Concrete mixer` | construction stage read |

**Do not** show Nation/State in tactical v1 — planner schema only.

---

## Tools: zone vs district picker

| Tool | Player name | Action | Sim effect |
|:---|:---|:---|:---|
| **Zone tool** | **Zone paint** | paint residential/commercial/industrial | zoning mask only |
| **District picker** | **Select district** | click boundary or list | opens growth inspector |
| **Block** | *(derived)* | click proposal card | pan + highlight |

**Rule:** Zone paint **does not** open district inspector. District select **does not** change zone mask.

---

## Coordinate with PLAN-SETTLEMENT-HIERARCHY-005

| Planner owns | Designer owns |
|:---|:---|
| `TownBook`, `DistrictBook` RON schema | display strings, picker UX |
| Block id allocation | Block label in proposal cards |
| Witness keys | Map chrome hierarchy |

**No duplicate Town management window** — town rollup is read-only summary in district inspector header:

`Town: {name} · Pop {pop} · Jobs {jobs}`

---

## Organic growth alignment

| UX surface | Hierarchy level |
|:---|:---|
| Pressure overlay | **District** |
| Auto-build policy | **District** |
| Proposal card “Block 12” | **Block** |
| Approve → construction site | **Building** |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-02 |
