# Operator unlock + test guide `v1`

**Audience:** human with a display (NEEDS-DISPLAY). Agents cannot self-certify pixels.  
**Repo:** `C:\dev\github\Rust_engine_template_01` · branch **`master`**  
**Date:** 2026-08-12

This is the **single start page**. Deep runbooks are linked; do not invent alternate bake/test paths.

---

## What you unlock (priority order)

| # | Session | Unlocks for agents | Time |
|:-:|:---|:---|:---|
| **1** | **A — Bug / renderer triage** | Clear bug list for `@coder` / `@sim-steward`; refreshes Stage5 / VFX witnesses | 20–40 min |
| **2** | **B — RPC-1-006** terrain bake pixel prove | Closes GPU terrain Phase 1; optional default flip later | 15–25 min |
| **3** | **C — G-PLAY-OPERATOR-01** | Closes **G-PLAY-01** (only open tensor gate) | ≥10 min play |
| **4** | **D — Warehouse keyframes (G4)** | Unpauses **MCP-PILOT-GRAMMAR-001** / Track B ship art · `@designer-mcp` / `@coder-mcp` | 45–90 min |

Do **A** first if you do not know current visual health. Do **D** when you want art fleet moving again.

---

## Session A — Bug situation + renderer

### A0. Clean shell (required)

```powershell
cd C:\dev\github\Rust_engine_template_01
Remove-Item Env:UI_LAYOUT_DEBUG,Env:STAGE5_VERBOSE,Env:STAGE5_PER_FRAME_HOOKS,Env:STAGE5_READINESS_VERBOSE,Env:VISUAL_DIAG,Env:STREAM_DIAG,Env:SIM_VIEW_SYNC_DEBUG,Env:VIEWPORT_DEBUG_OVERLAY,Env:TACTICAL_VFX_PROOF,Env:PERF_NO_VSYNC,Env:TERRAIN_GPU_BAKE_SPIKE,Env:PERF,Env:STALL,Env:STALL_SPAN_DEBUG -ErrorAction SilentlyContinue
```

Or: `.\tools\orchestrator\scripts\run_visual_test_clean.ps1`

Full env table: [`visual_test_runbook_v1.md`](visual_test_runbook_v1.md).

### A1. Stage5 / world renderer (primary)

```powershell
cargo run -p proc_A_dine01 --release -- --test visual --stay-open
```

**Look for (pass):**

- App stays up; no shader panic / Vulkan teardown
- Terrain + tiles visible on main map
- Minimap updates; pan/zoom ~2 min without soft-lock
- Exit cleanly → `debug_runs/stage5_full_app_live.json` written

**Known fixed vs still verify:** [`visual_run_blockers.md`](visual_run_blockers.md) (VR-10…17 fixed in code; **VR-16 sparks** and **VT-5 flicker** need eyes).

### A2. Fire / weather / VFX renderer

```powershell
cargo run -p proc_A_dine01 --release -- --test vfx
```

**Expect:** rain/precip, fire heat markers, sparks near burn, day/night void shift.  
**Note:** `TACTICAL_VFX_PROOF` locks zoom — do not leave that set for interactive play.

### A3. Optional stall triage (only if A1 feels janky)

```powershell
$env:PERF = "1"; $env:STALL = "1"; $env:RUST_LOG = "warn,stall=info,perf=info"
cargo run -p proc_A_dine01 --release -- --test visual --stay-open
```

Read `upd_span` using [`visual_test_runbook_v1.md`](visual_test_runbook_v1.md) § Reading `upd_span`.

### A4. Quiet perf truth (GPU program gate — optional)

```powershell
.\tools\orchestrator\scripts\run_demo_perf_truth.ps1
```

Then read `debug_runs/sim_spectrum_analytics_live.json` (gates in visual runbook § Perf truth).

### A5. Bug log (paste into chat / HANDOFF)

| ID | Symptom | When (A1/A2/…) | Panic? | Screenshot path | Suspected area |
|:---|:---|:---|:---:|:---|:---|
| B1 | | | | `debug_runs/captures/…` | terrain / fire / UI / camera |

**Rule:** one row per distinct bug. Prefer screenshot + witness path over long terminal paste.

---

## Session B — RPC-1-006 (GPU terrain bake prove)

