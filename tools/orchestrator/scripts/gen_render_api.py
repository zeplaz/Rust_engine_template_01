#!/usr/bin/env python3
"""One-shot: extract render/mod.rs pub use blocks into render/api.rs (RGR-M1-001)."""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MOD = ROOT / "src" / "render" / "mod.rs"
API = ROOT / "src" / "render" / "api.rs"


def main() -> None:
    text = MOD.read_text(encoding="utf-8")
    lines = text.splitlines(keepends=True)
    out_mod: list[str] = []
    blocks: list[str] = []
    skip = False
    buf: list[str] = []

    for line in lines:
        if not skip and line.startswith("pub use "):
            skip = True
            buf = [line]
            if line.rstrip().endswith(";"):
                blocks.append("".join(buf))
                skip = False
                buf = []
            continue
        if skip:
            buf.append(line)
            if line.rstrip().endswith(";"):
                blocks.append("".join(buf))
                skip = False
                buf = []
            continue
        out_mod.append(line)

    blocks = [
        b
        for b in blocks
        if not (
            "crate::gui" in b
            and "MinimapShellState" in b
        )
    ]

    api_blocks: list[str] = []
    for b in blocks:
        b2 = re.sub(r"pub use ([a-z_0-9]+)::", r"pub use super::\1::", b)
        api_blocks.append(b2)

    header = (
        "//! RGR-M1 explicit public render API — plugins, GPU spine, stage5, diagnostics.\n"
        "//! `crate::render::*` re-exports this module from `mod.rs`.\n\n"
    )
    shims = (
        "\n/// Deprecated GUI re-exports — import from `crate::gui` (RGR-M1-002/003).\n"
        "pub mod deprecated_gui_shims {\n"
        "    #[deprecated(\n"
        "        since = \"2026.07.04\",\n"
        "        note = \"import from crate::gui::{MinimapShellState, MinimapOverlayMask, MinimapPresentationMode}\"\n"
        "    )]\n"
        "    pub use crate::gui::{MinimapOverlayMask, MinimapPresentationMode, MinimapShellState};\n"
        "}\n"
    )
    API.write_text(header + "".join(api_blocks) + shims, encoding="utf-8")

    mod_text = "".join(out_mod).rstrip() + "\n\npub mod api;\npub use api::*;\n"
    MOD.write_text(mod_text, encoding="utf-8")
    print(f"api blocks: {len(api_blocks)}")


if __name__ == "__main__":
    main()
