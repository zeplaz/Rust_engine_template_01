#!/usr/bin/env python3
"""One real-world play check: build, run the game, capture, score, summarise.

Replaces hand-run build/launch/grep loops. Prints a short summary and writes
debug_runs/sim_review/playcheck_summary.md. The game process is always killed.

    python tools/sim_review/playcheck.py                 # build debug, seed 14046650793090929382
    python tools/sim_review/playcheck.py --no-build --frame 1800 --secs 300
    python tools/sim_review/playcheck.py --release        # release + `sim_dump` feature: real frame times

Frame time is always tracked: the dump's perf section (avg/p50/p95/p99/max, hitches, worst hitch
frames with their top PerfScopes) is scored with build-specific thresholds and drawn on the dashboard.
"""

from __future__ import annotations

import argparse
import hashlib
import os
import re
import json
import shutil
import subprocess
import sys
import time
from pathlib import Path

_REPO = Path(__file__).resolve().parents[2]
_OUT = _REPO / "debug_runs" / "sim_review"
_CAPTURES = _REPO / "debug_runs" / "captures"
_DUMP = _REPO / "debug_runs" / "sim_run_dump.json"
_ANSI = re.compile(r"\x1b\[[0-9;]*m")
_LEVEL = re.compile(r"\s(ERROR|WARN)\s+([\w:]+):\s*(.*)")


def _cargo_target_dir() -> Path:
    """Honor CARGO_TARGET_DIR (sandbox / CI) before the repo-local target/."""
    env = os.environ.get("CARGO_TARGET_DIR")
    if env:
        return Path(env)
    return _REPO / "target"


def _exe_path(release: bool) -> Path:
    root = _cargo_target_dir()
    return root / ("release" if release else "debug") / "proc_A_dine01.exe"


def say(msg: str) -> None:
    print(f"[playcheck {time.strftime('%H:%M:%S')}] {msg}", flush=True)


def build(release: bool) -> None:
    say(f"cargo build ({'release, sim_dump' if release else 'debug'}) ... first build can take several minutes")
    cmd = ["cargo", "build", "--bin", "proc_A_dine01", "--message-format=short"]
    if release:
        # Shipped release builds have no dump; the non-default `sim_dump` feature adds it back.
        cmd += ["--release", "--features", "sim_dump"]
    r = subprocess.run(cmd, cwd=_REPO, capture_output=True, text=True)
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


