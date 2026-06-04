# Agent / authoring notes

## Single canonical worktree (required)

**Primary (only):** `C:\dev\github\Rust_engine_template_01` · branch **`master`**.  
All work merges **into master** — never treat `ds5i` as the integration target. Retire `C:\Users\oz_\.cursor\worktrees\Rust_engine_template_01\ds5i`.  
Policy: [`src/dev/single_worktree_policy_v1.md`](src/dev/single_worktree_policy_v1.md). Gate record: [`src/dev/steward_w3_gate_v1.md`](src/dev/steward_w3_gate_v1.md).

---

## Stage 5 convergence (primary lane)

Visual / world representation work follows **convergent growth**, not a feature freeze. New systems must **attach to authoritative contracts** (`RepresentationResult`, `FireVisualFrame`, `SharedOverlayFieldBuffers`, projection graph, registry GPU upload) and must **not** introduce parallel extraction or duplicate LOD. Transitional scaffolds require a declared `ScaffoldContract` (`src/gui/representation_governance.rs`).

**Stage 5 operational gate: CLOSED** (2026-05-23). See [`src/dev/stage5_operational_signoff.md`](src/dev/stage5_operational_signoff.md). **Stage 5.5 tracks:** **DONE** — [`src/dev/stage5_5_active_todos.md`](src/dev/stage5_5_active_todos.md). **Stage 6 operational gate: CLOSED** (2026-05-23). See [`src/dev/stage6_operational_signoff.md`](src/dev/stage6_operational_signoff.md). **Wave S save spine:** [`src/dev/wave_s_open.md`](src/dev/wave_s_open.md) (S6-S1/S6-S3). **Active lane:** [`src/dev/post_stage6_active_todos.md`](src/dev/post_stage6_active_todos.md) · plan [`src/dev/post_stage6_design_plan.md`](src/dev/post_stage6_design_plan.md). **Stage tracks (7 lanes):** [`src/dev/stage_tracks_execution_index_v1.md`](src/dev/stage_tracks_execution_index_v1.md). **Closed:** Stage 6 [`src/dev/stage6_operational_signoff.md`](src/dev/stage6_operational_signoff.md). **Deferred:** [`src/dev/stage5_triage_backlog.md`](src/dev/stage5_triage_backlog.md). **Mission-critical checklist (historical):** [`src/dev/stage5_close_checklist.md`](src/dev/stage5_close_checklist.md).

**Regression (post-close):** `cargo test -p proc_A_dine01 --lib stage5` — if FULL_APP fails, fix spine only; defer infra to triage / Stage 5.5. Full rules: [`prompts/guides/stage5_convergence_directive_v1.md`](prompts/guides/stage5_convergence_directive_v1.md).

**Live todo board:** `STAGE5_TODOS` rows move to `Done` via per-row predicates (`sync_stage5_todo_board_predicates` in `src/dev/stage5_live_todos.rs`) plus closure witnesses — not on readiness green alone. Visual `--test visual` proof JSON (`debug_runs/stage5_full_app_live.json`) includes `readiness.live_todo_board` when the board resource is present.

**Agent debug JSON:** Live proofs under `debug_runs/` include `_agent_meta` (env flags, commands, cross-links). Index: `debug_runs/agent_debug_index.json` (refreshed on each proof write). Envelope: `src/dev/debug_run_envelope.rs`; guide: `debug_runs/README.md`.

**Visual run blockers (`--test visual`):** Active terminal failures (shader panic, VT-5 flicker, compile drift) — [`src/dev/visual_run_blockers.md`](src/dev/visual_run_blockers.md); deferred depth in [`src/dev/stage5_triage_backlog.md`](src/dev/stage5_triage_backlog.md) T0/T2/T3.

**Fire ecology F1 (sim, not Stage 5 gate):** Fuel + old-growth ignition gate — [`src/dev/fire_ecology_f1_todos.md`](src/dev/fire_ecology_f1_todos.md); live witness `debug_runs/fire_ecology_live.json`.

**Development planning loop:** [`src/dev/development_plan_index.md`](src/dev/development_plan_index.md) · `cargo orchestrate --plan-slice --skip-cargo` · `tools/orchestrator/scripts/invoke_slice.ps1`

### Operational readiness ≠ infrastructure hardening

