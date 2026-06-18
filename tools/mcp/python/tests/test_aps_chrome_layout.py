"""P7 Slice C guard — chrome de-clutter.

Asserts the two structural promises of Slice C, headlessly:
  * at most TWO always-on chrome rows sit ABOVE the work area (the notebook),
    at idle (the transient job strip and the bottom status log don't count);
  * the ``MetadataFlowPanel`` ("Where this data goes" guide) appears ONLY on the
    Assembly tab — not on every tab.

Skips cleanly when no Tk display is available.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

APS_PARENT = Path(__file__).resolve().parents[2]
if str(APS_PARENT) not in sys.path:
    sys.path.insert(0, str(APS_PARENT))

pytestmark = pytest.mark.aps_gui


def test_at_most_two_always_on_chrome_rows_above_work_area(aps_app) -> None:
    """The lane bar + the pipeline-spine row are the only always-on bands above
    the notebook. The job strip is hidden at idle; the status log is below it."""
    notebook = aps_app._notebook_container
    notebook_y = notebook.winfo_y()

    above_visible = []
    for child in aps_app.winfo_children():
        if child is notebook:
            continue
        if not child.winfo_ismapped():
            continue
        if child.winfo_y() < notebook_y:
            above_visible.append(child)

    assert len(above_visible) <= 2, (
        f"too many always-on chrome rows above the work area: {len(above_visible)} "
        f"(expected <= 2). Offenders: {[c.winfo_class() for c in above_visible]}"
    )


def test_status_log_is_below_the_work_area(aps_app) -> None:
    """The collapsible status log must not eat above-the-fold height (Slice C)."""
    notebook_y = aps_app._notebook_container.winfo_y()
    assert aps_app._status_log_frame.winfo_y() >= notebook_y, (
        "status log sits above the work area — it should be a bottom band"
    )


def test_metadata_flow_only_on_assembly(aps_app) -> None:
    """The MetadataFlowPanel guide appears only on the Assembly tab."""
    assert hasattr(aps_app.assembly, "metadata_flow"), "Assembly lost its metadata flow guide"

    others = {
        "materials": aps_app.materials,
        "catalog": aps_app.catalog,
        "atlas": aps_app.atlas,
    }
    aps_app._apply_lane("landscape", log=False)
    aps_app.update_idletasks()
    others.update(
        {
            "landscape_presets": aps_app.landscape_presets,
            "landscape_grammar": aps_app.landscape_grammar,
            "landscape_states": aps_app.landscape_states,
            "landscape_atlas": aps_app.landscape_atlas,
        }
    )
    leaked = [name for name, panel in others.items() if hasattr(panel, "metadata_flow")]
    assert not leaked, f"MetadataFlowPanel still present on non-Assembly tabs: {leaked}"


def test_pipeline_spine_is_single_advance_surface(aps_app) -> None:
    """Slice B — the spine has the advance plumbing; the lane bar has no flow row."""
    assert hasattr(aps_app.pipeline_status, "_advance_btn"), "spine lost its advance button"
    assert not hasattr(aps_app, "_lane_flow_host"), "lane bar still hosts a flow-verb row"
    assert not hasattr(aps_app, "_flow_buttons"), "old always-on flow-verb buttons survive"
