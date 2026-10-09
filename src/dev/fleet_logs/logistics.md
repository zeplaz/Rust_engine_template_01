# Fleet `logistics` — progress log

Mission: standing supply lines vs one-off multi-trip supply runs, draw demands. Plan:
[`plan_logistics_orders_v1.md`](../plan_logistics_orders_v1.md).

| Date | Commit | Landed | Next |
|:---|:---|:---|:---|
| 2026-10-08 | LO-0 | Research (Mars Tactics Steam feed — write-up not found; built from user brief) + code survey + plan | LO-1 sim core |
| 2026-10-09 | LO-1/2 | `src/economy/logistics/orders/` pure-Rust sim: vehicle pool, one-off runs with multi-trip batching + completion/cancel/pause/fail, draw demands (hysteresis, integer draw, starving), standing lines, priority + aging + round-robin fairness, owner = faction slot `u8`. 25 headless tests green. | LO-3 snapshot + Bevy adapter |
| 2026-10-09 | LO-3 | Snapshot (`orders/snapshot.rs`: site/order/demand/vehicle/fleet/event rows, AI helpers `sites_of`, `unserved_demands`), `SiteDistanceTravel` (metres, detour 1.3), Bevy adapter `LogisticsOrdersPlugin` (1 s steps from `SimTimeMicros`, `LogisticsOrderRequest` message = sole write path, `LogisticsOrderSite` component any lane can attach, depot fleet seed, hubs/flow facilities auto-tagged). 30 tests green. Found 5 **pre-existing** economy test failures (log_c_reservations, activation live_json ×2, spatial_district, logistics live_json) — queued as LO-H. | LO-4 UI panel |
