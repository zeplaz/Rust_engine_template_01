#!/usr/bin/env python3
"""kit_industrial_west_production_001 — PG-MODULE-AUDIT-002 gap closure batch."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools" / "mcp" / "python"))

from rust_engine_mcp.pg_module_audit_002 import (  # noqa: E402
    AUDIT_WITNESS_JSON,
    BATCH_ID,
    run_pg_module_audit_002,
    write_gap_artifacts,
)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--phase",
        default="full",
        choices=["sync", "specs", "geometry", "promote", "full", "all"],
    )
    parser.add_argument("--priorities", default="P0,P1,P2")
    parser.add_argument("--no-blender", action="store_true")
    parser.add_argument("--sync-only", action="store_true", help="Write specs + manifest only")
    args = parser.parse_args()

    priorities = tuple(p.strip() for p in args.priorities.split(",") if p.strip())
    phase = "sync" if args.sync_only else args.phase
    result = run_pg_module_audit_002(
        phase=phase,
        priorities=priorities,
        use_blender=not args.no_blender,
    )
    print(json.dumps(result, indent=2))
    witness = ROOT / AUDIT_WITNESS_JSON
    if not witness.is_file() and phase == "sync":
        write_gap_artifacts(priorities=priorities)
    return 0 if result.get("ok") else 1


if __name__ == "__main__":
    raise SystemExit(main())
