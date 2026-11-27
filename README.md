# Processor Alpha Dine (`proc_A_dine01`)

Rust + Bevy **0.20** simulation engine: production, logistics, terrain, water, combat, and play UI.

## Testing

**Start here:** [`docs/TESTING.md`](docs/TESTING.md) — unit tests, every `--test` scene, DamFlood / PlayQuad playchecks, dump→matrix scoring, and env vars.

```powershell
# DamFlood one-shot (build + run + capture + score)
python tools/sim_review/playcheck.py --scene dam

# Lib tests
cargo test -p proc_A_dine01 --lib
```

Sim-review tooling detail: [`tools/sim_review/README.md`](tools/sim_review/README.md).

## Quick start

```powershell
cargo run -p proc_A_dine01
cargo run -p proc_A_dine01 -- --test dam --stay-open   # DamFlood valley
```

Prerequisites: Rust stable toolchain; Python 3 for `tools/sim_review/` and validators.

## Docs map

| Doc | What |
|:---|:---|
| [`docs/TESTING.md`](docs/TESTING.md) | **Commands to run and what they do** |
| [`AGENTS.md`](AGENTS.md) | Agent / authoring rules (authority, UI, review path) |
| [`src/dev/coder_todos_v1.md`](src/dev/coder_todos_v1.md) | Open work board |
| [`src/dev/development_plan_index.md`](src/dev/development_plan_index.md) | Plan index |
| [`src/dev/plan_dam_flood_benchmark_v1.md`](src/dev/plan_dam_flood_benchmark_v1.md) | DamFlood benchmark stages |
| [`src/dev/codebase_index_v1.md`](src/dev/codebase_index_v1.md) | Module / concept index |
| [`tools/validators/README.md`](tools/validators/README.md) | Structured validate-report CLI |

## Layout

| Path | Owns |
|:---|:---|
| `src/` | Engine, ECS, sim, GUI, render |
| `src/scenario/dam_flood/` | DamFlood example harness |
| `src/construction/` | Construction execute funnel |
| `tools/sim_review/` | Dump scoring + playcheck |
| `tools/mcp/` | Deterministic art / MCP toolchain |
| `assets/` | Configs, scenarios, shaders, models |
