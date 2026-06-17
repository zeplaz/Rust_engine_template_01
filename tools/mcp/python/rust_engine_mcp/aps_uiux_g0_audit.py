"""DMCP-OVR-G0-AUDIT-001 — §2 ban-list + voice rules scan for APS Art Pipeline Suite."""

from __future__ import annotations

import ast
import json
import re
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/art_pipeline/dmcp_ovr_g0_audit_live.json"
AUDIT_DOC_REL = "src/dev/design_aps_uiux_g0_audit_v1.md"
CHARTER_REL = "src/dev/plan_aps_uiux_overhaul_20260616_v1.md"
DESIGN_SYSTEM_REL = "src/dev/aps_design_system_v1.md"
SWEEP_REL = "src/dev/aps_sweep_text_20260616_v1.md"
SUITE_REL = "tools/mcp/art_pipeline_suite"

# plan_aps_uiux_overhaul §2b + aps_design_system §6 DoD
BAN_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    ("gate_id_aps", re.compile(r"\bAPS-[A-Z0-9-]+\b")),
    ("gate_id_arch", re.compile(r"\bARCH-[A-Z0-9-]+\b")),
    ("gate_id_dmcp", re.compile(r"\bDMCP-[A-Z0-9-]+\b")),
    ("gate_id_build_set", re.compile(r"\bBUILD-SET\b")),
    ("gate_lg5", re.compile(r"\bLG-5\b")),
    ("gate_g_scope", re.compile(r"\bG[0-5]\b")),
    ("gate_p0", re.compile(r"\bP0\b(?!\s*issues)")),  # allow "P0 issues" in logs only if needed
    ("schema_assembly_snapshot", re.compile(r"\bassembly_snapshot\b")),
    ("schema_variant_set", re.compile(r"\bvariant_set(?:_v1)?\b")),
    ("schema_tile_batch", re.compile(r"\btile_batch(?:_v1)?\b")),
    ("schema_node_id", re.compile(r"\bnode_id\b")),
    ("schema_material_profile", re.compile(r"\bmaterial_profile\b")),
    ("schema_land_dna", re.compile(r"\bland_dna\b")),
    ("schema_topology_graph", re.compile(r"\btopology_graph\b")),
    ("schema_veg_catalog", re.compile(r"\bvegetation_variant_catalog\b")),
    ("env_rust_engine", re.compile(r"\bRUST_ENGINE_[A-Z0-9_]+\b")),
    ("tool_tilemapgen", re.compile(r"\btilemapgen\b", re.I)),
    ("tool_trimesh", re.compile(r"\btrimesh\b", re.I)),
    ("tool_rust_engine_mcp", re.compile(r"\brust_engine_mcp\b")),
    ("tool_mcp_word", re.compile(r"\bMCP\b")),
    ("agent_handle", re.compile(r"@coder\b")),
    ("voice_ship_truth", re.compile(r"\b[Ss]hip truth\b")),
    ("voice_authority_paren", re.compile(r"\bAUTHORITY\b")),
    ("schema_keyframe_pack", re.compile(r"\bkeyframe_pack\b")),
    ("engine_type_path", re.compile(r"VegetationExtractFrame|placement\.material_profile")),
)


@dataclass(frozen=True)
class Violation:
    rule: str
    path: str
    line: int
    excerpt: str
    source: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "rule": self.rule,
            "path": self.path,
            "line": self.line,
            "excerpt": self.excerpt,
            "source": self.source,
        }


def _rel(path: Path, root: Path) -> str:
    try:
        return str(path.relative_to(root)).replace("\\", "/")
    except ValueError:
        return str(path).replace("\\", "/")


def _scan_text(
    text: str,
    *,
    path: str,
    source: str,
    start_line: int = 1,
) -> list[Violation]:
    out: list[Violation] = []
    for i, line in enumerate(text.splitlines(), start=start_line):
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        for rule, pat in BAN_PATTERNS:
            if pat.search(line):
                out.append(
                    Violation(
                        rule=rule,
                        path=path,
                        line=i,
                        excerpt=stripped[:140],
                        source=source,
                    )
                )
                break
    return out


def _extract_string_literals(py_path: Path, root: Path) -> list[tuple[int, str, str]]:
    """Return (line, kind, value) for text= / LabelFrame title strings."""
    rel = _rel(py_path, root)
    try:
        tree = ast.parse(py_path.read_text(encoding="utf-8"), filename=rel)
    except SyntaxError:
        return []
    rows: list[tuple[int, str, str]] = []

    class V(ast.NodeVisitor):
        def visit_Call(self, node: ast.Call) -> None:
            kw = {k.arg: k.value for k in node.keywords if k.arg}
            if "text" in kw and isinstance(kw["text"], ast.Constant) and isinstance(kw["text"].value, str):
                rows.append((node.lineno, "text=", kw["text"].value))
            self.generic_visit(node)

        def visit_Dict(self, node: ast.Dict) -> None:
            self.generic_visit(node)

    V().visit(tree)
    return rows


def _scan_python_ui_strings(py_path: Path, root: Path) -> list[Violation]:
    rel = _rel(py_path, root)
    out: list[Violation] = []
    for lineno, kind, value in _extract_string_literals(py_path, root):
        for rule, pat in BAN_PATTERNS:
            if pat.search(value):
                out.append(
                    Violation(
                        rule=rule,
                        path=rel,
                        line=lineno,
                        excerpt=f"{kind}{value[:120]!r}",
                        source="ui_string",
                    )
                )
                break
    # LabelFrame super().__init__(..., text="...")
    text = py_path.read_text(encoding="utf-8")
    for m in re.finditer(r'text\s*=\s*"([^"]{4,})"', text):
        val = m.group(1)
        line_no = text[: m.start()].count("\n") + 1
        for rule, pat in BAN_PATTERNS:
            if pat.search(val):
                if any(v.line == line_no and v.excerpt == val for v in out):
                    continue
                out.append(
                    Violation(
                        rule=rule,
                        path=rel,
                        line=line_no,
                        excerpt=val[:140],
                        source="ui_string",
                    )
                )
                break
    return out


