"""OVR-P1-TOKENS-001 — spacing/density tokens exported and adopted by shell."""

from __future__ import annotations

import re
import sys
from pathlib import Path

APS_ROOT = Path(__file__).resolve().parents[2]
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))

from rust_engine_mcp.paths import repo_root

from art_pipeline_suite.aps_theme import GAP_MD, GAP_SM, INSET_PANEL, PAD_MD, PAD_SM

SUITE = repo_root() / "tools/mcp/art_pipeline_suite"
LEGAL_PADDING = {0, 2, 3, 4, 6, 8, 10, 12, 16, 24}

# Panels that must import spacing tokens (not literal-only padding).
TOKEN_ADOPTER_FILES = (
    "app.py",
    "aps_theme.py",
    "material_library_widget.py",
)


def test_density_token_aliases() -> None:
    assert PAD_SM == GAP_SM == 4
    assert PAD_MD == GAP_MD == 8
    assert INSET_PANEL == 8


def test_shell_imports_spacing_tokens() -> None:
    for name in TOKEN_ADOPTER_FILES:
        text = (SUITE / name).read_text(encoding="utf-8")
        assert "GAP_" in text or "PAD_" in text or "INSET_" in text, f"{name} missing spacing tokens"


def test_aps_theme_declares_full_gap_scale() -> None:
    theme = (SUITE / "aps_theme.py").read_text(encoding="utf-8")
    for name in ("GAP_XS", "GAP_SM", "GAP_MD", "GAP_LG", "GAP_XL", "PANE_MIN_LIST", "ROW_HEIGHT"):
        assert f"{name} =" in theme


def test_material_library_no_off_scale_card_padding() -> None:
    text = (SUITE / "material_library_widget.py").read_text(encoding="utf-8")
    offenders = [m.group(0) for m in re.finditer(r"padx=3|pady=3", text)]
    assert not offenders, "migrate card grid padding to GAP_SM/GAP_MD"


def test_no_out_of_scale_padding_literals_in_theme() -> None:
    """Theme module must not introduce ad-hoc padding ints outside the scale."""
    theme = (SUITE / "aps_theme.py").read_text(encoding="utf-8")
    offenders: list[str] = []
    for match in re.finditer(r"padding=\((\d+)", theme):
        if int(match.group(1)) not in LEGAL_PADDING:
            offenders.append(match.group(0))
    for match in re.finditer(r"padding=\((\d+),\s*(\d+)\)", theme):
        a, b = int(match.group(1)), int(match.group(2))
        if a not in LEGAL_PADDING or b not in LEGAL_PADDING:
            offenders.append(match.group(0))
    assert not offenders, f"out-of-scale padding in aps_theme.py: {offenders}"
