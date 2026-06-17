# DES-APS-E1-IA-OPTION-D-001 — Domain lane IA sign-off `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-E1-IA-OPTION-D-001** |
| **Program** | APS-E1 · blocks **APS-EVO-E1-DOMAIN-ROUTER-001** |
| **Parent** | [`design_aps_uiux_style_quality_20260616_v1.md`](design_aps_uiux_style_quality_20260616_v1.md) §2 |
| **Date** | 2026-06-16 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** — Option **D** signed |

---

## Decision

**Sign Option D: top-level Lane switch (Buildings ⇄ Landscape)** — **not** a 6th tab, **not** nested notebooks, **not** context-morph tabs.

| Rejected | Why |
|:---|:---|
| A — 6th tab "Landscape" | Mega-tab; building verbs bleed into veg |
| B — Tab groups | Tk keyboard/a11y nightmare |
| C — Auto-morph tabs | Spatial inconsistency; artist distrust |

---

## Signed behavior (@coder-mcp)

```text
┌─ Lane: [▣ Buildings] [ Landscape ] ─────────────────────────┐
│  Flow bar + authority strip + pipeline bar = lane-scoped       │
│  Notebook tab SET swaps on lane change                         │
├─ Buildings lane (default, byte-identical to today) ────────────┤
│  Tabs: Catalog · Assembly · Materials · Variants · Atlas       │
│  Authority: assembly_snapshot                                  │
├─ Landscape lane ───────────────────────────────────────────────┤
│  Tabs: Presets · Grammar · States · Atlas                      │
│  Authority: landscape_grammar preset (land_dna + topology_graph)│
└────────────────────────────────────────────────────────────────┘
```

### Isolation rules (hard)

1. `SuiteState.active_lane: "buildings" | "landscape"` — default `"buildings"`.
2. Lane switch **does not** carry selection across lanes (no silent cross-lane state).
3. Persist `active_lane` in `aps_ui_prefs.json`.
4. Lane identity = **word + chip + tint** — never color alone (`Buildings` / `Landscape`).
5. Ship Buildings lane first with router infra; Landscape tabs land when E2–E4 specs exist.

### Chrome hooks

| Surface | Buildings | Landscape |
|:---|:---|:---|
| Authority strip | `Ship truth: assembly_snapshot …` | `Ship truth: landscape_grammar preset …` |
| Pipeline STEPS | Catalog→Assembly→Materials→Variants→Atlas | Presets→Grammar→States→Atlas |
| Flow verbs | Send to Assembly · Bake variants · Pack atlas | Generate grammar · Bake states · Pack LG-5 atlas |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-16 |

```text
DES-APS-E1-IA-OPTION-D-001 Q✓
Option D lane switch signed — unblocks APS-EVO-E1-DOMAIN-ROUTER-001
```
