# APS tag tier-2 — archetype presets + sim-coupled tags `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-TAG-TIER2-001** |
| **Parent** | [`design_aps_tag_vocab_creative_pass_v1.md`](design_aps_tag_vocab_creative_pass_v1.md) |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **Handoff** | `@coder-mcp` preset loader · taxonomy JSON |
| **Verdict** | **PASS (qualified)** — spec only; wire after operator tier-1 rubric |

```text
DES-APS-TAG-TIER2-001 Q✓
One-click tag bundles per archetype + sim-coupled placement tags
```

---

## 1. Archetype tag presets (Variants + Assembly)

When artist selects archetype on Assembly or creates variant set from assembly, offer **optional preset** (never auto-apply without confirm):

| Archetype | Preset name | Mandate tags (Variants) | Semantic highlights (Assembly placement) |
|:---|:---|:---|:---|
| IndustrialWarehouse | `industrial_logistics` | `day_lit`, `service_continuity` | `loading_dock`, `rail_adjacent`, `industrial` |
| FactoryCluster | `process_plant` | `day_lit` | `utility`, `cooling_tower`, `pipework`, `transformer_yard` |
| CivicBlock | `civic_day` | `day_lit`, `heritage_integrity` | `civic`, `street_facing`, `signage` |
| RailEdge | `rail_corridor` | `day_lit` | `rail_adjacent`, `industrial`, `loading_dock` |

**UI:** `Apply tag preset…` button on Variants (mandate families) + Assembly tag section. Confirm dialog lists tags before apply.

---

## 2. Sim-coupled tags (presentation metadata only)

| Tag id | Category | When suggested | Player/sim story |
|:---|:---|:---|:---|
| `district_power_feed` | detail | Near substation placement in power UX slice | Grid tie-in read |
| `bilingual_signage` | detail | `language_ban` reaction session filter active | Resistance / locale visibility |
| `occupation_banner` | condition | Reaction territory event with policy layer | Transitional governance |
| `decommissioned` | condition | P0 gate fail on power-down scenario | Powered down shell |

Add to `aps_tag_taxonomy_v1.json` + `MANDATE_TAG_VOCAB` when coder-mcp wires.

---

## 3. Reaction session bundles

When reaction filter selects an event, offer **Suggested tags** chip row (one-click add to draft, still requires Apply layers):

| Event | Suggested mandate tags |
|:---|:---|
| Heritage site destruction | `burn_origin`, `heritage_marker`, `archive_slot` |
| Language ban | `censorship`, `language_script`, `signage_locale` |
| Transparent bilingual service | `bilingual_transparency`, `service_continuity` |

Uses catalog `tag_anchors` from `reaction_territory_events_v1.json`.

---

## 4. Acceptance

| # | Check |
|:---:|:---|
| T1 | Preset confirm dialog — no silent tag mutation |
| T2 | Preset JSON or in-code map versioned beside taxonomy |
| T3 | Suggested tags from reaction filter match catalog anchors |
| T4 | pytest extends `test_aps_tag_vocabulary.py` for new tag ids |

---

## 5. Exit predicate

Operator completes [`design_aps_tag_operator_rubric_v1.md`](design_aps_tag_operator_rubric_v1.md) tier-1 **PASS** → coder-mcp implements preset loader → witness `aps_tag_tier2_live.json`.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS (qualified)** | 2026-06-02 |
