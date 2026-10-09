"""Grouped catalog of micro CLI commands — derived from the live argparse tree, never hand-listed.

Used by ``cli commands``, ``cli help`` and the MCP ``micro_tool_help`` tool so the three can
never drift apart (the old hand-maintained list in ``server.micro_tool_help`` named ~20
commands that no longer existed).
"""

from __future__ import annotations

import argparse

# (group id, title, exact names, name prefixes). First match wins; order matters.
COMMAND_GROUPS: tuple[tuple[str, str, tuple[str, ...], tuple[str, ...]], ...] = (
    (
        "meta",
        "CLI meta",
        ("commands", "help"),
        (),
    ),
    (
        "core",
        "Core asset pipeline (spec -> geometry -> validate -> promote -> library)",
        (
            "ping",
            "locate-blender",
            "list-staging",
            "validate-spec",
            "write-spec",
            "run-geometry",
            "job-status",
            "validate-glb",
            "promote",
            "write-witness",
            "validate-report",
            "file-digest",
            "pixel-conclude",
            "generate-material-textures",
        ),
        ("library-",),
    ),
    (
        "tiles",
        "Tiles, atlases, keyframes, variants, assembly",
        ("build-iso-rig", "preview-assembly", "keyframe-matrix-export", "write-procedural-tiles-production-bake-witness"),
        ("tile-", "lod0-", "variant-", "assembly-", "write-tile-", "rail-warehouse-", "pg-module-"),
    ),
    (
        "effects",
        "VFX effect specs (pack / promote / capture)",
        ("artist-vfx-pipeline-witness",),
        ("effect-",),
    ),
    (
        "agents",
        "Agent queues, briefs, handoff, routing",
        (
            "get-que",
            "witness-brief",
            "review-order-brief",
            "slice-exec-brief",
            "handoff-brief",
            "orchestrator-brief",
            "token-savings-guide",
            "auto-fleet-brief",
            "pipeline-preflight",
            "place-feel-phase-note",
        ),
        ("agent-", "intel-officer-", "queue-", "mcp-"),
    ),
    ("ops", "Ops intelligence (blockers, drift, crash, dashboards)", (), ("ops-",)),
    (
        "grammar",
        "Building grammar, kits, city, build-quality (BQ)",
        ("designer-grammar-quality-loop", "single-archetype-ratio-guard", "warehouse-track-guard"),
        (
            "grammar-",
            "building-",
            "bq-",
            "city-",
            "kit-",
            "arch-",
            "site-",
            "spine-",
            "build-",
            "example-",
            "pilot-",
        ),
    ),
    ("aps", "Art Pipeline Suite (APS) witnesses + attestation", (), ("aps-", "apsr-")),
    (
        "landscape",
        "Landscape, vegetation, terrain, reaction territory",
        (),
        ("landscape-", "veg-", "reaction-", "terrain-"),
    ),
    ("power", "Power utility / nuclear batches", (), ("pwr-",)),
    ("dmcp", "Designer-MCP lane witnesses", (), ("dmcp-",)),
)

OTHER_GROUP = ("other", "Uncategorised")


def subparser_map(parser: argparse.ArgumentParser) -> dict[str, argparse.ArgumentParser]:
    for action in parser._actions:  # noqa: SLF001 — argparse has no public accessor
        if isinstance(action, argparse._SubParsersAction):  # noqa: SLF001
            return dict(action.choices)
    return {}


def _help_by_name(parser: argparse.ArgumentParser) -> dict[str, str]:
    out: dict[str, str] = {}
    for action in parser._actions:  # noqa: SLF001
        if isinstance(action, argparse._SubParsersAction):  # noqa: SLF001
            for choice in action._choices_actions:  # noqa: SLF001
                out[choice.dest] = choice.help or ""
    return out


def group_for(name: str) -> tuple[str, str]:
    for gid, title, exact, prefixes in COMMAND_GROUPS:
        if name in exact or any(name.startswith(p) for p in prefixes):
            return gid, title
    return OTHER_GROUP


def _usage_args(sp: argparse.ArgumentParser) -> str:
    usage = sp.format_usage().strip()
    # "usage: rust_engine_mcp.cli ping [-h] ..." -> args after the command name
    parts = usage.split(None, 3)
    tail = parts[3] if len(parts) > 3 else ""
    return tail.replace("[-h] ", "").replace("[-h]", "").strip()


def catalog(
    parser: argparse.ArgumentParser,
    *,
    grep: str = "",
    group: str = "",
    include_witness: bool = True,
) -> list[dict[str, str]]:
    helps = _help_by_name(parser)
    needle = grep.strip().lower()
    rows: list[dict[str, str]] = []
    for name, sp in subparser_map(parser).items():
        gid, title = group_for(name)
        if group and gid != group:
            continue
        if not include_witness and name.endswith("-witness"):
            continue
        help_text = helps.get(name, "")
        if needle and needle not in name.lower() and needle not in help_text.lower():
            continue
        rows.append(
            {
                "name": name,
                "group": gid,
                "group_title": title,
                "args": _usage_args(sp),
                "help": help_text,
            }
        )
    order = {gid: i for i, (gid, *_rest) in enumerate(COMMAND_GROUPS)}
    rows.sort(key=lambda r: (order.get(r["group"], len(order)), r["name"]))
    return rows


def render_text(rows: list[dict[str, str]]) -> str:
    if not rows:
        return "no matching commands"
    lines: list[str] = [f"{len(rows)} command(s) — python -m rust_engine_mcp.cli <command> ..."]
    current = None
    for row in rows:
        if row["group"] != current:
            current = row["group"]
            count = sum(1 for r in rows if r["group"] == current)
            lines.append("")
            lines.append(f"[{current}] {row['group_title']} ({count})")
        sig = f"{row['name']} {row['args']}".strip()
        lines.append(f"  {sig}" + (f"  — {row['help']}" if row["help"] else ""))
    return "\n".join(lines)