These are **different milestones** — do not collapse them.

| Lane | What it proves | Plain English |
|------|----------------|---------------|
| **Operational readiness** | FULL_APP green, spine valid, runtime stable, contracts pass | Playable, coherent, **testable**, **converged** |
| **Infrastructure hardening** | VM backlog, per-view isolation, editor parity, multiview correctness, replay | Scalability, **tooling**, **futureproofing**, **robustness** |

FULL_APP green means the **spine is authoritative and measurable** in the running app; it does **not** automatically close VM-06…VM-11 / isolation audits (see `prompts/guides/base_finsh_5.md`). Full distinction + perf playbook: [`prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md`](prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md).

## Serialization: RON vs JSON

- **Default for engine-owned serde assets** (terrain registries, transport R8, world-gen tuning, hybrid snapshot bodies, world subengine export): prefer **RON** on disk. Loaders use extension dispatch (`.ron` / `.json`; unknown extension often tries RON then JSON). Examples: `*.example.ron` beside legacy JSON where applicable.
- **JSON** is retained where **external tooling** or **human interchange** expects it (Python asset editor pages, some fixtures, explicit `.json` paths, HTTP APIs).
- **Documented in code** near loaders: `src/terrain/registry_serde_path.rs`, `src/terrain/generation/tuning_io.rs`, `src/systems/transport/persistence.rs`, `src/io/snapshot/mod.rs` (hybrid header may be JSON or **RON** line).

## Construction stage (parallel to Stage 5)

- **Not** part of `STAGE5_TODOS` / FULL_APP exit.
- **Invariants (hard rules):** [`src/dev/construction_invariants.md`](src/dev/construction_invariants.md) — preview never mutates gameplay; single execute funnel; no logic outside `src/construction/`.
- **Boards:** `BUILD-P*` + `FINISH-BUILD-*` + **Phase 2** `PHASE2-BUILD-*` + **Round 2/3** + **Operational** `CONSTRUCTION-OP-*` **done** (witness: `debug_runs/construction_stage_live.json`). Further construction work: product boards / Round 4 — not Stage 5.
- Implementation: [`src/construction/`](src/construction/). Spec: [`recovery_construction.md`](src/dev/recovery_construction.md) (Round 3 § line 1280+). Checklist: [`construction_recovery_todos.md`](src/dev/construction_recovery_todos.md). Ownership: [`construction_ownership.md`](src/dev/construction_ownership.md).
- Proof: `debug_runs/construction_stage_live.json` in sim. Do not fold construction closure into Stage 5 readiness.
- **Phase 4 (industrial activation):** after construction operational green — [`src/dev/industrial_activation_pipeline.md`](src/dev/industrial_activation_pipeline.md), bridge in `src/economy/activation/`. Not Stage 5.

## Transport R8 + construction

- `TransportNetworkSnapshot` may include a **`construction`** slice (corridor phases). **G4** load hydrates [`CorridorConstructionBook`](src/strategic/construction_book.rs) when that resource is present. Map editor **Save** embeds book rows from the live graph.

## Agent routing (three layers)

### 1. Cursor custom agents (authoritative roles) — [`.cursor/agents/`](.cursor/agents/)

Copied from user profile into the repo so the team shares one definition. Each file uses **`model: auto`** in frontmatter (agent picker or `@orchestrator` / `@planner` / `@coder` / `@designer` / **`@orchestrator-mcp` / `@planner-mcp` / `@coder-mcp` / `@designer-mcp`** / `@sim-steward`).

