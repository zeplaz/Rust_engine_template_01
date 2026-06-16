# Clippy policy (P1)

- **`too_many_arguments`:** allowed at crate level (`Cargo.toml` `[lints.clippy]`) — egui/HUD draw fns carry layout context by design.
- **`empty_line_after_outer_attr`:** warn — fix in touched modules; batch remaining in dedicated hygiene pass.
- **`-D warnings` (rustc):** enforced in `tools/orchestrator/ci/run.ps1` for `cargo build --lib` after `cargo check`.
- **Clippy `-D warnings`:** not CI-gated globally until style backlog triaged; use `cargo clippy -p proc_A_dine01 --lib` locally.
