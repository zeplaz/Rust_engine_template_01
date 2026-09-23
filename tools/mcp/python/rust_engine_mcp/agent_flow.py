"""Agent flow tier router — cheap research → hard gate → exec.

Deterministic control plane (no LLM). Call ``agent_flow_route`` before spawning
Task/subagents so parents spend tokens only on the hard gate when required.
"""

from __future__ import annotations

from typing import Any

# Cursor Task `model` slugs (must match parent allow-list).
MODEL_CHEAP = "composer-2.5-fast"
MODEL_HARD = "claude-opus-5-thinking-high"
MODEL_EXEC = "inherit"

HARD_TRIGGERS = (
    "authority",
    "dual writer",
    "dual-writer",
    "architecture",
    "migrate",
    "migration",
    "conflict",
    "ev/cx",
    "contested",
    "root cause",
    "root-cause",
    "which approach",
    "tradeoff",
    "trade-off",
    "design fork",
    "schedule",
    "systemset",
    "viewport drift",
    "render contract",
)

DOMAIN_KEYWORDS: dict[str, tuple[str, ...]] = {
    "fire": ("fire", "spark", "heat", "ember", "fuel", "smoke", "vfx", "combust"),
    "ui": ("ui", "hud", "egui", "tray", "onboard", "interaction", "bq-128", "minimap"),
    "render": (
        "render",
        "rtt",
        "shader",
        "viewport",
        "overlay",
        "gpu",
        "projection",
        "render_gui",
        "rgr-",
    ),
    "art": (
        "art",
        "g4",
        "atlas",
        "blender",
        "geometry",
        "warehouse",
        "keyframe",
        "aps",
        "tile",
        "grammar",
        "staging",
        "promote",
    ),
    "ops": ("ops", "status", "queue", "handoff", "witness", "blocker", "rollup", "drift"),
}


def _detect_domain(goal: str, domain: str) -> str:
    d = (domain or "auto").strip().lower()
    if d and d != "auto":
        return d if d in DOMAIN_KEYWORDS or d in {"engine", "sim"} else "ops"
    g = goal.lower()
    scores: dict[str, int] = {k: 0 for k in DOMAIN_KEYWORDS}
    for name, kws in DOMAIN_KEYWORDS.items():
        for kw in kws:
            if kw in g:
                scores[name] += 1
    best = max(scores, key=scores.get)
    return best if scores[best] > 0 else "ops"


def _needs_hard_gate(goal: str, force_hard: bool) -> bool:
    if force_hard:
        return True
    g = goal.lower()
    return any(t in g for t in HARD_TRIGGERS)