**Goal:** With spike **ON**, minimap/terrain label may honestly show `gpu_bake`; you confirm pixels look correct. **Do not** flip production default ON in this session unless pixels clearly beat default.

### B1. Baseline (flag OFF) — 30s

```powershell
Remove-Item Env:TERRAIN_GPU_BAKE_SPIKE -ErrorAction SilentlyContinue
cargo run -p proc_A_dine01 --release -- --test visual --stay-open
```

Note minimap terrain look. Witness after exit: `debug_runs/minimap_compositor_live.json` → expect terrain source **`world_raster`**.

### B2. Spike ON — pixel compare

```powershell
$env:TERRAIN_GPU_BAKE_SPIKE = "1"
cargo run -p proc_A_dine01 --release -- --test visual --stay-open
```

**Pass checklist:**

| ☐ | Check |
|:---:|:---|
| ☐ | Minimap / world terrain still readable (no black/magenta void) |
| ☐ | Materials/topo look coherent vs baseline |
| ☐ | Fire / weather still **overlays** (not baked into terrain) |
| ☐ | Capture 1–2 PNGs under `debug_runs/captures/rpc1_006/` |
| ☐ | After exit: `minimap_compositor_live.json` may show `gpu_bake` when consumers bound |

### B3. Honesty lint (machine)

```powershell
cd tools\mcp\python
python -m rust_engine_mcp.cli terrain-honesty-lint
```

→ `debug_runs/terrain_honesty_lint_live.json`

### B4. Sign-off note

Write one line in chat or `debug_runs/terrain_gpu_bake_rpc1_006_operator_live.json` (hand/agent):

- `operator_pixel_proof: true|false`
- `recommend_default_flip: false` until proven better than CPU path
- Capture paths

**Unlock:** `@coder` may proceed on bake hardening / default-flip PR only if you set `recommend_default_flip: true`.

---

## Session C — G-PLAY-01 (play acceptance)

**Gate:** only open sub-gate is **G-PLAY-OPERATOR-01**. Checklist: [`plan_g_play_close_001_checklist_v1.md`](plan_g_play_close_001_checklist_v1.md) · runbook: [`play_scenario_acceptance_runbook_v1.md`](play_scenario_acceptance_runbook_v1.md).

```powershell
# NO --test visual, NO harness seed
cargo run -p proc_A_dine01 --release
```

Walk checklist §1–8 (sim enter → build rail → mine/kiln/mixer → progress → logistics on minimap → pause/resume → 10 min pan/zoom).

**Stop / do not sign if:** panic, need harness seed, construction invisible, pause broken.

**On EXECUTED:** mark checklist Operator row + tell `@planner` to close G-PLAY-01 rollup.

---

## Session D — Fix the keyframe issue (warehouse G4)

### Why it is blocked

Witness `debug_runs/art_pipeline/warehouse_production_keyframe_g4_live.json`:

- `g4_3_keyframe_minimum_stills_review: fail` — **stills missing** (`clean_day.png` etc. do not exist)
- Headless / ortho bake is **rejected** as ship art
- Production path = **`Light_keysshotsetup` + `utils/keyframe_render.py`** only

Authority: [`utils/KEYFRAME_RENDER_README.md`](../../utils/KEYFRAME_RENDER_README.md) · full steps: [`pilot_grammar_operator_runbook_v1.md`](../../docs/archive/2026-06-src-dev/plans/pilot_grammar_operator_runbook_v1.md)

### D1. Install / open keyframe addon

```cmd
tools\mcp\scripts\open_keyframe_render.cmd
```

Or Blender → Preferences → Add-ons → Install → repo `utils/keyframe_render.py` (v1.3.1+). Quit Blender first if “already registered”.

### D2. Open assembly + append iso rig

1. Open `assets/staging/assemblies/industrial_west_7x5_s39_9fa1.blend` (or current warehouse assembly path from runbook).
2. **File → Append** → `utils/Tile_iso_rig_v1.blend` → collection **`TILE_ISO_RIG`**.
3. If rig has no animation:  
   `cd tools\mcp\python` → `python -m rust_engine_mcp.cli build-iso-rig` → re-append.
4. Outliner: select **`Camera`** / **`IsoCamera`** (not ASSEMBLY meshes).
5. **View → Cameras → Set as Active Camera** (fixes “Cannot render, no camera”).
6. Properties → **Output** → **Keyframes (legacy export)** → Animation source = Camera with curves → **Refresh keyframe list**.

