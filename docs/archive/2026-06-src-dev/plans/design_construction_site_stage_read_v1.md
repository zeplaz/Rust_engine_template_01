# DESIGN-CONSTRUCTION-STAGE-READ-001 — Site construction stage player read `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-CONSTRUCTION-STAGE-READ-001** |
| **Parent** | [`construction_product_roadmap_phases_2_10_v1.md`](construction_product_roadmap_phases_2_10_v1.md) Phase 2 |
| **Plan** | [`plan_construction_stage_pipeline_exec_002_v1.md`](plan_construction_stage_pipeline_exec_002_v1.md) |
| **Authority** | [`SiteConstructionPhase`](../strategic/site/resources.rs) |
| **Version** | `1.0.0` |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Unblocks** | Player comprehension of **CON-P2** pipeline |
| **No Rust** | HUD / minimap / F3 copy |

---

## Purpose

Players see **staged progress** from commit through **Operational** — not instant buildings. Copy maps 1:1 to `SiteConstructionPhase` enum (display names may differ).

**Commit sets `Planned`** — never `Operational` on commit ([`construction_invariants.md`](construction_invariants.md)).

---

## Phase → player strings

| `SiteConstructionPhase` | HUD / tray label | Substep (Clearing only) | Progress bar |
|:---|:---|:---|:---:|
| `Planned` | **Planned** | — | empty |
| `Surveying` | **Surveying site** | — | yes |
| `Clearing` | **Clearing land** | see below | yes |
| `Foundation` | **Foundation** | — | yes |
| `UnderConstruction` | **Building** | — | yes |
| `Provisioning` | **Provisioning** | utilities hookup | yes |
| `Operational` | **Operational** | — | full |
| `Damaged` | **Damaged** | — | warning stripe |
| `Offline` | **Offline** | — | grey |
| `Abandoned` | **Abandoned** | — | muted |

### Clearing substeps (display only)

| Substep | Label |
|:---|:---|
| `Trees` | **Clearing trees** |
| `Stumps` | **Removing stumps** |
| `Grade` | **Grading** |

---

## Build rail / site tooltip (selected site)

```text
{site_name} — {phase_label} ({progress_pct}%)
```

Example: `Concrete mixer — Foundation (42%)`

When `phase == Planned` after player commit: `Queued — construction will start next tick`

---

## Minimap / map icons

| State | Icon | Color |
|:---|:---|:---|
| Planned | hollow square | `#a0a0a0` |
| Active construction | crane / scaffold glyph | `#50a0e8` |
| Operational | solid dot (family hue) | catalog |
| Growth proposal (organic) | dashed square | district hue — **not** this doc |

**Under construction** sites: pulsing α on scaffold icon (subtle, 0.5 Hz).

---

## Command tray microcopy

| Context | String |
|:---|:---|
| Site selected, advancing | `Construction in progress` |
| Blocked (validation) | `Cannot advance — {reason}` |
| All sites operational in selection | `All sites operational` |

---

## F3 diagnostics one-liner

When `construction_site_stage_pipeline_001` present in witness:

```text
CON stage pipeline: {sites_active} active, {sites_operational} operational, witness={green}
```

Witness file: `debug_runs/construction_stage_live.json` → `/construction_site_stage_pipeline_001/green`

---

## Forbidden player copy

| String | Why |
|:---|:---|
| `Built instantly` | Violates P2 |
| `Operational` on commit toast | Misleading |
| `Zone complete` | Conflates zoning with site phase |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-02 |