def run_game(exe: Path, seed: str, frame: int, secs: int, scene: str, ticks: int, min_secs: int) -> list[str]:
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
        # --stay-open: FullCapture must not AppExit before dump ticks / playcheck kill.
        proc = subprocess.Popen(
            [str(exe), "--test", scene, "--stay-open"], cwd=_REPO, env=env,
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
                if (capture.exists() and capture.stat().st_mtime > started + 1 and dump_tick(started) >= ticks
                        and time.time() - started >= min_secs):
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



def perf_lines(perf: dict) -> list[str]:
    if not perf:
        return ["perf: no dump"]
    f = lambda k: float(perf.get(k) or 0)
    out = [
        f"perf [{perf.get('build')}] frames={perf.get('samples')} avg={f('avg_ms'):.1f} p50={f('p50_ms'):.1f} "
        f"p95={f('p95_ms'):.1f} p99={f('p99_ms'):.1f} max={f('max_ms'):.1f} ms | hitches(>{f('hitch_threshold_ms'):.0f}ms)="
        f"{perf.get('hitches')} ({f('hitches_per_min'):.1f}/min) stalls(>1s)={perf.get('stalls')}"
    ]
    for h in (perf.get("worst_hitches") or [])[:5]:
        scopes = ", ".join(f"{s['label']} {s['ms']:.1f}" for s in h.get("scopes") or [])
        out.append(f"  hitch frame={h.get('frame')} tick={h.get('tick')} dt={h.get('dt_ms', 0):.1f} "
                   f"main={h.get('main_ms', 0):.1f} outside_main={h.get('outside_main_ms', 0):.1f} [{scopes}]")
    return out


def score(since: float) -> list[str]:
    if not _DUMP.exists():
        return ["no sim_run_dump.json written (did the game reach the simulation?)"]
    try:
        if _DUMP.stat().st_mtime < since:
            return [
                f"stale sim_run_dump.json (mtime before this run); "
                f"game exited without a fresh dump — not scoring"
            ]
    except OSError:
        return ["could not stat sim_run_dump.json"]
    r = subprocess.run(
        [sys.executable, str(Path(__file__).parent / "score_dump.py")],
        cwd=_REPO, capture_output=True, text=True,
    )
    out = (r.stdout + r.stderr).strip().splitlines()
    return out[-70:]


def derive_seeds(base: str, n: int) -> list[str]:
    """Reproducible seed set: base, then sha256(base, i) truncated to u64 decimal."""
    out = [base]
    for i in range(1, n):
        digest = hashlib.sha256(f"{base}:{i}".encode("utf-8")).digest()
        out.append(str(int.from_bytes(digest[:8], "little")))
    return out[:n]


def run_one(exe: Path, seed: str, frame: int, secs: int, scene: str, ticks: int, min_secs: int) -> tuple[list[str], list[str]]:
    t0 = time.time()
    if _DUMP.exists():
        try:
            _DUMP.unlink()
        except OSError:
            pass
    lines = run_game(exe, seed, frame, secs, scene, ticks, min_secs)
    scored = score(t0)
    return lines, scored


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--no-build", action="store_true")
    ap.add_argument("--seed", default="14046650793090929382")
    ap.add_argument(
        "--seeds",
        type=int,
        default=1,
        help="run N seeds (default 1). Seed i>0 = sha256(base:i)→u64. Docs recommend N=3.",
    )
    ap.add_argument("--frame", type=int, default=1200, help="capture frame")
    ap.add_argument("--secs", type=int, default=420, help="hard time limit")
    ap.add_argument("--ticks", type=int, default=3000, help="sim tick the dump must reach before stopping")
    ap.add_argument("--scene", default="visual")
    ap.add_argument("--release", action="store_true", help="build/run release with the sim_dump feature")
    ap.add_argument("--exe", help="run this binary instead (implies --no-build)")
    ap.add_argument("--min-secs", type=int, default=None,
                    help="run at least this long (default 360 s: with opt-level 1 both builds reach --ticks in ~1 min, "
                         "before game-time loops such as weather solves and production have run)")
    a = ap.parse_args()
    min_secs = a.min_secs if a.min_secs is not None else 360
    n_seeds = max(1, int(a.seeds))

    exe = Path(a.exe) if a.exe else _exe_path(a.release)
    if not a.no_build and not a.exe:
        build(a.release)
    if not exe.is_file():
        print(f"missing exe: {exe} (CARGO_TARGET_DIR={os.environ.get('CARGO_TARGET_DIR')!r})")
        sys.exit(1)
    say(f"using exe {exe}")
    seeds = derive_seeds(a.seed, n_seeds)
    _OUT.mkdir(parents=True, exist_ok=True)

    if n_seeds == 1:
        t0 = time.time()
        # Drop stale dump so an early capture / crash cannot score a previous run.
        if _DUMP.exists():
            try:
                _DUMP.unlink()
            except OSError:
                pass
        lines = run_game(exe, seeds[0], a.frame, a.secs, a.scene, a.ticks, min_secs)
        s = summarise_log(lines)
        scored = score(t0)

        md = [f"# playcheck  {'release' if a.release else 'debug'} exe={exe.name} seed={seeds[0]} frame={a.frame} "
              f"scene={a.scene} ran={time.time() - t0:.0f}s", ""]
        md.append(f"panics: {len(s['panics'])}")
        md += [f"  {p[:200]}" for p in s["panics"][:5]]
        perf = {}
        try:
            perf = json.loads(_DUMP.read_text(encoding="utf-8")).get("perf") or {}
        except (OSError, ValueError):
            pass
        md += perf_lines(perf)
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
        return

    # Multi-seed: build once above; each run uses --no-build semantics and keeps dump_<seed>.json.
    say(f"multi-seed N={n_seeds}: {seeds}")
    summaries: list[str] = [
        f"# playcheck multi-seed N={n_seeds} base={a.seed}",
        "Verdict: a change is real only if |mean_after − mean_before| > 2 × max(stderr_before, stderr_after).",
        "",
    ]
    for seed in seeds:
        say(f"seed {seed}")
        lines, scored = run_one(exe, seed, a.frame, a.secs, a.scene, a.ticks, min_secs)
        s = summarise_log(lines)
        if _DUMP.exists():
            dest = _OUT / f"dump_{seed}.json"
            shutil.copy2(_DUMP, dest)
            say(f"kept {dest}")
        summaries.append(f"## seed {seed}")
        summaries.append(f"panics: {len(s['panics'])}")
        summaries += [f"  {l}" for l in scored[-20:]]
        summaries.append("")
    r = subprocess.run(
        [sys.executable, str(Path(__file__).parent / "score_dump.py"), "--multi", str(_OUT)],
        cwd=_REPO, capture_output=True, text=True,
    )
    summaries.append("## multi score")
    summaries += (r.stdout + r.stderr).strip().splitlines()
    text = "\n".join(summaries)
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