def _domain_packet(domain: str) -> dict[str, Any]:
    """Per-domain cheap / hard / exec owners + brief tools."""
    packets: dict[str, dict[str, Any]] = {
        "fire": {
            "cheap_agent": "explore",
            "cheap_chat": "Task(explore) — not an @chat agent",
            "cheap_task_type": "explore",
            "hard_agent": "debug-intelligence",
            "exec_agent": "coder",
            "exec_alt": "sim-steward",
            "briefs": [
                "witness_brief('debug_runs/fire_ecology_live.json', profile='fire_product')",
                "ops_get_project_brief()",
                "file_digest('src/dev/fire_ecology_f1_todos.md', max_lines=40)",
                "file_digest('src/dev/visual_run_blockers.md', max_lines=40)",
            ],
            "validate": ["validate_cargo_report(compress=4, use_cached=true)"],
            "witness_hint": "debug_runs/fire_ecology_live.json",
        },
        "ui": {
            "cheap_agent": "explore",
            "cheap_chat": "Task(explore) — not an @chat agent",
            "cheap_task_type": "explore",
            "hard_agent": "planner",
            "exec_agent": "coder",
            "exec_alt": "designer",
            "briefs": [
                "handoff_brief()",
                "agent_queue_next('coder')",
                "agent_queue_next('designer')",
                "file_digest('src/dev/post_stage6_active_todos.md', max_lines=50)",
            ],
            "validate": ["validate_bevy_report(compress=4)"],
            "witness_hint": "debug_runs/minimap_compositor_live.json",
        },
        "render": {
            "cheap_agent": "explore",
            "cheap_chat": "Task(explore) — not an @chat agent",
            "cheap_task_type": "explore",
            "hard_agent": "debug-intelligence",
            "exec_agent": "coder",
            "exec_alt": "sim-steward",
            "briefs": [
                "terrain_honesty_lint_tool()",
                "file_digest('src/dev/plan_rpc1_gpu_terrain_ab_v1.md', max_lines=50)",
                "file_digest('src/dev/visual_run_blockers.md', max_lines=50)",
                "orchestrator_brief(use_cached=true)",
                "handoff_brief()",
            ],
            "validate": ["validate_cargo_report(compress=4, use_cached=true)"],
            "witness_hint": "debug_runs/terrain_honesty_lint_live.json",
        },
        "art": {
            "cheap_agent": "explore",
            "cheap_chat": "Task(explore) — not an @chat agent",
            "cheap_task_type": "explore",
            "hard_agent": "designer-mcp",
            "exec_agent": "coder-mcp",
            "exec_alt": "designer-mcp",
            "briefs": [
                "pipeline_preflight()",
                "ops_get_active_blockers()",
                "handoff_brief()",
                "coder_mcp_drain_brief()",
            ],
            "validate": ["validate_report('mcp_spec'|asset path)", "witness_brief(..., profile='honesty')"],
            "witness_hint": "debug_runs/art_pipeline/kit_production_002_g4_live.json",
        },
        "ops": {
            "cheap_agent": "operations-intelligence",
            "cheap_chat": "@operations-intelligence",
            "cheap_task_type": "operations-intelligence",
            "hard_agent": "operations-intelligence",
            "exec_agent": "orchestrator",
            "exec_alt": "plan-orchestrator",
            "briefs": [
                "ops_get_project_brief()",
                "ops_get_active_blockers()",
                "handoff_brief()",
                "token_savings_guide()",
            ],
            "validate": ["validate_report('queue_integrity', compress=3)"],
            "witness_hint": "debug_runs/agent_ops/ops_project_brief_v1.json",
        },
        "engine": {
            "cheap_agent": "explore",
            "cheap_chat": "Task(explore) — not an @chat agent",
            "cheap_task_type": "explore",
            "hard_agent": "planner",
            "exec_agent": "coder",
            "exec_alt": "sim-steward",
            "briefs": ["ops_get_project_brief()", "handoff_brief()", "coder_drain_brief('c')"],
            "validate": ["validate_cargo_report(compress=4, use_cached=true)"],
            "witness_hint": "debug_runs/stage5_full_app_live.json",
        },
        "sim": {
            "cheap_agent": "explore",
            "cheap_chat": "Task(explore) — not an @chat agent",
            "cheap_task_type": "explore",
            "hard_agent": "planner",
            "exec_agent": "coder",
            "exec_alt": "sim-steward",
            "briefs": ["simulation_queue_brief()", "ops_get_project_brief()"],
            "validate": ["validate_cargo_report(compress=4, use_cached=true)"],
            "witness_hint": "debug_runs/weather_sim_live.json",
        },
    }
    return packets.get(domain, packets["ops"])


