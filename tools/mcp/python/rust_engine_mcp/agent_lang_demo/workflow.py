"""AGENT-LANG workflow engine — real MCP calls + symbolic paste lines."""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable

from .. import agent_doc_read, agent_queue
from ..mcp_productivity_p0 import pipeline_preflight
from ..paths import repo_root

AUTH_SPINE = "AUTH: MAT★ ⇢ APS★ ⇢ SNAP★ ⇢ WRK○ ⇢ ATL○ ⇢ RT○"

WITNESS_PATH = "debug_runs/agent_lang_demo_live.json"

AGENTS = ("operator", "orchestrator", "planner-mcp", "coder-mcp")

REF_DEFAULT = "src/dev/agent_lang_v1.md"


def _iso_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def _compact_json(data: Any, *, limit: int = 2400) -> str:
    text = json.dumps(data, indent=2, ensure_ascii=False)
    if len(text) <= limit:
        return text
    return text[: limit - 3] + "..."


def format_paste(
    *,
    blang: str,
    agent: str,
    status: str = "🟢",
    slice_id: str = "",
    route: str = "",
    ref: str = "",
) -> str:
    parts = [f"**{agent}**", blang]
    if slice_id:
        parts.append(f"⟨{slice_id}⟩")
    parts.append(status)
    if ref:
        parts.append(f"$ref:{ref}")
    if route:
        parts.append(route)
    return " · ".join(parts)


@dataclass
class StepResult:
    blang: str
    agent: str
    paste: str
    payload: dict[str, Any]
    handoff_to: str = ""
    ok: bool = True
    error: str = ""


@dataclass
class DemoSession:
    active_agent: str = "orchestrator"
    slice_id: str = "AGENT-LANG-DEMO-001"
    steps: list[StepResult] = field(default_factory=list)
    tools_called: list[str] = field(default_factory=list)

    def record(self, result: StepResult) -> StepResult:
        self.steps.append(result)
        return result


def _parse_ref(ref: str) -> str:
    text = ref.strip()
    if text.startswith("$ref:"):
        return text[5:].split("§", 1)[0]
    return text


