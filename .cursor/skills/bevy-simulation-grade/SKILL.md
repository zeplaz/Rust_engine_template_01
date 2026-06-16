---
name: bevy-simulation-grade
description: >-
  Applies Bevy 0.18 simulation-grade ECS patterns for Rust_engine_template_01:
  repo authority map (ViewAuthoritySystemSet, ViewportPipelineSet, FireVisualFrameSet),
  deterministic schedules, viewport/view contracts, projection graph, build ghosts,
  parallel sim boundaries. Use when implementing or reviewing src/ ECS, view, render,
  extraction, or scheduling. Prefer repo code over illustrative snippets in this skill.
bevy_version: "0.18"
repo: Rust_engine_template_01
---

`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md`

# Bevy Simulation-Grade ECS

**Engine:** Bevy **0.18** (see root `Cargo.toml`). Migration: [0.17 → 0.18](https://bevy.org/learn/migration-guides/0-17-to-0-18/).

**Scope:** Patterns for **this** engine. When this skill disagrees with `src/`, **`src/` wins**.

## Core principle

> No system writes without declared ownership; no resource without a single authority.

## Hard boundaries

| Layer | May | Must not |
|-------|-----|----------|
| Simulation | Own sim state | Read UI directly |
| View | Project sim → views | Ad-hoc `ResMut<ViewManager>` outside bridge |
| Render | Read snapshots / projection | Write simulation state |
| UI | Submit viewport requests, ghosts | Commit sim or own camera truth |

## Two schedule models (do not confuse)

| Model | What it is |
|-------|------------|
| **Conceptual spine** | `Input → ViewSync → Simulation → AuthorityResolve → Extraction → RenderPrep → Cleanup` — design vocabulary in [00](00-core-ecs-execution-model.md) |
| **Implementation spine** | Real `SystemSet`s in [07-repo-authority-map.md](07-repo-authority-map.md) — **use for `.before()` / `.after()`** |

There is **no** single `CoreSystemSet` enum in `src/` today. Order via `ViewRepresentationSystemSet`, `ViewportPipelineSet`, `ViewAuthoritySystemSet`, `MapCameraSystemSet`, `FireVisualFrameSet`.

## Which file to read

| Task | File |
|------|------|
| Repo sets, writers, frame order | **[07-repo-authority-map.md](07-repo-authority-map.md)** ← start here for coding |
| Bevy 0.18 guardrails (cameras, APIs) | [08-bevy-018-guardrails.md](08-bevy-018-guardrails.md) |
| Conceptual phases + parallel sim | [00-core-ecs-execution-model.md](00-core-ecs-execution-model.md) |
| `ViewId`, `ViewManager` bridge | [01-view-authority-viewmanager.md](01-view-authority-viewmanager.md) |
| `ViewportAuthority` → `ResolvedViewports` | [02-viewport-authority-pipeline.md](02-viewport-authority-pipeline.md) |
| `ViewRepresentationSnapshot`, GPU contract | [03-map-view-projection-and-render-contract.md](03-map-view-projection-and-render-contract.md) |
| `RenderProjectionContext`, fire extract | [04-render-projection-graph.md](04-render-projection-graph.md) |
| `BuildGhostState`, construction | [05-construction-ghost-overlay.md](05-construction-ghost-overlay.md) |
| Sim map pick/ghost projection, debug probe | **[09-sim-map-projection-placement.md](09-sim-map-projection-placement.md)** |
| `par_iter_mut`, cleanup | [06-parallel-simulation-and-cleanup.md](06-parallel-simulation-and-cleanup.md) |

## AGENT-LANG ritual (attach [agent-lang](../agent-lang/SKILL.md))

**ECS projection region** — symbols survive context drift:

| Region | `$sym:` anchor |
|:---|:---|
| View authority | `$sym:ViewAuthoritySystemSet@src/gui/view_authority.rs` |
| Viewport resolve | `$sym:ViewportPipelineSet@src/render/viewport_pipeline.rs` |
| Projection graph | `$sym:RenderProjectionContext@src/render/extraction/render_projection_graph.rs` |
| Stage 5 attach | `$sym:RepresentationResult@src/gui/representation_governance.rs` |

```text
Pre-change: name $sym:Writer → place in SystemSet (07) → BLANG:BEVY → 🟢/🔴
Dual writer suspected: ⟨DRIFT⟩ + debug-intelligence ΔWF→@coder
```

**Normative:** `$ref:.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md`

## Pre-change checklist

1. Name the **single authority** resource/system that may write.
2. Place the system in the correct **repo `SystemSet`** (07), not a invented set.
3. Cross-layer: read-only vs write (sim / view / render).
4. Views: **one** `sync_view_manager_bridge` per frame — sole `ResMut<ViewManager>` writer.
5. Viewports: resolve in `ViewportPipelineSet::Resolve` → `ResolvedViewports`.
6. GPU/raster: consume **`ViewRepresentationSnapshot`** after `SyncViewManager`, not stale `ViewManager`.
7. Parallel sim: no `ResMut` on view/render authority inside `par_iter_mut` workers.
8. Stage 5: attach to `RepresentationResult` / projection graph — no parallel extraction LOD.

## Related project skills

- **agent-lang** — `$sym:` schematization, BLANG:BEVY, ⟨DRIFT⟩ re-anchor
- **debug-intelligence** — witness JSON, VM drift (`debug_runs/`)
- **validation-first** — `validate-report` after `cargo check`
- **cleanup-completion-intelligence** — before deleting view/render shims

## Key repo paths

```text
src/gui/view_authority.rs          ViewAuthoritySystemSet, ViewManager bridge
src/gui/view_representation.rs     ViewRepresentationSystemSet
src/gui/map_camera.rs              MapCameraSystemSet, MapCameraDesired mirror
src/gui/viewport_authority.rs      ViewportAuthority (requests)
src/render/viewport_pipeline.rs    ViewportPipelineSet, ResolvedViewports
src/render/extraction/             FireVisualFrameSet, projection graph
src/dev/construction_invariants.md construction preview vs commit
```
