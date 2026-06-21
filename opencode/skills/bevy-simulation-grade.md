---
name: bevy-simulation-grade
description: >-
  Simulation-grade Bevy ECS guardrails for src/ work — single authority per
  resource, explicit SystemSets, immutable frame state, and hard Sim→View→Render→UI
  layer boundaries (never inverted). Use before editing schedules, system ordering,
  viewport/camera/render extraction, ECS resources, or view authority. Triggers:
  Bevy, ECS, SystemSet, schedule, ViewManager, viewport, render extraction, authority,
  par_iter, dual writer, fire overlay, projection graph.
metadata:
  bevy_version: "0.18"
---

`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md`

# bevy-simulation-grade — ECS authority guardrails

## The pattern (transferable)

Determinism over years of sim = **ownership discipline** ¬cleverness. Hard layer boundaries — never invert:

```text
⊚Sim ═▶ ⊚View ═▶ ⊚Render ═▶ ⊚UI         ⛔▶ inverted (UI ⛔▶ Sim · Render ⛔▶ Sim-during-extract)
  ⊚Sim    owns sim state ¬reads UI
  ⊚View   projects sim→views · ONE ViewManager writer ¬N rebuilds/frame
  ⊚Render reads snapshots ¬writes sim during extraction
  ⊚UI     visualizes ¬commits sim/camera authority
```

| # | Rule | Invariant |
|:--|:--|:--|
| 1 | single-authority 🏛 | exactly 1 system writes a resource/surface · 2nd writer = #1 drift source 🔴 |
| 2 | explicit SystemSets | order via named sets ¬ad-hoc `.after("string")` ⇒ real dependency graph |
| 3 | immutable-frame | build snapshots/registries/derived per frame ⊰ authority · downstream reads ¬mutates upstream truth |
| 4 | layer boundaries ⛓ | `Sim ═▶ View ═▶ Render ═▶ UI` · never inverted |
| 5 | parallel-safety | `par_iter_mut` over disjoint data only · ¬shared-resource writes in parallel bodies |

```text
🚦 before any schedule/authority edit ⊨ ⦃ name the single writer ║ place system in correct set ║ confirm cross-layer read/write boundary ⦄
```

## In this repo (Bevy 0.18) — authority map FIRST

```text
◎.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md   read BEFORE scheduling work
   + 08-bevy-018-guardrails.md (0.18 API)   + 00–06 (conceptual spine)
   + 09-sim-map-projection-placement.md (sim-map pick/ghost: visible_w/h ¬fixed_w/h; pick after ApplyCameraScissor)
```

| SystemSet | File | Role |
|:--|:--|:--|
| `ViewRepresentationSystemSet` | `src/gui/view_representation.rs` | UI collect → viewport → camera sync → render targets → world render → post-FX → overlay |
| `ViewportPipelineSet::Resolve` | `src/render/viewport_pipeline.rs` | writes **`ResolvedViewports`** |
| `MapCameraSystemSet` | `src/gui/map_camera.rs` | input → derive → smooth; **before** `SyncViewManager` |
| `ViewAuthoritySystemSet` | `src/gui/view_authority.rs` | `RegisterViewCameras` → **`SyncViewManager`** |
| `FireVisualFrameSet` | `src/render/extraction/fire_visual_extract.rs` | sim scan → clusters → GPU project; **after** view bridge |

Single writers (production):

```text
ViewManager ◂⊳ sync_view_manager_bridge   ·   ResolvedViewports ◂⊳ ViewportPipelineSet::Resolve
ViewProjectionAuthority ◂⊳ commit_pose     ·   ViewRepresentationSnapshot ◂⊳ build_view_representation_snapshot
```

Prefer these anchors (never ad-hoc system-name strings):

```rust
.after(ViewAuthoritySystemSet::SyncViewManager)   // after view bridge + snapshot
.after(FireVisualFrameSet::BuildProfiles)          // after fire profiles built
.in_set(ViewportPipelineSet::Resolve)              // inside viewport resolve
```

Migration tags to honor in code comments: **VM-06** (ViewManager sole writer) · **VM-09 / TRIAGE-VM-09-v2** (input commits `ViewProjectionAuthority` first).

## Verify after editing (form A gate)

```bash
node .claude/skills/agent-lang/driver.mjs validate-report cargo --cached --compress 4
```

```text
▢edit ─⬡[validate-report cargo --cached --compress 4 : 🟢]▶ ─⬡[API-sensitive(0.18) ▶ validate-report bevy -p <package>]▶ ★done
  reason on report ¬raw cargo → pairs-with [validation-first](../validation-first/SKILL.md)
  Δ dual-writer ∨ layer-cross ⟶ 🔴 stop ⤴ [debug-intelligence](../debug-intelligence/SKILL.md)
```

## Gotchas

```text
⚠ .after(some_system_fn) bare names   drift as systems move ▶ use SystemSet names from the table
⚠ sync_view_manager_world_main_from_authority   test-only · ¬scheduled in production ¬the bridge writer
⚠ diagnostics   ViewIsolationDiagnostics · ViewportPresentationMismatch · debug_runs/stage5_full_app_live.json (Stage 5 readiness)
```

## Source

```text
◎.cursor/skills/bevy-simulation-grade/   Cursor original (07 authority map = canonical scheduling reference)
```

```text
⟦/bevy-simulation-grade⟧ NEXT ⚑ read 07-authority-map → name single writer → .in_set/.after named SystemSet → validate-report cargo ★
```
