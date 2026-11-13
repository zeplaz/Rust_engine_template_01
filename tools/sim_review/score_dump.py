#!/usr/bin/env python3
"""Score a sim_run_dump.json against expected/matrix_v1.json. No green in the dump."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

_REPO = Path(__file__).resolve().parents[2]
_DEFAULT_DUMP = _REPO / "debug_runs" / "sim_run_dump.json"
_DEFAULT_MATRIX = Path(__file__).resolve().parent / "expected" / "matrix_v1.json"
_OUT_DIR = _REPO / "debug_runs" / "sim_review"


def _get(data: dict, path: str):
    cur = data
    for part in path.split("."):
        if not isinstance(cur, dict) or part not in cur:
            return None
        cur = cur[part]
    return cur


def _num(v) -> float:
    if isinstance(v, bool):
        return float(v)
    if isinstance(v, (int, float)):
        return float(v)
    raise TypeError(v)


def _scene_ok(row: dict, dump: dict) -> bool:
    want = row.get("when_scene")
    if not want:
        return True
    scene = ((dump.get("launch") or {}).get("test_scene")) or ""
    return scene in want


def _eval_rule(rule: str, dump: dict) -> tuple[bool, str]:
    counts = dump.get("counts") or {}
    if rule == "green_absent":
        ok = "green" not in dump
        return ok, "green field absent" if ok else "dump carries a green verdict"
    if rule == "veg_not_only_water":
        live = int(counts.get("veg_live_cells") or 0)
        water = int(counts.get("veg_water_cells") or 0)
        open_w = int(counts.get("water_open_cells") or 0)
        ok = live > 0 and (open_w == 0 or live > water * 0.05)
        return ok, f"live={live} veg_water={water} open_water={open_w}"
    if rule == "inst_tracks_heat":
        heat = int(counts.get("fire_heat_cells") or 0)
        inst = int(counts.get("fire_instance_rows") or 0)
        ok = (heat == 0 and inst == 0) or (heat > 0 and inst > 0)
        return ok, f"heat_cells={heat} inst={inst}"
    if rule == "sparks_not_bootstrap":
        sparks = int(counts.get("spark_rows") or 0)
        view = str(dump.get("spark_projection_view") or "")
        if sparks == 0:
            return True, "no sparks this dump"
        ok = view != "overlay_bootstrap"
        return ok, f"spark_rows={sparks} view={view or '(empty)'}"
    ai = dump.get("ai") or {}
    factions = ai.get("factions") or {}
    roster = [str(x) for x in (ai.get("roster_slots") or [])]
    if rule == "ai_roster":
        return len(roster) > 0, f"roster slots={roster}"
    if rule in ("ai_each_decided", "ai_each_issued_order"):
        key = "decisions" if rule == "ai_each_decided" else "orders_total"
        if not roster:
            return False, "no AI roster"
        per = {s: int((factions.get(s) or {}).get(key) or 0) for s in roster}
        return all(v >= 1 for v in per.values()), f"{key} per slot={per}"
    return False, f"unknown rule {rule}"


def _eval_op(op: str, actual, expect) -> bool:
    if op == "eq":
        return actual == expect
    if op == "gt":
        return _num(actual) > _num(expect)
    if op == "gte":
        return _num(actual) >= _num(expect)
    if op == "lt":
        return _num(actual) < _num(expect)
    if op == "lte":
        return _num(actual) <= _num(expect)
    raise ValueError(op)


def score(dump: dict, matrix: dict) -> dict:
    grade = dump.get("proof_grade")
    if grade != "sim_dump" or dump.get("_untrusted") is True:
        return {
            "schema": "sim_review_result_v1",
            "status": "reject",
            "summary": "Not a sim dump (fixture / untrusted live JSON). Run the sim.",
            "pass": 0,
            "fail": 1,
            "skip": 0,
            "rows": [
                {
                    "id": "dump.trusted",
                    "status": "fail",
                    "actual": grade,
                    "expect": "sim_dump",
                    "note": dump.get("_untrusted_reason") or "wrong proof_grade",
                }
            ],
        }
    rows_out = []
    passed = failed = skipped = 0
    for row in matrix.get("rows") or []:
        rid = row.get("id", "?")
        note = row.get("note", "")
        if not _scene_ok(row, dump):
            skipped += 1
            rows_out.append({"id": rid, "status": "skip", "note": "scene filter"})
            continue
        try:
            if "rule" in row:
                ok, detail = _eval_rule(str(row["rule"]), dump)
                actual = detail
                expect = row.get("rule")
            else:
                actual = _get(dump, str(row["path"]))
                expect = row.get("expect")
                if "expect_path" in row:
                    expect = _get(dump, str(row["expect_path"]))
                ok = _eval_op(str(row.get("op", "eq")), actual, expect)
                ref = f" ({row['expect_path']})" if "expect_path" in row else ""
                detail = f"{actual} {row.get('op')} {expect}{ref}"
        except Exception as exc:  # noqa: BLE001 — surface to the matrix
            ok = False
            actual = str(exc)
            expect = row.get("expect", row.get("rule"))
            detail = str(exc)
        status = "pass" if ok else "fail"
        if ok:
            passed += 1
        else:
            failed += 1
        rows_out.append(
            {
                "id": rid,
                "status": status,
                "actual": actual,
                "expect": expect,
                "note": f"{note} {detail}".strip(),
            }
        )
    status = "pass" if failed == 0 else "fail"
    return {
        "schema": "sim_review_result_v1",
        "status": status,
        "summary": f"{passed} pass / {failed} fail / {skipped} skip",
        "pass": passed,
        "fail": failed,
        "skip": skipped,
        "run_id": dump.get("run_id"),
        "tick": dump.get("tick"),
        "seed": dump.get("seed"),
        "rows": rows_out,
    }


_SPARKS = [
    ("burning", "fire: burning cells"),
    ("burnt", "fire: burnt cells"),
    ("regrowth", "fire: regrowth cells"),
    ("heat_cells", "fire: heat cells"),
    ("sites_operational", "sites operational"),
    ("produced", "units produced (cum)"),
    ("deliveries", "deliveries (cum)"),
    ("engaged", "formations engaged"),
    ("frame_ms", "frame ms"),
]


def _spark(vals: list[float], w: int = 260, h: int = 44) -> str:
    if len(vals) < 2:
        return "<em>no series</em>"
    lo, hi = min(vals), max(vals)
    span = (hi - lo) or 1.0
    pts = " ".join(
        f"{i * w / (len(vals) - 1):.1f},{h - 3 - (v - lo) / span * (h - 6):.1f}" for i, v in enumerate(vals)
    )
    return (
        f"<svg width='{w}' height='{h}' style='background:#1b1b1b'><polyline fill='none' stroke='#7cf' "
        f"stroke-width='1.5' points='{pts}'/></svg> <small>min {lo:.4g} max {hi:.4g} last {vals[-1]:.4g}</small>"
    )


def _kv(title: str, d: dict | None) -> str:
    if not isinstance(d, dict):
        return ""
    rows = "".join(
        f"<tr><td>{k}</td><td>{json.dumps(v) if isinstance(v, (dict, list)) else v}</td></tr>"
        for k, v in d.items()
    )
    return f"<div class='sec'><h3>{title}</h3><table>{rows}</table></div>"


def _sections(dump: dict | None) -> str:
    if not dump:
        return ""
    out = ["<h2>sections</h2><div class='grid'>"]
    for key, title in (
        ("construction", "Construction"),
        ("economy", "Production / economy"),
        ("logistics", "Logistics"),
        ("combat", "Fronts / combat"),
        ("fire", "Fire / ecology (first/last/min/max)"),
        ("perf", "Perf (frame ms)"),
    ):
        out.append(_kv(title, dump.get(key)))
    ai = dump.get("ai") or {}
    ai_rows = "".join(
        f"<tr><td>slot {slot}</td><td>{f.get('decisions')}</td><td>{f.get('orders_logistics')}</td>"
        f"<td>{f.get('orders_deployment')}</td><td>{f.get('orders_build')}</td><td>{f.get('last_goal')}</td></tr>"
        for slot, f in (ai.get("factions") or {}).items()
    )
    out.append(
        "<div class='sec'><h3>AI factions</h3><table><tr><th>slot</th><th>decisions</th><th>logistics</th>"
        f"<th>deployment</th><th>build</th><th>last goal</th></tr>{ai_rows}</table></div>"
    )
    out.append("</div><h2>time series</h2><table>")
    hist = dump.get("history") or []
    for key, title in _SPARKS:
        out.append(f"<tr><td>{title}</td><td>{_spark([float(h.get(key) or 0) for h in hist])}</td></tr>")
    out.append("</table>")
    return "".join(out)


def _html(result: dict, dump: dict | None = None) -> str:
    rows = []
    for r in result.get("rows") or []:
        cls = r.get("status", "")
        rows.append(
            "<tr class='{cls}'><td>{id}</td><td>{st}</td><td>{act}</td><td>{exp}</td><td>{note}</td></tr>".format(
                cls=cls,
                id=r.get("id"),
                st=r.get("status"),
                act=r.get("actual"),
                exp=r.get("expect"),
                note=r.get("note"),
            )
        )
    payload = json.dumps(result, indent=2)
    sections = _sections(dump)
    return f"""<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>sim review {result.get('run_id') or ''}</title>
