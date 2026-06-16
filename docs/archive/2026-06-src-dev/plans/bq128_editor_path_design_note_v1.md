# BQ-128 editor path — designer design note `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **UX-E02-BQ128-001** |
| **Planner plan** | [`bq128_editor_path_plan_v1.md`](bq128_editor_path_plan_v1.md) (**PLAN-UX-BQ128-001**) |
| **Backlog** | **BQ-128** — Blueprint preset Wave S path |
| **Lane** | **UX-E02** · Phase E product ([`post_stage6_active_todos.md`](post_stage6_active_todos.md)) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@designer` |
| **Status** | **SIGNED** |
| **Depends on** | **None** (may run **parallel** with other designer lanes) |
| **Blocked by** | — |
| **Authority** | [`experience_layer_ux_hud_designer_brief_v1.md`](../docs/archive/2026-06-prompts-guides/runbooks/guides/experience_layer_ux_hud_designer_brief_v1.md) §4 · [`rulebook_backlog_designer_brief_v1.md`](../docs/archive/2026-06-prompts-guides/runbooks/guides/rulebook_backlog_designer_brief_v1.md) **BQ-128** |
| **Sim invariants** | [`construction_invariants.md`](construction_invariants.md) — preview/queue never bypass execute |
| **Wave S IO** | [`wave_s_open.md`](wave_s_open.md) · [`src/io/save/wave_s_artifacts.rs`](../io/save/wave_s_artifacts.rs) |
| **Witness** | `debug_runs/wave_s_blueprint_roundtrip.json` · `debug_runs/wave_s_hydrate_live.json` |

---

## Executive summary

**BQ-128** defines the **editor-only** path for blueprint preset interchange: where presets live on disk, how the running app **imports** them into the construction planner UI, and how **capture** writes them back beside `manifest.ron` — **without** granting the shell a second commit authority.

**MVP (2026-05-25):** Round-trip RON + **Pending blueprints** panel import/export + bundle hydrate (**WS-A01…A04**). **BQ-128-APPLY-001 (2026-05-25):** **Apply ghost** per imported preset → `ConstructionQueueIntent::ApplyImportedPreset` (ghost only; Enter to commit).

---

## On-disk contract (locked)

| Artifact | Relative path | DTO | Schema |
|:---|:---|:---|:---:|
| Blueprint presets | `{save_bundle}/blueprints/presets.ron` | `BlueprintPresetCollectionR8` | **1** |
| Shell pointer | `product_shell.ron` → `blueprint_preset_ref` | optional `"blueprints/presets.ron"` | — |

**Constants:** `WAVE_S_BLUEPRINT_PRESETS_REL_PATH` in `wave_s_artifacts.rs`.

**Fixture proof:** [`debug_runs/wave_s_blueprint_roundtrip.json`](../debug_runs/wave_s_blueprint_roundtrip.json) — `roundtrip_ok: true`, `preset_count: 1`.

**Serialization default:** **RON** (engine-owned). JSON only for external tooling if explicitly added later — not MVP.

---

## Authority boundaries (non-negotiable)

| Rule | Rationale |
|:---|:---|
| Preset files are **data** only | No gameplay mutation on load |
| **Import** copies RON into panel buffer / future queue rows | Same as manual queue — still needs **Approve** + **Enter/confirm** |
| **Export** serializes `PendingConstructionQueue` | `blueprint_collection_from_pending` — offline authoring aid |
| **Capture Wave S** writes bundle artifacts | User gesture in command shell; uses `WorldSaveBundleSettings.bundle_dir` |
| **No** instant-build from preset click | CSTR-2 approval path ([`experience_layer_ux_hud_designer_brief_v1.md`](../docs/archive/2026-06-prompts-guides/runbooks/guides/experience_layer_ux_hud_designer_brief_v1.md) §4) |
| Construction logic stays in `src/construction/` | Panel emits `ConstructionQueueIntent` only |

---

## Session matrix — where BQ-128 applies

| Session | Blueprint presets | Primary chrome |
|:---|:---|:---|
| **Simulation** (`BaseState::Simulation`) | **YES** — primary editor path | Left **Construction** toolbox + **Pending blueprints** floating shell (`HudWidgetId::ConstructionQueue`) |
| **WorldGen / preview** | **NO** — out of scope | World Preview / generator sheet — no construction queue |
| **Map editor** (transport/strategic) | **Deferred** — separate book | Corridor/construction book in snapshot — not `presets.ron` |

**Designer verdict:** BQ-128 is a **sim-session construction tooling** path, not World Preview (D-WP) or Stage 5 spine proof.

---

## Editor path — primary (MVP landed)

```text
[Save bundle on disk]
  manifest.ron
  product_shell.ron  (optional blueprint_preset_ref)
  blueprints/presets.ron
        │
        │  WAVE_S_AUTOLOAD_SHELL=1  OR  "Restore layout from save bundle"
        ▼
