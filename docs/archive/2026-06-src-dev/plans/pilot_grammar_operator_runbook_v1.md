# Warehouse pilot — operator runbook (manual keyframe ship) `v1`

| Field | Value |
|:---|:---|
| **For** | **You** (human operator) — not @coder-mcp |
| **Program** | MCP-PILOT-GRAMMAR-001 Track B |
| **Status** | **PAUSED** (2026-06-03) — warehouse keyframe/G4 does **not** block APS, Bevy preview, grammar, or simulation HUD work. Resume when [`plan_bevy_hud_grammar_parallel_v1.md`](plan_bevy_hud_grammar_parallel_v1.md) says artist-ready. |
| **Assembly** | `industrial_west_7x5_s39_9fa1` |
| **Policy** | [`mcp_orchestrator_tile_fix_warehouse_slice_v2.md`](mcp_orchestrator_tile_fix_warehouse_slice_v2.md) |

---

## What you are doing (one sentence)

You are **taking 24 photos** of the warehouse building in Blender (3 lighting states × 8 rotations), saving them as PNGs, then running **two PowerShell commands** so the repo validators accept them as real ship art.

You are **not** running automated headless bake scripts.

---

## Before you start (5 minutes)

| Check | Command / look |
|:---|:---|
| Blender installed | Opens from Start menu |
| Repo path | `C:\dev\github\Rust_engine_template_01` |
| Pilot blend exists | `assets/staging/assemblies/industrial_west_7x5_s39_9fa1.blend` |
| Prep done | Run **Part 1** below once |

Optional: APS preview looked OK (`python -m rust_engine_mcp.cli preview-assembly ...`). Materials come from snapshot — **do not** paint materials in Blender by hand unless fixing a specific module.

---

## The 24 PNGs you must produce

Save into:

`assets/staging/tiles/tile_warehouse_industrial_v2_minimum_g4/`

**Exact filenames** (validators look for these):

| State | Files (8 facings each) |
|:---|:---|
| `clean_day` | `clean_day_f0.png` … `clean_day_f7.png` |
| `clean_night_on` | `clean_night_on_f0.png` … `clean_night_on_f7.png` |
| `damaged_night_on` | `damaged_night_on_f0.png` … `damaged_night_on_f7.png` |

Each PNG must be **128×128** (or larger — pack will use folder). **Facings must look different** (rotate building or rig per frame) — identical facings fail promotion.

Reference matrix: [`debug_runs/art_pipeline/warehouse_state_facing_matrix_v1.yaml`](../../debug_runs/art_pipeline/warehouse_state_facing_matrix_v1.yaml).

---

## Part 1 — Prep (once)

From repo root — pick **one** method (Windows execution policy often blocks `.ps1`):

**A — CMD (no PowerShell policy):**

```cmd
cd C:\dev\github\Rust_engine_template_01
tools\mcp\scripts\designer_mcp_pilot_grammar_prep.cmd
```

**B — PowerShell with bypass (one session):**

```powershell
cd C:\dev\github\Rust_engine_template_01
powershell -ExecutionPolicy Bypass -File tools\mcp\scripts\designer_mcp_pilot_grammar_prep.ps1
```

**C — Manual Python (same steps):**

```cmd
cd C:\dev\github\Rust_engine_template_01\tools\mcp\python
python -m rust_engine_mcp.cli generate-material-textures --profile steel_panel_01
python -m rust_engine_mcp.cli generate-material-textures --profile roof_metal_01
python -m rust_engine_mcp.cli generate-material-textures --profile brick_red_01
python -m rust_engine_mcp.cli generate-material-textures --profile wood_plank_01
python -m rust_engine_mcp.cli assembly-build-run C:\dev\github\Rust_engine_template_01\assets\staging\assemblies\industrial_west_7x5_s39_9fa1.json
```

This:

- Confirms grammar snapshot JSON exists
- Generates material texture PNGs for profiles
- Runs `assembly-build-run` → refreshes `industrial_west_7x5_s39_9fa1.blend` with snapshot materials

**Do not** run `tile_compile_minimum_bake.py` or `designer_mcp_pilot_grammar_keyframe.py` for ship.

---

## Part 2 — Blender (manual, ~30–60 min)

### 2.1 Open the assembly

1. Start **Blender**.
2. **File → Open** →  
   `C:\dev\github\Rust_engine_template_01\assets\staging\assemblies\industrial_west_7x5_s39_9fa1.blend`
3. In the **Outliner**, confirm collection **`ASSEMBLY`** has walls/roof — **no truck**, no `TILE_ISO_RIG` saved in file (if you see truck/rig, run prep/cleanup again).

### 2.2 Append the iso camera + lights rig

1. **File → Append**.
2. Navigate to `C:\dev\github\Rust_engine_template_01\utils\Tile_iso_rig_v1.blend`.
3. Open the blend, choose collection **`TILE_ISO_RIG`**, append.
4. If rig missing, rebuild once:

   ```powershell
   cd C:\dev\github\Rust_engine_template_01\tools\mcp\python
   python -m rust_engine_mcp.cli build-iso-rig
   ```

5. In the Outliner, **expand** collection **`TILE_ISO_RIG`** → select object **`IsoCamera`** (not the collection row). **View → Cameras → Active Camera**, **Numpad 0** to preview.

### 2.3 Enable the keyframe export addon

**Option A — APS (easiest launch)**

```powershell
cd C:\dev\github\Rust_engine_template_01\tools\mcp\python
python -m art_pipeline_suite.run
```

