"""CLI entry for SUBPROGRAM pixel_conclude. prog name: pixel-conclude."""

from __future__ import annotations

import argparse
import json
import sys

from .conclude import KINDS, pixel_conclude


def cli_main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="pixel-conclude")
    parser.add_argument("--image", required=True, help="PNG path")
    parser.add_argument("--kind", required=True, choices=list(KINDS))
    parser.add_argument("--out", default=None, help="Directory for the label map and matrix JSON")
    try:
        args = parser.parse_args(argv)
    except SystemExit as exc:
        code = exc.code
        return int(code) if isinstance(code, int) else 2
    try:
        result = pixel_conclude(args.image, args.kind, out_dir=args.out)
    except (FileNotFoundError, ValueError, OSError, RuntimeError) as exc:
        print(
            json.dumps(
                {
                    "ok": False,
                    "verdict": "incomplete",
                    "honest": True,
                    "error": str(exc),
                }
            ),
            file=sys.stderr,
        )
        return 2
    print(json.dumps(result, sort_keys=True))
    if result["verdict"] == "green":
        return 0
    return 1
