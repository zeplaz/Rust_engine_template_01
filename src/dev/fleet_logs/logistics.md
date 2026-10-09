# Fleet `logistics` — progress log

Mission: standing supply lines vs one-off multi-trip supply runs, draw demands. Plan:
[`plan_logistics_orders_v1.md`](../plan_logistics_orders_v1.md).

| Date | Commit | Landed | Next |
|:---|:---|:---|:---|
| 2026-10-08 | LO-0 | Research (Mars Tactics Steam feed — write-up not found; built from user brief) + code survey + plan | LO-1 sim core |
| 2026-10-09 | LO-1/2 | `src/economy/logistics/orders/` pure-Rust sim: vehicle pool, one-off runs with multi-trip batching + completion/cancel/pause/fail, draw demands (hysteresis, integer draw, starving), standing lines, priority + aging + round-robin fairness, owner = faction slot `u8`. 25 headless tests green. | LO-3 snapshot + Bevy adapter |