[hydrate_wave_s_artifacts_from_bundle]
        │
        ├──► HudLayoutStore + dock (BQ-130 sibling)
        └──► WaveSImportedBlueprints.collection
                    │
                    ▼
[Simulation] Construction toolbox → active build tool → map ghost
                    │
                    ▼
[Pending blueprints] panel (product shell window)
  · Queue rows · Approve / Clear intents
  · "Export presets (RON)"  → in-panel monospace buffer
  · "Import Wave S presets (N)"  → copies bundle collection into buffer
  · Enter / Shift+Enter → existing commit funnel (unchanged)
```

**Code anchors:**

| Step | Module |
|:---|:---|
| Hydrate | `io/save/wave_s_artifacts.rs` |
| Panel UI | `construction/pending_construction_panel.rs` |
| Dock / capture buttons | `gui/hud/dock_shell.rs` |
| DTO | `construction/blueprint_preset.rs` |

---

## Editor path — capture (write-back)

```text
[Command shell tray] → "Capture layout to Wave S DTO"
        │
        ▼
WaveSShellCapturePending.requested
        │
        ├── product_shell.ron  (layout + blueprint_preset_ref if queue non-empty)
        └── blueprints/presets.ron  (when PendingConstructionQueue has entries)
```

**User feedback:** last written path / error on shell tray (`dock_shell.rs`).

**Designer rule:** Capture is **explicit** — never silent autosave of blueprints on every queue edit.

---

## Editor path — phase 2 (coder — unblocked by this sign-off)

| ID | UX | Coder scope | Done when |
|:---|:---|:---|:---|
| **BQ-128-APPLY-001** | Preset **picker** row under Import | ☑ **done** — panel **Apply ghost** per preset; `apply_blueprint_preset_to_build_ghost` |
| **BQ-128-APPLY-002** | Merge vs replace | Radio: **Append queue** vs **Replace queue** on import | No silent wipe without confirm |
| **BQ-128-EXT-001** | External editor (optional) | Document path: edit `presets.ron` offline → Restore bundle | Out of egui scope |

**Out of scope for BQ-128:** Map-editor corridor phases, Python asset editor, blueprint approval AI, BQ-132 throughput hints (already stubbed in panel).

---

## Inputs & bindings (reference)

| Input | Action | Authority |
|:---|:---|:---|
| Shift+LMB (building tools) | Queue ghost as pending row | `build_queue_blueprint_on_shift_click_system` |
| Alt+drag (zones) | Paint pending tiles | Zone module |
| Backspace | Clear pending queue | `clear_pending_blueprints` |
| Enter | Commit valid ghost | Confirm funnel |
| Shift+Enter | Approve all + commit | Panel intents |
| Panel buttons | Approve / Clear / Export / Import | `ConstructionQueueIntent` only |

---

## Proof & verification

| Check | Command / artifact |
|:---|:---|
| Lib round-trip | `cargo test -p proc_A_dine01 --lib wave_s` |
| Blueprint fixture | `debug_runs/wave_s_blueprint_roundtrip.json` |
| Hydrate witness | `debug_runs/wave_s_hydrate_live.json` — `blueprint_count` when bundle has presets |
| Manual | Sim → Restore bundle → Pending panel → **Import Wave S presets (N)** → RON visible |

---

## §11 Designer sign-off

| # | Item | Done |
|:---|:---|:---:|
| 1 | On-disk path `blueprints/presets.ron` locked | ☑ |
| 2 | Sim-only primary editor path documented | ☑ |
| 3 | Import/export/capture flows match landed code (WS-A03) | ☑ |
| 4 | No sim authority violation in design | ☑ |
| 5 | Phase-2 picker scope listed for `@coder` | ☑ |
| 6 | WorldGen / map editor explicitly out of scope | ☑ |

**Verdict:** ☑ **SIGNED**

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-25 | **SIGNED** — unblocks **BQ-128-APPLY-001** coder |

---

## Unblocks

| Consumer | Notes |
|:---|:---|
| **@coder** **BQ-128-APPLY-001** | ☑ **done** — **Apply ghost** in pending panel |
| **UX-E02** board row | Mark done in `post_stage6_active_todos.md` |
| **Wave S** follow-on | WS-A03 import is **not** a substitute for picker — picker is phase 2 |

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | **UX-E02-BQ128-001** — editor path **SIGNED** |
