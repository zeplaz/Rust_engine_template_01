"""Art Pipeline Suite — four-workspace MCP UI."""

from .aps_theme import apply_theme, load_theme_mode

# Panels snapshot theme colors at import — apply dark/light before submodules load.
apply_theme(load_theme_mode())

