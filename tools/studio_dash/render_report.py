"""Shared static HTML report kit for APS / mission / world-builder tools.

No Grafana — self-contained HTML from JSON (sim_review pattern).
"""

from __future__ import annotations

import argparse
import html
import json
import sys
from pathlib import Path
from typing import Any


def _esc(v: Any) -> str:
    return html.escape(str(v))


def render_report(
    data: dict[str, Any],
    *,
    title: str | None = None,
) -> str:
    title = title or str(data.get("title") or data.get("validator") or "Studio report")
    status = data.get("status") or data.get("verdict") or data.get("summary") or ""
    rows: list[str] = []

    # Flat key/value summary
    skip = {
        "raw_log",
        "raw_log_path",
        "issues",
        "errors",
        "warnings",
        "metrics",
        "tasks",
        "massing_columns",
        "floors_histogram",
        "grammar_rule_chain",
        "grammar_chain",
        "modifiers",
        "connection_graph_ids",
        "massing_intent",
        "escalate",
        "ref_books_dual_cite",
        "report_kind",
        "series",
        "chart",
    }
    # Republic lineage first when present
    for k in (
        "report_kind",
        "frame_id",
        "seed",
        "pilot_id",
        "arch_build_grammar_preset_id",
        "material_strategy",
        "column_count",
        "presentation_only",
        "sim_occupation",
    ):
        if k in data and data[k] is not None:
            rows.append(f"<tr><th>{_esc(k)}</th><td>{_esc(data[k])}</td></tr>")
    for k, v in data.items():
        if k in skip or isinstance(v, (dict, list)):
            continue
        if k in {
            "report_kind",
            "frame_id",
            "seed",
            "pilot_id",
            "arch_build_grammar_preset_id",
            "material_strategy",
            "column_count",
            "presentation_only",
            "sim_occupation",
        }:
            continue
        rows.append(f"<tr><th>{_esc(k)}</th><td>{_esc(v)}</td></tr>")

    chain = data.get("grammar_rule_chain")
    chain_html = ""
    if isinstance(chain, dict) and chain:
        crows = "".join(
            f"<tr><th>{_esc(k)}</th><td>{_esc(v)}</td></tr>" for k, v in chain.items()
        )
        chain_html = (
            "<h2>Grammar rule chain</h2>"
            f"<table><tbody>{crows}</tbody></table>"
        )

    grammar_steps = data.get("grammar_chain")
    grammar_chain_html = ""
    if isinstance(grammar_steps, list) and grammar_steps:
        grows = []
        for step in grammar_steps[:24]:
            if not isinstance(step, dict):
                continue
            grows.append(
                "<tr><td>{st}</td><td>{gid}</td><td>{rk}</td><td>{ref}</td></tr>".format(
                    st=_esc(step.get("step")),
                    gid=_esc(step.get("grammar_id")),
                    rk=_esc(step.get("rule_key")),
                    ref=_esc(step.get("grammar_ref")),
                )
            )
        grammar_chain_html = (
            "<h2>Grammar chain (param frame)</h2>"
            "<table><thead><tr><th>step</th><th>grammar_id</th>"
            "<th>rule_key</th><th>grammar_ref</th></tr></thead>"
            f"<tbody>{''.join(grows)}</tbody></table>"
        )

    modifiers = data.get("modifiers")
    modifier_html = ""
    if isinstance(modifiers, list) and modifiers:
        mrows = []
        for m in modifiers[:24]:
            if not isinstance(m, dict):
                continue
            mrows.append(
                "<tr><td>{mid}</td><td>{en}</td><td>{params}</td></tr>".format(
                    mid=_esc(m.get("modifier_id")),
                    en=_esc(m.get("enabled", True)),
                    params=_esc(m.get("params")),
                )
            )
        modifier_html = (
            "<h2>Modifiers</h2>"
            "<table><thead><tr><th>id</th><th>enabled</th><th>params</th></tr></thead>"
            f"<tbody>{''.join(mrows)}</tbody></table>"
        )

    graph_ids = data.get("connection_graph_ids")
    graphs_html = ""
    if isinstance(graph_ids, list) and graph_ids:
        glis = "".join(f"<li>{_esc(g)}</li>" for g in graph_ids[:24])
        graphs_html = f"<h2>Connection graphs</h2><ul>{glis}</ul>"

    intent = data.get("massing_intent")
    intent_html = ""
    if isinstance(intent, dict) and intent:
        irows = "".join(
            f"<tr><th>{_esc(k)}</th><td>{_esc(v)}</td></tr>" for k, v in intent.items()
        )
        intent_html = (
            "<h2>Massing intent</h2>"
            f"<table><tbody>{irows}</tbody></table>"
        )

    escalate = data.get("escalate")
    escalate_html = ""
    if isinstance(escalate, dict) and escalate:
        erows = "".join(
            f"<tr><th>{_esc(k)}</th><td>{_esc(v)}</td></tr>" for k, v in escalate.items()
        )
        escalate_html = (
            "<h2>Escalate hints</h2>"
            f"<table><tbody>{erows}</tbody></table>"
        )

    dual = data.get("ref_books_dual_cite")
    dual_html = ""
    if isinstance(dual, dict) and dual:
        new_rows = dual.get("new_rows") or []
        legacy = dual.get("legacy_keywords") or []
        dual_html = (
            "<h2>Ref-books dual cite</h2>"
            "<table><tbody>"
            f"<tr><th>NEW rows</th><td>{_esc(', '.join(str(x) for x in new_rows))}</td></tr>"
            f"<tr><th>LEGACY keywords</th><td>{_esc(', '.join(str(x) for x in legacy))}</td></tr>"
            "</tbody></table>"
        )

    # Issues / errors lists
    issues = data.get("errors") or data.get("issues") or []
    issue_html = ""
    if isinstance(issues, list) and issues:
        lis = "".join(f"<li>{_esc(i)}</li>" for i in issues[:40])
        issue_html = f"<h2>Issues</h2><ul>{lis}</ul>"

    # Metrics dict → bar-ish table
    metrics = data.get("metrics")
    metric_html = ""
    if isinstance(metrics, dict) and metrics:
        mrows = []
        for mid, m in list(metrics.items())[:60]:
            if isinstance(m, dict):
                val = m.get("mean", m.get("value", m))
            else:
                val = m
            mrows.append(f"<tr><td>{_esc(mid)}</td><td>{_esc(val)}</td></tr>")
        metric_html = (
            "<h2>Metrics</h2><table><thead><tr><th>id</th><th>value</th></tr></thead>"
            f"<tbody>{''.join(mrows)}</tbody></table>"
        )

    # Floors histogram (massing QC)
    floors_hist = data.get("floors_histogram")
    floors_html = ""
    if isinstance(floors_hist, dict) and floors_hist:
        frows = []
        for band in ("1", "2", "3", "4"):
            if band in floors_hist:
                frows.append(
                    f"<tr><td>floors={_esc(band)}</td><td>{_esc(floors_hist[band])}</td></tr>"
                )
        for k, v in floors_hist.items():
            if str(k) not in {"1", "2", "3", "4"}:
                frows.append(f"<tr><td>{_esc(k)}</td><td>{_esc(v)}</td></tr>")
        floors_html = (
            "<h2>Floors histogram</h2>"
            "<table><thead><tr><th>band</th><th>count</th></tr></thead>"
            f"<tbody>{''.join(frows)}</tbody></table>"
        )

    # Massing columns table
    massing_cols = data.get("massing_columns")
    massing_html = ""
    if isinstance(massing_cols, list) and massing_cols:
        crows = []
        for c in massing_cols[:80]:
            if not isinstance(c, dict):
                continue
            tile = c.get("tile") or []
            crows.append(
                "<tr><td>{tile}</td><td>{floors}</td><td>{kind}</td><td>{w}</td></tr>".format(
                    tile=_esc(tile),
                    floors=_esc(c.get("floors")),
                    kind=_esc(c.get("color_kind")),
                    w=_esc(c.get("weight")),
                )
            )
        massing_html = (
            "<h2>Massing columns</h2>"
            "<table><thead><tr><th>tile</th><th>floors</th>"
            "<th>color_kind</th><th>weight</th></tr></thead>"
            f"<tbody>{''.join(crows)}</tbody></table>"
        )

    # Tasks list
    tasks = data.get("tasks")
    task_html = ""
    if isinstance(tasks, list) and tasks:
        trows = []
        for t in tasks[:40]:
            if isinstance(t, dict):
                trows.append(
                    "<tr><td>{id}</td><td>{kind}</td><td>{st}</td><td>{seed}</td></tr>".format(
                        id=_esc(t.get("task_id")),
                        kind=_esc(t.get("kind")),
                        st=_esc(t.get("status")),
                        seed=_esc(t.get("seed")),
                    )
                )
        task_html = (
            "<h2>Tasks</h2><table><thead><tr><th>id</th><th>kind</th>"
            "<th>status</th><th>seed</th></tr></thead>"
            f"<tbody>{''.join(trows)}</tbody></table>"
        )

    # Simple SVG spark from numeric list if present
    series = data.get("series") or data.get("chart")
    chart_html = ""
    if isinstance(series, list) and series and all(isinstance(x, (int, float)) for x in series):
        w, h = 320, 80
        mx = max(series) or 1
        pts = []
        for i, y in enumerate(series[:64]):
            x = i * (w / max(1, len(series[:64]) - 1))
            yy = h - (float(y) / mx) * (h - 4) - 2
            pts.append(f"{x:.1f},{yy:.1f}")
        chart_html = (
            f"<h2>Series</h2><svg width='{w}' height='{h}' viewBox='0 0 {w} {h}'>"
            f"<polyline fill='none' stroke='#6cf' stroke-width='2' points='{' '.join(pts)}'/>"
            f"</svg>"
        )

    return f"""<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>{_esc(title)}</title>
<style>
body {{ font: 14px/1.4 system-ui, sans-serif; margin: 24px; background: #12141a; color: #e8eaed; }}
h1 {{ font-size: 1.4rem; }}
table {{ border-collapse: collapse; width: 100%; max-width: 960px; }}
th, td {{ border-bottom: 1px solid #333; padding: 6px 8px; text-align: left; }}
th {{ color: #9aa0a6; width: 28%; }}
.badge {{ display: inline-block; padding: 2px 8px; border-radius: 4px; background: #2a2f3a; }}
</style></head><body>
<h1>{_esc(title)}</h1>
<p class="badge">{_esc(status)}</p>
<table><tbody>
{''.join(rows)}
</tbody></table>
{issue_html}
{chain_html}
{grammar_chain_html}
{modifier_html}
{graphs_html}
{intent_html}
{escalate_html}
{dual_html}
{metric_html}
{floors_html}
{massing_html}
{task_html}
{chart_html}
</body></html>
"""


def write_report(data: dict[str, Any], out_path: Path, *, title: str | None = None) -> Path:
    out_path = Path(out_path)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(render_report(data, title=title), encoding="utf-8")
    return out_path


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description="Render studio HTML dashboard from JSON")
    p.add_argument("json_path")
    p.add_argument("-o", "--out", required=True)
    p.add_argument("--title", default=None)
    args = p.parse_args(argv)
    data = json.loads(Path(args.json_path).read_text(encoding="utf-8"))
    path = write_report(data, Path(args.out), title=args.title)
    print(path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
