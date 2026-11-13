# Build orchestrator (`cargo orchestrate`)

Small Rust CLI that runs `cargo check/clippy/test` with JSON diagnostics, classifies warnings against
`knowledge/*.json` (migration state, owners, `@orchestrator-*` source markers) and writes reports to
`reports/` (gitignored, regenerated every run).

```powershell
cargo orchestrate                      # alias in .cargo/config.toml
cargo orchestrate -- --skip-test --skip-clippy
cargo run --manifest-path tools/orchestrator/Cargo.toml -- --help
```

Other scripts here: `scripts/build_deep_debug.ps1` / `run_deep_debug.ps1` (deep-debug engine build),
`scripts/run_visual_test_clean.ps1` (clean env for `--test visual`), `scripts/bake_*_icon_atlas*.py` (UI icon atlas bakes),
`scripts/check_orphan_modules.py` (module index vs GLB files).
