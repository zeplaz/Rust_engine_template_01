"""DMCP-OVR-ARTIST-ACCEPT-001 — post-overhaul artist acceptance witness."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/art_pipeline/dmcp_ovr_artist_accept_live.json"
DELIVERABLE_REL = "src/dev/design_aps_artist_ship_review_uiux_v1.md"
PRIOR_REL = "src/dev/design_aps_artist_ship_review_20260616_v1.md"
CLOSE_REL = "debug_runs/aps_uiux_overhaul_close_live.json"
SIGNOFF_REL = "src/dev/design_aps_uiux_overhaul_signoff_v1.md"


def refresh_dmcp_ovr_artist_accept_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    deliverable = root / DELIVERABLE_REL
    signoff = root / SIGNOFF_REL
    close_path = root / CLOSE_REL
    close_machine_ok = False
    if close_path.is_file():
        close = json.loads(close_path.read_text(encoding="utf-8"))
        ban = close.get("ban_list") or {}
        close_machine_ok = bool(close.get("pytest_aps_ok")) and bool(ban.get("ui_clean"))
    doc_ok = deliverable.is_file()
    signoff_ok = signoff.is_file()
    green = doc_ok and signoff_ok and close_machine_ok
    body: dict[str, Any] = {
        "gate": "DMCP-OVR-ARTIST-ACCEPT-001",
        "program_id": "PLAN-APS-UIUX-OVERHAUL-001",
        "green": green,
        "verdict": "PASS_WITH_NOTES" if green else "PENDING",
        "ship_score": "8 / 10",
        "prior_score": "7 / 10 (DMCP-E0-ARTIST-REVERDICT-001)",
        "delta": "+1 UI/UX overhaul (text, IA, spine, layout, onboarding)",
        "deliverable": DELIVERABLE_REL,
        "supersedes": PRIOR_REL,
        "checks": {
            "deliverable_on_disk": doc_ok,
            "designer_signoff_on_disk": signoff_ok,
            "close_machine_ok": close_machine_ok,
        },
        "blocks": [],
        "unblocks": ["PLAN-APS-UIUX-OVERHAUL-001 program close"],
        "_agent_meta": {
            "schema": "dmcp_ovr_artist_accept_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "DMCP_OVR_ARTIST_ACCEPT",
            "relative_path": WITNESS_REL,
            "ritual": "BLANG:WIT-HON→Q✓ DMCP-OVR-ARTIST-ACCEPT-001",
            "agent": "designer-mcp",
        },
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body
