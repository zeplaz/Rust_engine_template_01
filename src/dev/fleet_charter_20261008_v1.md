# Fleet charter — 2026-10-08 (v1)

Eleven parallel Claude Code sessions ("fleets") work in the **single canonical checkout**
`C:\dev\github\Rust_engine_template_01` on **`master`** (see `single_worktree_policy_v1.md`).
No worktrees, no side branches. Baseline checkpoint: `374b0494`.

## Fleets and file ownership

A fleet may freely edit files it **owns**. Anything else: read freely, but edit only if the
owning fleet has no uncommitted change in that file (`git status --porcelain -- <path>`), keep
the edit minimal, and say so in the commit message. Cross-fleet asks go in
`src/dev/fleet_board_20261008_v1.md` (append-only, one line per ask: `[from→to] ask`).

| # | Fleet tag | Mission | Owns (primary) |
|---|-----------|---------|----------------|
| 1 | `tooling` | Dev tooling, editors, studios/suites (APS, map/world editor, MCP CLI, orchestrator scripts) | `tools/`, `src/gui/editor/`, `src/bin/`, `.claude/skills/`, `.cursor/` |
| 2 | `effects` | VFX / fire / particles / weather / sim-effect spine | `src/render/fire_vfx/`, `src/render/fx_spine/`, `src/render/particle_domains/`, `src/render/weather_vfx/`, `src/sim/effects/`, `src/systems/fire/`, `assets/shaders/fire/` |
| 3 | `logistics` | Logistics: continuous supply lines vs one-off multi-trip orders, draw demands | `src/economy/`, `src/strategic/logistics_net.rs`, `src/strategic/transport_bridge.rs`, `src/strategic/network_flow.rs`, `src/systems/transport/`, `src/gui/logistics_*` |
| 4 | `ai` | Strategic / tactical AI, behavior brain, macro AI | `src/ai/`, `src/strategic/behavior_*`, `src/strategic/hybrid_*`, `src/strategic/mission_*`, `src/strategic/agent_batch_scoring.rs`, `src/strategic/ai_explainability.rs`, `src/systems/agents/` |
| 5 | `graphics` | Graphical overhaul: blended 2.5D / 3D models on the iso world | `src/render/` (except effects-owned dirs), `src/gui/map_view/`, `src/gui/view_*`, `src/gui/viewport_*`, `assets/shaders/` (except fire) |
| 6 | `interaction` | Adversarial test + fix of mouse, menus, icon & building world placement | `src/gui/input_*`, `src/gui/hud/`, `src/gui/*menu*`, `src/gui/strategic_icon_instances.rs`, `src/gui/tactical/`, `src/construction/` input/ghost/snap/place files |
| 7 | `front` | Frontlines, theaters, pressure points, deployment, fortifications, faction symbols, balance, combat | `src/strategic/frontline.rs`, `src/strategic/zones.rs`, `src/strategic/faction_plugin.rs`, `src/strategic/fracture_*`, `src/systems/damage/`, `src/systems/collision/`, new `src/combat/` |
| 8 | `drain` | Drain existing work queues in `tools/orchestrator/queues/` | queue/HANDOFF rows it claims (code edits follow the ownership table) |
| 9 | `legacy` | Find old/half-wired systems, old config & loose files; bring them correctly into the game or retire them | `assets/config/`, repo-root loose files, `src/infrastructure/`, `src/bevysubengines/` |
| 10 | `tests` | Build health first, then test cleanup / consolidation | `src/**/tests.rs`, `src/**/integration_tests.rs`, `*_live_proof.rs`, `src/dev/*.rs`, CI config |
| 11 | `hygiene` | File management, docs sprawl, code consistency, clippy, reliability | `src/dev/*.md`, `docs/`, `debug_runs/`, `prompts/`, `src/utils/` |

Ambiguous file → the fleet whose mission it serves most directly; if still unclear, ask on the board.

## Checkpoint protocol (mandatory)

1. **One checkpoint commit per completed slice.** Small and frequent beats big and late.
2. **Commit only your own paths:** new files need `git add <path>` first; then `git commit -m "<msg>" -- <path> <path> …` (pathspec commit).
   **Never** `git add -A`, `git add .`, `git commit -a`, `git stash`, `git reset --hard`,
   `git checkout -- <others' files>`, rebase, or push.
3. Message format: `[<fleet-tag>] <imperative summary>` + body (what/why, verification run) +
   trailer `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
4. **Gate:** code commits require `cargo check --lib` green (plus targeted tests for touched
   modules). Doc-only / data-only commits may skip cargo. If the build is red because of
   *another* fleet's file, do not fix it — note it on the board and wait/retry; fleet `tests`
   owns build health.
5. `index.lock` exists → another fleet is committing; wait a few seconds and retry. Never delete it
   unless it is older than 10 minutes.
6. Keep the tree compiling between commits — stage multi-file refactors so every intermediate
   save still builds, or land them quickly.
7. Large deletes: classify first (`cleanup-completion-intelligence` skill / `cleanup-intelligence`
   agent); prefer completion over destruction.
8. Write a short progress log at `src/dev/fleet_logs/<fleet-tag>.md` (append per checkpoint:
   date, commit sha, what landed, what's next) and commit it with the slice.

## Shared constraints

- Follow `AGENTS.md`: attach to authoritative contracts (`RepresentationResult`,
  `FireVisualFrame`, projection graph, registry upload); no parallel extraction or duplicate LOD;
  scaffolds need a `ScaffoldContract`; construction invariants in `construction_invariants.md`.
- Bevy **0.19**; RON for engine-owned serde assets.
- Cargo shares one `target/` — builds serialize on the cargo lock. Prefer `cargo check --lib` and
  filtered `cargo test -p proc_A_dine01 --lib <module>` over full test runs.
- Metric units in all new data/docs.
