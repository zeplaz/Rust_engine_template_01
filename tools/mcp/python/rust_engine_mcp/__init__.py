"""Designer art MCP — see tools/mcp/README.md and plan_designer_mcp_art_toolchain_exec_001_v1.md."""

import os as _os

if _os.environ.get("RUST_ENGINE_TEST_WRITE_GUARD"):
    # Child process of a guarded pytest session: roll back our repo writes too.
    from . import test_write_guard as _test_write_guard

    _test_write_guard.install_from_env()
