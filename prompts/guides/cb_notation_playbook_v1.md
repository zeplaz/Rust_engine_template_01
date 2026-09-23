# CB-notation playbook (repo adoption) `v1`

**Source research:** `C:\Users\oz_\Downloads\notation\` (DEV-15789)  
**Already in-repo:** [`prompts/SYMBOLIC_LANGUAGE.meta.md`](../SYMBOLIC_LANGUAGE.meta.md) §3.12 · [`tools/mcp/MICRO_TOOLS_REGISTRY_v1.md`](../../tools/mcp/MICRO_TOOLS_REGISTRY_v1.md) signature-book form

> Do **not** dump the full research into every session. Amortize the **law** below; leave prose for one-offs.

## One law

**Replace whole concepts, not words.** Glyph-for-word loses. Bind a reused concept to a cheap ASCII handle once (`G ≜ …`), then reuse.

## Blend (default)

| Faction | Job |
|:---|:---|
| prose | causal leap / one-off |
| `●◐○` | dense status vectors |
| `∀ ⇒ ≥` | gates / invariants |
| bound handle | concept reused ≥3× |
| emoji | sparse headline only |

## Agent I/O rules (this repo)

1. **MCP schema tax** — prefer signature-book / brief tools (`token_savings_guide`, `agent_flow_route`) over raw schemas + full files.
2. **Sacred tool names** — never strip disambiguating segments (`agent_flow_route`, not `flow_route`).
3. **Deterministic L0** — Python/MCP lints + digests before any L1 hard model (see `terrain_honesty_lint`, `pilot_hardcode_lint`).
4. **Deep reasoning only when dense** — HYP/EV/INFER + ρ (`cb-notation-reasoning.md`); skip for short status.
5. **Break-even** — codebook setup (~70–120 tok) needs reuse; one-shot answers stay prose.

## Workflow hook

```text
BLANG:FLOW L0
  → agent_flow_route(goal)
  → deterministic_*_lint / file_digest / witness_brief   ← no LLM
  → [L1 HARD only if hard_gate]
  → L2 EXEC
```

## Import map

| Downloads file | Use here |
|:---|:---|
| `cb-notation-summary.md` | this playbook |
| `cb-notation.md` | full spec — read on demand via `file_digest` |
| `cb-notation-reasoning.md` | L1 HARD diagnostics only |
| `cb-notation-presentation.md` | archive / evidence, not session orient |
