#!/usr/bin/env python3
"""One-shot frame + massing QC HTML (sim_review pattern: JSON in → dashboard out).

Writes ``debug_runs/studio_dash/frame_qc_smoke.html`` and
``debug_runs/studio_dash/massing_qc_smoke.html`` from checked-in examples.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    from .frame_qc import frame_qc_view
    from .massing_qc import massing_qc_view
    from .render_report import write_report
except ImportError:
    from frame_qc import frame_qc_view  # type: ignore
    from massing_qc import massing_qc_view  # type: ignore
    from render_report import write_report  # type: ignore


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _out_dir() -> Path:
    p = _repo_root() / "debug_runs" / "studio_dash"
    p.mkdir(parents=True, exist_ok=True)
    return p


def run_smoke(
    *,
    frame_json: Path | None = None,
    massing_json: Path | None = None,
    out_dir: Path | None = None,
) -> tuple[Path, Path]:
    root = _repo_root()
    frame_json = frame_json or (
        root / "tools" / "mcp" / "schemas" / "examples" / "param_frame_rail_warehouse_v1.json"
    )
    massing_json = massing_json or (
        root
        / "tools"
        / "mcp"
        / "schemas"
        / "examples"
        / "massing_preview_pilot_rail_warehouse_v1.json"
    )
    dest = out_dir or _out_dir()

    frame_raw = json.loads(frame_json.read_text(encoding="utf-8"))
    massing_raw = json.loads(massing_json.read_text(encoding="utf-8"))

    frame_path = write_report(
        frame_qc_view(frame_raw),
        dest / "frame_qc_smoke.html",
        title="param_frame QC (smoke)",
    )
    massing_path = write_report(
        massing_qc_view(massing_raw),
        dest / "massing_qc_smoke.html",
        title="massing_preview QC (smoke)",
    )
    (dest / "frame_qc_smoke.json").write_text(
        json.dumps(frame_qc_view(frame_raw), indent=2), encoding="utf-8"
    )
    (dest / "massing_qc_smoke.json").write_text(
        json.dumps(massing_qc_view(massing_raw), indent=2), encoding="utf-8"
    )
    return frame_path, massing_path


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description="Studio dash QC smoke (frame + massing HTML)")
    p.add_argument("--frame", type=Path, default=None, help="param_frame JSON path")
    p.add_argument("--massing", type=Path, default=None, help="massing_preview JSON path")
    p.add_argument("-o", "--out", type=Path, default=None, help="output directory")
    args = p.parse_args(argv)
    frame_p, massing_p = run_smoke(frame_json=args.frame, massing_json=args.massing, out_dir=args.out)
    print(f"wrote {frame_p}")
    print(f"wrote {massing_p}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())