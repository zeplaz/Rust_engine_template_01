"""OVR-P56-ONBOARD-001 — first-run onboarding prefs + witness."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

ONBOARDING_PREFS_KEY = "onboarding_seen_v1"
WITNESS_REL = "debug_runs/aps_uiux_onboard_live.json"


def onboarding_prefs_path(*, repo: Path | None = None) -> Path:
    return (repo or repo_root()) / "debug_runs/aps_ui_prefs.json"


def load_onboarding_seen(*, repo: Path | None = None) -> bool:
    path = onboarding_prefs_path(repo=repo)
    if not path.is_file():
        return False
    try:
        body = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    return bool(body.get(ONBOARDING_PREFS_KEY))


def mark_onboarding_seen(*, repo: Path | None = None) -> None:
    path = onboarding_prefs_path(repo=repo)
    path.parent.mkdir(parents=True, exist_ok=True)
    body: dict[str, Any] = {}
    if path.is_file():
        try:
            raw = json.loads(path.read_text(encoding="utf-8"))
            if isinstance(raw, dict):
                body = raw
        except (OSError, json.JSONDecodeError):
            body = {}
    body[ONBOARDING_PREFS_KEY] = True
    path.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")


def refresh_aps_onboard_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    meta = (root / "tools/mcp/art_pipeline_suite/metadata_flow_panel.py").read_text(encoding="utf-8")
    collapsed_default = "return False" in meta and "_initial_expanded" in meta
    body: dict[str, Any] = {
        "gate_id": "OVR-P56-ONBOARD-001",
        "green": collapsed_default,
        "metadata_collapsed_default": collapsed_default,
        "onboarding_prefs_key": ONBOARDING_PREFS_KEY,
        "onboarding_seen": load_onboarding_seen(repo=root),
        "_agent_meta": {
            "schema": "aps_uiux_onboard_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "OVR_P56_ONBOARD",
            "relative_path": WITNESS_REL,
        },
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body
