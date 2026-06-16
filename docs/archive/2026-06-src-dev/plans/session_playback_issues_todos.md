# Session playback issues — todo board



**Created:** 2026-05-22 (from live `--test visual` session feedback)



**Status:** All PLAY items implemented (2026-05-22). Proof: `cargo test -p proc_A_dine01 --lib`; live JSON via `cargo run -p proc_A_dine01 -- --test visual` (`stage5_full_app_live.json` → `fire_playback` block).



Priority order: **P0 panels/perf** → **P1 tile occupation visuals (build/road)** → **P1 fire stability**.



**Build/road model (canonical — not “zoom scaling buildings”):** Structures **occupy world tiles**. Gameplay keeps **which tiles** and **what state** (planned, under construction, built, damaged, road lane, etc.). The map only **draws** those cells; zoom changes screen projection, not occupancy. Syx-style top-down **tileism**: one tactical cell = one visual cell (colors now, tiled textures later).



---



## PLAY-01 — Editor/tools panels must not open in simulation



- [x] **PLAY-01a** — `apply_simulation_hud_defaults` on `BaseState::Simulation` enter.

- [x] **PLAY-01b** — `dismiss_world_gen_preview_chrome` on sim enter.

- [x] **PLAY-01c** — Diagnostics sections `default_open(false)` in sim (`diagnostics_ui.rs` + `BaseState`).

- [x] **PLAY-01d** — Document sim HUD vs editor HUD in `AGENTS.md`.



---



## PLAY-02 — Simulation frame time / lag



- [x] **PLAY-02a** — Profiling targets documented below (`streaming_apply`, `egui_world_gen_ui`); full 60s capture optional per `operational_readiness_vs_infrastructure_perf_v1.md`.

- [x] **PLAY-02b** — Throttle `maintain_test_scene_fire_overlay`.

- [x] **PLAY-02c** — WorldGen UI `run_if(visible)`; preview texture sync gated when chrome hidden.

- [x] **PLAY-02d** — `ProductShellUpdateBudget::set_bypass_throttle(false)` on sim enter.



**Perf profiling note (PLAY-02a):** In sim, watch `perf` target for `STALL culprit=streaming_apply` and `worldgen_chrome::egui` / `egui_world_gen_ui`. World preview raster is already gated by `world_preview_pipeline_enabled` + chrome dismiss on sim enter.



---



## PLAY-BUILD — Tile occupation visuals



- [x] **PLAY-BUILD-01** … **09** — Occupancy, mock shapes RON + registry, roads, projection, labels, F7 toggle.

- [x] **PLAY-BUILD-06** — `map_egui_projection::tile_world_xy_stable_under_zoom_pan`.

- [x] **PLAY-BUILD-08** — Opaque tooltip at footprint (`build_footprint_overlay.rs`); no `Build intent` window.

- [x] **PLAY-BUILD-10** — Tile occupation invariants in `src/dev/construction_invariants.md`.



---



## PLAY-06 — Fire stability



- [x] **PLAY-06a** — `chunk_surface_fire.rs` smolder tuning.

- [x] **PLAY-06b** — `chunk_fire_overlay.rs` slower decay + rain damp on existing heat.

- [x] **PLAY-06c** — Hold overlay when snapshot empty (`fire_visual_extract.rs`).

- [x] **PLAY-06d** — Test harness throttle.

- [x] **PLAY-06e** — `FirePlaybackStabilityWitness` in live JSON `fire_playback` block.



---



## Commands



```powershell

cargo test -p proc_A_dine01 --lib

cargo run -p proc_A_dine01 -- --test visual

```


