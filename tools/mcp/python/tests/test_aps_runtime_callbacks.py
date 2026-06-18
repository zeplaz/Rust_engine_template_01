"""APS runtime callback smoke — drive the GUI callbacks that crashed at runtime.

Import/unit tests never construct the real Tk panels nor invoke their event
callbacks, so the runtime tracebacks (B1 grammar-OFF generate, B2 slot preview
render, the slot-preview placeholder TypeError, and the assembly-preview thumb
2-arg mismatch) shipped undetected. These tests build a HIDDEN Tk root, wire up
the actual panels, and call the callbacks directly — asserting "no exception".

Skips cleanly when no display / Tk is available (CI without a desktop); runs and
passes locally on Windows where `python` (3.14) has PIL + mcp + jsonschema.
"""

from __future__ import annotations

import os
import sys
from pathlib import Path

import pytest
import tkinter as tk

# rust_engine_mcp is importable from tools/mcp/python (pytest rootdir); the
# art_pipeline_suite package lives one level up under tools/mcp.
APS_PARENT = Path(__file__).resolve().parents[2]
if str(APS_PARENT) not in sys.path:
    sys.path.insert(0, str(APS_PARENT))

pytest.importorskip("PIL")

# Keep the Bevy worker out of the loop — these are pure UI-callback smoke tests,
# not a render-farm round trip. The browser/three.js path is exercised instead.
os.environ.setdefault("RUST_ENGINE_BEVY_PREVIEW", "0")

pytestmark = pytest.mark.aps_gui


def _make_assembly_panel(gui_panel_host):
    from art_pipeline_suite.assembly_panel import AssemblyPanel
    from art_pipeline_suite.state import SuiteState

    logs: list[str] = []
    state = SuiteState()
    panel = AssemblyPanel(gui_panel_host, state, on_log=logs.append, start_job=None)
    panel.pack(fill="both", expand=True)
    gui_panel_host.winfo_toplevel().update_idletasks()
    return panel, logs


def test_on_generate_with_building_grammar(gui_panel_host) -> None:
    """B1 guard — grammar ON generate produces a snapshot, no traceback."""
    panel, _logs = _make_assembly_panel(gui_panel_host)
    panel.use_grammar_var.set(True)
    panel._on_grammar_toggle()
    gui_panel_host.winfo_toplevel().update_idletasks()

    panel.on_generate()
    gui_panel_host.winfo_toplevel().update_idletasks()

    snap = panel._snapshot
    assert snap is not None, "grammar generate produced no snapshot"
    assert snap.get("module_placements"), "grammar snapshot has no placements"
    assert snap.get("grammar_rule_chain"), "grammar snapshot missing rule chain"


def test_on_generate_without_building_grammar(gui_panel_host) -> None:
    """B1 guard — grammar OFF generate produces a valid snapshot, no traceback.

    The grammar-OFF path also auto-selects placement 0, which runs
    SlotPreviewPanel.show_placement -> render_module_isolated (the call that
    used to TypeError on the missing placeholder ``color=`` kwarg).
    """
    panel, _logs = _make_assembly_panel(gui_panel_host)
    panel.use_grammar_var.set(False)
    panel._on_grammar_toggle()
    gui_panel_host.winfo_toplevel().update_idletasks()

    panel.on_generate()
    gui_panel_host.winfo_toplevel().update_idletasks()

    snap = panel._snapshot
    assert snap is not None, "grammar-OFF generate produced no snapshot"
    assert snap.get("module_placements"), "grammar-OFF snapshot has no placements"
    # Non-grammar snapshots carry no grammar rule chain — the UI must tolerate it.
    assert "grammar_rule_chain" not in snap


def test_grammar_toggle_then_off_generate(gui_panel_host) -> None:
    """B1 guard — the exact user sequence: grammar ON, generate, uncheck, generate."""
    panel, _logs = _make_assembly_panel(gui_panel_host)
    top = gui_panel_host.winfo_toplevel()

    panel.use_grammar_var.set(True)
    panel._on_grammar_toggle()
    panel.on_generate()
    top.update_idletasks()

    panel.use_grammar_var.set(False)
    panel._on_grammar_toggle()
    top.update_idletasks()
    panel.on_generate()
    top.update_idletasks()

    assert panel._snapshot is not None
    assert panel._snapshot.get("module_placements")


def test_on_placement_select_runs_slot_preview(gui_panel_host) -> None:
    """Crash-1 / B2 guard — placement select -> slot preview render, no TypeError.

    render_module_isolated falls back to a labeled placeholder when trimesh is
    absent or returns a blank/black render; either way it must not raise and the
    combined/module thumbs must not be a black tile.
    """
    panel, _logs = _make_assembly_panel(gui_panel_host)
    top = gui_panel_host.winfo_toplevel()
    panel.use_grammar_var.set(False)
    panel._on_grammar_toggle()
    panel.on_generate()
    top.update_idletasks()

    assert panel.placement_list.size() > 0, "no placements to select"
    panel.placement_list.selection_clear(0, tk.END)
    panel.placement_list.selection_set(0)
    panel.on_placement_select()
    top.update_idletasks()

    assert panel._selected_node_id, "placement select did not resolve a node id"


