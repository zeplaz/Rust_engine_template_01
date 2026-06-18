# APS UI/UX Onboarding Spec `v1` — OVR-DES-P56-ONBOARD-SPEC-001

| Field | Value |
|:---|:---|
| **ID** | **OVR-DES-P56-ONBOARD-SPEC-001** |
| **Program** | PLAN-APS-UIUX-OVERHAUL-001 |
| **Phase** | P5.6 |
| **Date** | 2026-06-17 |
| **Owner** | `@designer` |
| **Authority** | [`aps_design_system_v1.md`](aps_design_system_v1.md) §0 #2 |
| **Outline** | [`design_aps_uiux_onboard_outline_v1.md`](design_aps_uiux_onboard_outline_v1.md) |
| **Spine** | [`design_aps_uiux_spine_spec_v1.md`](design_aps_uiux_spine_spec_v1.md) |
| **Implements** | `OVR-P56-ONBOARD-001` · `test_aps_onboarding.py` |
| **Verdict** | **PASS** |

```text
OVR-DES-P56-ONBOARD-SPEC-001 Q✓
Unblocks: OVR-P56-ONBOARD-001 (if not already landed)
```

---

## 1. First-run welcome card

**Placement:** inline below Row 2 chrome — **not modal**.

**Trigger:**

| Event | Show |
|:---|:---|
| `onboarding_seen_v1` absent | Buildings welcome |
| First switch to Landscape + `onboarding_landscape_seen_v1` absent | Landscape 4-step card |
| Help → `Show getting started` | Either card (non-blocking) |

### Buildings copy (verbatim)

**Title:** `How the Art Pipeline Suite works`

**Body:**
```text
1. Catalog — pick a building module.
2. Materials — create the looks you'll assign.
3. Assembly — place pieces and run Ship check.
4. Variants — set states for tile baking.
5. Atlas — pack and register tiles for the game.

Start on Catalog: select a module, then follow the pipeline above.
```

### Landscape copy (verbatim)

```text
1. Presets — choose a landscape.
2. Grammar — lay out regions and corridors.
3. States — author growth and fire looks.
4. Atlas — bake and register tiles.

Start on Presets: pick a preset and run Check schema.
```

### Actions

| Button | Behavior |
|:---|:---|
| `Start on Catalog` / `Start on Presets` | Dismiss card; select tab 0; spine `▣` on step 0 |
| `Don't show again` | `onboarding_seen_v1=true` (+ landscape flag if applicable) |
| `Show advanced data flow` | Expands metadata panel with copy-pack §4 blocks |

**Ban:** no schema keys, gate IDs, `Ship truth`, or `rust_engine_mcp` in welcome card.

---

## 2. Metadata panel defaults

| Context | Default |
|:---|:---|
| All tabs | **Collapsed** |
| First expand | Copy-pack §4 prose — not engineer diagram |
| Checkbox | `Show how tags & materials reach runtime` |

**Migration:** if `metadata_flow_seen_*` true but `onboarding_seen_v1` absent → set onboarding seen (veterans not re-nagged).

---

## 3. Empty-state catalog (verbatim strings)

| Surface | Condition | Headline | Sub / action |
|:---|:---|:---|:---|
| Catalog | no selection | `No module selected.` | `Select a module from the list to begin.` |
| Catalog | empty list | `Module list is empty.` | `Check assets path or import modules.` |
| Materials | empty | `No materials yet.` | `Create a material or import from disk.` |
| Assembly | no snapshot | `No Assembly yet.` | `Generate Assembly from Catalog.` |
| Assembly | no piece | `Select a piece on the grid to edit.` | — |
| Variants | empty | `No variant layers yet.` | `Add a layer or load a variant set.` |
| Atlas (B) | no folder | `No tile atlas yet.` | `Pack atlas when variants are ready.` |
| Presets | none | `No landscape selected.` | `Select a preset and run Check schema.` |
| Grammar | no graph | `No layout graph yet.` | `Generate grammar from your preset.` |
| States | empty | `No state rows yet.` | `Bake states from the catalog.` |
| Atlas (L) | no pack | `No landscape atlas yet.` | `Pack landscape atlas from States.` |
| Slot preview | nothing | `Nothing to preview.` | `Select a module or piece.` |

Pattern: `{What this is}. {One action}.` — centre in panel, not tooltip-only.

---

## 4. Progressive disclosure defaults

| Panel | Section | Default |
|:---|:---|:---:|
| Assembly | Setup / grammar advanced | collapsed |
| Assembly | Metadata flow | collapsed |
| Grammar | Iterate / shape bias | collapsed |
| Variants | Agent patch / raw JSON | collapsed |
| Atlas | Advanced / log / lod0 | collapsed |
| Status log | — | collapsed first run |
| Grammar inspector | — | collapsed (all tiers) |

---

## 5. Persistence (`aps_ui_prefs.json`)

| Key | Type | Purpose |
|:---|:---|:---|
| `onboarding_seen_v1` | bool | Welcome dismissed |
| `onboarding_landscape_seen_v1` | bool | Landscape intro shown once |
| `metadata_flow_expanded_{context}` | bool | Per-tab metadata |
| `metadata_flow_seen_{context}` | bool | Existing — **do not** auto-expand on first seen |

---

## 6. Spine-as-teacher

Onboarding **does not** duplicate the stepper.

| Moment | Spine |
|:---|:---|
| Welcome dismiss | `▣` on step 0; primary verb if prereqs pass |
| Step complete | P4.5 rules — pill updates, no auto tab steal |
| Pipeline complete | Hint: `Pipeline complete — review Atlas registration.` |

Optional (P2): 1s border tint on pipeline row first run — not blocking.

---

## 7. Verification

**Headless:** `test_aps_onboarding.py` — first-run renders welcome not schema diagram; empty-state strings present.

**NEEDS-DISPLAY:** operator dismiss + return visit; landscape first-switch card.

---

## 8. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-17 |