def execute_blang(
    action: str,
    session: DemoSession,
    *,
    ref: str = REF_DEFAULT,
    witness_path: str = WITNESS_PATH,
    queue_agent: str = "",
) -> StepResult:
    """Run one BLANG action against live MCP helpers."""
    agent = session.active_agent
    blang = action.upper().replace("BLANG:", "BLANG:")
    if not blang.startswith("BLANG:"):
        blang = f"BLANG:{blang}"

    try:
        if blang == "BLANG:PRE":
            payload = pipeline_preflight(queue="grammar")
            session.tools_called.append("pipeline_preflight")
            paste = format_paste(blang=blang, agent=agent, status="🟢" if payload.get("ok") else "🔴")
            return session.record(StepResult(blang, agent, paste, payload))

        if blang == "BLANG:GUIDE":
            payload = agent_queue.token_savings_guide()
            session.tools_called.append("token_savings_guide")
            paste = format_paste(blang=blang, agent=agent, status="🟢")
            return session.record(StepResult(blang, agent, paste, payload))

        if blang == "BLANG:HO":
            payload = agent_queue.handoff_brief()
            session.tools_called.append("handoff_brief")
            st = "🟢" if payload.get("ok") else "🔴"
            paste = format_paste(
                blang=blang,
                agent=agent,
                status=st,
                ref="tools/orchestrator/queues/HANDOFF.md",
            )
            return session.record(StepResult(blang, agent, paste, payload))

        if blang == "BLANG:Q+":
            target = queue_agent or agent
            payload = agent_queue.agent_queue_next(target, mark_in_progress=False)
            session.tools_called.append("agent_queue_next")
            sl = (payload.get("slice") or {}).get("id") or session.slice_id
            session.slice_id = str(sl)
            action_kind = payload.get("action", "idle")
            st = "🟢" if action_kind == "work" else "🟡"
            paste = format_paste(blang=blang, agent=agent, status=st, slice_id=session.slice_id)
            return session.record(StepResult(blang, agent, paste, payload))

        if blang == "BLANG:REF":
            path = _parse_ref(ref)
            payload = agent_doc_read.agent_doc_touch(
                path,
                agent=agent,
                intent="ref",
                session_hint=session.slice_id,
            )
            session.tools_called.append("agent_doc_touch")
            st = "🟢" if payload.get("ok") else "🔴"
            paste = format_paste(blang=blang, agent=agent, status=st, ref=path)
            return session.record(StepResult(blang, agent, paste, payload))

        if blang == "BLANG:WIT":
            payload = agent_queue.witness_brief(witness_path)
            session.tools_called.append("witness_brief")
            brief = payload.get("brief") or {}
            green = brief.get("green")
            st = "🟢" if green else "🟡" if payload.get("ok") else "🔴"
            paste = format_paste(blang=blang, agent=agent, status=st, ref=witness_path)
            return session.record(StepResult(blang, agent, paste, payload))

        if blang == "BLANG:RUN":
            event = {
                "slice_id": session.slice_id,
                "tools_called": list(dict.fromkeys(session.tools_called)),
                "demo": True,
            }
            payload = agent_doc_read.agent_run_append(event, agent=agent)
            session.tools_called.append("agent_run_append")
            paste = format_paste(blang=blang, agent=agent, status="🟢", slice_id=session.slice_id)
            return session.record(StepResult(blang, agent, paste, payload))

        if blang == "BLANG:Q✓":
            note = "agent-lang demo slice — no queue mutation"
            paste = format_paste(
                blang=blang,
                agent=agent,
                status="🟢",
                slice_id=session.slice_id,
                route="⟨COMMIT:WIT⟩",
            )
            payload = {
                "ok": True,
                "slice_id": session.slice_id,
                "note": note,
                "queue_update_skipped": True,
                "hint": "Demo does not write grammar queue; use real agent_queue_update in production",
            }
            return session.record(StepResult(blang, agent, paste, payload))

        return session.record(
            StepResult(
                blang,
                agent,
                format_paste(blang=blang, agent=agent, status="🔴"),
                {"ok": False, "error": f"unknown action: {action}"},
                ok=False,
                error=f"unknown action: {action}",
            )
        )
    except Exception as exc:  # noqa: BLE001 — demo UI surface
        return session.record(
            StepResult(
                blang,
                agent,
                format_paste(blang=blang, agent=agent, status="🔴"),
                {"ok": False, "error": str(exc)},
                ok=False,
                error=str(exc),
            )
        )


def handoff(session: DemoSession, to_agent: str, *, route: str = "") -> StepResult:
    """Symbolic ΔWF between agents."""
    from_agent = session.active_agent
    session.active_agent = to_agent
    r = route or f"ΔWF→@{to_agent}"
    paste = f"**{from_agent}** → **{to_agent}** · {r} · ⟨{session.slice_id}⟩ · 🔗"
    payload = {"from": from_agent, "to": to_agent, "route": r, "slice_id": session.slice_id}
    return session.record(
        StepResult("HANDOFF", from_agent, paste, payload, handoff_to=to_agent)
    )


def delimiter(session: DemoSession, tag: str) -> StepResult:
    """Stream delimiter — ⟨BRK⟩ ⟨CONT⟩ ⟨DRIFT⟩."""
    agent = session.active_agent
    tag = tag.strip().upper()
    if not tag.startswith("⟨"):
        tag = f"⟨{tag.strip('<>')}⟩"

    payload: dict[str, Any] = {"tag": tag, "slice_id": session.slice_id, "agent": agent}
    if tag == "⟨DRIFT⟩":
        payload["recover"] = [
            "BLANG:REF on normative spec",
            "BLANG:HO for active programs",
            f"re-state ⟨{session.slice_id}⟩",
        ]
        path = _parse_ref(REF_DEFAULT)
        payload["reanchor"] = agent_doc_read.agent_doc_touch(
            path, agent=agent, intent="ref", session_hint="DRIFT"
        )
        session.tools_called.append("agent_doc_touch")

    paste = f"**{agent}** · {tag} · ⟨{session.slice_id}⟩"
    return session.record(StepResult(tag, agent, paste, payload))