Atlas tab → **Keyframe render addon** (opens Blender with addon script).

**Option B — CLI**

```powershell
cd C:\dev\github\Rust_engine_template_01\tools\mcp\python
python -c "from tools.mcp.module_viewer.pipeline_runner import open_keyframe_render_addon; print(open_keyframe_render_addon())"
```

Or: Blender → **Edit → Preferences → Add-ons → Install** → pick `utils/keyframe_render.py` → enable **Keyframe Renderer**.

### 2.4 Set render output

1. **Render Properties** (camera icon) → **Output**:
   - **File format:** PNG
   - **Color:** RGBA
   - **Resolution:** 128 × 128 (or 256 — downscale later if needed)
2. Set output folder to the staging tile folder (paste path):

   `C:\dev\github\Rust_engine_template_01\assets\staging\tiles\tile_warehouse_industrial_v2_minimum_g4\`

   (Blender may use `//` relative to blend — saving PNGs next to the assembly is OK; **move/rename** into the folder above before Part 3.)

### 2.5 For each state × facing (24 renders)

The rig / legacy light setup animates **variant** (day, night-on, damaged) and **rotation** on the timeline.

1. Scrub the **timeline** (frames 1, 2, 3, …).
2. For each frame you need:
   - Building looks like **clean day** / **clean night (lights on)** / **damaged night** per matrix.
   - Building is **rotated** to a new facing (8 different angles across the pilot).
3. **Properties → Output → Keyframes** (addon). **Animation source** = **`IsoCamera`** or a **Light** in `TILE_ISO_RIG` (never a `wall_steel_…` mesh).
4. If the rig has timeline keyframes: **Refresh keyframe list**, check frames, set **File Name** (e.g. `clean_day_f0`), **Render Keyframes to Images**.
5. If **Refresh** warns *no Action/F-Curves*: either run `build-iso-rig` (see §2.2) and re-append, **or** scrub timeline, pose building/lights, click **Add current frame**, set **File Name**, **Render** — repeat per PNG.

Repeat until you have **24 files** with the exact names in the table above.

**Tips**

- **Animation source** empty or “no Action on rig”: you selected the **collection** or a procedural rig without legacy keyframes — expand `TILE_ISO_RIG`, pick **IsoCamera**, or rebuild iso rig / use **Add current frame**.
- Legacy reference: `utils/Light_keysshotsetup.blend` shows how states were authored before `Tile_iso_rig_v1`.
- **Eyeball check:** open `clean_day_f0.png` vs `clean_day_f4.png` — they must not be identical.

### 2.6 Delete old rejected stills

In `tile_warehouse_industrial_v2_minimum_g4\`, delete grey slab PNGs from the old headless run if still present. Keep only your new 24 + files you create in Part 3.

---

## Part 3 — Finish in PowerShell (after Blender)

One script — pack, marker, validators:

```cmd
cd C:\dev\github\Rust_engine_template_01
tools\mcp\scripts\operator_warehouse_keyframe_finish.cmd
```

Or: `powershell -ExecutionPolicy Bypass -File tools\mcp\scripts\operator_warehouse_keyframe_finish.ps1`

This script:

1. Counts `clean_day_f*.png`, `clean_night_on_f*.png`, `damaged_night_on_f*.png` (8 each)
2. Writes `keyframe_manual.export` (required for ship — **not** headless fake marker)
3. Runs `tile-atlas-pack … -pk` on that folder
4. Runs `designer_mcp_warehouse_phase_c.ps1` against the **same** folder

If it fails, read the printed `validate-report` summary (validation-first — do not paste full cargo/blender logs unless asked).

---

## Part 4 — Designer G4 (you + @designer-mcp)

1. **You** flip through all 24 PNGs at 100% zoom — warehouse readable, no truck, night/damage states obvious, facings rotate.
2. If happy, ask **@designer-mcp** to run witness / signoff (or run Phase C again after fixes).

**Forbidden:** approving grey identical slabs or marking green on schema-only.

---

## What each old instruction meant

| Old line | Plain meaning |
|:---|:---|
| Open `…7x5….blend` | Part 2.1 |
| Append `Tile_iso_rig_v1` → `TILE_ISO_RIG` | Part 2.2 |
| Run `keyframe_render.py` | Part 2.3–2.5 (Blender UI addon, not a terminal command) |
| Export to `keyframe_stills/…` | **Use** `tile_warehouse_industrial_v2_minimum_g4` with names above (validators point here) |
| `keyframe_manual.export` | Part 3 script writes this |
| `tile-atlas-pack` | Part 3 script |
| `designer_mcp_warehouse_phase_c.ps1` | Part 3 script (after manual PNGs exist) |

---

## Troubleshooting

| Problem | Fix |
|:---|:---|
| “I only have one grey PNG” | You ran headless bake — delete it; do Part 2 in Blender UI |
| Keyframes panel empty | Select animated rig object; append `TILE_ISO_RIG` |
| phase_c fails `art_quality` | Missing/wrong `keyframe_manual.export` or headless marker — rerun Part 3 |
| `FacingRotationMissing` | Re-render facings — rotate building 45° per facing |
| P0 / StylePackDrift | @coder-mcp fixes modules — you can still test Blender, but ship may wait |
| Blender won’t start from APS | Install Blender; set path in MCP config / `BLENDER` env |

---

## After you pass G4

@coder-mcp runs atlas **register**; @coder runs map stamp smoke. You do **not** need to edit `_tile_atlas_index.ron` by hand.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Operator-facing steps aligned to repo tools |
