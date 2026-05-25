# UI Phase 2 — coder execution queue `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.5.0` |
| **Date** | 2026-05-24 |
| **Active sprint** | **Phase 2** — proof / polish / product (no blocking design gates) |
| **Owner** | `@coder` / `ui_layout_agent` |
| **Index** | [`src/dev/ui_phase2_sprint_queue.md`](../../../src/dev/ui_phase2_sprint_queue.md) |
| **Mock** | [`ui_phase0_panel_mocks_v1.md`](ui_phase0_panel_mocks_v1.md) |
| **Sign-off** | [`ui_phase2_designer_signoff_v1.md`](ui_phase2_designer_signoff_v1.md) |
| **Authoritative code** | [`simulation_shell_phase2.rs`](../../../src/gui/hud/simulation_shell_phase2.rs) |
| **Witness** | `debug_runs/ui_shell_migration_live.json` |

**Goal:** Close Phase 2 UI shell **fast**, then **stop blocking** Phase E product lanes (industrial chain, GPU minimap M1, construction P9 verify).

---

## @coder — pick up here (Sprint 2C — DEFERRED)

```
Lane: UI Phase 2C — left chrome layout (P4 mock delta)
Read: prompts/guides/ui/ui_phase2_coder_queue_v1.md (§ Sprint 2C)
      prompts/guides/ui/ui_phase2_designer_signoff_v1.md (§ Phase 2C, @coder item 4)
Playbook: tools/orchestrator/agents/ui_layout_agent.md
Blocked: @designer picks 2C-A / 2C-B / 2C-C / 2C-D first — then mock § P4 update — then implement
Do NOT: implement 2C layout until designer records chosen option in ui_phase0_panel_mocks_v1.md § P4
First step (designer): pick 2C-A/B/C/D · amend mock · sign P4 row
Then (coder): implement per option table · refresh witness · optional 2C-C viewport review
Verify: cargo test -p proc_A_dine01 --lib simulation_shell_phase2
        cargo run -p proc_A_dine01 -- --test frame
        # 2C-C only: cargo run … --test capture + viewport_drift witness + @sim-steward
Exit: P4 PASS in sign-off · mock § P4 matches live chrome width budget
```

### Sprint status board

| Sprint | State | Notes |
|:---:|:---|:---|
| **1 — 2A-Tail** | **DONE** | F-10 + 1.6 witness clicks green (2026-05-24) |
| **2 — 2B-Build** | **Done** | 2B-01/02/04 + witness fields in `build_proof_payload` |
| **3 — 2B-Dedupe** | **Done** | 2B-03 gates · `enforce_simulation_product_egui_gates` · 3.3 shim optional |
| **4 — Sign-off** | **DONE** | Witness + designer **SIGNED** · `--test visual` green 2026-05-24 |
| **2C — Left chrome** | **DONE** | **2C-B** mock + witness · [`ui_phase0_panel_mocks_v1.md`](ui_phase0_panel_mocks_v1.md) § P4 |

**Tests (2026-05-24 sweep):** `simulation_shell_phase2` **7/7** · `simulation_egui_gate_witness_sync` **1/1** · `cargo check` green.

---

## Global authority rules (all sprints)

1. **Presentation-only** in 2A/2B shell — ops strip, tray, minimap chrome do not mutate sim state.
2. **Build tool authority (Sprint 2+)** — `BuildStripState` has a single write path; Bevy rail delegates to existing construction APIs.
3. **PLAY-01** — gate egui shells off in sim via `apply_simulation_hud_defaults`; do not delete editor/dev paths.
4. **No parallel HUD** — extend `SimulationShellPhase2Plugin`; no second ops strip or tray.
5. **Regression** — every sprint: `cargo test -p proc_A_dine01 --lib stage5`; Sprint 4 adds mode **②** below.

---

## Agent / coder commands

