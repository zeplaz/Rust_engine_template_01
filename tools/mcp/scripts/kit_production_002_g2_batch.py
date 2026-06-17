#!/usr/bin/env python3
"""MCP-P2-KIT002-G2 — roof_industrial_shed_2u production bpy + promote."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools" / "mcp" / "python"))

from rust_engine_mcp import kit_production_002  # noqa: E402


def main() -> int:
    p = argparse.ArgumentParser(description="kit_production_002 G2 roof batch")
    p.add_argument(
        "--phase",
        default="full",
        choices=["geometry", "promote", "witness", "full"],
    )
    p.add_argument("--no-promote", action="store_true")
    args = p.parse_args()
    out: dict = {"phase": args.phase}
    if args.phase in ("geometry", "full"):
        out["geometry"] = kit_production_002.run_kit_production_002_g2_geometry()
        if not out["geometry"].get("ok"):
            print(json.dumps(out, indent=2))
            return 1
    if args.phase in ("promote", "full") and not args.no_promote:
        out["promote"] = kit_production_002.promote_kit_production_002_g2_roof()
    if args.phase in ("witness", "full", "promote", "geometry"):
        out["witness"] = kit_production_002.refresh_kit_production_002_g2_witness()
    print(json.dumps(out, indent=2))
    green = bool((out.get("witness") or {}).get("green"))
    if args.phase == "promote" and out.get("promote"):
        green = True
    return 0 if green or args.phase == "geometry" else 1


if __name__ == "__main__":
    raise SystemExit(main())
