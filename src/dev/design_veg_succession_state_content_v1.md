# Succession stage → atlas variant rows `v1` (DMCP-SUCCESSION-STATE-CONTENT-001)

| Field | Value |
|:---|:---|
| **Program** | **APS-E3** (content parallel to schema) |
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-17 |
| **ECS** | `SuccessionTopologyStage` in `landscape_grammar_lg2.rs` |

Content authority for variant rows — catalog on disk (**DMCP-E3 PASS** 2026-06-17).

---

## Stage → variant_key mapping

| `SuccessionTopologyStage` | Atlas suffix | Extract glyph | Tint bias (LG-4) |
|:---|:---|:---:|:---|
| `Grass` | `_regrowth_grass` | `,` | +yellow channel |
| `Shrub` | `_regrowth_shrub` | `.` | +mid green |
| `YoungForest` | `_regrowth_canopy` | `*` | +dark green |
| `OldGrowth` | _(clean row)_ | `#` | baseline canopy |
| `BurnScar` | `_scar` | `x` | −saturation, ash |

---

## Disturbance overlay rows

| Event | Variant pattern | Notes |
|:---|:---|:---|
| Fire tick 0 | `{topology}_burn_00` | Pairs with `DisturbanceKind::Fire` |
| Fire mid | `{topology}_burn_04` | Operator-readable peak |
| Fire late | `{topology}_burn_07` | Optional per topology (matrix §3) |
| Post-fire gap | `{topology}_scar` | Before regrowth ticks advance |

---

## Gap / regen / shrub / sapling / canopy content strings (APS)

| Stage | Display string | Operator hint |
|:---|:---|:---|
| Gap | “Disturbance gap — bare soil” | After fire or construction |
| Regen grass | “Pioneer grass — year 0–1” | Low fuel |
| Shrub | “Shrub thicket — year 2–4” | Medium fuel |
| Sapling | “Young stems — year 5–10” | Rising canopy |
| Canopy | “Closed canopy — mature” | Old-growth target |

---

## Catalog handoff (when E3 unblocks)

Planner-mcp schema → designer-mcp fills `_vegetation_variant_catalog.ron` with rows above + `sim_tags` per warehouse fire pattern.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** (content only — catalog file blocked on schema) | 2026-06-17 |