| # | Label | How to run | Behavior |
|:---:|:---|:---|:---|
| **①** | **Frame layout test** | Menu: **Debug ① — Frame layout test** · CLI: `cargo run -p proc_A_dine01 -- --test frame` | 192² world → sim, `UI_LAYOUT_DEBUG=1`, **stays open** — use for F-03 hover, F-06 peek, F-08 egui parity, 2B layout |
| **②** | **Full capture proof** | `cargo run -p proc_A_dine01 --release -- --test visual` or `--test capture` | Auto world-gen → sim → **≥90 sim frames** → proof JSON → graceful exit (**30** post-proof frame cushion) |
| **②b** | **Capture + stay open** | `cargo run -p proc_A_dine01 --release -- --test capture --stay-open` | Same harness as ②; window stays open after proof — use for **witness click pass** (Alerts / INTEL / Escape) |
| **③** | **Demo world (stay open)** | Menu: **Debug ③ — Demo world** · CLI: `cargo run -p proc_A_dine01 -- --test demo` | Auto gen → sim, **no auto-exit** — use for 2B-01 build-rail tool clicks |
| **④** | **Unittest fixture** | `cargo run -p proc_A_dine01 -- --test unittest` · optional `--unittest-world PATH` | Loads `assets/fixtures/unittest_world.ron` (deterministic params), then capture path |

**Quick copy-paste**

```powershell
# ① Layout — inspect frame, stay open (Sprint 1 material / Sprint 2 layout)
cargo run -p proc_A_dine01 -- --test frame

# ② CI / regression capture (auto exit) — Sprint 4 gate
cargo run -p proc_A_dine01 --release -- --test visual

# ② Witness clicks — capture then inspect / click
cargo run -p proc_A_dine01 --release -- --test capture --stay-open

# ③ Demo — build rail / PLAY-01 manual (Sprint 2–3)
cargo run -p proc_A_dine01 -- --test demo

# ④ Deterministic world params
cargo run -p proc_A_dine01 -- --test unittest
```

**Proof JSON refreshed by ②:** `debug_runs/stage5_full_app_live.json`, `debug_runs/ui_shell_migration_live.json` (sim frames ≥90).

---

## Sweep summary (2026-05-23)

### Landed since sign-off v1.2.0

| ID | Item | Evidence |
|:---|:---|:---|
| ✅ | **F-01** ◆ + number badge | `format_ops_strip_alert_badge` → `◆0` / `◆99+` |
| ✅ | **F-02** 2px strip top offset | `OPS_STRIP_TOP_OFFSET_PX = 2` |
| ✅ | **F-04** 48px rail icon grid | `⏱` `⛭` `◎` `☰` on `LeftContextRail` |
| ✅ | **F-07** gold bar + vellum tabs | `sync_context_tray_tab_chrome_system` + `bevy_bg_vellum()` |
| ✅ | **F-09** minimap chrome ≤2px | `last_image_rect` + `MINIMAP_CHROME_STROKE_PAD_PX = 1` |
| ✅ | Phase 2 zone expansion | Ops strip: TIME · ALERTS · INTEL · WX · PWR · **▼ TRAY** |
| ✅ | Tray peek cycle | `cycle_tray_affordance` via `OpsStripZone::TrayAffordance` |
| ✅ | **Build rail** (Bevy) | `BuildRailRoot` + `sync_build_rail_from_strip_system` (read-only highlight) |
| ✅ | Palette v2 fields in code | `bg_vellum`, `bg_paper`, `fg_data` in `palette.rs` |
| ✅ | B0001 tray visibility fix | `ParamSet` in `sync_context_tray_visibility_system` |
| ✅ | Witness `phase2` + `backends` blocks | `build_proof_payload` |

| ✅ | **F-03** hover `accent_hot` | `sync_ops_strip_zone_hover_system` (2026-05-24) |
| ✅ | **F-06** peek → expand | `on_tab_pressed` + 48px peek body (2026-05-24) |
| ✅ | **F-08** egui `flat_v2_tray_tab` | gold/vellum in `info_tabs.rs` (2026-05-24) |
| ✅ | **F-11** token doc | [`palette_v2_tokens.md`](palette_v2_tokens.md) (2026-05-24) |
| ✅ | **2B-01** build rail writes | `build_rail_tool_click_system` + toggle deselect (2026-05-24) |
| ✅ | **2B-04** stack hidden when collapsed | `sync_command_left_stack_visibility` (2026-05-24) |
| ✅ | `bevy_accent_hot()` | `palette.rs` (2026-05-24) |
| ✅ | Phase 2 lib tests | 7/7 `simulation_shell_phase2` + 1/1 `simulation_session` gate (2026-05-24) |
| ✅ | **2B-02** BuildToolbox gated in sim | `build_toolbox_egui_dock_active` + `suppress_simulation_floating_shell_slots` |
| ✅ | **2B-03** side status rail gated | `side_status_rail_egui_active` + `product_egui_shell_active` on `hud_root_tick` |
| ✅ | **2B-04** stack hidden when collapsed | `sync_command_left_stack_visibility` |
| ✅ | Phase 2B witness payload | `phase2b_closed`, gate flags in `build_proof_payload` |

