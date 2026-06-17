# APS UI/UX Onboarding Outline `v1` — DES-OVR-P56-ONBOARD-OUTLINE-001

| Field | Value |
|:---|:---|
| **ID** | **DES-OVR-P56-ONBOARD-OUTLINE-001** |
| **Program** | PLAN-APS-UIUX-OVERHAUL-001 |
| **Phase** | P5.6 (onboarding) — **outline only** |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **Authority** | [`aps_design_system_v1.md`](aps_design_system_v1.md) §0 #2 |
| **Depends** | [`design_aps_uiux_spine_spec_v1.md`](design_aps_uiux_spine_spec_v1.md) (spine teaches) |
| **Final spec** | `design_aps_uiux_onboard_spec_v1.md` (`OVR-DES-P56-ONBOARD-SPEC-001`) |
| **Implements** | `OVR-P56-ONBOARD-001` (later) |
| **Verdict** | **PASS** — outline ready; full spec blocked on P4.5 landing |

```text
DES-OVR-P56-ONBOARD-OUTLINE-001 Q✓
Outline for @designer finalize → OVR-DES-P56-ONBOARD-SPEC-001 after OVR-P45-SPINE-001 Q✓
```

---

## 0. Intent

A first-time artist should know **what this tool is**, **which lane to use**, and **the one action to take next** — without reading engineering diagrams or agent docs.

**Teacher model:** the P4.5 spine is the ongoing guide; onboarding is a **one-time orientation** that points at the spine, then gets out of the way.

---

## 1. Document map (finalize in full spec)

| § | Section | Owner file(s) |
|:---:|:---|:---|
| 1 | First-run greeting | `app.py`, `metadata_flow_panel.py` |
| 2 | Lane chooser copy | `domain_router.py` authority + welcome |
| 3 | Empty-state catalog | each `*_panel.py` |
| 4 | Progressive disclosure defaults | `assembly_panel.py`, grammar panels |
| 5 | Dismiss + persistence | `state.py`, `debug_runs/aps_ui_prefs.json` |
| 6 | Spine-as-teacher hooks | `pipeline_status_bar.py` (read-only copy) |
| 7 | Guard tests | `test_aps_onboarding.py` |

---

## 2. First-run greeting (replace schema diagram)

### 2.1 When

| Trigger | Show |
|:---|:---|
| First launch per machine (`onboarding_seen_v1` absent) | Welcome card |
| Return visit | No card; metadata panel **collapsed** |
| User clicks "Show how this works again" | Welcome card (non-modal) |

### 2.2 Welcome card content (draft)

**Title:** `How the Art Pipeline Suite works`

**Body (Buildings default):**

```text
1. Catalog — pick a building module.
2. Materials — create the looks you'll assign.
3. Assembly — place pieces and run Ship check.
4. Variants — set states for tile baking.
5. Atlas — pack and register tiles for the game.

Start on Catalog: select a module, then use the pipeline above.
```

**Landscape variant** (shown when user first switches to Landscape, once):

```text
1. Presets — choose a landscape.
2. Grammar — lay out regions and corridors.
3. States — author growth and fire looks.
4. Atlas — bake and register tiles.

Start on Presets: pick a preset and validate it.
```

**Actions:**

