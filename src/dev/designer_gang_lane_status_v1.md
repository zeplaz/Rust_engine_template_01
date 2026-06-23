# Designer gang lane status `v1` — who is doing what (2026-06-02)

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
| **coder-mcp** | `APS-GRAM-TIER-GATES-LIVE-001` · `DES-APS-SESSION-DUMP-001` | APS-PRESENCE-CORRECTION | Guard parity **done**; tier gates + session dump parallel |
| **coder-mcp** | `APS-E1-CHROME-001` · MCP-P2-* · VEG-F02 | APS-OPTION-D / MCP-P2 | Chrome blocked on DES mockup — **PASS on disk** |
| **coder** | `BUILD-READ-REWIRE-003/004` · `SIM-EFFECT-*` · `APS-QC-REWIRE` | POST-DRAIN phase 4/5 | Reopened by INTEL-OFFICER — witnesses mostly green |
| **planner** | `PLAN-APS-PRESENCE-PLAN-EDIT-001` | APS-PRESENCE-CORRECTION | Amend G0→G3 plan examples |
| **operator** | `G-PLAY-01` · `PERF-SHELL-001` | POST-DRAIN phase 2/3 | Needs display session |
| **operator** | `OVR-APS-PRESENCE-OPERATOR-001` | APS-PRESENCE | **Blocked** until session dump green |
| **sim-steward** | VM-09 bridge tail | infra | Lockstep / invert bridge |
| **orchestrator-mcp** | `WH-TRACK-B-PAUSE` | grammar continuation | Track B pause witness |

---

## P0 — designer lane (this session)

| ID | Action | Status |
|:---|:---|:---|
| **DES-APS-DEFAULT-PRESENCE-AUDIT-001** | Audit PASS — registered | **done** |
| **DES-APS-ASSEMBLY-EMPTY-G2-001** | Copy spec for coder-mcp | **done** |
| **INTEL-OFFICER re-close** | Power · P55 · VM-11 · T1 APS blocked→done | **in flight** |
| **OVR-DES-P55-PREVIEW-SPEC-001** | Spec PASS — unblocks `OVR-P55-PREVIEW-001` | **re-close** |
| **DES-POWER-NODE-HOVER / VOLTAGE** | PASS specs — registry sync | **re-close** |

---

## Dependency spine (APS presence)

```text
APS-GUARD-BRIEF-PARITY-001          [coder-mcp DONE]
        ├─ APS-GRAM-TIER-GATES-LIVE-001   [coder-mcp PICK]
        ├─ DES-APS-SESSION-DUMP-001       [coder-mcp PICK]
        └─ DES-APS-ASSEMBLY-EMPTY-G2-001  [designer copy DONE → coder-mcp impl]
                ↓
PLAN-APS-PRESENCE-PLAN-EDIT-001     [planner PICK]
OVR-APS-PRESENCE-OPERATOR-001       [operator WAIT]
```

---

## Coder spine (post-drain — not designer)

| Track | Hot rows | Designer already shipped |
|:---|:---|:---|
| **T2 BUILD-READ** | REWIRE 003/004 · pilot lint | [`design_build_read_hud_v2.md`](design_build_read_hud_v2.md) |
| **T4 FIRE** | F2 extract done · ignition P0 | [`fire_f2_extract_readability_pass_001.md`](fire_f2_extract_readability_pass_001.md) |
| **T5 SIM-HUD** | ESC cascade done · theme wired | F2 overlay/minimap/pause v2 specs |
| **T6 POWER** | COD-POWER-* waits on design PASS | All Track A–D specs PASS on disk |
| **T8 INFRA** | VM-10 lockstep green · VM-11 audit | [`design_vm11_preview_audit_v1.md`](design_vm11_preview_audit_v1.md) |

---

## Designer queue hygiene targets

| Queue | Issue | Fix |
|:---|:---|:---|
| `multi_parallel_home_queues_v1.json` | T1 APS rows `blocked` despite PASS specs | `done` + `witness` + `exit_predicate` |
| `designer_signoff_registry.json` | INTEL-OFFICER reopen stubs | Restore **SIGNED** where spec **PASS** |
| `aps_uiux_overhaul_queue.json` | P55 spec `reopened` | Mark **done** — unblocks coder-mcp preview |
| `designer_active_queue.json` | `multi_parallel_ready` stale reopen | Align with registry |

---

## Do not open (designer)

- New tier matrix specs — [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md) still authoritative
- Landscape LG-5 matrix amend — separate spine per audit §3.2
- Stage 5 / construction param re-litigation — closed boards

---

## Changelog

| Date | Notes |
|:---|:---|
| 2026-06-02 | Initial gang rollup + APS presence designer tail |
