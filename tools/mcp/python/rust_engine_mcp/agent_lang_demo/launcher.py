"""Launch AGENT-LANG demo UI."""

from __future__ import annotations

import argparse
import json
import sys

from .app import run_app
from .workflow import AUTH_SPINE, write_demo_witness, DemoSession, DEMO_SCRIPT


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="AGENT-LANG multi-agent workflow demo")
    parser.add_argument("--headless", action="store_true", help="Run scripted demo without UI; write witness")
    args = parser.parse_args(argv)

    if args.headless:
        session = DemoSession()
        for step in DEMO_SCRIPT:
            if step.agent:
                session.active_agent = step.agent
            step.fn(session)
        wit = write_demo_witness(session)
        print(json.dumps({"ok": wit.get("green"), "witness": wit.get("written"), "auth": AUTH_SPINE}, indent=2))
        return 0 if wit.get("green") else 1

    run_app()
    return 0


if __name__ == "__main__":
    sys.exit(main())
