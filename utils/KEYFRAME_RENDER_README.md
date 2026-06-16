# keyframe_render.py — legacy iso still export

**Spine:** [`LEGACY_ART_PIPELINE_README.md`](LEGACY_ART_PIPELINE_README.md) · operator runbook [`docs/archive/2026-06-src-dev/plans/pilot_grammar_operator_runbook_v1.md`](../docs/archive/2026-06-src-dev/plans/pilot_grammar_operator_runbook_v1.md)

## Where is the UI?

Blender **Properties** panel (wrench icon) → **Output** tab → scroll to **Keyframes (legacy export)**.

Not in the 3D View sidebar. Not in MCP Art Pipeline Suite (APS only launches Blender with this script).

## Why "No object animation data found"?

The warehouse **assembly blend** (`industrial_west_7x5_s39_9fa1.blend`) contains **static** module GLBs in collection `ASSEMBLY`. They have **no** keyframes.

Rotation / day-night / damaged variants live on the **iso rig** you must append:

1. **File → Append** → `utils/Tile_iso_rig_v1.blend` → collection **`TILE_ISO_RIG`**
2. In the **Outliner**, expand **`TILE_ISO_RIG`** (the collection row is not an object — clicking it shows a bounds box only).
3. Select **`IsoCamera`** or a **Light** (Key / Fill / Sun), not `wall_steel_…` under **ASSEMBLY**.
4. **Animation source** → pick the object that shows **F-Curves** (not “no Action on rig”).
5. **Refresh keyframe list** — or, if the rig has no baked animation, scrub the timeline and use **Add current frame** per still (24× manual ship).

If **Animation source** only lists objects with **“no Action on rig”**, rebuild the rig from legacy lights:

```powershell
cd C:\dev\github\Rust_engine_template_01\tools\mcp\python
python -m rust_engine_mcp.cli build-iso-rig
```

Requires `utils/Light_keysshotsetup.blend` in the repo. Then re-append **`TILE_ISO_RIG`** and Refresh again.

Reference: open `utils/Light_keysshotsetup.blend` to see how variant keyframes were authored before the slim iso rig.

### Blender 5.0 / 5.1 (Steam) — “Camera” + Action + Legacy Slot

The rig camera may be named **`Camera`**, not `IsoCamera`. In the Properties → Object → Animation panel you see an **Action** and **Legacy Slot** — that is correct.

Blender 5 **removed** `action.fcurves` on the Action block. Keyframes live in the slot’s **channelbag**. Addon **v1.3.0+** reads those via `bpy_extras.anim_utils`.

**Workflow:**

1. Outliner → select **`Camera`** (under `TILE_ISO_RIG`).
2. Keyframes panel → **Use active object** (eyedropper).
3. **Animation source** should show `Camera (N keyframe curves)` or `(Action assigned — click Refresh)`.
4. **Refresh keyframe list**.

To inspect curves manually: **Dope Sheet** editor → mode **Action** → channel list for that Action (same data as Legacy Slot).

If Refresh still reports zero curves, use **Add current frame** while scrubbing the timeline (manual ship path).

### “Cannot render, no camera”

Selecting **Camera** in the Outliner is not enough — Blender needs **`scene.camera`** for `bpy.ops.render.render`. Use **View → Cameras → Set as Active Camera**, or let addon **v1.3.1+** set it automatically from your rig **Camera** before each still. Re-install from repo `utils/keyframe_render.py` (not only the copy under `%AppData%\Roaming\Blender Foundation\...\addons\`).

## Install (persistent)

1. **Quit Blender completely** if you previously ran `open_keyframe_render.cmd` or installed an older copy (avoids "already registered" errors).
2. Blender → **Edit → Preferences → Add-ons → Install**
3. Select `utils/keyframe_render.py` from this repo (not a copy under Blender's addons folder unless you intend to maintain that copy separately).
4. Enable **Render: Keyframe Renderer (Rust Engine legacy)**

If install fails with `already registered as a subclass 'keyframe_checkbox_group'`:

- Preferences → Add-ons → search **Keyframe** → **Disable** and **Remove** any old entry, restart Blender, install again from repo `utils/keyframe_render.py`.

One-shot launch (registers for that session only — still install for the Output panel):

```cmd
tools\mcp\scripts\open_keyframe_render.cmd
```

## Output files

With **File Name** = `clean_day_f0` and frame 3 selected, Blender writes next to the saved `.blend`:

```text
3_clean_day_f0_.png
```

Rename to pilot names if needed (`clean_day_f0.png`) before `operator_warehouse_keyframe_finish.ps1`.

**Save the blend** before rendering so output path `//` resolves.

## v1.1 changes (2026-06)

- **Animation source** dropdown (rig vs active object)
- **Refresh keyframe list** button
- **Select all** checkboxes
- Status line + Blender **Info** reports after render
- Clear error when ASSEMBLY modules have no animation
