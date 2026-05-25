# Stage 6 design decisions (Package E)

**Date:** 2026-05-23  
**Agent:** designer lane (docs only)

---

## S6-P2 — Inspector / registry table surface

**Decision:** **egui F8 dev overlay** remains the default in-engine surface for registry inspection during S6; a **desktop asset tool** (Python editor) stays the authority for bulk edits. Full Bevy inspector parity is **deferred** — track as **BQ** if product requires in-sim entity inspection at scale.

**Rationale:** S6 exit is residency/virtualization, not tooling rewrite. egui path already wired to shell; external editor owns RON/JSON assets.

---

## S6-25 — Minimap ghost-band tint

**Decision:** **Diagnostics only** for S6 exit (F3 residency strip + `ResidencyOverlayConsumerDto.chunks[].ghost_band`). **Minimap tint** (DQ-S6-04-B) deferred to post–S6-3 UX slice — no shader work in S6 gate.

**Spec when implemented:** ghost-band chunks use muted cyan outline at 40% alpha; core chunks unchanged.

---

## S6-S2 — HUD layout Wave S parallel

**Decision:** **May parallel S6-1** only for **BQ-130** (layout rects) using existing `HudLayoutCollectionR8` in `ProductShellPersistenceBundleR8` — schema is locked in `shell_persistence.rs`. **Do not** block S6-1 on Wave S save expansion (BQ-128/133).

---

## Sign-off

| ID | Status |
|----|--------|
| S6-P2 | Documented |
| S6-25 | Spec deferred (diagnostics-only for S6) |
| S6-S2 | Parallel allowed for BQ-130 only |
