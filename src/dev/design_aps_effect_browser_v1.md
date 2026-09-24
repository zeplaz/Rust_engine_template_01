# APS EffectSpec browse / preview / assign `v1` — VSS-T4-005

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-EFFECT-BROWSER-001** |
| **Program** | PLAN-ARTIST-VFX-TOOLCHAIN-001 · VSS-T4-005 |
| **Date** | 2026-09-24 |
| **Owner** | `@designer` |
| **Authority** | [`plan_artist_vfx_toolchain_v1.md`](plan_artist_vfx_toolchain_v1.md) §P4 · [`design_aps_preview_ladder_v1.md`](design_aps_preview_ladder_v1.md) · [`design_aps_mat_browse_v1.md`](design_aps_mat_browse_v1.md) |
| **Schema** | `tools/mcp/schemas/effect_spec_v1.schema.json` (G0 green) |
| **Pattern** | `MaterialBrowserPanel` / `mount_material_library` — **mirror, do not fork** |
| **Handoff** | `tools/mcp/art_pipeline_suite/effect_browser_impl_packet_v1.md` → `@coder-mcp` |
| **Witness** | `debug_runs/artist_vfx_pipeline_live.json` |
| **Verdict** | **PASS** |

```text
DES-APS-EFFECT-BROWSER-001 Q✓
APS Effects tab — browse batch · preview ladder · assign → scenario markers
⛔ fork fire_vfx draw · ⛔ sim HUD chrome · ⛔ in-game spark/smoke product path
```

---

## 0. Scope fence (hard)

| In scope (APS artist panel) | Out of scope (other owners) |
|:---|:---|
| Browse `assets/effects/registry/` + staging | In-game fire / spark / smoke absence |
| Preview ladder gated by `honest_gate` | Yellow/green chrome jank (sim HUD) |
| Assign effect → scenario `TriggerEffect` / trigger RON marker | Runtime emit spine (VSS-T4-005b **done**) |
| Tooltip + status copy | New GPU / particle draw path |

**Authority rule:** `lane: particle_instanced | gpu_field` already routes through `fire_vfx` / weather field spines. This panel **never** invents a parallel preview draw — preview uses staging thumbs / capture frames / honest_gate badges only.

---

## 1. UX goals

1. Artist finds any registry EffectSpec in ≤2 clicks (batch filter + search).
2. Preview honesty is **visible and non-bypassable** — `pending` / `dishonest_gate` cannot look like ship-ready.
3. Assign binds `effect_id` to a **scenario marker** (trigger RON + optional scenario step), not to Assembly slots.
4. Interaction states match Materials: idle · hover · selected · assign-active · blocked · loading.

---

## 2. Interaction problems (today)

| # | Problem |
|:---:|:---|
| P1 | No APS surface for EffectSpec — artists edit JSON / registry folders by hand |
| P2 | `preview_witness.honest_gate` stays `pending` with `frames_captured: 0` — no UI surface for that honesty |
| P3 | Scenario `TriggerEffect` / trigger RON is disconnected from the consumable registry browser |
| P4 | Risk of forking a “VFX preview” that looks like Materials but drives a fake particle sandbox |

---

## 3. Proposed interaction model

### 3.1 Mount pattern (mirror Materials)

```text
effect_browser.py
  EffectBrowserPanel(EffectLibraryWidget)   ← only wrapper
  mount_effect_library(master, mount=…)     ← only factory

MOUNT_BROWSE = "browse"   # Effects tab (studio)
MOUNT_ASSIGN = "assign"   # right rail / marker assign strip
```

Same as `MaterialBrowserPanel` + `mount_material_library` — **one** widget class, mount presets only.

### 3.2 Tab placement

| Lane | Tab | Label |
|:---|:---|:---|
| Buildings notebook | new tab after **Materials** | **Effects** |

```text
Catalog → Materials → Effects → Assembly → Variants → Atlas
```

Landscape lane: **no Effects tab** in v1 (effects are world/scenario consumables, not landscape presets).

Update `domain_router.BUILDINGS_TAB_LABELS` + pipeline step key `effects`.

### 3.3 Layout (Effects tab — studio)

