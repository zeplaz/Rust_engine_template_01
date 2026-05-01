# G3B — Production HUD remediation `v1`

> **STATUS:** **Core shipped** — site-bound HUD, pick/focus, and F9/F10 surfaces are in code paths below (**§ Done**). **§ Next** is **product/design-gated** polish; track and resolve via designer brief [`rulebook_backlog_designer_brief_v1.md`](../../../guides/rulebook_backlog_designer_brief_v1.md) §4 **BQ-103**.

**Pair:** G3 index [`g3_gui_todos_steps_v1.md`](g3_gui_todos_steps_v1.md) · GUI orchestrator [`../../../guides/gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) · gap orchestrator [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md).

---

## Scope

Bevy HUD + F10 picker: physical inventory for the resolved focus (hub roll-up or single storage).

---

## 1. Owning spec

- **G3B** + production economy specs + ECS on storage / site entities.

---

## 2. Recorded answers (policy)

| Topic | Decision |
|:---|:---|
| **Sites** | **[`LogisticsSiteRoot`](../../../../src/entities/production/core/logistics_site.rs)** on hub; **[`LogisticsSiteMember { hub }`](../../../../src/entities/production/core/logistics_site.rs)** on trucks / sidings / warehouses. Clicking a member resolves focus to **hub**; HUD merges **all** `ResourceStorage` on hub + members. |
| **Capacity** | **[`ResourceStorageCapacity.max_amounts`](../../../../src/entities/production/core/resources.rs)** — summed per type across merged entities for bar denominator; UI shows `stock/cap` as text hint. |
| **Focus** | **[`HudLogisticsFocus`](../../../../src/gui/logistics_focus.rs)**; set by pointer, F10 list, or F9. |
| **Flow** | `ResourceProducer` / `ResourceConsumer` read from **hub** when focus is a site hub; single-storage otherwise. |

---

## Done (shipped)

- Site-bound HUD (no global pool): **[`HudLogisticsFocus`](../../../../src/gui/logistics_focus.rs)** targets a **hub** or standalone storage.
- **Primary-click** on entities with [`ResourceStorage`](../../../../src/entities/production/core/resources.rs) (after auto [`Pickable`](https://docs.rs/bevy)) sets focus; **[`LogisticsSiteRoot`](../../../../src/entities/production/core/logistics_site.rs) / [`LogisticsSiteMember`](../../../../src/entities/production/core/logistics_site.rs)** roll up member inventories.
- **[`ResourceStorageCapacity`](../../../../src/entities/production/core/resources.rs)** drives **stock/cap** bars and `/cap` hint.
- **F9** cycles storages; **F10** opens egui **[`LogisticsTargetsPanelPlugin`](../../../../src/gui/logistics_targets_panel.rs)** (building-panel style list).
- **[`InGameHudPlugin`](../../../../src/gui/in_game_hud.rs)** is registered on [`EnginePlugin`](../../../../src/engine/engine_with_worldgen.rs).
- 3D pick path: `attach_storage_picking_hooks` + `On<Pointer<Click>>` in [`in_game_hud.rs`](../../../../src/gui/in_game_hud.rs); list panel [`logistics_targets_panel.rs`](../../../../src/gui/logistics_targets_panel.rs); site graph [`logistics_site.rs`](../../../../src/entities/production/core/logistics_site.rs).

---

## 3. Living map

| Mechanism | Source |
|:---|:---|
| 3D pick | `attach_storage_picking_hooks` + `On<Pointer<Click>>` in [`in_game_hud.rs`](../../../../src/gui/in_game_hud.rs) |
| List panel | [`logistics_targets_panel.rs`](../../../../src/gui/logistics_targets_panel.rs) |
| Site graph | [`logistics_site.rs`](../../../../src/entities/production/core/logistics_site.rs) |

---

## Next (optimization — BQ-103)

Do not expand this list without updating **BQ-103** in [`rulebook_backlog_designer_brief_v1.md`](../../../guides/rulebook_backlog_designer_brief_v1.md) §4.

1. Replace entity-id buttons with **names** / scenario IDs when available.
2. Gate **F9** behind `dev_tools` if desired.
3. In-transit / pipeline row (not warehouse stock).
4. Richer Bevy bars / tooltips.
