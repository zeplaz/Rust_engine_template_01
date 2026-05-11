//! **P1 color boundary** — raw `Color32` / `from_rgb` live only under `style/` (palette, theme, swatch conversion).
//!
//! Use repo grep as the backlog driver:
//! `rg "Color32::|from_rgb" src/gui` → anything outside `style/` is migration debt.
//!
//! Future: optional dev feature could call [`forbid_raw_colors`] from a test or hook to assert zero hits.

/// Placeholder for a future strict check (CI / `--features ui_color_guard`); grep is the policy until then.
pub fn forbid_raw_colors() {}
