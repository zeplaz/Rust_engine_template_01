# Studio dashboards

Static HTML from JSON for APS, mission tools, and world-builder — same loop as `tools/sim_review/` (JSON artifact → `debug_runs/studio_dash/*.html`).

## Frame QC + massing QC

| Kind | Helper | Example input |
|:---|:---|:---|
| **Param frame** | `frame_qc.frame_qc_view` | `tools/mcp/schemas/examples/param_frame_rail_warehouse_v1.json` |
| **Massing preview** | `massing_qc.massing_qc_view` | `tools/mcp/schemas/examples/massing_preview_pilot_rail_warehouse_v1.json` |
| **HTML** | `render_report.write_report` | self-contained dark-theme table report |

```text
python tools/studio_dash/render_report.py <report.json> -o debug_runs/studio_dash/out.html
python tools/studio_dash/qc_smoke.py
python -m rust_engine_mcp.cli studio-dash-render <report.json> -o debug_runs/studio_dash/out.html
```

Smoke (checked-in examples → HTML):

```powershell
python tools/studio_dash/qc_smoke.py
# wrote debug_runs/studio_dash/frame_qc_smoke.html
# wrote debug_runs/studio_dash/massing_qc_smoke.html
```

Mission / worldgen / QC adapters:

```python
from studio_dash.mission_adapter import (
    emit_frame_qc_dashboard,
    emit_massing_qc_dashboard,
    emit_mission_dashboard,
    emit_worldgen_dashboard,
)
emit_frame_qc_dashboard(payload)  # ParamFrame dict
emit_massing_qc_dashboard(massing_preview_dict)
```

Tests: `python -m pytest tools/studio_dash/tests/test_render_report.py -q`

Voxel presentation demo: [`demo_massing_override.md`](demo_massing_override.md) · parent plan slice **S3** (`plan_ref_books_upgrade_v3.md`).

Charter: `src/dev/plan_artist_studio_expansion_v1.md` · dual-cite **IO-09** · **ECS-15** · **PCG-06** + `asset_pipeline` · `shape_grammars`.