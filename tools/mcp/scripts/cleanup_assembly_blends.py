"""
Delete junk staging assembly .blend files (rig+truck embedded) and rebuild ASSEMBLY-only blends.

  python tools/mcp/scripts/cleanup_assembly_blends.py
  python tools/mcp/scripts/cleanup_assembly_blends.py --rebuild-only
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
if str(ROOT / "tools" / "mcp" / "python") not in sys.path:
    sys.path.insert(0, str(ROOT / "tools" / "mcp" / "python"))

from rust_engine_mcp.blender_runner import build_iso_rig_blend  # noqa: E402
from rust_engine_mcp.tile_pipeline import assembly_build_run  # noqa: E402

ASSEMBLIES = ROOT / "assets" / "staging" / "assemblies"


def _purge_blends() -> list[str]:
    removed: list[str] = []
    for path in sorted(ASSEMBLIES.iterdir()):
        name = path.name
        if name.endswith(".blend") or name.endswith(".blend1"):
            path.unlink(missing_ok=True)
            removed.append(str(path.relative_to(ROOT)))
    return removed


def _rebuild_from_snapshots() -> list[dict]:
    results: list[dict] = []
    for snap in sorted(ASSEMBLIES.glob("*.json")):
        if snap.name.endswith(".example.json"):
            continue
        rel = str(snap.relative_to(ROOT)).replace("\\", "/")
        results.append({"snapshot": rel, **assembly_build_run(snap)})
    return results


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rebuild-only", action="store_true")
    parser.add_argument("--skip-rig", action="store_true")
    args = parser.parse_args()

    if not args.rebuild_only:
        removed = _purge_blends()
        print(f"removed {len(removed)} blend backup(s)")
        for r in removed:
            print(f"  {r}")

    if not args.skip_rig:
        rig = build_iso_rig_blend()
        print("iso_rig", rig.get("ok"), rig.get("blend_path"))
        if not rig.get("ok"):
            print(rig.get("log", "")[-2000:])
            return 1

    results = _rebuild_from_snapshots()
    failed = [r for r in results if not r.get("ok")]
    for r in results:
        print(r.get("snapshot"), r.get("ok"), r.get("blend_path"))
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
