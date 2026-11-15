# Studio dashboards

Static HTML from JSON for APS, mission tools, and world-builder.

```text
python tools/studio_dash/render_report.py <report.json> -o debug_runs/studio_dash/out.html
python -m rust_engine_mcp.cli studio-dash-render <report.json> -o debug_runs/studio_dash/out.html
```

Mission / worldgen:

```python
from studio_dash.mission_adapter import emit_mission_dashboard, emit_worldgen_dashboard
emit_worldgen_dashboard({"status": "ok", "towns": 3})
```

Charter: `src/dev/plan_artist_studio_expansion_v1.md`.
