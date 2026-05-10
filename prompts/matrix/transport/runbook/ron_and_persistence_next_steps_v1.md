# RON-first persistence — transport done, repo backlog `v1`

> **Policy:** Prefer **RON** for serde game-data authored by tooling and the engine. Reserve **JSON** where humans or external stacks require it (browser snippets, some Python editors, legacy fixtures).

## Done (this pass)

- **R8 transport** dev save/load canonical path → `*.ron` (pretty).
- **G4 `LoadTransportNetworkSnapshotFromDisk`:** extension `.ron` / `.json`; unknown → try RON then JSON.
- **Hybrid dev snapshot** body: UTF-8 **RON** first, JSON fallback (existing `.sav` files with JSON body still load).
- **World gen tuning overlay** (`tuning_io`): canonical file **`world_gen_tuning.ron`**; startup + F8 **Reload** prefer RON if present, else `world_gen_tuning.json`; per-path load uses extension (unknown → RON then JSON). F8 **Save** writes RON. Examples: `world_gen_tuning.example.ron` + `.example.json`.

## TODO — migrate or justify JSON retention

| Area | Today | Target / note |
|:---|:---|:---|
| Terrain registries | `*.tag_registry.json`, `material_registry.json`, … | RON loaders + `*.ron` examples; keep JSON loaders until matrix U3/U4 confirms. |
| `world_gen_tuning.json` | Python asset editor | **Exception:** keep JSON for tooling; runtime/editor canonical RON (`WORLD_GEN_TUNING_RON_PATH`) — see `tuning_io.rs`. |
| `plant_definitions.json` | Embedded defaults | RON + `include_str!` or keep JSON with explicit rationale in module doc. |
| World generator export | `world_*.json` in subengine | RON or binary snapshot per orchestrator. |
| `io/snapshot` hybrid **header** line | JSON | Acceptable single-line metadata; optional RON header in `v1` format bump. |
| Python asset tools | JSON file dialogs | Bridge: export RON where possible; JSON only for legacy pages. |
| `reqwest` JSON API | HTTP | Correct to stay JSON. |

## Phased build plan (tests at each step)

Each phase ships **loaders or I/O paths** plus **automated tests** before moving on. Reuse patterns from [`src/systems/transport/persistence.rs`](../../../src/systems/transport/persistence.rs) and [`src/terrain/generation/tuning_io.rs`](../../../src/terrain/generation/tuning_io.rs): extension dispatch (`.ron` / `.json`, unknown → RON then JSON), and “prefer RON on disk when both names exist” where there are paired defaults.

| Phase | Scope | Deliverables | Tests (minimum) |
|:---|:---|:---|:---|
| **P1** | **Tag registry** | Shared terrain registry text helper (recommended) or per-type dispatch; `tag_registry.example.ron`; wire all call sites that read tag JSON. | Parse `example.json` ⇄ RON round-trip (`serde_json::Value` or `PartialEq` structs); `from_path` respects `.json` / `.ron`; unknown extension → RON then JSON. |
| **P2** | **Material registry** | `material_registry.example.ron`; path-based load; family resolution unchanged. | Same as P1; optional `CARGO_MANIFEST_DIR` load of example files. |
| **P3** | **Terrain family registry** | `terrain_family_registry.example.ron`; extend Bevy `AssetLoader` extensions if applicable. | Parse tests + smoke `hash_terrain_family_registry` / schema gate. |
| **P4** | **Transport fixtures** | `assets/test_fixtures/transport/*.ron` mirrors; retire duplicate JSON when CI is RON-only. | G4 / `persistence` tests against `.ron` paths; keep one JSON regression test if needed. |
| **P5** | **`plant_definitions`** | RON-first default or disk path + JSON fallback; document exception in module doc if JSON stays. | `include_str!` or path round-trip; boot-path smoke if wired. |
| **P6** | **World generator export** (subengine `world_*.json`) | Choose RON vs binary; versioned writer + reader. | Small grid export → re-import round-trip. |
| **P7** | **Hybrid snapshot header** | Optional RON in format `v1`; keep JSON decode. | `io/snapshot` tests: legacy JSON header + new RON header. |
| **P8** | **Format-exception docs** | `AGENTS.md` or authoring-meta appendix: JSON vs RON per surface. | Review checklist only. |
| **P9** | **R8 construction book** in transport snapshot | Serde optional block + hydrate. | Snapshot round-trip + hydrate unit test. |

**Order rationale:** P1 should introduce the shared helper so P2–P3 stay small.

## Wiring follow-ups (execution)

1. Add `*.ron` mirrors under `assets/test_fixtures/transport/` and retire duplicate JSON when CI uses RON only.
2. **Construction book** in R8 snapshot extension fields (serde optional block) — separate PR.
3. Document per-crate **format exceptions** in `AGENTS.md` or `system_runbook_authoring_meta` appendix.
4. **Terrain registries** (material / tag / family JSON): add RON loaders + examples, same extension dispatch pattern as `tuning_io` / transport.

Version: `v1.1.0`