### Optional / closed (non-blocking)

| ID | Gap | Notes |
|:---|:---|:---|
| ○ **P4-ART-01** | Placeholder icon atlas | **Optional** traced PNG — [`ui_phase4_icon_atlas_brief_v1.md`](ui_phase4_icon_atlas_brief_v1.md) |
| ✅ **2C** | Left chrome layout | **CLOSED** — **2C-B** mock + witness (2026-05-24) |
| ◐ **3.3** | `ui_shell_migration.rs` shim classify | **Done** — transitional **B** shim; preserve until import convergence |

---

## Sprint 2C — left chrome layout (DEFERRED)

**Goal:** Close P4 **CONDITIONAL** — reconcile mock § P4 with live `CommandLeftStackOverlay` tree.

**Workflow:** **`@designer` picks 2C-A/B/C/D → updates `ui_phase0_panel_mocks_v1.md` § P4 → `@coder` implements** (sign-off @coder item **4**).

**Blocked on:** designer layout decision (sign-off § Phase 2C).

### Authoritative code refs

| Symbol | Role | Location |
|:---|:---|:---|
| `CommandLeftStackOverlay` | Absolute overlay; does not shrink map hole | `in_game_hud.rs` spawn ~L776 |
| `LeftContextRail` | 48px context/HUD rail (`CONTEXT_RAIL_W_PX`) | `in_game_hud.rs` ~L799 · `simulation_shell_phase2.rs` |
| `BuildRailRoot` | **52px** build-tool column (Phase 2B) | `in_game_hud.rs` spawn ~L811–L821 |
| `LeftContextStackBody` | 400px narrative stack (`LEFT_CONTEXT_STACK_W_PX`) | `in_game_hud.rs` ~L873 |
| `CommandLeftStackState` | Collapse/expand authority | `in_game_hud.rs` |
| `sync_command_left_stack_visibility` | Rail/stack/build visibility matrix | `in_game_hud.rs` ~L1215 |
| `reset_simulation_map_viewport_on_left_stack_toggle` | Viewport settle latch on toggle | `in_game_hud.rs` ~L299 |
| `build_rail_tool_click_system` | Build rail → `BuildStripState` | `simulation_shell_phase2.rs` |
| `sync_build_rail_from_strip_system` | Highlight sync (read-only) | `simulation_shell_phase2.rs` |

**Width budget** (`column_gap` 6px on overlay):

| State | LeftContextRail | BuildRailRoot | LeftContextStackBody | Footprint |
|:---|:---:|:---:|:---:|:---:|
| **Collapsed** (PLAY-01 default) | 48px visible | 52px visible | hidden | **~106px** |
| **Expanded** | hidden | 52px visible | 400px visible | **~458px** |

Map hole stays full-width today — overlay occludes the left edge (`reset_simulation_map_viewport_on_left_stack_toggle` does not inset geometry).

### Layout options (`@designer` picks one)

| Option | Summary | Coder touch | Viewport |
|:---|:---|:---|:---:|
| **2C-A** | Merge build tools into **48px** P4 column | Collapse `BuildRailRoot` into `LeftContextRail`; denser icon stack | Low |
| **2C-B** | Amend mock — canonical dual column **48 + 52** (+ gap) | Doc-first; code optional width normalize | None |
| **2C-C** | Inset map hole when left chrome present | Flex row or `SimulationMapViewportFill` margins from chrome measure | **High** — `@sim-steward` |
| **2C-D** | Hide build rail when stack expanded; tools in stack footer | Toggle `BuildRailRoot` in `sync_command_left_stack_visibility` | Low |

### Execution steps

| Step | Owner | Task | Verify |
|:---:|:---:|:---|:---|
| **2C-0** | `@designer` | Record chosen **2C-A/B/C/D** in [`ui_phase0_panel_mocks_v1.md`](ui_phase0_panel_mocks_v1.md) § P4 (+ diagram if needed) | Mock diff review |
| **2C-1** | `@coder` | Implement per option table (≤3 files per step) | **①** `--test frame` |
| **2C-2** | `@coder` | Refresh `debug_runs/ui_shell_migration_live.json` if width/visibility fields added | **②b** capture |
| **2C-3** | `@designer` | P4 row **PASS** in sign-off | Panel table updated |