def test_slot_preview_render_not_black(tk_root) -> None:
    """B2 guard — module isolated render is never a uniformly-black image.

    Either trimesh produces a framed render or we degrade to the gray labeled
    placeholder; a fully black tile is the bug being regression-locked.
    """
    from rust_engine_mcp import assembly
    from rust_engine_mcp.aps_slot_preview import render_module_isolated

    snap = assembly.generate_assembly_snapshot(
        style_pack_id="style_industrial_west",
        width=4,
        depth=3,
        floors=2,
        seed=42,
        source_tier="production",
        write=False,
    )
    placements = snap["module_placements"]
    assert placements, "fixture snapshot produced no placements"
    glb_rel = placements[0]["glb_path"]
    img = render_module_isolated(glb_rel, size=96)
    assert img is not None, "module render returned None (should fall back to placeholder)"
    lo, hi = img.convert("L").getextrema()
    assert hi >= 24, f"module thumbnail is (near-)black: extrema={(lo, hi)}"


def test_assembly_preview_apply_result_with_png(gui_panel_host, tmp_path) -> None:
    """Crash-2 guard — _apply_preview_result with a png drives _on_preview_thumb(image, result)."""
    from art_pipeline_suite.assembly_preview_panel import AssemblyPreviewPanel
    from rust_engine_mcp.paths import repo_root
    from PIL import Image

    # A non-black PNG under the repo root so _load_thumbnail resolves + displays it.
    png_abs = repo_root() / "debug_runs" / "_aps_runtime_callback_preview.png"
    png_abs.parent.mkdir(parents=True, exist_ok=True)
    Image.new("RGB", (48, 48), (90, 140, 200)).save(png_abs)
    png_rel = png_abs.relative_to(repo_root()).as_posix()

    received: list[tuple[object, object]] = []

    def on_thumb(image, result):
        # Mirrors AssemblyPanel._on_assembly_preview_thumb(self, image, _result):
        # crash-2 was calling this with a single argument.
        received.append((image, result))

    panel = AssemblyPreviewPanel(gui_panel_host, on_log=lambda _l: None, on_preview_thumb=on_thumb)
    panel.pack()
    top = gui_panel_host.winfo_toplevel()
    top.update_idletasks()

    result = {
        "assembly_id": "smoke",
        "mode": "browser_threejs",
        "preview_url": "http://127.0.0.1:0/",
        "modules_loaded": 1,
        "material_profiles_sample": ["mat_demo"],
        "missing_glb": [],
        "png": png_rel,
    }
    panel._apply_preview_result(result)
    top.update_idletasks()

    assert received, "_on_preview_thumb was not invoked"
    assert received[0][1] is result, "_on_preview_thumb did not receive the result dict (2nd arg)"

    try:
        png_abs.unlink()
    except OSError:
        pass


def test_assembly_preview_apply_result_black_png_labels(gui_panel_host) -> None:
    """B2 guard — a black preview PNG is labeled, not pasted as a black tile."""
    from art_pipeline_suite.assembly_preview_panel import AssemblyPreviewPanel
    from rust_engine_mcp.paths import repo_root
    from PIL import Image

    png_abs = repo_root() / "debug_runs" / "_aps_runtime_callback_black.png"
    png_abs.parent.mkdir(parents=True, exist_ok=True)
    Image.new("RGB", (48, 48), (0, 0, 0)).save(png_abs)
    png_rel = png_abs.relative_to(repo_root()).as_posix()

    panel = AssemblyPreviewPanel(gui_panel_host, on_log=lambda _l: None)
    panel.pack()
    top = gui_panel_host.winfo_toplevel()
    top.update_idletasks()

    panel._load_thumbnail(png_rel)
    top.update_idletasks()

    assert panel._thumb_photo is None, "black PNG should not become a displayed photo"
    assert "unavailable" in panel._thumb_label.cget("text").lower()

    try:
        png_abs.unlink()
    except OSError:
        pass


def test_tooltip_lifecycle_hides(tk_root) -> None:
    """B3 guard — a shown tooltip is destroyed by hide_all_tooltips (tab change)."""
    from art_pipeline_suite.aps_tooltips import _Tooltip, bind_aps_tooltip, hide_all_tooltips

    btn = tk.Button(tk_root, text="hover me")
    btn.pack()
    bind_aps_tooltip(btn, "asm_generate")
    tk_root.update_idletasks()

    _Tooltip.show_for(btn, "tooltip body")
    tk_root.update_idletasks()
    assert _Tooltip._current is not None, "tooltip did not show"

    hide_all_tooltips()
    tk_root.update_idletasks()
    assert _Tooltip._current is None, "tooltip survived hide_all_tooltips (would float)"


def test_scrollable_frame_constructs_and_updates(tk_root) -> None:
    """B4 guard — ScrollableFrame builds, debounced scrollregion applies cleanly."""
    from art_pipeline_suite.scrollable import ScrollableFrame

    frame = ScrollableFrame(tk_root, enable_horizontal=True)
    frame.pack(fill=tk.BOTH, expand=True)
    for i in range(40):
        tk.Label(frame.interior, text=f"row {i}").pack(anchor=tk.W)
    tk_root.update_idletasks()
    # Force the debounced idle pass to run; must not raise.
    frame._apply_scrollregion()
    tk_root.update_idletasks()
    assert frame._last_scrollregion is not None
