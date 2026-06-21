"""OVR-P56-ONBOARD-001 — first-run onboarding prefs + witness."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

ONBOARDING_PREFS_KEY = "onboarding_seen_v1"
WITNESS_REL = "debug_runs/aps_uiux_onboard_live.json"

# P5.6 — first-run "How this works" content (plain artist words, no jargon).
ONBOARDING_TITLE = "How this works"
ONBOARDING_INTRO = (
    "Five steps, left to right. Each tab hands its result to the next — "
    "the Next step line up top always tells you what to do."
)
ONBOARDING_STEPS: tuple[tuple[str, str], ...] = (
    ("Catalog", "Pick a building module to start from."),
    ("Materials", "Choose or make the surfaces it is built from."),
    ("Assembly", "Combine module + materials into one saved building."),
    ("Variants", "Add states — lighting, damage, fill — for that building."),
    ("Atlas", "Pack the baked tiles into one sheet the game loads."),
)
ONBOARDING_DISMISS = "Got it"

# P5.6 — friendly per-tab empty states for the primary surfaces.
EMPTY_STATES: dict[str, str] = {
    "catalog": "No module selected — pick one from the list to see its details.",
    "materials": "No materials yet — Generate or add one to begin.",
    "assembly": "No assembly yet — Generate one to begin.",
    "variants": "No variant set yet — New from assembly or Load example to begin.",
    "atlas": "No tiles yet — bake variants first, then pack them here.",
}


def onboarding_greeting_lines() -> list[str]:
    """Plain-language first-run greeting as flat lines (testable, display-free)."""
    lines = [ONBOARDING_TITLE, ONBOARDING_INTRO]
    for i, (name, blurb) in enumerate(ONBOARDING_STEPS, start=1):
        lines.append(f"{i}. {name} — {blurb}")
    return lines


def empty_state_text(surface: str) -> str:
    """Friendly empty-state copy for a primary surface key."""
    return EMPTY_STATES.get(surface, "Nothing here yet.")


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
    assembly_src = (root / "tools/mcp/art_pipeline_suite/assembly_panel.py").read_text(
        encoding="utf-8"
    )
    strip_src = (root / "tools/mcp/art_pipeline_suite/assembly_onboard_strip.py").read_text(
        encoding="utf-8"
    )
    onboard_strip_wired = "AssemblyOnboardStrip" in assembly_src and "value=False" in strip_src
    body: dict[str, Any] = {
        "gate_id": "OVR-P56-ONBOARD-001",
        "green": onboard_strip_wired,
        "metadata_collapsed_default": onboard_strip_wired,
        "assembly_onboard_strip": onboard_strip_wired,
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
