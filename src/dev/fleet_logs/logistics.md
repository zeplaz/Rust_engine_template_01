# Fleet `logistics` — progress log

Mission: standing supply lines vs one-off multi-trip supply runs, draw demands. Plan:
[`plan_logistics_orders_v1.md`](../plan_logistics_orders_v1.md).

| Date | Commit | Landed | Next |
|:---|:---|:---|:---|
| 2026-10-08 | LO-0 | Research (Mars Tactics Steam feed — write-up not found; built from user brief) + code survey + plan | LO-1 sim core |
| 2026-10-09 | LO-1/2 | `src/economy/logistics/orders/` pure-Rust sim: vehicle pool, one-off runs with multi-trip batching + completion/cancel/pause/fail, draw demands (hysteresis, integer draw, starving), standing lines, priority + aging + round-robin fairness, owner = faction slot `u8`. 25 headless tests green. | LO-3 snapshot + Bevy adapter |
| 2026-10-09 | LO-3 | Snapshot (`orders/snapshot.rs`: site/order/demand/vehicle/fleet/event rows, AI helpers `sites_of`, `unserved_demands`), `SiteDistanceTravel` (metres, detour 1.3), Bevy adapter `LogisticsOrdersPlugin` (1 s steps from `SimTimeMicros`, `LogisticsOrderRequest` message = sole write path, `LogisticsOrderSite` component any lane can attach, depot fleet seed, hubs/flow facilities auto-tagged). 30 tests green. Found 5 **pre-existing** economy test failures (log_c_reservations, activation live_json ×2, spatial_district, logistics live_json) — queued as LO-H. | LO-4 UI panel |
| 2026-10-09 | LO-4 | `src/gui/logistics_orders_panel.rs` (Shift+F6 / button in F6 targets panel): New-order form with ⟳ Supply line vs ➜ Supply run toggle + plain-language hint, line form publishes destination demand when missing; tabs Runs (progress bar `24/40 t · trip 3/5`, ⛟ count, next-drop ETA, pause/cancel/priority), Lines (fill %, t/min), Demands (health dot, on-hand/target, "+ Create line" for unserved), Fleet (dev + truck/tanker/troop), Sites (stock, dev seed, from/to pickers); recent events. Module declared via `#[path]` in `logistics_targets_panel.rs` because `gui/mod.rs` was dirty. 3 panel tests. | LO-5 map overlay + stock bridge; LO-H economy test failures |
