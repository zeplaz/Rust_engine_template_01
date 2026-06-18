# Nuclear PWR plant massing concept `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-ART-NUCLEAR-PLANT-001** |
| **Program** | PLAN-POWER-GRID-ART-ASSETS-001 · Lane A |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Depends** | [`design_utility_industrial_style_v1.md`](design_utility_industrial_style_v1.md) |
| **Handoff** | **DMCP-SPEC-NUCLEAR-PWR-001** |
| **Verdict** | **PASS** |

```text
DES-ART-NUCLEAR-PLANT-001 Q✓
PWR silhouette + zone massing — hands off to designer-mcp AssetSpec
```

---

## 0. Catalog anchor

| Field | Value |
|:---|:---|
| Building id | `nuclear_plant_pwr` (planned) |
| Footprint | **6×6** tiles (site) |
| Tier | `power_tier: generation` · `utility_role: nuclear` |
| Style pack | `style_industrial_west` + utility bible §1 |

---

## 1. Silhouette (@ 64px iso)

**Must read:** **dome** (containment) + **turbine hall** (low box) + **cooling** (tower or bank).

```text
        ╭───╮
       │ dome│     ← containment — primary landmark
    ┌──┴────┴──┐
    │ turbine  │   ← long low hall
    └──┬───┬───┘
       │   │
      tower  aux diesel pad
```

**Height:** dome apex ≤3 grid units · hall ≤1.5u · cooling tower ≤2.5u.

---

## 2. Site zones (facility grammar)

| Zone | % mass | Content |
|:---|:---:|:---|
| **primary** | 40% | Containment + turbine hall |
| **utility** | 25% | Switchyard edge, transformer row |
| **service** | 15% | Admin / security shack |
| **buffer** | 20% | Gravel setback, fence |

Aligns with [`design_facility_site_zone_taxonomy_v1.md`](design_facility_site_zone_taxonomy_v1.md).

---

## 3. Module whitelist (DMCP)

| Module id | Grid | Notes |
|:---|:---|:---|
| `containment_dome_pwr` | 3×3 | Cylindrical dome, concrete |
| `turbine_hall_1u` | 2×1 | Repeat for hall length |
| `cooling_tower_1u` | 1×1 | Or `cooling_bank_2u` variant |
| `diesel_gen_pad_2x2` | 2×2 | Backup — links diesel glyph |
| `switchyard_edge_1u` | 1×1 | Bus + fence segment |
| `fence_chainlink_1u` | 1×1 | Perimeter |
| `warning_sign_nuclear_1u` | 1×1 | Yellow + trefoil simplified |

---

## 4. State read (gameplay)

| State | Visual |
|:---|:---|
| Normal | Dome + steady stack |
| SCRAM | Amber dome wash + `icon_scram` HUD |
| Meltdown | Column effect + danger adjunct |
| Diesel run | Aux pad exhaust animation |

Pairs [`power_glyphs_spec_v1.md`](../../assets/ui/infrastructure/power_glyphs_spec_v1.md) `node_plant_nuclear_*`.

---

## 5. DMCP handoff checklist

- [x] AssetSpec `kit_nuclear_pwr_production_001` — 6×6 site assembly
- [x] `containment_dome_pwr` hero module — production tier (spec listed)
- [ ] Facility needs strip row: `grammar_facility_brief` nuclear generation
- [x] Staging witness under `debug_runs/art_pipeline/dmcp_nuclear_pwr_spec_live.json`

**Next owner:** `@coder-mcp` · **MCP-PWR-NUCLEAR-BATCH-001** (bpy — unblocked)

---

## 6. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |
