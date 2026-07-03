# Reaction-territory events schema `v1` — DES-REACTION-TERRITORY-EVENTS-001

| Field | Value |
|:---|:---|
| **Gate** | **DES-REACTION-TERRITORY-EVENTS-001** |
| **Owner** | `@designer-mcp` (schema) → **`@coder-mcp`** (resolver + APS wire) |
| **Date** | 2026-06-02 |
| **Authority** | [`scarydayzx.txt`](../../docs/extr_cell_and_liquidation/scarydayzx.txt) |
| **Catalog** | [`reaction_territory_events_v1.yaml`](../../tools/mcp/schemas/examples/reaction_territory_events_v1.yaml) |
| **JSON Schema** | [`reaction_territory_events_v1.schema.json`](../../tools/mcp/schemas/reaction_territory_events_v1.schema.json) |
| **Verdict** | **PASS** — spec_only · no Bevy sim in this gate |

```yaml
order_critique:
  request_summary: "Map cultural liquidation doc events to variant layers + occupied metrics"
  rules_audit:
    doc_authority: pass
    variant_layer_indirection: pass
    no_sim_implementation: pass
  proceed: yes
```

---

## 1. Problem

Occupation / cultural-survival events in the design doc (`heritage_site_destruction`, `language_ban`, …) define **metric deltas** and **agent responses**, but APS preview and tile variant graphs need a **deterministic join**:

```text
doc event → abstract variant layer → concrete variant_key (per archetype)
         → tag_anchors (semantic slots)
         → preview_states (APS strip)
```

**Rule:** This catalog is **schema only**. Sim metric application and resolver wiring are **@coder-mcp** — not designer-mcp bpy.

---

## 2. Pattern (heritage destruction)

```yaml
heritage_site_destruction:
  variant_keys: [damaged_heavy, burning, scar_recovery_0]
  metric_deltas:
    heritage_integrity_index: -0.25
    cultural_continuity_index: -0.18
  tag_anchors: [burn_origin, heritage_marker, archive_slot]
  preview_states: [damaged, burning, clean]  # clean = recovery terminus
```

| Field | Meaning |
|:---|:---|
| `variant_keys` | **Abstract layers** — not direct atlas keys |
| `metric_deltas` | Deltas on `occupied_region_metrics` (clamped 0–1 where indexed) |
| `tag_anchors` | Semantic bind points for variant graph / metadata |
| `preview_states` | APS 4-state strip subset (`clean`/`damaged`/`burning`/`night`/`scar`) |

---

## 3. Layer resolution (`variant_layer_resolution`)

Abstract layers resolve per **domain** at runtime:

| Domain | `damaged_heavy` | `burning` | `scar_recovery_0` |
|:---|:---|:---|:---|
| `heritage_civic` | `damaged_heavy` | `burning_00` | `clean_day` |
| `building_rowhouse` | `damaged_day` | `burning_00` | `clean_day` |
| `landscape_topology` | `topology_patch_scar` | `topology_patch_burn_04` | `topology_patch_regrowth_grass` |

**CMCP task:** `resolve_reaction_territory_variant(event_id, domain) → concrete variant_key[]`

---

## 4. Tag anchors

| Anchor | Binds |
|:---|:---|
| `burn_origin` | fire frame axis · sim_fire |
| `heritage_marker` | site marker role · heritage_integrity |
| `archive_slot` | institutional_memory · record_preservation |
| `language_script` | language_vitality · signage_locale |
| `censorship_overlay` | censorship · night_off |
| `service_continuity` | essential_service_continuity · legitimacy |

---

## 5. Event catalog (v1)

| Event ID | Trigger | Layers |
|:---|:---|:---|
| `heritage_site_destruction` | `destruction_of_heritage_sites` | damaged_heavy · burning · scar_recovery_0 |
| `language_ban` | `language_ban` | censorship_dim |
| `transparent_bilingual_service_continuation` | — (legitimate change) | service_lit |
| `forced_assimilation_in_schools` | `forced_assimilation_in_schools` | censorship_dim · damaged_heavy |
| `archive_seizure_or_censorship` | `seizure_or_censorship_of_archives` | damaged_heavy · scar_recovery_0 |
| `forced_renaming` | `forced_renaming` | censorship_dim · damaged_heavy |
| `banning_cultural_or_religious_practices` | `banning_of_cultural_or_religious_practices` | censorship_dim |
| `removal_of_children_from_institutions` | `removal_of_children_from_community_institutions` | damaged_heavy · scar_recovery_0 |
| `forced_displacement` | `forced_displacement` | damaged_heavy · scar_recovery_0 |
| `erasure_of_local_history` | `erasure_of_local_history` | damaged_heavy · scar_recovery_0 |
| `imperial_institution_replacement` | `replacement_of_local_institutions_with_imperial_administration` | censorship_dim |

Full rows: YAML catalog § `events` (**11** events · **10/10** liquidation triggers).

---

## 6. Handoff (@coder-mcp)

| Task | Deliverable |
|:---|:---|
| **CMCP-REACTION-TERRITORY-RESOLVE-001** | Python resolver: event + domain → concrete keys |
| **CMCP-REACTION-TERRITORY-PREVIEW-001** | APS preview strip maps `preview_states` |
| **Optional** | Extend `variant_matrix_*.yaml` with `reaction_territory_ref` column |

**Witness:** `debug_runs/art_pipeline/dmcp_reaction_territory_events_live.json`  
**CLI:** `python -m rust_engine_mcp.cli dmcp-reaction-territory-events-witness`

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-06-02 |

```text
DES-REACTION-TERRITORY-EVENTS-001 Q✓ — schema on disk · resolver open @coder-mcp
```