def agent_flow_route(
    goal: str,
    domain: str = "auto",
    *,
    force_hard: bool = False,
    skip_hard: bool = False,
) -> dict[str, Any]:
    """Emit a 3-tier dispatch packet for multi-agent sessions.

    L0 CHEAP — research / digests / queue (composer-2.5-fast / explore)
    L1 HARD  — contested decisions only (claude-opus-5-thinking-high)
    L2 EXEC  — implement from filled packet (inherit / @coder…)
    """
    goal_s = (goal or "").strip()
    if not goal_s:
        return {
            "schema": "agent_flow_route_v1",
            "ok": False,
            "error": "goal required",
            "hint": "agent_flow_route(goal='…', domain='fire|ui|render|art|ops|auto')",
        }

    detected = _detect_domain(goal_s, domain)
    pkt = _domain_packet(detected)
    hard = False if skip_hard else _needs_hard_gate(goal_s, force_hard)

    phases: list[dict[str, Any]] = [
        {
            "id": "L0_CHEAP",
            "purpose": "research + digests + queue picks — fill packet, do not decide contested roots",
            "model": MODEL_CHEAP,
            "task_subagent_type": pkt["cheap_task_type"],
            "chat_agent": pkt.get("cheap_chat") or f"@{pkt['cheap_agent']}",
            "tools": pkt["briefs"],
            "output": "research_packet.yaml (paths, witness digests, open questions ≤5)",
            "budget": "max 1 explore Task · prefer MCP briefs over Read",
        }
    ]

    if hard:
        phases.append(
            {
                "id": "L1_HARD",
                "purpose": "hard question only — authority / architecture / EV/Cx / contested root",
                "model": MODEL_HARD,
                "task_subagent_type": pkt["hard_agent"],
                "chat_agent": f"@{pkt['hard_agent']}",
                "input": "research_packet.yaml from L0",
                "output": "decision_packet.yaml (root_cause, owner, confidence, reject_alts)",
                "budget": "≤1 hard Task · no file edits · no cargo walls",
                "skip_if": None,
            }
        )
    else:
        phases.append(
            {
                "id": "L1_HARD",
                "purpose": "SKIP — goal has no hard-gate triggers",
                "model": None,
                "skipped": True,
                "skip_reason": "no HARD_TRIGGERS matched; set force_hard=true to compel",
            }
        )

    phases.append(
        {
            "id": "L2_EXEC",
            "purpose": "implement / validate / witness from filled packets",
            "model": MODEL_EXEC,
            "task_subagent_type": pkt["exec_agent"],
            "chat_agent": f"@{pkt['exec_agent']}",
            "chat_agent_alt": f"@{pkt['exec_alt']}",
            "tools": pkt["validate"],
            "output": "witness path + Q✓ note",
            "budget": "one owner · validation-first · WIT-HON before Q✓",
            "requires": ["L0 research_packet"] + (["L1 decision_packet"] if hard else []),
        }
    )

    return {
        "schema": "agent_flow_route_v1",
        "ok": True,
        "goal": goal_s,
        "domain": detected,
        "domain_requested": domain or "auto",
        "hard_gate": hard,
        "models": {
            "cheap": MODEL_CHEAP,
            "hard": MODEL_HARD,
            "exec": MODEL_EXEC,
        },
        "phases": phases,
        "parent_policy": [
            "Call agent_flow_route BEFORE spawning Tasks",
            "L0 always · L1 only if hard_gate · L2 only after packet filled",
            "Never put research Raw-Read walls on L1/L2 models",
            "Task usage error → foreground @chat_agent with same packet (no Task retry)",
            "Multitask OFF when Task quota empty — use @agents in chat",
        ],
        "never": [
            "Run L2 without L0 packet",
            "Use HARD model for file_digest / queue / ops_get_project_brief",
            "Paste cargo walls into any tier",
            "Spawn parallel Tasks that share file/authority ownership",
        ],
        "witness_hint": pkt["witness_hint"],
        "blang": "BLANG:FLOW → L0 → [L1?] → L2 → WIT-HON → Q✓",
    }


def agent_flow_policy() -> dict[str, Any]:
    """Static tier policy — pair with token_savings_guide."""
    return {
        "schema": "agent_flow_policy_v1",
        "ok": True,
        "tiers": {
            "L0_CHEAP": {
                "model": MODEL_CHEAP,
                "use_for": ["research", "digests", "queue", "witness_brief", "file_digest", "status"],
                "agents": ["explore", "operations-intelligence", "shell"],
            },
            "L1_HARD": {
                "model": MODEL_HARD,
                "use_for": list(HARD_TRIGGERS[:8]) + ["…see HARD_TRIGGERS"],
                "agents": [
                    "planner",
                    "debug-intelligence",
                    "operations-intelligence",
                    "designer-mcp",
                ],
            },
            "L2_EXEC": {
                "model": MODEL_EXEC,
                "use_for": ["implement", "validate", "witness write", "Q✓"],
                "agents": ["coder", "coder-mcp", "designer", "sim-steward"],
            },
        },
        "entry": "agent_flow_route(goal, domain='auto')",
        "cli": "python -m rust_engine_mcp.cli agent-flow-route --goal '…' [--domain fire]",
    }