```powershell
# Baseline (all options)
cargo test -p proc_A_dine01 --lib simulation_shell_phase2
cargo run -p proc_A_dine01 -- --test frame

# Regression spine
cargo test -p proc_A_dine01 --lib stage5

# 2C-C only — viewport authority
cargo run -p proc_A_dine01 --release -- --test capture
# witness: debug_runs/viewport_drift.json — @sim-steward gate ≤2px after stack toggle
```

**Exit:** mock § P4 matches live chrome · chosen option implemented · designer P4 **PASS** · sign-off item **4** closed.

### Do not touch in Phase 2 (next lanes)

| Lane | Doc | Notes |
|:---|:---|:---|
| GPU minimap M1 | [`ux_gpu_minimap_design_v1.md`](../../../src/dev/ux_gpu_minimap_design_v1.md) | Phase 3 / UX-E01 — after 2B sign-off |
| 9-slice atlases | [`ui_texture_assets.json`](../../../tools/orchestrator/knowledge/ui_texture_assets.json) | Phase 3 art — flat v2 sufficient until then |
| Industrial E2E | [`post_stage6_active_todos.md`](../../../src/dev/post_stage6_active_todos.md) IND-E01 | **Unblocked** — build rail writes `BuildStripState` (2026-05-24) |
| F3 editor egui | Keep dev-gated | Remove from **sim default**, not delete |

---

## Execution plan (multi-step)

Run **one sprint per session**; max **3 files** per step (runbook §6 halt rule).

```text
2A-Tail → 2B-Build → 2B-Dedupe → Sign-off refresh → Product lanes (parallel)
```

---

### Sprint 1 — **2A-Tail** — **DONE**

**Exit:** F-10 doc + witness interaction flags after **1.6** — green 2026-05-24.

| Step | Status | Task | Verify |
|:---:|:---:|:---|:---|
| **1.1** | ✅ | **F-03** hover token-driven | **①** frame |
| **1.2** | ✅ | **F-08** `flat_v2_tray_tab` on info tabs | **①** frame |
| **1.3** | ✅ | **F-06** `on_tab_pressed` peek → expand | `cargo test … simulation_shell_phase2` |
| **1.4** | ✅ | **F-11** `palette_v2_tokens.md` | doc |
| **1.5** | ✅ | **F-10** HUD focus order note | `ui_layout_agent.md` § focus order |
| **1.6** | ✅ | Witness click flags in JSON | **②b** capture + `replay_ui_shell_witness_interactions_system` |

```powershell
cargo test -p proc_A_dine01 --lib simulation_shell_phase2 stage5
cargo run -p proc_A_dine01 -- --test frame          # Sprint 1 steps 1.1–1.3
cargo run -p proc_A_dine01 --release -- --test capture --stay-open   # 1.6 witness clicks
cargo run -p proc_A_dine01 --release -- --test visual                # optional CI pass
```

---

### Sprint 2 — **2B-Build** — **Done**

**Exit:** Bevy build rail authoritative; egui toolbox hidden in sim — **met**.

| Step | Status | Task | Touch (≤3 files) | Verify |
|:---:|:---:|:---|:---|:---|
| **2.1** | ✅ | **2B-01** rail → `BuildStripState` | `simulation_shell_phase2.rs` | **③** demo |
| **2.2** | ✅ | **2B-04** hide narrative stack when collapsed | `in_game_hud.rs` | **①** frame |
| **2.3** | ✅ | **2B-02** BuildToolbox off in sim | `simulation_session.rs`, `shell_framework.rs` | **③** demo |
| **2.4** | ✅ | Witness `phase2b_closed` + gate flags in JSON | re-run **②b** after 1.6 | JSON |

```powershell
cargo test -p proc_A_dine01 construction:: --lib
cargo run -p proc_A_dine01 -- --test demo
```

**Authority rule:** `BuildStripState` ← only build rail + existing keyboard cycle; UI reads, sim owns.

---

### Sprint 3 — **2B-Dedupe** — **Done** *(3.3 optional)*

**Exit:** One tool rail in sim; egui fallbacks editor-gated only — **met in code**.

| Step | Status | Task | Touch (≤3 files) | Verify |
|:---:|:---:|:---|:---|:---|
| **3.1** | ✅ | **2B-03** gate egui side status rail in sim | `ui_gates.rs`, `hud_root_tick.rs` | **③** demo |
| **3.2** | ✅ | Floating shells collapsed in sim | `simulation_session.rs`, `dock_shell.rs` | **①** frame |
| **3.3** | ✅ | Classify `ui_shell_migration.rs` shim (**B** transitional) | `ui_shell_migration.rs` doc | grep gate before delete |

