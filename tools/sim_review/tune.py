#!/usr/bin/env python3
"""Hill-climb FrontBalance constants against multi-seed matrix scores (T7 / LEARN-01).

CQ-78: T7 is **not built** — only `--dry-run` (default) is supported. Does not write
assets/ or src/. A future Accept step needs T6 multi-seed dumps + the verdict rule.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

_REPO = Path(__file__).resolve().parents[2]
_OUT = _REPO / "debug_runs" / "sim_review"
_BALANCE_RON = _REPO / "assets" / "configs" / "combat" / "front_balance_v1.ron"

# ≤ 6 named knobs (documentation whitelist for the first hill-climb).
WHITELIST: list[tuple[str, float, float, float]] = [
    # name, default_hint, lo, hi
    ("combat.k_attrition", 0.002, 0.0005, 0.008),
    ("combat.morale_loss_per_strength", 1.5, 0.5, 3.0),
    ("combat.attack_mult", 1.2, 0.8, 1.6),
    ("combat.defend_mult", 1.0, 0.7, 1.4),
    ("pressure.w_threat", 1.0, 0.4, 1.6),
    ("control.stamp_gain_per_s", 0.03, 0.01, 0.08),
]

VERDICT = (
    "accept a step only if |mean_after − mean_before| > 2 × max(stderr_before, stderr_after)"
)


def dry_run() -> int:
    print("tune.py dry-run — hill climbing with random restarts (not launched)")
    print("objective: mean matrix score from score_dump.py --multi (T6)")
    print(f"verdict: {VERDICT}")
    print(f"proposal out: {_OUT / 'tune_proposal.ron'} (not written in dry-run)")
    print(f"source defaults: {_BALANCE_RON}")
    print("whitelist (name  lo..hi  hint_default):")
    for name, hint, lo, hi in WHITELIST:
        print(f"  {name:36}  [{lo}, {hi}]  ~{hint}")
    print("planned perturbations: ±10% then ±25% of span around current, 3 random restarts")
    print("no annealing / genetic until one accepted hill-climb step exists")
    return 0


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description="FrontBalance constant tuner (proposal only)")
    p.add_argument("--dry-run", action="store_true", help="print whitelist + plan; do not launch game")
    args = p.parse_args(argv)
    if args.dry_run:
        return dry_run()
    print("tune.py: pass --dry-run (live hill-climb needs T6 dump_<seed>.json set)", file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
