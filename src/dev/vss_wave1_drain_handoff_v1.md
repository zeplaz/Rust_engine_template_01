# VSS-WAVE1-DRAIN-HANDOFF-001 — Multi-channel agent drain + token contingency `v1`

```text
⟦SYMLANG⟧⟐v1  ◈HANDOFF  ◈DRAIN
⟨ID⟩ VSS-WAVE1-DRAIN-001
Parent: $ref:src/dev/plan_vfx_spectator_scale_program_v1.md
Queue: $ref:tools/orchestrator/queues/vss_wave1_drain_queue.json
Drain status (L0): $ref:debug_runs/agent_ops/vss_wave1_drain_status.yaml
Continuity: $ref:prompts/guides/vss_agent_continuity_stack_v1.md
Status: ACTIVE — 2026-08-28
```

**If parent agent hits token limit:** next agent reads **only** this file + queue JSON + their slice witness — NOT full chat history.

---

## Wave 1 parallel channels (launch together)

| Ch | Slice | Agent | Model tier | Authority touch | Stop rule |
|:---:|:---|:---|:---|:---|:---|
| **A** | VSS-T2-001 | @coder | L2 exec | `test_harness.rs`, witness only | No camera edits |
| **B** | VSS-T1-002 | @coder | L2 exec | `world_scale_contract.rs`, `tile_world_fallback.rs` | No harness fire logic |
| **C** | VSS-T4-002 | @designer-mcp | L2 exec | schema + examples only | No Rust |
| **D** | VSS-T4-003 prep | @coder-mcp | L0→L2 | `validators/effect_spec.py` stub | No promote yet |
| **E** | Witness digest | @operations-intelligence | **L0 cheap** | read-only | YAML packet only |
| **F** | Dual-write map | @sim-steward | L0→L1 | read-only routing | No impl |

**SERIAL — never same session:** A ∩ B on `test_harness.rs` same frame. If both active, **A first** (harness flags), then **B** (camera constants).

---

## Token contingency stack (mandatory)

```text
SESSION START
  1. node .claude/skills/agent-lang/driver.mjs boot <agent>
  2. Read THIS handoff + vss_wave1_drain_queue.json (slice row only)
  3. agent_flow_route(goal=slice_id, domain=auto)
  4. L0: witness_brief · slice_exec_brief · token_savings_guide
  5. L2: implement ONLY if packet filled

NEVER
  - Full tactical_map_debug JSON
  - Full cargo stderr
  - Re-read entire plan_vfx + plan_world_scale + plan_artist in one turn

ALWAYS
  - symbolic_build_digest_tool after cargo
  - tribunal_on_touch on trip paths
  - WIT-HON before Q✓
```

---

## Save stack (human + agent improvable)

| Layer | Path | Purpose |
|:---|:---|:---|
| **S0** | `tools/orchestrator/queues/vss_wave1_drain_queue.json` | Machine slice status |
| **S1** | `debug_runs/agent_ops/vss_001_kickoff_packet.yaml` | Q/C/E + risks (old but stable) |
| **S2** | `debug_runs/symbolic_digests/*.notes.md` | Human notes on build digests |
| **S3** | `debug_runs/tribunals/*.json` | Dissent preserved |
| **S4** | Per-slice witness JSON | Scoped green truth |

**Balance old vs new:** Read S0+slice row first (new). Touch S1 only if routing conflict. S2–S4 on demand.

---

## Cheap-agent triggers (worth it)

| Trigger | Cheap agent | Output |
|:---|:---|:---|
| Queue stale vs witness | @operations-intelligence | 20-line ΔWF |
| "Is this dead code?" | @cleanup-intelligence | classify packet |
| Authority drift | @debug-intelligence | routing YAML |
| Schema only | @planner-mcp | no bpy |
| Token budget low | @flow-controller | compress phases |

Old agents / archived plans **trigger** DR rows and tribunal — run L0 brief before dismissing.

---

## Parent token exhaustion protocol

```text
Parent low tokens:
  1. Write slice status → vss_wave1_drain_queue.json
  2. invoke_handoff.ps1 -Goal "Resume VSS Wave1 Ch X" -Lane VSS-001
  3. New chat: @main-thread-orchestrator OR disable Multitask + @coder in chat
  4. Resume Task ID from queue row if background worker finished
```

---

## Exit (Wave 1)

All must be true or explicit dissent filed:

- [x] `spectator_parity_live.json` — **all product paths green** (scenario + save + interactive) VSS-T2-003
- [x] `world_scale_contract_live.json` — T1-003 designer sign-off (`buildings_per_frame` 10.15, dissent on bpf cap)
- [x] `artist_vfx_pipeline_live.json` — T4-003 shipped + **T4-004** `EffectConsumableRegistry`; G4 `honest_gate` pending
- [x] `symbolic_build_encoding_live.json` — fresh digest (T5-001)

---

## @orchestrator-status

`@orchestrator-status: ACTIVE`  
`@orchestrator-owner: @main-thread-orchestrator` on Task quota failure
