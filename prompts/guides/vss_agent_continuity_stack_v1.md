# GUIDE-VSS-AGENT-CONTINUITY-001 — Balance old context vs new work `v1`

```text
⟦SYMLANG⟧⟐v1  ◈GUIDE
Parent: $ref:src/dev/vss_wave1_drain_handoff_v1.md
```

Agents must **balance** stale program docs (old) against live witnesses (new). Both matter.

---

## Read order (token-cheap)

| Priority | Source | When |
|:---:|:---|:---|
| 1 | `vss_wave1_drain_queue.json` **your slice row** | Every session |
| 2 | `witness_brief <path>` | Before editing lane |
| 3 | TRIP_*.md for touched path | Before write |
| 4 | `symbolic_build_digest_tool` | After build |
| 5 | Full plan doc | Only if slice blocked |
| 6 | AGENTS.md | Boot only — not every turn |

---

## Old vs new

| Old (stable intent) | New (ground truth) |
|:---|:---|
| `plan_vfx_spectator_scale_program_v1.md` | `*_live.json` witnesses |
| `vss_001_kickoff_packet.yaml` | `vss_wave1_drain_queue.json` status |
| HANDOFF narrative | `git diff` + symbolic digest |
| Queue row `done` | Witness `green: false` → **witness wins** |

**Old agents** (archived plans, closed MIG rows) still **trigger**:
- Check `plan_deferral_registry_v1.md` DR-* before re-picking
- File tribunal dissent if reviving closed work

---

## Reminder card (paste in Task prompt)

```yaml
continuity:
  slice_id: VSS-T2-001
  read_first: tools/orchestrator/queues/vss_wave1_drain_queue.json
  witness: debug_runs/spectator_parity_live.json
  trip: src/dev/TRIP_VSS_TEST_HARNESS.md
  forbid: raw cargo walls, full witness JSON
  tools: symbolic_build_digest_tool, witness_brief, tribunal_on_touch
  dissent_slot: required
```

---

## Self-balance rule

If context feels **too new** (no plan): read child plan one-pager.  
If context feels **too old** (plan says done): read witness.  
If they disagree: `@operations-intelligence` L0 brief → HANDOFF — do not implement.