| Agent file | Role | Delegates to |
|------------|------|--------------|
| [`.cursor/agents/orchestrator.md`](.cursor/agents/orchestrator.md) | Sequencing only — never writes production code | `planner` first, then `coder` / `designer` |
| [`.cursor/agents/orchestrator-mcp.md`](.cursor/agents/orchestrator-mcp.md) | **MCP art pipeline** sequencing — gates spec→validate→promote | `planner-mcp` → `designer-mcp` → `coder-mcp` |
| [`.cursor/agents/planner.md`](.cursor/agents/planner.md) | Architecture plans, phases, authority map (readonly) | — |
| [`.cursor/agents/planner-mcp.md`](.cursor/agents/planner-mcp.md) | MCP toolchain architecture — schemas, tool categories, batch rollout | — |
| [`.cursor/agents/coder.md`](.cursor/agents/coder.md) | ECS, render, viewport, logistics, diagnostics `src/` | — |
| [`.cursor/agents/coder-mcp.md`](.cursor/agents/coder-mcp.md) | `tools/mcp/` Python/CLI, bpy ops, validators | — |
| [`.cursor/agents/designer.md`](.cursor/agents/designer.md) | HUD, overlays, multiview UX, ghosts (presentation) | — |
| [`.cursor/agents/designer-mcp.md`](.cursor/agents/designer-mcp.md) | MCP art pipeline — AssetSpec, quality gates; **critiques orders**, no shortcuts | MCP skills · `@coder-mcp` for tool execution |
| [`.cursor/agents/sim-steward.md`](.cursor/agents/sim-steward.md) | **Simulation steward** — bevy-simulation-grade + debug-intelligence + cleanup-completion-intelligence; **sequential shifts A→B→C** in main chat when Task quota blocked | `coder` / `planner` / `designer` for out-of-scope slices |
| [`.cursor/agents/main-thread-orchestrator.md`](.cursor/agents/main-thread-orchestrator.md) | **Main-thread continuity** — Task attempt + fail-cycle escalation + foreground queue when Task/debug/cleanup fail; never stop on usage errors | Runs Shift A→B→C inline or via `@sim-steward` |
| [`.cursor/agents/coparent-orchestrator.md`](.cursor/agents/coparent-orchestrator.md) | **Secondary pathways** — parallel lanes (operator, VFX capture, designer tails, parametric placement); conflict matrix vs primary P1 | Promotes slices to `@orchestrator`; routes drift to `@sim-steward` |

**Handoff chain (orchestrator.md):** `orchestrator` → **`planner`** (plan) → **`coder`** / **`designer`** (implement) → verification (`cargo check` / tests / witness JSON).

**MCP art pipeline (separate lane):** `orchestrator-mcp` → `designer-mcp` (spec + run jobs) → `coder-mcp` (toolchain) → validate → promote → **`coder`** (Bevy registry). **Consumers** (`coder`, `designer`, `planner`) **use** MCP via CLI/validation-first — they **do not** build `tools/mcp/`. Guide: [`src/dev/agent_mcp_consumer_guide_v1.md`](src/dev/agent_mcp_consumer_guide_v1.md). Skills: `.cursor/skills/mcp-asset-pipeline`, `mcp-production-rules`, `validation-first`.

**Construction + growth product lane:** [`src/dev/construction_economy_growth_vision_v1.md`](src/dev/construction_economy_growth_vision_v1.md) · index [`src/dev/construction_procedural_growth_index_v1.md`](src/dev/construction_procedural_growth_index_v1.md) · fleet prompts [`src/dev/fleet_longrun_prompts_20260602_v1.md`](src/dev/fleet_longrun_prompts_20260602_v1.md).

**MCP art lane (orchestrator-mcp.md):** `orchestrator-mcp` → **`planner-mcp`** (if architecture) → **`designer-mcp`** (critique + spec + sign-off) → **`coder-mcp`** (toolchain) → validate → promote → registry.

**Iso tile bake spine (mandatory for ship art):** [`src/dev/design_tile_bake_spine_convergence_v1.md`](src/dev/design_tile_bake_spine_convergence_v1.md) — production = `Light_keysshotsetup.blend` + `utils/keyframe_render.py` → `tile-atlas-pack` (tilemapgen). **`tile_ortho_bake` / lod0 pilot atlases are CI/smoke only** — not building production templates. Agents/skills: `tile-generation`, `mcp-production-rules` (`bake_source: keyframe_pack` when `ship: true`).

**`@coparent-orchestrator`** runs parallel secondary lanes without preempting primary P1.

### Subagent continuity (mission-critical)

Task background workers can fail with *“Switch to Auto”* — that blocks **Task quota only**, not the repo. Full playbook: [`prompts/guides/subagent_continuity_playbook_v1.md`](prompts/guides/subagent_continuity_playbook_v1.md).

