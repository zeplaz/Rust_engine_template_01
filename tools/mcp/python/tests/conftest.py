"""Shared APS pytest fixtures — one headless app per session (no multi-window spam)."""

from __future__ import annotations

import os
import sys
from pathlib import Path

import pytest

_APS_MCP = Path(__file__).resolve().parents[2]
_APS_PY = _APS_MCP / "python"
for _p in (_APS_MCP, _APS_PY):
    if str(_p) not in sys.path:
        sys.path.insert(0, str(_p))


def pytest_configure(config: pytest.Config) -> None:
    os.environ.setdefault("APS_TEST_HEADLESS", "1")
    os.environ.setdefault("RUST_ENGINE_BEVY_PREVIEW", "0")
    config.addinivalue_line(
        "markers",
        "aps_gui: Tkinter smoke (headless root; use pytest -k 'aps and not aps_gui' for fast gate)",
    )


@pytest.fixture(scope="session")
def _aps_headless_env() -> None:
    os.environ["APS_TEST_HEADLESS"] = "1"
    os.environ.setdefault("RUST_ENGINE_BEVY_PREVIEW", "0")


@pytest.fixture(scope="session")
def _aps_app_session(_aps_headless_env: None):
    """Single headless ArtPipelineSuiteApp for all GUI tests (one Tk root)."""
    pytest.importorskip("PIL")
    pytest.importorskip("tkinter")
    from art_pipeline_suite.app import ArtPipelineSuiteApp
    from art_pipeline_suite.aps_headless import apply_headless_root

    try:
        app = ArtPipelineSuiteApp()
    except Exception as exc:  # noqa: BLE001
        pytest.skip(f"no APS Tk display: {exc}")
    apply_headless_root(app)
    app._apply_lane("buildings", log=False)
    app.update_idletasks()
    try:
        yield app
    finally:
        try:
            app.destroy()
        except Exception:  # noqa: BLE001
            pass


@pytest.fixture
def aps_app(_aps_app_session):
    """Reset lane/geometry between tests."""
    from art_pipeline_suite.aps_theme import DEFAULT_WINDOW_SIZE

    w, h = DEFAULT_WINDOW_SIZE
    _aps_app_session.geometry(f"{w}x{h}")
    _aps_app_session._apply_lane("buildings", log=False)
    _aps_app_session.update_idletasks()
    return _aps_app_session


@pytest.fixture
def tk_root(aps_app):
    """Alias — all GUI tests share the one session app (avoids second Tk / pyimage bugs)."""
    return aps_app


@pytest.fixture
def gui_panel_host(aps_app):
    """Off-screen frame for mounting a panel without disturbing app chrome."""
    from tkinter import ttk

    host = ttk.Frame(aps_app)
    host.place(x=-8000, y=-8000, width=640, height=480)
    aps_app.update_idletasks()
    try:
        yield host
    finally:
        host.place_forget()
        host.destroy()
