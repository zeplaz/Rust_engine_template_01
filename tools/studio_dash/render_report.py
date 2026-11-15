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
    skip = {"raw_log", "raw_log_path", "issues", "errors", "warnings", "metrics", "tasks"}
    for k, v in data.items():
        if k in skip or isinstance(v, (dict, list)):
            continue
        rows.append(f"<tr><th>{_esc(k)}</th><td>{_esc(v)}</td></tr>")

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
{metric_html}
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