| Channel | When to use |
|---------|-------------|
| **Main chat (Auto)** | Default implementation after any Task failure |
| **`@coder` / `@planner` / `@sim-steward` / `@main-thread-orchestrator` / `@coparent-orchestrator` in chat** | Same roles as Task; `.cursor/agents/*` use `model: auto` (different meter than Task). **`@sim-steward`** = shifts A→B→C; **`@main-thread-orchestrator`** = Task fail-cycle + slice queue; **`@coparent-orchestrator`** = secondary parallel pathways |
| **Task tool** | **Separate subagent quota** — when exhausted, **do not retry** (incl. `composer-2.5-fast`); use Auto / `@coder` |
| **`HANDOFF.md`** | Session handoff — template: [`tools/orchestrator/queues/HANDOFF.template.md`](tools/orchestrator/queues/HANDOFF.template.md); script: `tools/orchestrator/invoke_handoff.ps1` |
| **SDK `@cursor/sdk`** | Local `Agent.prompt` + `composer-2` when IDE Task pool is exhausted |

**Parent rule:** Task `status: error` (usage) → **implement in foreground same turn**; do not stop after reporting the error.

| Do this | Not this |
|---------|----------|
| Foreground fix + `cargo test` | Retry Task (any model) after usage errors |
| `@coder` + playbook path + 3 file paths | Empty “continue the plan” delegations |
| `invoke_handoff.ps1 -Goal … -Lane …` before leaving a lane | Lose context across sessions |

Multitask mode **requires** Task — if subagent quota is dry, **turn off Multitask** and work in a normal agent chat.

### What to adjust so “agents” work

| Adjustment | Why |
|------------|-----|
| **Disable Multitask mode** for implementation sessions | Stops the parent from *only* spawning Task workers that hit the empty subagent pool |
| **Use `@coder` / `@planner` in chat** (not “run subagent in background”) | Custom agents use `model: auto` → same path as **this** working Auto session |
| **Do not expect `Task(coder)` to work** until Cursor **usage** shows subagent/agent budget | Fast model name does not bypass a **zero** Task pool |
| **Orchestrator plans in chat; coder implements in chat or same thread** | Matches updated [`.cursor/agents/orchestrator.md`](.cursor/agents/orchestrator.md) |
| **Admin / plan: raise agent or subagent limits** | Only product-side way to re-enable Task background workers |
| Optional: **Cursor SDK** + API key | Separate billing from IDE Task (see playbook §7) |

**Sanity check:** In a new chat, type `@coder` and a small task (e.g. “run `cargo test -p proc_A_dine01 construction:: --lib` and report”). If that works but Multitask Task fails, your setup is correct — use chat agents, not Task.

### 2. Repo lane playbooks — `tools/orchestrator/agents/*.md`

Per-subsystem **DO NOT TOUCH**, safe edits, and exit criteria (e.g. `viewport_cleanup_agent`, `stage5_readiness_agent`). The **coder** / **designer** agents must read the matching playbook before editing that lane.

### 3. Skills + governance

- **bevy-simulation-grade** (personal skill, `~/.cursor/skills/`) — attach for **coder** work on ECS/view/render.
- **debug-intelligence** (project skill, [`.cursor/skills/debug-intelligence/`](.cursor/skills/debug-intelligence/SKILL.md)) — witness JSON, viewport/render drift, VM-* migration debt; compresses evidence and routes to `@planner` / `@coder` / `@designer` (does not implement fixes).
- **cleanup-completion-intelligence** (project skill, [`.cursor/skills/cleanup-completion-intelligence/`](.cursor/skills/cleanup-completion-intelligence/SKILL.md)) — before deleting or consolidating modules; classifies obsolete / transitional / dormant / incomplete and prefers completion plans over destructive cleanup.
- **AGENTS.md** + [`prompts/guides/stage5_convergence_directive_v1.md`](prompts/guides/stage5_convergence_directive_v1.md) — Stage 5 vs construction vs infrastructure.

Index: [`.cursor/skills/README.md`](.cursor/skills/README.md).

