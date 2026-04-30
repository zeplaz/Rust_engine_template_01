# Strategic platforms — implementation questions `v1`

**Pair with:** [`spec/README.md`](spec/README.md), `platforms_ew_munitions_v1.md`, `phased_engine_delivery_v1.md`, `prompts/matrix/strategic_platforms/strategic_platforms_matrix_v1.md`

## Data & authority

1. **Munition flight state:** component bundle per entity vs child entities for stages?
2. **Track ID** type — `u64` globally unique vs `(owner, local_idx)`?
3. **Server validation** of fire orders: azimuth limits, friendly corridors, min range — which invariants v0?

## LOD

4. **Promotion** on launch: chunk set expansion formula tied to `terrain_world/chunks_streaming_v1.md`?
5. **Demotion** after detonation: dwell time before corridor collapse?

## Sensors / EW

6. **Radar equation** level: table lookup vs simplified `SNR = f(R, ERP, σ)`?
7. **Jamming** stacking: multiple jammers — sum in dB-domain with cap?

## Integration

8. **Magazine** linkage to `ProductionManifest` resource types — which existing enums?
9. **Naval** coupling to hydrology events (dam breach affects port) — event bus topic name 📎?

## Testing

10. **Headless** integration test: spawn launcher + target, assert damage component delta after fixed ticks.
11. **Determinism** of fire-control not required for MP — but **server replay** blob for QA 📎?

## Tools

12. **egui** track table columns: range, mode, lethality estimate — which derived from server snapshot?

## Road vehicles & damage (code already present)

13. **`RoadVehicle`** carries **`RoadVehicleDamageInfo`** (`src/entities/vehicles/runtime.rs`) — when does this become ECS components vs remain a helper struct?
14. **`RoadVehicleToolsUiPlugin`** is registered in `engine_with_worldgen.rs` next to production tools — panel contract vs **`ProductionToolsUiPlugin`** 📎?
15. **Cargo / capacity** enums from `ResourceType` — linkage to `production_economy` manifests for convoy logistics 📎?
