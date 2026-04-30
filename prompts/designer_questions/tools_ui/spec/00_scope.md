# Tools UI — scope `00`

## In scope

- **egui** tooling: diagnostics, editors, cross-domain inspect (`tooling_cross_domain_v1.md`).
- **Bevy UI** player HUD: charts, alerts — rules in `debug_perf_ui_split_v1.md`.
- **Feature / runtime gates** so retail builds strip or disable dev panels.

## Out of scope

- Non-engine product storefronts; keep to **sim + editor** surfaces.

## Authority

- Tools that **mutate sim** respect MP server validation — same commands as gameplay where applicable.