### D3. Export 24 stills

Folder:

`assets/staging/tiles/tile_warehouse_industrial_v2_minimum_g4/`

Names (exact):

- `clean_day_f0.png` … `clean_day_f7.png`
- `clean_night_on_f0.png` … `clean_night_on_f7.png`
- `damaged_night_on_f0.png` … `damaged_night_on_f7.png`

**Save the .blend** before render so `//` output resolves. Rename Blender’s `3_clean_day_f0_.png`-style outputs if needed.

### D4. Finish script (pack + validators)

```powershell
.\tools\mcp\scripts\operator_warehouse_keyframe_finish.ps1
```

Expect: 24/24 found → atlas pack → Phase C validators green · `keyframe_manual.export` with `method: keyframe_render.py`.

### D5. Confirm G4

Re-check / refresh warehouse G4 witness — minimum stills must exist; `proceed_ship` only after Phase C pass.

**Unlock:** unpause **MCP-PILOT-GRAMMAR-001**; `@designer-mcp` can resume Track B; landscape `ship:false` atlases remain separate until their own G4.

### Common keyframe failures

| Symptom | Fix |
|:---|:---|
| “No object animation data” | You selected ASSEMBLY mesh — select Camera under `TILE_ISO_RIG` |
| Zero curves after Refresh | Blender 5: use addon 1.3+; or scrub + **Add current frame** |
| “Cannot render, no camera” | Set Active Camera; reinstall addon from repo |
| Finish script Missing PNGs | Wrong folder or wrong filenames (must be `*_f0`…`*_f7`) |
| G4 still fails after pack | Headless marker left behind — finish script removes fake `blender_keyframe_light_rig` marker |

---

## Operator bugs (2026-08-12 session)

Ingested from `Documents/bug_out` → [`debug_runs/operator_bugs/`](../../debug_runs/operator_bugs/).

| ID | Severity | Symptom | Likely cause | Workaround now |
|:---|:---|:---|:---|:---|
| **B2** | **P0** | Build menu dead; stuck `Place 2x2`; cannot place/cancel | Shift+LMB queue had **no Enter drain** for buildings; Esc kept tool; tray queue read-only | **Fixed:** Esc clears queue+tool; Enter builds queued; tray **Build queued** / **Clear queue** |
| **B1** | P2 | Yellow bar in water/sky — **static, not cursor** | Likely leftover road/path ghost or bad world polyline (not unified cursor) | Esc with Roads selected; capture again with placement debug OFF |
| **B3** | P1 | No fire on `--test vfx` | `VfxFireTestHighlightPlugin` was **never wired**; heat overlay off in VfxSandbox | **Fixed:** plugin wired in test mode + heat markers on; rebuild + `--test vfx` |
| **B4** | P1 | `Path not found …\assets\assets\…` on visual | AssetServer got `assets/`-prefixed paths (doubled) | **Fixed in tree** — rebuild then re-run visual; GLBs exist |
| **B5** | P3 | 10 rustc warnings | Hygiene only — compile still succeeds | Optional later; not a blocker |

Triage packet: [`operator_session_20260812_triage_live.json`](../../debug_runs/operator_bugs/operator_session_20260812_triage_live.json). **Next coder pick: B2.**

---

## After you finish — tell agents

Paste this block (fill verdicts):

```text
OPERATOR SESSION 2026-08-12
A visual: PASS|FAIL — notes: …
A vfx: PASS|FAIL — notes: …
B RPC-1-006: PASS|FAIL — recommend_default_flip: false|true — captures: …
C G-PLAY: EXECUTED|BLOCKED — checklist: plan_g_play_close_001_checklist_v1.md
D keyframes: 24/24|incomplete — finish.ps1: PASS|FAIL
Bugs: (table rows or none)
Next agent pick unlocked: coder TODO-05 | terrain default flip | designer-mcp pilot | none
```

---

## Do not

- Claim G-PLAY or RPC-1-006 closed from lib tests alone  
- Use `tile_ortho_bake` / lod0 pilot atlases as production ship art  
- Leave `TERRAIN_GPU_BAKE_SPIKE=1` as permanent shell default until signed  
- Flip terrain production default ON without Session B pass  
- Dump full cargo walls into chat — use witness paths + bug table  
