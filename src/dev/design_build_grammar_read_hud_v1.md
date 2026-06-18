# Building Grammar Read HUD `v1` — DES-BUILD-READ-HUD-001

| Field | Value |
|:---|:---|
| **ID** | **DES-BUILD-READ-HUD-001** |
| **Program** | PLAN-BUILD-READABILITY-001 / grammar evolution consumer |
| **Date** | 2026-06-17 |
| **Owner** | `@designer` |
| **Authority** | [`design_build_readability_v1.md`](design_build_readability_v1.md) · [`grammar_labels_v1.json`](../../assets/configs/buildings/grammars/grammar_labels_v1.json) |
| **Consumer** | sim construction HUD · placement debug · commit toast |
| **Verdict** | **PASS** |

```text
DES-BUILD-READ-HUD-001 Q✓
Human grammar fields on sim-side — not APS panels
```

---

## 0. Scope

When `arch_build_grammar` / procedural commit produces a building, the **sim HUD** (not APS) shows readable grammar choices — same labels as APS inspector, shorter form.

**Ban:** `IndustrialWarehouse`, `long_hall`, `assembly_snapshot` in player-facing strip.

---

## 1. Context strip templates (sim)

| Moment | Template |
|:---|:---|
| Ghost preview | `BUILD · {archetype_label} · {district_label} · click map to lock` |
| Adjust valid | `BUILD · {archetype_label} · locked · Ctrl rotate · Shift scale` |
| After commit | `Placed · {archetype_label} · {massing_label}` |
| Blocked | `Blocked — {reason}` |

**Label source:** `human_label()` from `aps_grammar_labels.py` (shared module).

---

## 2. Grammar summary chip (placement debug / F3)

When grammar snapshot attached to ghost:

```text
Style: {archetype_label} · {massing_label} · {age_label}
```

Example: `Style: Rail edge warehouse · L-shaped yard · Weathered`

Collapse engineer chain to **three words max** on HUD; full chain in debug panel only.

---

## 3. Site overlay legend extension

When site stub active ([`design_build_toolbox_hud_v1.md`](design_build_toolbox_hud_v1.md) §4):

| Zone | Label | Grammar link |
|:---|:---|:---|
| Primary | `Building` | massing footprint cells |
| Loading | `Load` | `l_shape` wing |
| Utility | `Svc` | `yard_complex` void |
| Rail | `Rail` | `RailEdge` districts |

---

## 4. Toast on commit

| Event | Copy |
|:---|:---|
| Success | `✓ {archetype_label} committed` |
| Grammar stale | `◐ Style outdated — regenerate in APS before next variant bake` |

---

## 5. Archetype label table (G1)

| id | HUD label |
|:---|:---|
| `IndustrialWarehouse` | Industrial warehouse |
| `FactoryCluster` | Factory cluster |
| `RailEdge` | Rail edge warehouse |

---

## 6. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-17 |

**@coder:** wire in `contextual_tip.rs` / placement debug — reuse MCP label helper or mirror JSON.
