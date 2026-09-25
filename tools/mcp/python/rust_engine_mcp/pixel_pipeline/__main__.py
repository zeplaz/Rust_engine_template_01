"""python -m rust_engine_mcp.pixel_pipeline --image <png> --kind world|gui|ui|art"""

from .cli import cli_main

raise SystemExit(cli_main())