```powershell
cargo run -p proc_A_dine01 -- --test demo
cargo test -p proc_A_dine01 --lib stage5
```

---

### Sprint 4 — **Sign-off + gate lift** — **DONE**

| Step | Task | Verify |
|:---:|:---|:---|
| **4.1** | Designer §1–§5 → **SIGNED** in [`ui_phase2_designer_signoff_v1.md`](ui_phase2_designer_signoff_v1.md) | All ☑ or accepted ◐ |
| **4.2** | Update [`ui_phase0_panel_mocks_v1.md`](ui_phase0_panel_mocks_v1.md) § Phase 2 → **closed**; note F-05 mock amend | Doc |
| **4.3** | Stage 5 + visual spine | **②** `--test visual` + `cargo test -p proc_A_dine01 --lib stage5` | ✅ 2026-05-24 |

```powershell
cargo test -p proc_A_dine01 --lib stage5
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Handoff — next phases (after Sprint 4)

**Immediate (Phase 2C — DEFERRED):** `@designer` picks **2C-A/B/C/D** → mock § P4 → `@coder` § Sprint 2C.

Pick **one primary** per cycle ([`post_stage6_active_todos.md`](../../../src/dev/post_stage6_active_todos.md)):

| Priority | Lane | Coder entry | Blocked by UI until |
|:---:|:---|:---|:---|
| **1** | **IND-E01** industrial chain | `economy/activation/` + sim place chain | **Unblocked** (2B-01 landed) |
| **2** | **UX-E01 M1** GPU minimap | [`ux_gpu_minimap_design_v1.md`](../../../src/dev/ux_gpu_minimap_design_v1.md) · **queue:** [`ui_phase3_coder_queue_v1.md`](ui_phase3_coder_queue_v1.md) | M1 code done — witness §3.2 |
| **3** | **CON-E01 P9** | construction witnesses in sim | Optional — parallel |
| **4** | **Phase 3 UI art** | 9-slice sheets per `ui_texture_assets.json` | Designer asset drop |

---

## Quick reference — files

| Area | File |
|:---|:---|
| Ops strip + tray + build rail systems | `src/gui/hud/simulation_shell_phase2.rs` |
| Bevy spawn (strip, rail, tray, minimap chrome) | `src/gui/in_game_hud.rs` |
| PLAY-01 sim defaults | `src/gui/hud/simulation_session.rs` |
| egui overlay tabs | `src/gui/hud/info_tabs.rs`, `overlay_shell.rs` |
| Tokens | `src/gui/style/palette.rs` → **`palette_v2_tokens.md`** (Sprint 1.4) |
| Panel state / Escape | `src/gui/hud/panel_state.rs` |
| Playbook | `tools/orchestrator/agents/ui_layout_agent.md` |

---

## Definition of done (Phase 2 UI shell)

- [x] F-01–F-09, F-11 landed (2026-05-24)
- [x] Bevy build rail **writes** `BuildStripState` (2B-01)
- [x] egui **BuildToolbox** / **side status rail** gated in sim (2B-02/03)
- [x] Left narrative stack hidden when collapsed (2B-04)
- [x] `palette_v2_tokens.md` committed
- [x] `simulation_shell_phase2` + gate tests **8/8**
- [x] Sign-off **SIGNED** (2026-05-24)
- [x] Interaction witness flags true after sim click pass (1.6)
- [x] Witness JSON refreshed with `phase2b_closed` (2.4)
- [x] `--test visual` green (Sprint 4.3)

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-23 | Post-sweep queue; 2A mostly landed; 4 sprints to product handoff |
| v1.1.0 | 2026-05-24 | @coder handoff block; authority rules; visual verify; B0001 + witness blocks landed |
| v1.2.0 | 2026-05-24 | Agent command table ①–④ mapped per sprint step |
| v1.3.0 | 2026-05-24 | Sprint board: 1 @ 95%, 2 @ 60%, active → 3; IND-E01 unblocked; 6/6 tests |
| v1.4.0 | 2026-05-24 | Sweep reconcile: Sprints 2–3 done; active → 4 prep; 8/8 tests; witness stale called out |
| v1.5.0 | 2026-05-24 | Sprint **2C** block: code refs, verify commands, designer-first workflow; Sprint 4 closed |
