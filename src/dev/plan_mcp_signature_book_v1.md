---
name: plan_mcp_signature_book_v1
owner: "@coder-mcp"
status: ready-to-implement
gate: callability ≥95% (separate axis from token-Δ)
---

`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md`

# Plan — MCP signature-book schemas (cut the ~96% schema tax)

**Source:** `$REPORT ≜ C:\dev\out_\[Gemini Conversation] Agent I_O Notation Research Report` (external,
referenced — never copied; valid lines 1–~2655). Findings: **§13 / W1 / WS10 (L435-503, 618-639,
1872-1945)**. Parent: `$ref:src/dev/plan_io_notation_upgrade_v1.md §6` · form: `$ref:prompts/SYMBOLIC_LANGUAGE.meta.md §3.12 SIGNATURE-BOOK`.

## Problem

```text
◎server.py ▷⊳ FastMCP "rust-engine-art" exposes 83 @mcp.tool() fns
  each tool's description(docstring)+inputSchema(signature) is injected on EVERY request
  ⟶ tool schemas ≈ 96% of the always-on token budget ($REPORT §13)   ← the real lever, not replies
```

## Target

```text
publish each tool as a ONE-LINE signature  ⟶  −92% @ 100% callability (WS10, $REPORT §13)
form:  name(req, req, [opt]) :type =default -> result        + a once codebook for shared fields
e.g.   geometry_run_job(job_path, [out_dir]) -> status
       validate_report(validator, [target, --compress:int=4, --cached]) -> report
```

```text
⚡ SACRED  keep the EXACT tool name (mcp__…rust-engine-art__<name>). Stripping the disambiguating
          segment drops cold-callability 100% → 12% ($REPORT §13 L468-474). Amortise ONLY the constant prefix.
```

## Approach (pick B if FastMCP allows)

```text
◆ how to emit the book ?
 A docstring-rewrite  ─ rewrite each @mcp.tool docstring → the one-line sig ; FastMCP derives the rest
 B list_tools post-process (PREFER) ─ keep Python signatures as source-of-truth; wrap/override the
    emitted tool list so description = sig line + a shared-field codebook, inputSchema kept minimal
    (required names+types preserved, verbose docstrings/`default:null` repeats stripped)
```

Files: `tools/mcp/python/rust_engine_mcp/server.py` (the 83 `@mcp.tool()` defs / FastMCP instance) ·
a new `schema_book.py` render helper (sig line from a fn signature) · tests in `tools/mcp/python/tests/`.

## GATE — executability first (report it on a SEPARATE axis from tokens)

```text
⬡ cold-callability ≥95%  ws10-style harness: fresh sub-agents emit a tool CALL from the sig line ALONE
                          (no full schema) → score call-validity. $REPORT §13 ws10_baseline/encodings/score.
⬡ lossless              any datum a consumer reads back (results/contracts) round-trips 100% (WS12).
⬡ token Δ                measure full-registry before/after (expect the WS10 ~10:1 ratio).
```

## Acceptance

```text
☐ signature book emitted for ALL 83 tools · exact names intact (SACRED)
☐ cold-callability ≥95% (harness green) — reported separately from token-Δ
☐ no required-arg / type dropped · results round-trip lossless
☐ pytest green (tools/mcp/python/) · CLI↔MCP parity preserved (MICRO_TOOLS_REGISTRY_v1.md updated)
☐ results/diff outputs as ●◐○ vectors where applicable ($REPORT W5)
```

## Boundaries

```text
@coder-mcp owns this (builds tools/mcp/). consumer-vs-builder: @coder/@designer do NOT build it.
docs already adopt the form (MICRO_TOOLS_REGISTRY_v1.md §Schema form · tools/mcp/README.md). This is the SERVER change that realises the −92%.
```

```text
⟦/plan_mcp_signature_book_v1⟧ NEXT ⚑ @coder-mcp → render sig book (B) → ws10 callability harness ≥95% → token Δ → pytest → registry parity
```
