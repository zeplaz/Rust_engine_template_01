# APS-VALIDATOR-PLAIN-SIGN-001 — planner sign-off `v1`

| Field | Value |
|:---|:---|
| **Slice ID** | **APS-VALIDATOR-PLAIN-SIGN-001** |
| **Spec** | [`aps_validator_plain_language_v1.md`](aps_validator_plain_language_v1.md) |
| **MCP pair** | **MCP-P0-PLAIN-001** — `validate_p0_gate_plain` ([`plan_mcp_productivity_chain_v1.md`](plan_mcp_productivity_chain_v1.md)) |
| **APS Tk pair** | **APS-VALIDATOR-PLAIN-002** — `rust_engine_mcp/aps_validator_plain.py` |
| **Status** | **SIGNED** |
| **Date** | 2026-06-03 |
| **Owner** | `@planner` sign · `@coder-mcp` implement |

---

## Verdict

The plain-language table in **APS-VALIDATOR-PLAIN-001** is **authoritative** for artist-facing P0 gate messages. Technical `ValidationIssue.hint` + `field` remain secondary (expandable).

**Sign-off scope:**

- Production, grammar, and materials signatures covered
- Display contract: sentence first, arrow hint second, technical block collapsed
- Markdown authoritative; JSON mirror optional later

---

## Implementation contract (@coder-mcp)

| Surface | Behavior |
|:---|:---|
| **MCP** `validate_p0_gate_plain(path)` | Returns `{ status, artist_messages[], signature_count, technical }` |
| **APS Tk** `assembly_panel.on_validate_p0` | Same mapper; no modal on fail — inline panel (APS-UX-NONBLOCK-001) |
| **Loader** | `aps_validator_plain.py` reads signatures from spec; unknown signature → generic sentence + raw hint |

**Ship rule:** MCP tool and APS inline panel must use **same** mapper — no drift.

---

## Acceptance

| # | Criterion |
|:---:|:---|
| 1 | Every signature in spec table has unit test row |
| 2 | `validate_p0_gate_plain` witness in `aps_validator_plain_002_live.json` or MCP productivity witness |
| 3 | Unknown signature does not panic — falls back gracefully |

---

## Orchestrator paste

```text
@coder-mcp — ship MCP-P0-PLAIN-001 same day as APS-VALIDATOR-PLAIN-002

Spec SIGNED: aps_validator_plain_signoff_v1.md
Implement: aps_validator_plain.py + validate_p0_gate_plain MCP tool
Pair with APS-UX-NONBLOCK-001 for inline display (no modal on P0 fail)
```
