# PLAN-VEG-VARIANT-GRAPH-DECISION-001 — ARCH-002 reuse vs flat veg catalog `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ PLAN-ARCH-002-VEG-PATCH-001
Date: 2026-06-16
Status: **SIGNED** (@planner-mcp)
Parent: ARCH-002 · APS-EVO-E3
Schema graph: $ref:tools/mcp/schemas/variant_graph_v1.schema.json
Schema catalog: $ref:tools/mcp/schemas/vegetation_variant_catalog_v1.schema.json
```

## Decision

| Option | Verdict |
|:---|:---:|
| **A — Reuse `variant_graph_v1` for veg state patches** | **REJECT** |
| **B — Flat `vegetation_variant_catalog_v1` for veg** | **ACCEPT** |

---

## Rationale

`variant_graph_v1` targets **assembly graph nodes** (`node_id`, `role`, `semantic_tag`) on building snapshots — material/visibility/emission/decal **per mesh node**.

Vegetation extract resolves **per-chunk / per-row** state from sim overlays — no assembly graph, no Blender node tree. Forcing variant_graph would:

- Duplicate `variant_key` on fake `node_id`s
- Break parity with `variant_key_for_burn_row` (string keys, not node patches)
- Confuse APS Variants tab (building layers vs iso tile slots)

**Complement:** Buildings keep `variant_set_v1` + optional `variant_graph_v1`. Landscape keeps **catalog + tile_batch variants** only.

---

## Future bridge (non-goal v1)

If LG-6 multi-mesh vegetation props ship, revisit **per-prop** material overrides via variant_graph on a **building-style assembly** — not on grammar extract rows.

---

## Sign-off

| Role | Date | Action |
|:---|:---|:---|
| **@planner-mcp** | 2026-06-16 | **SIGNED** |

```text
⟦/PLAN-ARCH-002-VEG-PATCH-001⟧  catalog schema authoritative for E3
```
