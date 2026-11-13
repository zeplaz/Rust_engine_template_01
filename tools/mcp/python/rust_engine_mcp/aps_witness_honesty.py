"""Legacy debug_runs/*_live.json writer used by a few remaining module-level helpers.

The witness-honesty gate was retired with the witness system; review goes through
tools/sim_review/. This only writes the JSON body with a small provenance envelope.
"""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root


def write_aps_live_witness(
    body: dict[str, Any],
    rel_path: str,
    *,
    schema: str,
    profile: str,
    source_system: str = "aps_witness",
    ritual: str | None = None,
    exit_predicate_must: list[dict[str, Any]] | None = None,
    repo: Path | None = None,
) -> dict[str, Any]:
    root = repo or repo_root()
    rel = rel_path.replace("\\", "/")
    payload = dict(body)
    payload["_agent_meta"] = {
        "schema": schema,
        "profile": profile,
        "relative_path": rel,
        "source_system": source_system,
        "written_at_epoch_secs": int(time.time()),
    }
    out = root / rel
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    payload["written"] = rel
    return payload