| Cursor agent | Repo playbook(s) | Also read |
|--------------|------------------|-----------|
| `orchestrator` | [`agent_queue.md`](tools/orchestrator/queues/agent_queue.md), all playbooks as needed | [`NEXT.md`](tools/orchestrator/NEXT.md), **debug-intelligence** for multi-domain drift; delegate drift/cleanup lanes to **`sim-steward`**; parallel lanes to **`coparent-orchestrator`** |
| `orchestrator-mcp` | MCP exec plan, [`tools/mcp/README.md`](tools/mcp/README.md) | All four MCP skills; **`designer-mcp`** gate before tool tasks |
| `coparent-orchestrator` | [`HANDOFF.md`](tools/orchestrator/queues/HANDOFF.md), machine queues | **debug-intelligence** + **cleanup-completion-intelligence** + bevy-simulation-grade conflict matrix |
| `planner` | `migration_tasks.md`, matrices | [`llm_agent_brief.md`](prompts/llm_agent_brief.md), **debug-intelligence** |
| `planner-mcp` | MCP exec plan, MCP drafts | All four MCP skills |
| `coder` | `viewport_cleanup_agent`, `render_pipeline_agent`, `stage5_readiness_agent`, … | bevy-simulation-grade, **debug-intelligence**; **cleanup-completion-intelligence** before removals |
| `coder-mcp` | [`tools/mcp/MICRO_TOOLS_REGISTRY_v1.md`](tools/mcp/MICRO_TOOLS_REGISTRY_v1.md) | All four MCP skills |
| `designer` | `ui_layout_agent` | [`ui_boundary_guide_v1.md`](prompts/guides/ui_boundary_guide_v1.md) |
| `designer-mcp` | MCP art exec plan | All four MCP skills |
| `sim-steward` | `stage5_readiness_agent`, `viewport_cleanup_agent`, `render_pipeline_agent` | All three skills (personal **bevy-simulation-grade** + project **debug-intelligence** + **cleanup-completion-intelligence**); [`subagent_continuity_playbook_v1.md`](prompts/guides/subagent_continuity_playbook_v1.md) |
| `main-thread-orchestrator` | Same as sim-steward + orchestrator continuity §10 | Fail-cycle ledger in `HANDOFF.md`; [`main-thread-orchestrator.md`](.cursor/agents/main-thread-orchestrator.md) |

**Cycles:** Stage 5 regression → `stage5_readiness_agent`; infrastructure → `viewport_cleanup_agent` + `render_pipeline_agent`; after edits → `cargo orchestrate`. See [`tools/orchestrator/NEXT.md`](tools/orchestrator/NEXT.md).

## Simulation HUD vs editor HUD (PLAY-01)

| Session | Default chrome | Editor-only |
|---------|----------------|-------------|
| **Simulation** (`BaseState::Simulation`) | Collapsed command tray; WorldGen/preview dismissed; scenario script closed; non-essential product shells hidden/minimized | Scenario script panel (`map_editor` plugin), verbose diagnostics sections |
| **Editor / WorldGen** | Full tools, preview chrome, `WorldGenUiState.visible` may drive raster | — |

Entry hook: [`src/gui/hud/simulation_session.rs`](src/gui/hud/simulation_session.rs) (`apply_simulation_hud_defaults` on `OnEnter(Simulation)`). Diagnostics: collapsed sections in sim (`diagnostics_ui.rs` + `BaseState`). World preview raster: `world_preview_chrome_active` / `world_preview_pipeline_enabled`.

**Session playback (PLAY):** done — [`src/dev/session_playback_issues_todos.md`](src/dev/session_playback_issues_todos.md).

**Post-PLAY follow-up:** [`src/dev/post_play_followup_todos.md`](src/dev/post_play_followup_todos.md) — **closed** (2026-05-22).

**Active execution list:** [`src/dev/next_action_todos.md`](src/dev/next_action_todos.md) — doc reconcile, proof refresh, perf, undo/redo, infra hardening. Handoff: [`tools/orchestrator/queues/HANDOFF.md`](tools/orchestrator/queues/HANDOFF.md).

## Build orchestrator (diagnostics pipeline)

After `cargo check` / test cycles, run **`cargo orchestrate`** (or `tools/orchestrator/hooks/post_build.ps1`). The orchestrator parses **`--message-format=json`** diagnostics, classifies warnings by **migration state** (`WarningState` + `do_not_touch`), and writes reports under [`tools/orchestrator/`](tools/orchestrator/). It preserves architectural intent — **not** auto-delete warnings. Tag in-progress code with `@orchestrator-status`, `@orchestrator-owner`, `@orchestrator-do-not-cleanup`. Optional: `$env:RUST_ENGINE_ORCHESTRATE=1` before check to chain the hook.
