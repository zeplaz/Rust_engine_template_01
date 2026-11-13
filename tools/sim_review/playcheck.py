#!/usr/bin/env python3
"""One real-world play check: build, run the game, capture, score, summarise.

Replaces hand-run build/launch/grep loops. Prints a short summary and writes
debug_runs/sim_review/playcheck_summary.md. The game process is always killed.

    python tools/sim_review/playcheck.py                 # build debug, seed 14046650793090929382
    python tools/sim_review/playcheck.py --no-build --frame 1800 --secs 300
"""

from __future__ import annotations

import argparse
import os
import re
import json
import subprocess
import sys
import time
from pathlib import Path

_REPO = Path(__file__).resolve().parents[2]
_OUT = _REPO / "debug_runs" / "sim_review"
_CAPTURES = _REPO / "debug_runs" / "captures"
_DUMP = _REPO / "debug_runs" / "sim_run_dump.json"
_EXE = _REPO / "target" / "debug" / "proc_A_dine01.exe"
_ANSI = re.compile(r"\x1b\[[0-9;]*m")
_LEVEL = re.compile(r"\s(ERROR|WARN)\s+([\w:]+):\s*(.*)")


def say(msg: str) -> None:
    print(f"[playcheck {time.strftime('%H:%M:%S')}] {msg}", flush=True)


def build() -> None:
    say("cargo build (debug) ... first build can take several minutes")
    r = subprocess.run(
        ["cargo", "build", "--bin", "proc_A_dine01", "--message-format=short"],
        cwd=_REPO, capture_output=True, text=True,
    )
    errors = [l for l in r.stderr.splitlines() if l.startswith("error")]
    if r.returncode != 0:
        print("BUILD FAILED")
        print("\n".join(errors[:20]))
        sys.exit(1)
    say("build ok")


def dump_tick(since: float) -> int:
    try:
        if _DUMP.stat().st_mtime < since:
            return 0
        return int(json.loads(_DUMP.read_text(encoding="utf-8")).get("tick") or 0)
    except (OSError, ValueError):
        return 0


def run_game(seed: str, frame: int, secs: int, scene: str, ticks: int) -> list[str]:
    env = dict(os.environ)
    env.update({
        "RUST_ENGINE_SIM_DUMP": "1",
        "WORLD_GEN_SEED": seed,
        "VFX_CAPTURE": "1",
        "VFX_CAPTURE_FRAME": str(frame),
        "VFX_CAPTURE_NO_MACRO_ZOOM": "1",
        "RUST_BACKTRACE": "1",
    })
    capture = _CAPTURES / "tactical_rtt_live.png"
    started = time.time()
    log_path = _OUT / "playcheck_run.log"
    _OUT.mkdir(parents=True, exist_ok=True)
    with open(log_path, "w", encoding="utf-8", errors="replace") as log:
        proc = subprocess.Popen(
            [str(_EXE), "--test", scene], cwd=_REPO, env=env,
            stdout=log, stderr=subprocess.STDOUT,
        )
        say(f"game started pid={proc.pid}; waiting for capture (limit {secs}s), log {log_path}")
        last_note = started
        try:
            while time.time() - started < secs:
                if time.time() - last_note > 30:
                    last_note = time.time()
                    say(f"  {int(time.time() - started)}s, log {log_path.stat().st_size // 1024} KiB")
                if proc.poll() is not None:
                    break
                # Captured this run AND the dump reached the requested sim tick: done.
                if capture.exists() and capture.stat().st_mtime > started + 1 and dump_tick(started) >= ticks:
                    time.sleep(3)
                    break
                time.sleep(2)
        finally:
            say(f"stopping game (exit={proc.poll()})")
            if proc.poll() is None:
                proc.kill()
                proc.wait(timeout=30)
    text = log_path.read_text(encoding="utf-8", errors="replace")
    return [_ANSI.sub("", l) for l in text.splitlines()]


def summarise_log(lines: list[str]) -> dict:
    panics = [l.strip() for l in lines if "panicked at" in l or "Encountered a panic" in l]
    levels: dict[str, dict[str, int]] = {"ERROR": {}, "WARN": {}}
    for l in lines:
        m = _LEVEL.search(l)
        if not m:
            continue
        lvl, target, msg = m.groups()
        key = f"{target}: {re.sub(r'[0-9]+(\.[0-9]+)?', '#', msg)[:140]}"
        levels[lvl][key] = levels[lvl].get(key, 0) + 1
    return {"panics": panics, "levels": levels}



def score() -> list[str]:
    if not _DUMP.exists():
        return ["no sim_run_dump.json written (did the game reach the simulation?)"]
    r = subprocess.run(
        [sys.executable, str(Path(__file__).parent / "score_dump.py")],
        cwd=_REPO, capture_output=True, text=True,
    )
    out = (r.stdout + r.stderr).strip().splitlines()
    return out[-70:]


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--no-build", action="store_true")
    ap.add_argument("--seed", default="14046650793090929382")
    ap.add_argument("--frame", type=int, default=1200, help="capture frame")
    ap.add_argument("--secs", type=int, default=420, help="hard time limit")
    ap.add_argument("--ticks", type=int, default=3000, help="sim tick the dump must reach before stopping")
    ap.add_argument("--scene", default="visual")
    a = ap.parse_args()

    if not a.no_build:
        build()
    t0 = time.time()
    lines = run_game(a.seed, a.frame, a.secs, a.scene, a.ticks)
    s = summarise_log(lines)
    scored = score()

    md = [f"# playcheck  seed={a.seed} frame={a.frame} scene={a.scene} ran={time.time() - t0:.0f}s", ""]
    md.append(f"panics: {len(s['panics'])}")
    md += [f"  {p[:200]}" for p in s["panics"][:5]]
    perf = {}
    try:
        perf = json.loads(_DUMP.read_text(encoding="utf-8")).get("perf") or {}
    except (OSError, ValueError):
        pass
    md.append(f"perf (ms/frame, from dump): {perf or 'no dump'}")
    for lvl in ("ERROR", "WARN"):
        items = sorted(s["levels"][lvl].items(), key=lambda kv: -kv[1])
        md.append(f"{lvl}: {sum(s['levels'][lvl].values())} total, {len(items)} distinct")
        md += [f"  {n:>5}x {k}" for k, n in items[:8]]
    md.append("score:")
    md += [f"  {l}" for l in scored]
    md.append(f"captures: {_CAPTURES / 'tactical_rtt_live.png'} | {_CAPTURES / 'window_live.png'}")
    text = "\n".join(md)
    (_OUT / "playcheck_summary.md").write_text(text, encoding="utf-8")
    print(text)


_LOCK = _OUT / "playcheck.lock"


def locked_main() -> None:
    """One playcheck at a time: runs share the exe, dump, captures and summary."""
    _OUT.mkdir(parents=True, exist_ok=True)
    if _LOCK.exists() and time.time() - _LOCK.stat().st_mtime < 3600:
        print(f"another playcheck is running ({_LOCK.read_text().strip()}); "
              f"wait for it, or delete {_LOCK} if it is stale")
        sys.exit(2)
    _LOCK.write_text(f"pid={os.getpid()} started={time.strftime('%H:%M:%S')}")
    try:
        main()
    finally:
        _LOCK.unlink(missing_ok=True)


if __name__ == "__main__":
    locked_main()