```text
┌ Effects ──────────────────────────────────────────────────────────┐
│ Browse registry effects, preview honesty, assign to scenario     │
│ markers — does not draw in the game view.                        │
├──────────────┬─────────────────────┬─────────────────────────────┤
│ Batches      │ Effects             │ Preview ladder              │
│ (tree)       │ (list)              │ + Assign strip              │
│ min 160      │ min 220             │ min 280                     │
│              │                     │                             │
│ ▼ Reference  │ spark_shower   ◐    │  [P0][P1][P2][P3][P4]     │
│   vss_t4_…   │ smoke_column   ○    │  ┌─────────────────┐      │
│ ▼ Staging    │ rain_streaks   ○    │  │ thumb / stub    │      │
│ ▼ Recent     │                     │  └─────────────────┘      │
│              │                     │  ◐ pending — no frames     │
│              │                     │  [Assign to marker…]       │
└──────────────┴─────────────────────┴─────────────────────────────┘
```

**Intro copy:** `Browse registry effects, check preview honesty, then assign to a scenario marker — this panel does not draw particles in the game.`

### 3.4 Browse list

| Column | Content |
|:---|:---|
| Label | `display_name` (fallback mono `effect_id`) |
| Lane | caption: `particle` / `gpu_field` / `overlay` / `embellish` |
| Honesty | glyph+word — never hue-only |

| Gate | Glyph+word | Assign |
|:---|:---|:---|
| `honest` | `✓ honest` | enabled |
| `pending` | `◐ pending — no frames` | **blocked** (tooltip: capture preview first) |
| `dishonest_gate` | `⊘ dishonest — do not assign` | **blocked** |
| missing witness | `○ no preview witness` | **blocked** |

Source of truth: registry `manifest.json` / `effect_spec.json` `preview_witness.honest_gate`, mirrored in `artist_vfx_pipeline_live.json`.

**Search:** `effect_id` · `display_name` · `batch_id` · `lane` — 200ms debounce · Esc clears.

**Batch tree:** `All` · per `batch_id` · `Staging` (under `assets/staging/` not yet promoted) · `Recent` (12 ids).

### 3.5 Preview ladder (honest_gate)

Reuse [`design_aps_preview_ladder_v1.md`](design_aps_preview_ladder_v1.md) **levels**, remapped for effects:

| Level | Artist sees | Unlock |
|:---|:---|:---|
| **P0** | Footprint / spawn-hook token (tile grid + hook kind badge) | always |
| **P1** | Staging thumb or placeholder card (lane icon + seed) | always |
| **P2** | Spec strip — duration · rate · lod bands (read-only) | always |
| **P3** | Capture frame strip (`frames_captured` ≥ 1) | `honest_gate != dishonest_gate` |
| **P4** | Ship chip — `development_tier: production` + `honest` | `honest_gate == honest` |

**Locked teaser:** `Preview unlocks when honest_gate is honest — run effect preview / promote capture.`

**Rules:**

- Select → debounce **300ms** (smoothness charter).
- Loading: `⟳ Loading preview…` in pane.
- Stale job cancel on reselect.
- **Never** spawn Bevy fire particles inside APS for preview — thumbs/captures only.
- `dishonest_gate` shows hatch + ⊘ on P3/P4 chips (shape, not color alone).

### 3.6 Assign → scenario markers

**Assign target** = scenario marker, not Assembly slot.

```text
○ idle → select effect → ★ selected
  → [Assign to marker…] → ◐ marker picker
      → commit → writes trigger + optional scenario step
  ═[honest_gate ≠ honest]▶ ⊘ blocked
```

| Field | Spec |
|:---|:---|
| Marker id | default = `effect_id` (editable) |
| Cells | tile list editor (chunk_x, chunk_y, cell, spark) — seed from spawn_hook when `world_xy` |
| Output | `assets/scenarios/triggers/{marker_id}.trigger.ron` |
| Optional | append `TriggerEffect(effect_id: "…")` to selected scenario RON |
| Callback | `on_assign_effect(effect_id, marker_id)` — panel logs + status strip |

**Blocked copy:** `⊘ Assign blocked — preview honesty is {pending|dishonest}. Capture frames before binding to a scenario.`

**Success copy:** `✓ Assigned {display_name} → trigger {marker_id}. Scenario will TriggerEffect this id — game draw uses existing fire_vfx spine.`

### 3.7 Explicit states (form B)

```text
○idle ─hover▶ ◐hover ─click▶ ★selected ─Assign▶ ⧗assigning ─commit▶ ★assigned
                              │
                     ═[¬honest]▶ ⊘blocked (hatch + word)
                     ═[loading]▶ ⟳loading
Esc / clear → ○idle
```

