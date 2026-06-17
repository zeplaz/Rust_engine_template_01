# DES-APS-STATE-AXIS-LABELS-001 — Succession + burn axis labels `v2`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-STATE-AXIS-LABELS-001** |
| **Program** | APS-E3 · unblocks **APS-EVO-E3-VEG-STATE-AXIS-001** |
| **Schema** | [`vegetation_variant_catalog_v1.schema.json`](../../tools/mcp/schemas/vegetation_variant_catalog_v1.schema.json) |
| **Naming** | [`plan_veg_variant_key_naming_v1.md`](plan_veg_variant_key_naming_v1.md) |
| **Date** | 2026-06-17 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |

---

## States tab wireframe (Landscape lane · not Buildings Variants)

```text
┌─ States — succession + disturbance matrix ─────────────────────────────────────────┐
│ ENGINE_READ_PATH (landscape_states) — collapsible ▼                                  │
├──────────────────────────────────────────────────────────────────────────────────────┤
│ Axis editors (catalog.axes — saved to _vegetation_variant_catalog.ron)               │
│                                                                                      │
│  Succession stage     [Grass ▼] [Shrub] [Sapling] [Canopy] [OldGrowth] [BurnScar]   │
│  Regrowth macro       [None ▼] [Scar] [Nuclei] [Front] [Closing] [Mature]            │
│  Burn frames          8 frames · veg_burn_00 … veg_burn_07  [Preview frame ▼ 03]    │
│                                                                                      │
├──────────────────────────────────────────────────────────────────────────────────────┤
│ Variant catalog rows (entries[])                                                   │
│ ┌──────────────┬─────────────────────────┬──────────┬─────────────────────────────┐  │
│ │ variant_key  │ Resolver (plain)        │ Status   │ Atlas slot                  │  │
│ ├──────────────┼─────────────────────────┼──────────┼─────────────────────────────┤  │
│ │ topology_patch│ Patch topology sprite    │ ○ pending│ landscape_lg5_pilot_v1      │  │
│ │ veg_burn_03  │ Active fire · frame 3    │ ○ pending│ —                           │  │
│ │ veg_regrowth_nuclei │ Regrowth · nuclei │ ○ pending│ —                           │  │
│ └──────────────┴─────────────────────────┴──────────┴─────────────────────────────┘  │
│                                                                                      │
│ [Load catalog] [Save catalog] [Validate] [Bake states → tile_batch]                │
│ ○ States pending — select preset + grammar first                                     │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

**Hard rule:** No building Variants layer comboboxes (lighting/damage/fill/power). Landscape States is a **separate panel** (`landscape_states_panel.py`).

---

## Axis 1 — Succession stage (`resolver.kind = succession_stage`)

| Schema enum | UI label (combobox) | Short (tree column) | Tooltip |
|:---|:---|:---|:---|
| `Grass` | **Pioneer grass** | grass | Bare / pioneer cover after gap |
| `Shrub` | **Shrub thicket** | shrub | Low woody regrowth |
| `Sapling` | **Young stems** | sapling | Establishing canopy |
| `Canopy` | **Closed canopy** | canopy | Mature closed cover |
| `OldGrowth` | **Old growth** | old growth | Long-horizon climax stage |
| `BurnScar` | **Burn scar** | burn scar | Persistent post-fire scar on succession graph |

**Multi-select chips:** artist toggles which stages appear in `catalog.axes.succession_stages[]`.

---

## Axis 2 — Regrowth macro (`resolver.kind = regrowth_macro`)

| Schema enum | UI label | Short | Tooltip |
|:---|:---|:---|:---|
| `None` | **No regrowth** | none | Undisturbed macro phase |
| `Scar` | **Scar hold** | scar | Ash scar before nuclei |
| `Nuclei` | **Regrowth nuclei** | nuclei | Spot regrowth seeds |
| `Front` | **Regrowth front** | front | Advancing edge |
| `Closing` | **Canopy closing** | closing | Gaps filling in |
| `Mature` | **Regrowth mature** | mature | Hands off to succession stage |

**Multi-select:** `catalog.axes.regrowth_macro_phases[]`.

---

## Axis 3 — Active burn frames (`resolver.kind = active_burn_frame`)

| Pattern | UI label | Short | Tooltip |
|:---|:---|:---|:---|
| `veg_burn_00` | **Fire start** | burn 0 | Ignition frame |
| `veg_burn_01` … `06` | **Fire frame N** | burn N | Mid burn loop |
| `veg_burn_07` | **Fire end** | burn 7 | Late / ember frame |

**Frame count:** `catalog.axes.burn_frame_count` (default **8**, max 16). Preview combobox shows `veg_burn_{:02}` labels only — not color-only.

| Preview value | Display |
|:---|:---|
| frame 0 | `Fire start (veg_burn_00)` |
| frame 3 | `Fire mid (veg_burn_03)` |
| frame 7 | `Fire end (veg_burn_07)` |

---

## Catalog row — resolver plain labels (`entries[]` tree)

| `resolver.kind` | Plain label template |
|:---|:---|
| `topology_kind` | `{topology_kind} topology sprite` — e.g. `Patch topology sprite` |
| `active_burn_frame` | `Active fire · frame {index}` |
| `regrowth_macro` | `Regrowth · {phase_label}` |
| `succession_stage` | `Succession · {stage_label}` |
| `default` | `Default fallback` |

### `variant_key` display (monospace secondary)

Show `variant_key` in **State key** column; **Label** column uses plain text above. Never label-only from color.

---

## Status column vocabulary (States tree)

| Internal | Display | fg token |
|:---|:---|:---|
| `blocked` | `○ blocked — no preset` | `COLOR_MUTED` |
| `await_grammar` | `◐ await grammar` | `COLOR_WARN` |
| `scaffold` | `◐ scaffold` | `COLOR_WARN` |
| `pending` | `○ pending` | `COLOR_MUTED` |
| `valid` | `✓ valid` | `COLOR_PASS` |
| `fail` | `✗ FAIL` | `COLOR_FAIL` |

After **Bake states:** pipeline pill `✓ States valid` or `◐ States saved (QC not run)`.

---

## LG-5 gate labels (Atlas tab cross-ref — scope-explicit)

| Gate | UI label | Status words |
|:---|:---|:---|
| G0 | **Schema valid** | `✓ schema PASS` / `✗ schema FAIL` |
| G1 | **Topology wired** | `✓ wired` / `○ pending` |
| G2 | **Bake output** | `✓ bake OK` / `◐ partial` |
| G3 | **Keyframe stills** | `✓ stills present` / `○ missing` |
| G4 | **Art QC** | `✓ art PASS` / `✗ art FAIL` |
| G5 | **Ship target** | `✓ ship OK` / `⊘ teach pilot only` |

Never collapse to one `✓ Atlas valid` without G-scope in tooltip.

---

## Inline hints (bottom of States tab)

| Condition | Copy |
|:---|:---|
| No preset | `○ States pending — select a landscape preset on Presets tab` |
| Preset, no grammar | `◐ States blocked — generate grammar on Grammar tab` |
| Ready to bake | `Bake states prepares LG-5 tile batch — then Pack LG-5 atlas on Flow bar` |
| Catalog invalid | `✗ Catalog FAIL — fix rows before bake` |

---

## Buildings Variants tab

Unchanged (lighting / damage / fill / material). **Hidden** when Landscape lane active — no dual meaning.

---

## Tooltip keys (`aps_tooltips.py`)

| Key | Text |
|:---|:---|
| `state_succession_axis` | Succession stages written to catalog.axes — long-term cover ladder. |
| `state_regrowth_axis` | Regrowth macro phases — transient post-disturbance window. |
| `state_burn_frames` | Eight-frame burn loop (veg_burn_00–07) — matches engine VEG_BURN_FRAME_COUNT. |
| `state_bake` | Expand catalog entries to tile_batch variants — does not replace preset authority. |
| `state_catalog_validate` | Validate against vegetation_variant_catalog_v1 schema before bake. |

---

## @coder-mcp implementation checklist

| ☐ | Wire `LandscapeStatesPanel` labels from tables above (not scaffold `_STATE_ROWS`) |
| ☐ | Combobox values = schema enums; display = UI label column |
| ☐ | Tree **Label** column = plain resolver label; **State key** = `variant_key` |
| ☐ | Status column uses glyph + word + `validation_foreground` |
| ☐ | `mark_states_ready()` only after validate PASS |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-17 |

```text
DES-APS-STATE-AXIS-LABELS-001 Q✓ — unblocks APS-EVO-E3-VEG-STATE-AXIS-001
```