@dataclass
class DemoScriptStep:
    label: str
    fn: Callable[[DemoSession], StepResult | None]
    agent: str | None = None


def _script() -> list[DemoScriptStep]:
    def s1(sess: DemoSession) -> StepResult:
        sess.active_agent = "orchestrator"
        return execute_blang("PRE", sess)

    def s2(sess: DemoSession) -> StepResult:
        return execute_blang("GUIDE", sess)

    def s3(sess: DemoSession) -> StepResult:
        return execute_blang("HO", sess)

    def s4(sess: DemoSession) -> StepResult:
        return handoff(sess, "planner-mcp", route="ΔWF→@planner-mcp ⟨AGENT-LANG-003-BLANG⟩")

    def s5(sess: DemoSession) -> StepResult:
        return execute_blang("REF", sess, ref=REF_DEFAULT)

    def s6(sess: DemoSession) -> StepResult:
        return execute_blang("Q+", sess, queue_agent="planner-mcp")

    def s7(sess: DemoSession) -> StepResult:
        return handoff(sess, "coder-mcp", route="ΔWF→@coder-mcp spec landed")

    def s8(sess: DemoSession) -> StepResult:
        return execute_blang("REF", sess, ref="tools/mcp/MICRO_TOOLS_REGISTRY_v1.md")

    def s9(sess: DemoSession) -> StepResult:
        return execute_blang("RUN", sess)

    def s10(sess: DemoSession) -> StepResult:
        return execute_blang("Q✓", sess)

    return [
        DemoScriptStep("1 Orchestrator BLANG:PRE", s1, "orchestrator"),
        DemoScriptStep("2 Orchestrator token guide", s2, "orchestrator"),
        DemoScriptStep("3 Orchestrator handoff brief", s3, "orchestrator"),
        DemoScriptStep("4 → Planner-MCP", s4, "planner-mcp"),
        DemoScriptStep("5 Planner $ref digest", s5, "planner-mcp"),
        DemoScriptStep("6 Planner queue next", s6, "planner-mcp"),
        DemoScriptStep("7 → Coder-MCP", s7, "coder-mcp"),
        DemoScriptStep("8 Coder registry $ref", s8, "coder-mcp"),
        DemoScriptStep("9 Coder run telemetry", s9, "coder-mcp"),
        DemoScriptStep("10 Close slice Q✓", s10, "coder-mcp"),
    ]


DEMO_SCRIPT = _script()


def write_demo_witness(session: DemoSession) -> dict[str, Any]:
    """Persist proof witness — debug_runs/agent_lang_demo_live.json."""
    ok = all(s.ok for s in session.steps if s.blang not in ("HANDOFF",))
    witness: dict[str, Any] = {
        "schema": "agent_lang_demo_witness_v1",
        "green": ok,
        "status": "ok" if ok else "failed",
        "summary": f"AGENT-LANG demo — {len(session.steps)} steps, {len(session.tools_called)} tools",
        "auth_spine": AUTH_SPINE,
        "slice_id": session.slice_id,
        "tools_called": list(dict.fromkeys(session.tools_called)),
        "agents_touched": list(dict.fromkeys(s.agent for s in session.steps)),
        "step_pastes": [s.paste for s in session.steps[-12:]],
        "ts": _iso_now(),
    }
    out = repo_root() / WITNESS_PATH
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(witness, indent=2) + "\n", encoding="utf-8")
    witness["written"] = str(out.relative_to(repo_root())).replace("\\", "/")
    return witness
