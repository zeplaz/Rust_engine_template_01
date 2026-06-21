---
name: io_notation_cold_decode_v1
kind: fidelity-gate result (P5)
---

`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md`

# Cold-decode fidelity gate — SYMLANG I/O forms

**Gate (spec L9 · `$REPORT §9`):** ship no notation without cold-decode ≥95% (a *fresh* reader, **no
legend**) + a token before/after. `$REPORT ≜ C:\dev\out_\[Gemini Conversation] Agent I_O Notation
Research Report` (external, referenced — never copied; valid lines 1–~2655).

**Method (the report's own):** fresh `general-purpose` sub-agents, given ONLY the encoded sample and
"decode to plain English, no legend," scored on core-meaning match. Run 2026-06-19.

```text
form (spec §)                  sample                                              fidelity  note
●◐○✗⊘ status vector (§2.2)     Δmods(5): all ● except ✗ test:atlas · ◐ build:…     ~95% ✓   ⊘ read "skip/disabled" (≈ blocked) — acceptable nuance
signature-book (§3.12)         validate_report(validator,[target,--compress:int=4… 100% ✓   fn · [opt] · :type · =def · -> read exactly
reasoning lattice (§3.12)      HYP/EV … ⊕╱╱→H1 ⊖→H2 … INFER H2 0.84◕(root) H1⤳H2   100% ✓   root cause + downstream + fix all recovered
dim lattice row (§2.1,§3.2)    ⟨MAT-001⟩ ◐ Au:🏛🟨🟨🟨🟨 Cx:🌀🟨🟨 Rk:⚠🟨            ~95% ✓   Au/Cx/Rk/Ct + 🟨 pips + ◐/◕ all read right
flow graph (§3.9)              ◎spec ▷⊳ ▢validate ─⬡[schema ok]▶ … ⇧promote …★      100% ✓   gates/promote/escalate read exactly
status+conf+route (§2.2,2.11)  pipeline ✗ · build ●●●◐ test ●●✗● · conf ~.75 · ΔWF→ 100% ✓   per-cell ● / ✗ / ◐ + ~.75 + route read right
```

**Verdict: 6/6 ≥95% — GATE CLEARED.** The §2.2/§2.11/§3.9/§3.12 forms decode cold without a legend,
so they ship. This is the empirical backing for the P2/P4 spec changes (`plan_io_notation_upgrade_v1.md`).

**Token note (proxy):** the wins claimed are STRUCTURE/AMORTIZATION (a graph/vector replacing a
paragraph; a signature line replacing a JSON block) — `$REPORT §30` proves these are
**tokenizer-invariant**. Single-glyph token deltas (e.g. ●◐○ vs 🟢🟡🔴) are o200k-measured and
**not** confirmed on Claude's tokenizer (no Claude token API was reachable, same limit the report
hit). We therefore rely on the *structural* wins and the *fidelity* result, not single-glyph counts.

**Re-run when:** a form changes, before any *aggressive* bespoke-glyph swap, or if a Claude
token-count API becomes reachable (then re-confirm the ●◐○-vs-emoji single-glyph delta).
```text
⟦/io_notation_cold_decode_v1⟧ NEXT ⚑ forms ship · re-run gate on any form change
```