| Button | Behavior |
|:---|:---|
| `Start on Catalog` / `Start on Presets` | Dismiss card; select tab 0; set spine current |
| `Don't show again` | Set `onboarding_seen_v1=true` |
| `Show advanced data flow` | Expands legacy metadata body (today's diagram rewrite per copy pack §4) |

**Ban:** no schema keys, gate IDs, or `Ship truth:` in welcome card.

### 2.3 Metadata panel default

| Context | Default |
|:---|:---|
| All tabs | **Collapsed** |
| First expand | Show copy-pack §4 prose blocks — not engineer diagram |
| Checkbox label | `Show how tags & materials reach runtime` (existing) |

---

## 3. Empty-state map (per surface)

Pattern: `{What this is}. {One action}.`

| Surface | Empty condition | Headline | Primary action |
|:---|:---|:---|:---|
| **Catalog** | no module selected | `No module selected.` | `Select a module from the list to begin.` |
| **Materials** | empty library | `No materials yet.` | `Create a material or import from disk.` |
| **Assembly** | no snapshot | `No Assembly yet.` | `Generate Assembly from Catalog.` |
| **Assembly** | snapshot, no selection | `Select a piece on the grid to edit.` | — |
| **Variants** | no variant set | `No variant layers yet.` | `Add a layer or load a variant set.` |
| **Atlas (B)** | no folder | `No tile atlas yet.` | `Pack atlas when variants are ready.` |
| **Presets** | none selected | `No landscape selected.` | `Select a preset and run Check schema.` |
| **Grammar** | no graph | `No layout graph yet.` | `Generate grammar from your preset.` |
| **States** | empty matrix | `No state rows yet.` | `Bake states from the catalog.` |
| **Atlas (L)** | no pack | `No landscape atlas yet.` | `Pack landscape atlas from States.` |
| **Slot preview** | no selection | `Nothing to preview.` | `Select a module or piece.` |
| **Catalog list** | zero modules | `Module list is empty.` | `Check assets path or import modules.` |

Place empty copy in-panel (centred or below list), not tooltip-only.

---

## 4. Progressive disclosure (defaults)

| Panel | Section | Default | Rationale |
|:---|:---|:---:|:---|
| Assembly | Setup / grammar advanced | collapsed | Happy path = grid + material assign |
| Assembly | Metadata flow | collapsed | Welcome + spine teach first |
| Grammar | Iterate / pressure | collapsed | Power users only |
| Variants | JSON raw view | collapsed | Declarative UI first |
| Atlas | Debug / legacy Blender | hidden unless env flag | Existing `RUST_ENGINE_ART_DEBUG_GUI` |
| Status log | expanded | **collapsed** first run | Notebook priority |

---

## 5. Persistence keys (`aps_ui_prefs.json`)

| Key | Type | Purpose |
|:---|:---|:---|
| `onboarding_seen_v1` | bool | Welcome card dismissed forever |
| `onboarding_landscape_seen_v1` | bool | Landscape 4-step intro shown once |
| `metadata_flow_expanded_{context}` | bool | Per-tab metadata expand (existing) |
| `metadata_flow_seen_{context}` | bool | Existing — **stop auto-expand on first seen** |

Migration: if `metadata_flow_seen_*` true but `onboarding_seen_v1` absent, set onboarding seen (don't re-nag veterans).

---

## 6. Spine-as-teacher (no extra UI)

Onboarding does **not** duplicate the stepper. It references it.

| Moment | Spine behavior |
|:---|:---|
| Welcome dismiss | `▣` on step 0; primary verb enabled if prereqs pass |
| Step completes | P4.5 advance rules (pill updates, no tab steal) |
| All steps valid | Row-1 hint: `Pipeline complete — review Atlas registration.` |

Optional first-run pulse: highlight pipeline row once (border tint 1s) — **P5.6 nice-to-have**, not blocking.

---

## 7. Acceptance sketch (full spec will lock)

- [ ] First launch: welcome card visible; metadata **not** auto-expanded with schema diagram
- [ ] Dismiss remembered across sessions
- [ ] Every primary tab has empty-state copy from §3
- [ ] Advanced sections collapsed per §4
- [ ] `test_aps_onboarding.py`: headless first-run + empty-state strings present
- [ ] Feel: NEEDS-DISPLAY operator walk

---

## 8. Open questions (resolve in `onboard_spec_v1`)

| # | Question | Lean |
|:---:|:---|:---|
| Q1 | Modal vs inline welcome card? | **Inline** below Row 2 — avoids blocking window |
| Q2 | Per-lane onboarding vs Buildings-only first? | **Buildings first**; Landscape intro on first lane switch |
| Q3 | Video / illustration? | **No** — text + spine sufficient for v1 |
| Q4 | Re-show entry point? | Help menu → `Show getting started` |

---

## 9. Handoff chain

```text
OVR-DES-P45-SPINE-SPEC-001 (this wave)
    → OVR-P45-SPINE-001 (@coder-mcp)
    → OVR-DES-P56-ONBOARD-SPEC-001 (@designer finalize)
    → OVR-P56-ONBOARD-001 (@coder-mcp)
```

---

## 10. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** (outline) | 2026-06-02 |

```text
DES-OVR-P56-ONBOARD-OUTLINE-001 Q✓ — sections + empty-state map drafted
```