def _scan_tooltips(root: Path) -> list[Violation]:
    path = root / SUITE_REL / "aps_tooltips.py"
    if not path.is_file():
        return []
    rel = _rel(path, root)
    out: list[Violation] = []
    mod = ast.parse(path.read_text(encoding="utf-8"), filename=rel)
    for node in mod.body:
        if isinstance(node, ast.Assign):
            if not isinstance(node.value, ast.Dict):
                continue
            for k, v in zip(node.value.keys, node.value.values, strict=False):
                if isinstance(v, ast.Constant) and isinstance(v.value, str):
                    for rule, pat in BAN_PATTERNS:
                        if pat.search(v.value):
                            key = k.value if isinstance(k, ast.Constant) else "?"
                            out.append(
                                Violation(
                                    rule=rule,
                                    path=rel,
                                    line=v.lineno or 0,
                                    excerpt=f"{key}: {v.value[:100]}",
                                    source="tooltip",
                                )
                            )
                            break
    return out


def _scan_module_string_constants(root: Path) -> list[Violation]:
    out: list[Violation] = []
    for rel_path in (
        f"{SUITE_REL}/domain_router.py",
        f"{SUITE_REL}/aps_theme.py",
        f"{SUITE_REL}/metadata_flow_panel.py",
    ):
        path = root / rel_path
        if not path.is_file():
            continue
        rel = _rel(path, root)
        text = path.read_text(encoding="utf-8")
        # triple-quoted diagram blocks in metadata_flow
        for m in re.finditer(r'"""(.*?)"""', text, re.DOTALL):
            block = m.group(1)
            line_no = text[: m.start()].count("\n") + 1
            for rule, pat in BAN_PATTERNS:
                if pat.search(block):
                    out.append(
                        Violation(
                            rule=rule,
                            path=rel,
                            line=line_no,
                            excerpt=block.strip().splitlines()[0][:120],
                            source="diagram_block",
                        )
                    )
                    break
        for m in re.finditer(r'"([^"]{12,})"', text):
            val = m.group(1)
            if "Ship truth" in val or "assembly_snapshot" in val or "land_dna" in val:
                line_no = text[: m.start()].count("\n") + 1
                for rule, pat in BAN_PATTERNS:
                    if pat.search(val):
                        out.append(
                            Violation(
                                rule=rule,
                                path=rel,
                                line=line_no,
                                excerpt=val[:140],
                                source="module_constant",
                            )
                        )
                        break
    return out


def run_ban_list_audit(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    suite = root / SUITE_REL
    violations: list[Violation] = []
    violations.extend(_scan_tooltips(root))
    violations.extend(_scan_module_string_constants(root))
    if suite.is_dir():
        for py in sorted(suite.glob("*.py")):
            violations.extend(_scan_python_ui_strings(py, root))

    by_rule: dict[str, int] = {}
    by_file: dict[str, int] = {}
    p0_rules = {
        "gate_id_aps",
        "gate_id_arch",
        "gate_id_dmcp",
        "gate_lg5",
        "gate_g_scope",
        "engine_type_path",
        "voice_ship_truth",
        "schema_land_dna",
        "schema_topology_graph",
        "tool_rust_engine_mcp",
        "agent_handle",
    }
    p0_count = sum(1 for v in violations if v.rule in p0_rules)
    for v in violations:
        by_rule[v.rule] = by_rule.get(v.rule, 0) + 1
        by_file[v.path] = by_file.get(v.path, 0) + 1

    return {
        "gate": "DMCP-OVR-G0-AUDIT-001",
        "charter_section": "plan_aps_uiux_overhaul §2b",
        "violation_count": len(violations),
        "p0_violation_count": p0_count,
        "by_rule": dict(sorted(by_rule.items(), key=lambda x: -x[1])),
        "by_file": dict(sorted(by_file.items(), key=lambda x: -x[1])[:15]),
        "violations": [v.to_dict() for v in violations[:80]],
        "truncated": len(violations) > 80,
        "audit_complete": True,
        "ui_clean": len(violations) == 0,
        "verdict": "FAIL" if violations else "PASS",
        "blocks": [] if violations else ["OVR-P2-TEXT-001"],
        "unblocks_p2_when_clean": True,
    }


def refresh_dmcp_ovr_g0_audit_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    audit = run_ban_list_audit(repo=root)
    green = bool(audit.get("audit_complete"))
    body: dict[str, Any] = {
        **audit,
        "green": green,
        "deliverable": AUDIT_DOC_REL,
        "charter": CHARTER_REL,
        "design_system": DESIGN_SYSTEM_REL,
        "sweep_ref": SWEEP_REL,
        "guard_test": "tools/mcp/python/tests/test_aps_no_jargon.py",
        "handoff": "OVR-P2-TEXT-001 + OVR-DES-P2-COPY-PACK-001",
        "parallel_with": "OVR-P1-TOKENS-001",
        "_agent_meta": {
            "schema": "dmcp_ovr_g0_audit_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "DMCP_OVR_G0_AUDIT",
            "source_system": "aps_uiux_g0_audit",
            "relative_path": WITNESS_REL,
            "ritual": "BLANG:WIT-HON→Q✓ DMCP-OVR-G0-AUDIT-001" if green else "BLANG:WIT-HON FAIL pre-P2",
            "agent": "designer-mcp",
        },
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