<style>
body {{ font: 14px/1.4 sans-serif; margin: 24px; background: #111; color: #eee; }}
h1 {{ font-size: 20px; }}
.pass {{ color: #8d8; }} .fail {{ color: #f88; }} .skip {{ color: #aaa; }}
table {{ border-collapse: collapse; width: 100%; }}
td, th {{ border-bottom: 1px solid #333; padding: 6px 8px; text-align: left; vertical-align: top; }}
tr.fail {{ background: #2a1111; }} tr.pass {{ background: #112211; }}
.grid {{ display: flex; flex-wrap: wrap; gap: 16px; }} .sec {{ min-width: 320px; }} .sec table {{ width: auto; }}
h3 {{ margin: 8px 0 4px; font-size: 15px; }}
pre {{ background: #1b1b1b; padding: 12px; overflow: auto; }}
</style></head><body>
<h1>Sim review — {result.get('status')} · {result.get('summary')}</h1>
<p>run {result.get('run_id')} · tick {result.get('tick')} · seed {result.get('seed')}</p>
<p>Dump is counts only. This table is the verdict. Fixture <code>*_live.json</code> files are not a run.</p>
<table><thead><tr><th>id</th><th>status</th><th>actual</th><th>expect</th><th>note</th></tr></thead>
<tbody>
{''.join(rows)}
</tbody></table>
{sections}
<h2>raw</h2><pre>{payload}</pre>
</body></html>
"""


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description="Score sim_run_dump.json")
    p.add_argument("dump", nargs="?", default=str(_DEFAULT_DUMP))
    p.add_argument("--matrix", default=str(_DEFAULT_MATRIX))
    p.add_argument("--out", default=str(_OUT_DIR))
    args = p.parse_args(argv)
    dump_path = Path(args.dump)
    if not dump_path.is_file():
        print(f"no dump at {dump_path} — run the sim in debug (RUST_ENGINE_SIM_DUMP=1)", file=sys.stderr)
        return 2
    dump = json.loads(dump_path.read_text(encoding="utf-8"))
    matrix = json.loads(Path(args.matrix).read_text(encoding="utf-8"))
    result = score(dump, matrix)
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    (out / "matrix.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    (out / "dashboard.html").write_text(_html(result, dump), encoding="utf-8")
    print(result["summary"])
    for row in result["rows"]:
        print(f"  {row['status']:4}  {row['id']:28}  {row.get('actual')}")
    print(f"wrote {out / 'dashboard.html'}")
    return 0 if result["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
