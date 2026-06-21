"""APS-STUDIO-CLOSE-001 — APS studio polish rollup witness."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.aps_uiux_onboard import refresh_aps_onboard_witness
from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/art_pipeline/aps_studio_close_live.json"


def refresh_aps_studio_close_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    onboard = refresh_aps_onboard_witness(repo=root)
    assembly_src = (root / "tools/mcp/art_pipeline_suite/assembly_panel.py").read_text(
        encoding="utf-8"
    )
    variant_src = (root / "tools/mcp/art_pipeline_suite/aps_preview_variant_state.py").is_file()
    onboard_strip = "AssemblyOnboardStrip" in assembly_src
    preview_state = (root / "tools/mcp/python/tests/test_aps_preview_variant_state.py").is_file()
    p7 = (root / "tools/mcp/python/tests/test_aps_p7_wave2_style.py").is_file()
    green = (
        bool(onboard.get("green"))
        and onboard_strip
        and variant_src
        and preview_state
        and p7
    )
    body: dict[str, Any] = {
        "gate": "APS-STUDIO-CLOSE-001",
        "green": green,
        "onboard_witness": onboard.get("green"),
        "assembly_onboard_strip": onboard_strip,
        "preview_variant_state_module": variant_src,
        "p7_style_tests_on_disk": p7,
        "depends_on": ["OVR-P55-PREVIEW-002", "OVR-P56-ONBOARD-001"],
        "_agent_meta": {
            "schema": "aps_studio_close_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "APS_STUDIO_CLOSE",
            "relative_path": WITNESS_REL,
        },
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body
