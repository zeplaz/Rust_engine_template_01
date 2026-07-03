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
| **coder_a** | `CITY-G0-S11-001` | PLAN-CITY-GRAMMAR-v1 | Typed grammar ids (G0a) |
| **coder_b** | `CITY-G0-S1C-001` | PLAN-CITY-GRAMMAR-v1 | `building_grammar.rs` split (G0b) |
| **coder** | `MIG-P0-G2-001` · `CITY-G0-WIT-001` after G0a/b | MIG + city | Determinism witness (G0c) |
| **designer-mcp** | **blocked** | PLAN-CITY-GRAMMAR-v1 | `DES-CITY-BLOCK-RECIPE-001` after G0c |
| **operator** | `APS-G4 pixel follow-up` (optional) · `G-PLAY-01` | POST-DRAIN | Needs display session |
| **operator** | `DES-APS-TAG-RUBRIC-001` tier-1 walk | APS tags | Charter ready |

---

## P0 — designer lane (APS building UI — closed)

| ID | Action | Status |
|:---|:---|:---|
| **DES-APS-DEFAULT-PRESENCE-AUDIT-001** | Audit PASS | **done** |
| **DES-APS-ASSEMBLY-EMPTY-G2-001** | Copy spec | **done** · coder-mcp wired |
| **APS-UX-AUDIT-001 v2** | PASS WITH NOTES | **done** · polish tail shipped |
| **DES-APS-GRAM-TIER-004** | G0/G1 empty copy | **done** — **building** tier only |
| **DES-APS-TAG-TIER2-001** | Preset spec | **done** · coder-mcp wired |
| **DES-APS-TAG-RUBRIC-001** | Operator charter | **done** (operator pending) |

---

## P2 — city grammar (designer-mcp — NOT YET ACTIVE)

| ID | Phase | Status | Blocker | Notes |
|:---|:---|:---|:---|:---|
| **DES-CITY-BLOCK-RECIPE-001** | G1 / CITY-C3 | **DRAFT** | `CITY-G0-WIT-001` | Charter on disk — designer-mcp PASS after G0c |
| **DES-CITY-PALETTE-VARIATION-001** | G2 / CITY-C5 | **blocked** | G1 gate (`CITY-G1-C3-001`) | Kit × palette charter — **designer-mcp** |

**Queue:** [`city_grammar_queue.json`](../../tools/orchestrator/queues/city_grammar_queue.json)

**Gap vs APS specs:** Existing APS designer work covers **building grammar UI** (tier chips, tags, presence). City plan adds a **block tier** above buildings — **no designer spec on disk yet**; rows above are seeded and blocked correctly on G0.

---

## Dependency spine (updated)

```text
APS-PRESENCE + G4-COVERAGE          [coder-mcp DONE 2026-07-03]
        ↓
CITY-G0-S11 / G0-S1C              [coder_a / coder_b PICK]
        ↓
CITY-G0-WIT-001                   [coder — determinism witness]
        ↓
DES-CITY-BLOCK-RECIPE-001         [designer-mcp — first city designer pick]
        ↓
CITY-G1 (C1→C2→C3)               [coder + designer-mcp recipe]
        ↓
DES-CITY-PALETTE-VARIATION-001    [designer-mcp G2]
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
| 2026-07-03 | G4 closed · city grammar designer-mcp rows seeded · APS building designer lane closed |
