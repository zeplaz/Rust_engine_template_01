"""P7 META-GUARD — scan the *real* artist-visible surface of the Art Pipeline Suite.

The prior `aps_uiux_g0_audit` only saw static ``text="..."`` constants plus the
tooltip dict, so every runtime string (f-strings, ``StringVar.set(...)``,
``.configure(text=...)``, ``messagebox.*`` titles/messages, dynamically-built
``LabelFrame`` titles, and ``_log(...)`` status lines) sailed past it — a false
green. This module walks those contexts via the AST and flags two classes:

* **ban-list jargon / gate IDs** (reuses ``aps_uiux_g0_audit.BAN_PATTERNS``) seen
  in *any* of the runtime contexts above, not just literal ``text=`` constants;
* **off-glossary terminology** — words that are jargon for the artist even though
  they are not gate IDs (``Material profile`` → Material, ``Node id`` → Piece id,
  ``StylePack`` → Building style, ``Archetype`` → an artist word, ``Snapshot`` →
  Assembly, and the loose ``Validate`` button that should read ``Check schema``).

A tooltip-dict path-field exception keeps raw schema names allowed where the
glossary says so (path tooltips only).
"""

from __future__ import annotations

import ast
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from rust_engine_mcp.aps_uiux_g0_audit import BAN_PATTERNS
from rust_engine_mcp.paths import repo_root

SUITE_REL = "tools/mcp/art_pipeline_suite"

# Functions whose first string argument is shown to the artist (status log,
# inline validation result lines, lane banners, next-step guidance, …).
_TEXT_SINK_FUNCS = frozenset(
    {
        "_log",
        "_on_log",
        "_set_validation_result",
        "_set_status",
        "_show_validation_report",
        "_inline_hint",
        "_set_hint",
    }
)

# Helpers whose *text* argument (positional or kwarg) is shown to the artist.
# ``set_inline_status(widget, var, text, ...)`` — 3rd positional is the message.
# ``configure_preview_label(label, state, detail=...)`` — detail kwarg is shown.
_STATUS_HELPER_ARGINDEX = {"set_inline_status": 2}
_STATUS_HELPER_KWARGS = {"configure_preview_label": ("detail",)}

# kwargs/attrs that carry artist-visible text when assigned a string.
_TEXT_KWARGS = frozenset({"text", "title", "message", "label"})

# §2a canonical glossary — words that are engineer/schema dialect on screen.
# (rule, compiled pattern, the canonical replacement the artist should see)
TERM_PATTERNS: tuple[tuple[str, re.Pattern[str], str], ...] = (
    ("term_material_profile", re.compile(r"\bMaterial profile\b"), "Material"),
    ("term_node_id", re.compile(r"\bNode id\b"), "Piece id (or hide)"),
    ("term_stylepack", re.compile(r"\bStylePack\b"), "Building style"),
    ("term_archetype", re.compile(r"\bArchetype\b"), "an artist word"),
    ("term_snapshot", re.compile(r"\bSnapshot\b"), "Assembly"),
    ("term_p0_gate", re.compile(r"\bP0 gate\b"), "Ship check"),
    # the loose "Validate" button/dialog (the strict one is "Run ship check");
    # the glossary word is "Check schema". Flags the *bare* button label and the
    # "Validate (production)" dialog title — not descriptive imperatives like
    # "Validate GLB"/"Validate preset" which name their object and are in voice.
    ("term_validate_loose", re.compile(r"^\s*Validate\s*(?:\((?:production|prod)\))?\s*$"), "Check schema"),
    ("term_validation_caps", re.compile(r"\bValidation:\s*(PASS|FAIL)\b"), "Check passed / failed"),
)


@dataclass(frozen=True)
class SurfaceHit:
    rule: str
    path: str
    line: int
    context: str  # text= | configure | set | messagebox | log_sink | labelframe_title
    excerpt: str
    suggestion: str = ""

    def to_dict(self) -> dict[str, Any]:
        return {
            "rule": self.rule,
            "path": self.path,
            "line": self.line,
            "context": self.context,
            "excerpt": self.excerpt,
            "suggestion": self.suggestion,
        }


def _rel(path: Path, root: Path) -> str:
    try:
        return str(path.relative_to(root)).replace("\\", "/")
    except ValueError:
        return str(path).replace("\\", "/")


def _string_value(node: ast.AST) -> str | None:
    """Best-effort literal text from a const, f-string, or ``+`` concat.

    f-string holes / non-literal pieces become ``{}`` so the surrounding literal
    fragments (which is where the jargon lives) are still scanned.
    """
    if isinstance(node, ast.Constant) and isinstance(node.value, str):
        return node.value
    if isinstance(node, ast.JoinedStr):
        parts: list[str] = []
        for v in node.values:
            if isinstance(v, ast.Constant) and isinstance(v.value, str):
                parts.append(v.value)
            else:
                parts.append("{}")
        return "".join(parts)
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add):
        left = _string_value(node.left)
        right = _string_value(node.right)
        if left is not None or right is not None:
            return (left or "{}") + (right or "{}")
    return None


def _func_name(node: ast.Call) -> str | None:
    f = node.func
    if isinstance(f, ast.Attribute):
        return f.attr
    if isinstance(f, ast.Name):
        return f.id
    return None


