"""P7 Wave-2 — style cohesion guards (hex, font floor, status atom, material cards)."""

from __future__ import annotations

import re
import sys
from pathlib import Path

import pytest

APS_ROOT = Path(__file__).resolve().parents[2]
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))

from rust_engine_mcp.paths import repo_root

from art_pipeline_suite import aps_inline_feedback
from art_pipeline_suite.aps_theme import COLOR_PASS

SUITE = repo_root() / "tools/mcp/art_pipeline_suite"

# Files allowed raw hex (data palettes, canvas drawing, token module).
HEX_ALLOWLIST = {
    "aps_theme.py",
    "footprint_canvas.py",
    "atlas_preview_panel.py",
    "landscape_grammar_panel.py",
    "assembly_panel.py",  # material swatch heuristic colors
    "status_log_panel.py",
}

# Legacy chrome hexes that must not reappear in panel UI strings.
BANNED_UI_HEX = (
    'foreground="#555"',
    'foreground="#444"',
    'foreground="#0a4a7a"',
    'foreground="#8b0000"',
    'font=("Segoe UI", 7)',
)

SUB_NINE_FONT = re.compile(r'\(\s*"(?:Segoe UI|Consolas)"\s*,\s*([0-8])\b')
RAW_HEX_IN_KW = re.compile(
    r"""(?:foreground|background|bg|fg|fill|outline)\s*=\s*["']#([0-9a-fA-F]{3,8})["']"""
)


def _panel_py_files() -> list[Path]:
    return sorted(SUITE.glob("*.py"))


def test_no_banned_legacy_chrome_literals() -> None:
    offenders: list[str] = []
    for path in _panel_py_files():
        if path.name in HEX_ALLOWLIST:
            continue
        text = path.read_text(encoding="utf-8")
        for banned in BANNED_UI_HEX:
            if banned in text:
                offenders.append(f"{path.name}: {banned}")
    assert not offenders, "legacy hardcoded chrome:\n" + "\n".join(offenders)


def test_no_sub_nine_font_literals_on_content() -> None:
    offenders: list[str] = []
    allow = {("footprint_canvas.py", "glyph_size"), ("aps_theme.py", "FONT_CAPTION")}
    for path in _panel_py_files():
        for i, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            m = SUB_NINE_FONT.search(line)
            if not m:
                continue
            if any(path.name == fname and token in line for fname, token in allow):
                continue
            offenders.append(f"{path.name}:{i}: {line.strip()}")
    assert not offenders, "sub-9px font literals:\n" + "\n".join(offenders)


def test_material_cards_no_bullet_glyph_vocabulary() -> None:
    import art_pipeline_suite.aps_theme as theme

    text = (SUITE / "material_library_widget.py").read_text(encoding="utf-8")
    assert '"●"' not in text
    assert "'●'" not in text
    glyph, label, fg = aps_inline_feedback.material_texture_status("ready")
    assert glyph == "✓"
    assert label == "ready"
    assert fg == theme.COLOR_PASS


def test_status_atom_pass_uses_pass_color_not_accent() -> None:
    import art_pipeline_suite.aps_theme as theme

    _g, _w, fg, _bg = aps_inline_feedback.status_atom("pass")
    assert fg == theme.COLOR_PASS
    assert fg != theme.COLOR_ACCENT


def test_hex_guard_catches_deliberate_violation(tmp_path: Path) -> None:
    """Prove the banned-hex scan fires — not a trivial pass."""
    probe = tmp_path / "probe_panel.py"
    probe.write_text('foreground="#555"\n', encoding="utf-8")
    text = probe.read_text(encoding="utf-8")
    assert 'foreground="#555"' in text
    offenders = [b for b in BANNED_UI_HEX if b in text]
    assert offenders == ['foreground="#555"']


def test_pipeline_pills_use_theme_backgrounds() -> None:
    import art_pipeline_suite.aps_theme as theme
    from art_pipeline_suite.pipeline_pills import pill_bg_map

    theme.apply_theme("light")
    bg = pill_bg_map()
    assert bg["valid"] == theme.COLOR_PASS_BG
    assert bg["saved_qc_not_run"] == theme.COLOR_WARN_BG
    theme.apply_theme("dark")
    bg_dark = pill_bg_map()
    assert bg_dark["valid"] == theme._DARK_TOKENS["COLOR_PASS_BG"]


def test_no_raw_hex_in_chrome_panels_except_allowlist() -> None:
    offenders: list[str] = []
    for path in _panel_py_files():
        if path.name in HEX_ALLOWLIST:
            continue
        for i, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            if RAW_HEX_IN_KW.search(line):
                offenders.append(f"{path.name}:{i}: {line.strip()[:100]}")
    assert not offenders, "raw hex in UI kwargs (use aps_theme tokens):\n" + "\n".join(offenders[:20])


@pytest.mark.skipif(not SUITE.joinpath("scrollable.py").is_file(), reason="suite missing")
def test_scrollable_uses_attach_wheel_area_not_recursive_bind() -> None:
    text = (SUITE / "scrollable.py").read_text(encoding="utf-8")
    assert "attach_wheel_area" in text
    assert "_bind_wheel_recursive" not in text


def test_dark_theme_is_default_and_applies_tokens() -> None:
    import os

    import art_pipeline_suite.aps_theme as theme

    prev = os.environ.pop("APS_THEME", None)
    try:
        theme.apply_theme("dark")
        assert theme.theme_mode() == "dark"
        assert theme.COLOR_PANEL_BG == theme._DARK_TOKENS["COLOR_PANEL_BG"]
        theme.apply_theme("light")
        assert theme.COLOR_PANEL_BG == theme._LIGHT_TOKENS["COLOR_PANEL_BG"]
        os.environ.pop("APS_THEME", None)
        assert theme.load_theme_mode() in ("light", "dark")
    finally:
        if prev is not None:
            os.environ["APS_THEME"] = prev
        theme.apply_theme("light")


def test_assembly_generate_defaults_to_grammar_path() -> None:
    src = (SUITE / "assembly_panel.py").read_text(encoding="utf-8")
    assert "Use building style rules (recommended)" in src
    assert "BooleanVar(value=bool(archetypes))" in src
    assert "_grammar_combo_maps" in src
    assert "Kit grammar reference (advanced)" in src


def test_dark_theme_sets_readable_tk_defaults() -> None:
    import art_pipeline_suite.aps_theme as theme

    theme.apply_theme("dark")
    assert theme.COLOR_TEXT_BODY == theme._DARK_TOKENS["COLOR_TEXT_BODY"]
    assert theme.COLOR_INPUT_BG == theme._DARK_TOKENS["COLOR_INPUT_BG"]
    # Body text must be light on dark surfaces (not legacy #333).
    assert theme.COLOR_TEXT_BODY.startswith("#e") or int(theme.COLOR_TEXT_BODY[1:3], 16) > 0xC0
    theme.apply_theme("light")
