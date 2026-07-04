# Designer gang lane status `v1` — who is doing what (2026-07-03)

| Field | Value |
|:---|:---|
| **Owner** | `@designer` (read-only rollup for coordination) |
| **Hub** | [`agent_hub_queue_v1.json`](../../tools/orchestrator/queues/agent_hub_queue_v1.json) |
| **Handoff** | [`HANDOFF.md`](../../tools/orchestrator/queues/HANDOFF.md) |

**Designer rule:** specs + registry + queue hygiene only — no Rust/MCP implementation in this lane.

---

## P0 — active picks (other lanes)

| Lane | PICK now | Program | Notes |
|:---|:---|:---|:---|
| **coder-mcp** | **idle** | APS spine | G4 + tier-2 + polish tail **done** 2026-07-03 |
| **coder_a** | **idle** | CITY-G0 | G0a done 2026-07-03 |
| **coder_b** | **idle** | CITY-G0 | G0b done 2026-07-03 |
| **coder** | **`CITY-G2-C5-001`** | PLAN-CITY-GRAMMAR-v1 G2 | Palette resolver — charter signed |
| **designer-mcp** | **idle** | CITY | Block + palette charters **PASS** |
| **operator** | `APS-G4 pixel follow-up` (optional) · `G-PLAY-01` | POST-DRAIN | Needs display session |
| **operator** | `DES-APS-TAG-RUBRIC-001` tier-1 walk | APS tags | Charter ready |

---

## P0 — designer lane (APS + city — closed)

| ID | Action | Status |
|:---|:---|:---|
| **DES-APS-*** building UI tail | tags · UX audit · tier exposure | **done** |
| **DES-SIM-HUD-COHESION/COPY/TRAY** | Track F P0 specs | **done** · registry re-signed |
| **DES-CITY-BLOCK-DEBUG-READ-001** | F3 block/recipe overlay legend | **done** 2026-07-03 |

---

## P2 — city grammar (designer-mcp — G1/G2 charters closed)

| ID | Phase | Status | Notes |
|:---|:---|:---|:---|
| **DES-CITY-BLOCK-RECIPE-001** | G1 / CITY-C3 | **PASS** | BlockRecipe v1 + 3 RON examples |
| **DES-CITY-BLOCK-DEBUG-READ-001** | G1 debug | **PASS** | F3 overlay legend — engineer path |

**Queue:** [`city_grammar_queue.json`](../../tools/orchestrator/queues/city_grammar_queue.json) · active phase **G2**

---

## Dependency spine (updated)

```text
CITY-G0 + G1 (C1→C4→C3)          [DONE 2026-07-03]
DES-CITY-BLOCK-RECIPE-001         [designer-mcp PASS]
DES-CITY-PALETTE-VARIATION-001    [designer-mcp PASS 2026-07-03]
        ↓
CITY-G2-C5-001                    [coder PICK — module_index + tile_atlas_index]
```

---

## Do not open (designer)

- New **building** tier matrix specs — [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md) still authoritative for APS
- Block-tier UI in APS before G1 engine lands — charter recipes first (designer-mcp)
- Stage 5 / construction param re-litigation — closed boards

---

## Changelog

| Date | Notes |
|:---|:---|
| 2026-06-02 | Initial gang rollup + APS presence designer tail |
| 2026-07-03 | Registry hygiene 24× SIGNED · **DES-CITY-BLOCK-DEBUG-READ-001** |
