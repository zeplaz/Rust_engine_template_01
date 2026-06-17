# APS preset browse QC criteria `v1` (DMCP-E2-PRESET-QC-CRITERIA-001)

| Field | Value |
|:---|:---|
| **Program** | **APS-E2** |
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-17 |
| **Verdict** | **PASS** |

Plain-language **must-read** fields when an artist opens a landscape preset in APS browse (Option D domain toggle).

---

## 1. Must-read panel (every preset)

| # | Label (APS) | Source field | Pass if |
|:---:|:---|:---|:---|
| Q1 | **Preset name** | `preset_id` + display string | Human title from [`_display_strings_v1.json`](../assets/configs/landscape/presets/_display_strings_v1.json) |
| Q2 | **District read** | `landscape_program.district_ref` or class | One line: ag / industrial / military / settlement / fire recovery |
| Q3 | **Topology summary** | `topology_graph[]` | Plain list: “Patch, Corridor, Ring, Cluster, Fringe” — **not** raw `preset_id` enums |
| Q4 | **Pressure headline** | top-2 λ fields | e.g. “High disturbance · Moderate moisture” |
| Q5 | **Ship status** | validator + `_meta.not_a_ship_target` | **Ship** / **Teach** / **Draft** badge |
| Q6 | **Chart family** | `chart_id` | Optional tooltip — not blocking |

---

## 2. Topology summary template

```
Topologies: {kinds_plain} ({count} kinds)
Nested depth: {max_depth}
Required: {required_topologies_joined}
```

**Example (fire_recovery_v0):**
> Topologies: irregular patch, rail corridor, natural cluster, edge fringe (4 kinds)  
> Nested depth: 2 · Required: PATCH_IRREGULAR, CORRIDOR_RAILSIDE, CLUSTER_NATURAL, FRINGE_EDGE

---

## 3. QC pass / fail

| Verdict | When |
|:---|:---|
| **PASS** | Q1–Q5 present · topology summary uses plain language · validator `landscape_grammar` pass |
| **FAIL** | Missing preset file · schema fail · topology list empty · `not_a_ship_target` hidden while teach preset shown as ship |

---

## 4. Artist action hints

| Badge | Copy |
|:---|:---|
| Ship | “Ready for map rollout — atlas may still be pilot tier.” |
| Teach | “Grammar teach preset — do not register as production atlas.” |
| Draft | “Fix validator errors before bake.” |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-06-17 |