def _is_messagebox_call(node: ast.Call) -> bool:
    f = node.func
    return (
        isinstance(f, ast.Attribute)
        and isinstance(f.value, ast.Name)
        and f.value.id in ("messagebox", "tkMessageBox")
    )


def _is_set_on_textlike(node: ast.Call) -> bool:
    """``something.set("literal")`` where the target looks like a text var."""
    f = node.func
    if not (isinstance(f, ast.Attribute) and f.attr == "set"):
        return False
    tgt = f.value
    name = getattr(tgt, "attr", None) or getattr(tgt, "id", None) or ""
    low = str(name).lower()
    return any(tok in low for tok in ("var", "text", "status", "label", "hint", "legend", "summary"))


def _is_labelframe_title(node: ast.Call) -> bool:
    """``ttk.LabelFrame(parent, text="...")`` / ``super().__init__(..., text=...)``.

    Both already flow through the generic ``text=`` kwarg check; this exists so a
    *dynamic* title (f-string) is still caught — the generic path handles that.
    """
    return False


def _collect_strings(py_path: Path, root: Path) -> list[tuple[int, str, str]]:
    """Return (line, context, value) for every artist-visible string sink."""
    rel = _rel(py_path, root)
    try:
        tree = ast.parse(py_path.read_text(encoding="utf-8"), filename=rel)
    except SyntaxError:
        return []
    rows: list[tuple[int, str, str]] = []

    class V(ast.NodeVisitor):
        def visit_Call(self, node: ast.Call) -> None:
            fname = _func_name(node)

            # text= / title= / message= / label= keyword (const OR f-string OR concat)
            for kw in node.keywords:
                if kw.arg in _TEXT_KWARGS:
                    val = _string_value(kw.value)
                    if val:
                        ctx = "configure" if fname in ("configure", "config") else f"{kw.arg}="
                        rows.append((node.lineno, ctx, val))

            # messagebox.* positional args (title, message)
            if _is_messagebox_call(node):
                for arg in node.args:
                    val = _string_value(arg)
                    if val:
                        rows.append((node.lineno, "messagebox", val))

            # text-sink functions: first positional argument is shown text
            if fname in _TEXT_SINK_FUNCS and node.args:
                val = _string_value(node.args[0])
                if val:
                    rows.append((node.lineno, "log_sink", val))

            # status helpers with the message at a known positional index
            if fname in _STATUS_HELPER_ARGINDEX:
                idx = _STATUS_HELPER_ARGINDEX[fname]
                if len(node.args) > idx:
                    val = _string_value(node.args[idx])
                    if val:
                        rows.append((node.lineno, "status_helper", val))

            # status helpers with the message in a kwarg (e.g. detail=)
            if fname in _STATUS_HELPER_KWARGS:
                for kw in node.keywords:
                    if kw.arg in _STATUS_HELPER_KWARGS[fname]:
                        val = _string_value(kw.value)
                        if val:
                            rows.append((node.lineno, "status_helper", val))

            # StringVar-like .set("literal")
            if _is_set_on_textlike(node) and node.args:
                val = _string_value(node.args[0])
                if val:
                    rows.append((node.lineno, "set", val))

            self.generic_visit(node)

    V().visit(tree)
    return rows


def _ban_hits(value: str) -> list[tuple[str, str]]:
    out: list[tuple[str, str]] = []
    for rule, pat in BAN_PATTERNS:
        if pat.search(value):
            out.append((rule, ""))
    return out


def _term_hits(value: str) -> list[tuple[str, str]]:
    out: list[tuple[str, str]] = []
    for rule, pat, suggestion in TERM_PATTERNS:
        if pat.search(value):
            out.append((rule, suggestion))
    return out


def scan_surface(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    suite = root / SUITE_REL
    hits: list[SurfaceHit] = []
    if suite.is_dir():
        for py in sorted(suite.glob("*.py")):
            # the tooltip dict is scanned by the g0 audit (path-field exception
            # lives there); skip it here to avoid double-flagging path tooltips.
            if py.name == "aps_tooltips.py":
                continue
            rel = _rel(py, root)
            for lineno, ctx, value in _collect_strings(py, root):
                for rule, _ in _ban_hits(value):
                    hits.append(
                        SurfaceHit(
                            rule=rule,
                            path=rel,
                            line=lineno,
                            context=ctx,
                            excerpt=value.strip()[:140],
                        )
                    )
                for rule, suggestion in _term_hits(value):
                    hits.append(
                        SurfaceHit(
                            rule=rule,
                            path=rel,
                            line=lineno,
                            context=ctx,
                            excerpt=value.strip()[:140],
                            suggestion=suggestion,
                        )
                    )

    by_rule: dict[str, int] = {}
    by_file: dict[str, int] = {}
    by_context: dict[str, int] = {}
    for h in hits:
        by_rule[h.rule] = by_rule.get(h.rule, 0) + 1
        by_file[h.path] = by_file.get(h.path, 0) + 1
        by_context[h.context] = by_context.get(h.context, 0) + 1

    return {
        "gate": "P7-META-GUARD-SURFACE-001",
        "surface_clean": len(hits) == 0,
        "violation_count": len(hits),
        "by_rule": dict(sorted(by_rule.items(), key=lambda x: -x[1])),
        "by_file": dict(sorted(by_file.items(), key=lambda x: -x[1])),
        "by_context": dict(sorted(by_context.items(), key=lambda x: -x[1])),
        "violations": [h.to_dict() for h in hits],
        "scan_complete": True,
    }