Every control must expose: idle · hover · selected · blocked · loading · assigned.

---

## 4. Visual hierarchy Δ

1. **Batch tree** — navigation
2. **Effect list** — primary selection
3. **Preview ladder + honesty badge** — truth surface
4. **Assign strip** — commit action (secondary until honesty green)

⛔ Do not put Assign above honesty. ⛔ Do not chrome-compete with Materials tab (shared theme tokens only).

---

## 5. A11y

| Rule | Spec |
|:---|:---|
| Honesty | glyph + word (`✓`/`◐`/`⊘`/`○`) — never hue alone |
| Contrast | `COLOR_TEXT_*` from `aps_theme` |
| Keyboard | ↑↓ list · Enter select · Esc clear search · Tab to Assign |
| Reduced noise | no animated particle preview loops in panel |
| Tooltips | `eff_browse_*` keys in `aps_tooltips.py` |

---

## 6. Viewport / multiview impact

**None.** APS Tk panel only — no Bevy camera, no overlay, no ViewManager touch.

---

## 7. Required engine / tool hooks (`@coder-mcp`)

| Hook | Path / API |
|:---|:---|
| List registry | scan `assets/effects/registry/*/effect_spec.json` (+ staging) |
| Honesty read | `preview_witness.honest_gate` from spec/manifest / pipeline witness |
| Validate | `validate_report effect_spec` before assign commit (optional soft warn) |
| Write trigger | RON writer for `assets/scenarios/triggers/{id}.trigger.ron` |
| Optional scenario patch | append `TriggerEffect` step |
| Mount factory | `effect_browser.mount_effect_library` |
| Tab wire | `app.py` buildings notebook + `domain_router` |
| Tooltips | `aps_tooltips.py` keys below |
| Tests | `tools/mcp/python/tests/test_effect_browser.py` |
| Witness refresh | `artist_vfx_pipeline_live.json` → `aps_panel: shipped` when green |

**Tooltip keys (add):**

| Key | Copy |
|:---|:---|
| `tab_effects` | Browse EffectSpecs, check preview honesty, assign to scenario markers. |
| `eff_honesty_pending` | No preview frames yet — assign stays blocked. |
| `eff_honesty_honest` | Preview capture passed — safe to assign. |
| `eff_honesty_dishonest` | Preview witness is dishonest — do not assign or promote. |
| `eff_assign_marker` | Write a scenario trigger RON bound to this effect_id. |
| `eff_preview_ladder` | Preview levels unlock with honest_gate — not a live game particle view. |

---

## 8. Diagnostics required

| Artifact | When |
|:---|:---|
| `debug_runs/aps_effect_browser_live.json` | panel mount + list ≥3 reference effects |
| Refresh `debug_runs/artist_vfx_pipeline_live.json` | `pending.aps_panel` → shipped path + pytest note |
| WIT-HON on both | before Q✓ |

---

## 9. Risks / tradeoffs

| Risk | Mitigation |
|:---|:---|
| Artists expect live particles in APS | Explicit intro + tooltip: thumbs only |
| Assign while `pending` | Hard-block Assign button |
| Confusion with Materials assign | Different target noun: **marker**, not **slot** |
| Scope creep into sim HUD | Fence §0 — refuse chrome / in-game work |
| G4 capture still stub | Panel surfaces `pending` honestly; P4 locked until frames exist |

---

## 10. Acceptance

| # | Check |
|:---:|:---|
| E1 | Effects tab lists spark_shower · smoke_column · rain_streaks |
| E2 | Honesty glyph+word matches witness `honest_gate` |
| E3 | Assign disabled when ≠ `honest` |
| E4 | Assign writes trigger RON under `assets/scenarios/triggers/` |
| E5 | No import of fire_vfx / Bevy particle emit from panel code |
| E6 | Preview ladder P4 hidden until `honest` |
| E7 | Pytest `test_effect_browser.py` green |
| E8 | `artist_vfx_pipeline_live.json` refreshed (`aps_panel` not pending) |

---

## 11. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-09-24 |

**Residual for `@coder-mcp`:** implement packet · wire tab · tests · witness refresh.  
**Residual (shared):** G4 preview worker → `frames_captured` + `honest_gate: honest` (unlocks Assign + P4).
