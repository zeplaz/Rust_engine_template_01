---
name: plan_io_notation_upgrade_v1
status: ready-to-run
authored-from: CB-notation research report (external, referenced — never copied)
---

`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md` — this plan is authored in SYMLANG.

# Plan — Agent I/O Notation upgrade (CB-notation → SYMLANG + all skills/agents/MCP docs)

## §0 Source (REFERENCE — never copy)

```text
$REPORT ≜ C:\dev\out_\[Gemini Conversation] Agent I_O Notation Research Report
  external · read-only · DO NOT copy its text into any artifact — cite it by anchor
  ⚠ valid content = lines 1–~2655 ; lines 2656–3951 are binary corruption — never read past 2655
  Read in ≤700-line windows (Read caps ~25k tok). Refs below use  $REPORT §N Lx-Ly.
```

When executing any work item, **read its `$REPORT` anchor first**, apply, then gate. The report
is the authority for *why*; this plan is the authority for *what/where*. Every recommendation
below carries its anchor so the rationale stays one hop away without duplicating the text.

## §1 Thesis — what the report changes (reflection, not just adoption)

The report is **not** "add more glyphs." Its headline is that **pure-symbolic is the *worst*
encoding** (176 tok > prose 159 > blend 144 — `$REPORT §5 L134-158`), and several of SYMLANG's
own choices are measured losers. We rebalance toward **the blend**, gate hard on **cold-decode +
token-delta**, swap **expensive glyphs for cheap-clear ones**, and chase the **real lever (MCP
schema tax)**.

```text
◆ the 7 findings → our delta
 1 BLEND≻pure          $REPORT §5,§19 L134,L1386   prose+emoji(sparse)+●◐○+∀⇒≥+handles+→@role ; SYMLANG must STOP over-symbolising
 2 COLD-DECODE = gate  $REPORT §9 L275-301          a token win that won't decode cold (fresh reader, no legend) = NOT a win → make it a LAW
 3 dense status ●◐○    $REPORT §8,§16 L257-266,1339 ●◐○ = ½ tokens of 🟢🟡🔴, equally glance-legible ; emoji ONLY for sparse headline status
 4 our glyphs cost     $REPORT results L1062-1071    ⟦META⟧=7 tok · ◔◑◕●=7 · ⟨ ⟩=3 each — SYMLANG's own delimiters are expensive → audit + cheapen
 5 ¬domain glyph lex   $REPORT §11 Rec5 L1670-1702   terse English term beats a bespoke glyph (decode-cold cost ≈ 12 tool calls/44k tok) → gate §2.13 hard
 6 single-glyph fragile $REPORT §30 L1704-1736       ∀⇒○ = 1 tok on o200k but 2–3 on older BPE ; STRUCTURE/AMORTIZATION wins are tokenizer-invariant
 7 MCP schema = 96% tax $REPORT §13,W1 L435-503,618  the ~real budget is tool schemas on every request → signature-book = −92% @100% callable
```

```text
KEEP (validated by the report) :
  standard notation ∀⇒≥□ (free, legend-less, computable — §8) · graph/DSM motif-factoring (§11 L1641)
  reasoning lattice HYP/EV/INFER+ρ (§12) · table/default-exception/composite-alphabet (§6) · →@role routing
DROP / GATE :
  glyph-for-word substitution · bespoke domain lexicons (gate) · emoji for DENSE status · reliance on single-glyph token wins
```

## §2 Scope — every artifact this plan touches

```text
SURFACE                         FILES                                                      OWNER-LANE
A SYMLANG spec (the language)   prompts/SYMBOLIC_LANGUAGE.meta.md (canonical)              §5 below
                                .claude/SYMBOLIC_LANGUAGE.md (redirect — leave)
                                claude-portal-skills/SYMBOLIC_LANGUAGE.meta.md (generic)   §5 (port the generic-safe edits)
B .claude skills (source)       .claude/skills/*/SKILL.md  (10)                            §4 P1-P3
C .claude agents                .claude/agents/*.md  (13)                                  §4 P1-P3
D .cursor skills+agents         derived from B/C via sync (see §3) — do NOT hand-edit      §3
E portal (generic templates)    claude-portal-skills/skills+agents (7+11)                  §4 (mirror, kept generic)
F MCP md / docs                 tools/mcp/README.md · MICRO_TOOLS_REGISTRY_v1.md ·         §6 (W1 lever)
                                CLAUDE.md (if present) · prompts/llm_agent_brief.md
G MCP server/CLI (schema text)  tools/mcp/python/rust_engine_mcp/  (signature-book)        §6 (W1 — server-side)
```

## §3 Deployment mechanism (do NOT hand-edit twice)

`.claude/skills` is the **source of truth**; `.cursor/skills` is **derived**. The repo already ships
the sync path (seen in `agent-lang/SKILL.md` §Skill-parity):

```text
edit .claude/skills/<x>/SKILL.md  ⟶  powershell -NoProfile -File .cursor/skills/sync-claude-skills/scripts/sync.ps1  ⟶  .cursor/skills/*
```

```text
⚑ Pre-req W0 (verify before any sync): confirm sync.ps1 covers BOTH skills AND agents.
   IF it covers skills only ⟶ extend it to mirror .claude/agents → .cursor/agents (or hand-port agents once, documented).
   $ref:.cursor/skills/sync-claude-skills/SKILL.md
Portal (E) is a SEPARATE generic copy (no project bindings) — apply only the generic-safe edits (§5 marks them ⟦GEN-SAFE⟧).
```

## §4 Phased work plan (maps `$REPORT §16` P1–P6 + `§17` W1–W7 onto our repo)

Order = risk-adjusted payoff. **Each phase has a gate; do not proceed past a red gate.**

```text
◆ P1 · FREE LEVERAGE (zero-legend, do first)          $REPORT §16 L572-574 · §8 · §14
  change   keep ∀⇒≥□ standard notation ; ensure gates/invariants in skills+agents use it (already partial)
  files    B,C all ; A spec §8 unchanged (validated)
  ⬡gate    spot-read 3 real docs stay legible (no regression)            risk ~none

◆ P2 · STATUS REBALANCE — emoji ⟶ ●◐○ for DENSE       $REPORT §8 L257-266 · results §15,§16 L1306-1353
  change   dense status vectors/clusters use ●◐○✗ (●=pass ◐=running ○=skip ✗=fail) ;
           reserve 🟢🟡🔴/emoji for SPARSE headline status only ; graded confidence ◔◑◕● ⟶ write ~.5/~.75/1.0  ($REPORT L2398)
  files    A spec §2.1/§2.2/§2.11 (§5 below) ; then B,C status lines
  ⬡gate    cold-decode ≥95% on a 10-item status sample  +  token before/after vs the emoji form (measure on Claude's tokenizer, not o200k)
  ⚠note    single-glyph wins are tokenizer-fragile ($REPORT §30) — if ●◐○ does NOT beat words on Claude's tokenizer, keep words+1-line key

◆ P3 · REASONING LATTICE in the analysis agents        $REPORT §12 L377-433 · lexicon spec L807-966 · W3 L656
  change   debug-intelligence · operations-intelligence · sim-steward emit deep diagnoses as
           HYP/EV/INFER + computed ρ (ρ(h) ∝ π(h)·∏ₑ LR(e,h)) instead of prose ; map our §2.5 review tags onto it
  files    .claude/agents/{debug-intelligence,operations-intelligence,sim-steward}.md ; A spec §12 hook (§5)
  ⬡gate    oracle-checkable on one real witness/incident ; reviewer can follow the lattice ; round-trips to JSON lossless

◆ P4 · GLYPH-COST AUDIT of SYMLANG itself              $REPORT results L1062-1071 · §11 L1670-1702
  change   audit SYMLANG's own delimiters: ⟦META⟧=7tok, ◔◑◕●=7, ⟨ ⟩=3 each (measured) ;
           swap expensive constructs for cheaper-clear ones where it does NOT cost scannability ;
           GATE §2.13 domain glyphs (see §5) — default to terse English term
  files    A spec (§5) ; ripple to B,C where the expensive forms appear
  ⬡gate    each swap: token before/after + cold-decode ≥95% ; net token reduction with no fidelity loss

◆ P5 · GATES & GUARDRAILS (CI)                          $REPORT §16 P5 L578 · §9 · sequencing rule L581
  change   make "cold-decode score + token before/after" a REQUIRED gate before any notation ships ;
           wire to the repo's witness-honesty path (WIT-HON: validate-report witness_honesty / queue_integrity)
  files    prompts/SYMBOLIC_LANGUAGE.meta.md §1+§9 (the LAW) ; agent-lang skill loop (already has WIT-HON)
  ⬡gate    a notation change with no cold-decode score + token delta is BLOCKED (mirror the BLANG:Q✓ ⟸ WIT-HON rule)

◆ P6 · TOOL / MCP LAYER — the big lever                 $REPORT §13,W1 L435-503,618-639 · §6 below
  see §6 (signature-book schemas + tool-result vectors + Δ-handoff). Gated on CALLABILITY, not just tokens.
```

## §5 SYMLANG spec edits (`prompts/SYMBOLIC_LANGUAGE.meta.md` — canonical; mirror ⟦GEN-SAFE⟧ to portal)

```text
E1 §1 LAW  ⟦GEN-SAFE⟧   add L-COLD: "no notation ships without (a) cold-decode ≥95% (fresh reader, no legend)
                         AND (b) a token before/after vs terse prose. A token win that doesn't decode = not a win."  ($REPORT §9)
E2 §1 LAW  ⟦GEN-SAFE⟧   add L-BLEND: "default = the blend; pure-symbolic is the worst encoding ($REPORT §5). Symbolise
                         only status(dense)/rules/relations/recurring-concepts; leave narrative as prose."
E3 §2.2 status ⟦GEN-SAFE⟧ redefine the dense-status spine to ●=pass ◐=running ○=skip ✗=fail ; demote 🟢🟡🔴 + emoji
                         to "SPARSE headline status only" ($REPORT §8 L257-266). Keep ✅+evidence-closer rule.
E4 §2.11 confidence ⟦GEN-SAFE⟧ note: ◔◑◕● cost ~7 tok as a set — for inline confidence prefer ~.5/~.75/1.0 ($REPORT L2398);
                         keep ◔◑◕● only where a glance-bar earns it.
E5 §2.13 domain glyphs ⟦GEN-SAFE⟧ GATE it: a domain glyph is allowed ONLY when it (a) recurs ≥3× in the artifact,
                         (b) cold-decodes ≥95% OR carries a 1-line in-context legend, AND (c) beats the terse English
                         term on payload+fidelity. DEFAULT = terse English term. ($REPORT §11 Rec5 L1670-1702)
E6 §3.x add a "STATUS-VECTOR" form (●◐○ default+exception) + a "SIGNATURE-BOOK" form for tool schemas + a
                         "Δ-HANDOFF table" form (col-header + legend, lossless) ($REPORT §6,§13,W7)
E7 §12-link            point the reasoning section at the HYP/EV/INFER + ρ scaffold; keep our edge glyphs but mark the
                         lexicon spec as the canonical deep-reasoning form ($REPORT §12 L807-966)
E8 §0b OUTCOME MAP     add a 4th outcome test row: "decodes cold (fidelity)" alongside ⏩/💰↓/🎯 ($REPORT §9)
```

Apply E1–E8 to the canonical spec; apply only the ⟦GEN-SAFE⟧ ones (all of E1–E8 are generic) to
`claude-portal-skills/SYMBOLIC_LANGUAGE.meta.md`, keeping the portal project-agnostic.

## §6 The W1 lever — MCP signature-book schemas (biggest token win)

```text
$REPORT §13 L435-503 · W1 L618-639 · WS10 L1872-1945
finding   MCP tool schemas injected every request = ~96% of always-on budget. Signature book = −92% @ 100% callable.
our case  rust_engine_mcp exposes ~90 CLI commands; the MCP server surface is the cinesite-relevant tax.
do        1 emit each MCP tool as ONE signature line:  name(req, [opt]):type =default -> result   (+ tiny shared-field codebook)
          2 tool-result outputs (status/diff) as ●◐○ vectors ($REPORT W5 L686 · WS11 L1949 : −73% vs JSON)
          3 cross-agent handoffs as the Δ-handoff table ($REPORT W7 L726 · WS12 : lossless, −61% vs JSON)
files     tools/mcp/README.md · tools/mcp/MICRO_TOOLS_REGISTRY_v1.md (document the signature-book form) ;
          tools/mcp/python/rust_engine_mcp/ (server-side schema text — the actual lever)
⚡SACRED   never strip the exact tool/command name — stripping the disambiguating segment drops callability 100%→12%
          ($REPORT §13 L468-474). Amortising the constant mcp__…server__ prefix is safe.
⬡gate     CALLABILITY ≥95% (cold sub-agents emit calls from the encoding alone) reported on a SEPARATE axis from token-delta.
          A schema that won't call is worth 0.  ($REPORT §16 P6 L579)
scope     the −92% is realised SERVER-SIDE (the MCP server emits the book). MD docs (F) document + adopt the form;
          they cannot by themselves cut the injected schema tax.
```

## §7 MCP md files (F) — concrete doc upgrades

```text
tools/mcp/README.md              add a "Tool schema = signature book" section (the W1 form) + the SACRED-name rule
MICRO_TOOLS_REGISTRY_v1.md       render the tool list AS the signature book (name(req,[opt]):type=def -> result)
prompts/llm_agent_brief.md       §tool layer: cite $REPORT §13 ; status outputs → ●◐○ ; diagnoses → HYP/EV/INFER
CLAUDE.md (if present)           note the blend + cold-decode gate as the house notation rule
```

## §8 Run order (checklist)

```text
☑ W0  sync.ps1 verified — covers SKILLS only (agents = open, below)
☑ P1  standard notation kept (validated free leverage)
☑ E1,E2,E8  SYMLANG §1 L9/L10 (cold-decode + blend) + §0b fidelity gate — canonical + portal
☑ P2 / E3,E4  status spine ●◐○✗ + confidence ~.5/.75/1.0 in spec ; swept files = NO dense-emoji violations found (already sparse-legit)
☑ P4 / E5,E6,E7  §2.13 domain glyphs GATED · §3.12 I/O forms added · glyph-cost note (◔◑◕●/⟦META⟧ expensive)
☑ P3  reasoning lattice (HYP/EV/INFER+ρ) → .claude debug/ops/sim-steward  ⚠ not yet in .cursor (see open)
☑ sync .claude/skills → .cursor/skills (10) ; portal SYMLANG mirror (generic) ; spec is $ref'd by .cursor → auto-current
☑ P6 / §7  MCP md docs adopt signature-book + SACRED name + ●◐○ results + HYP/EV/INFER (registry · README · llm_agent_brief)
☑ P5  cold-decode + token gate locked into spec L9 + §9 ENFORCEMENT (mirrors the BLANG:Q✓ ⟸ WIT-HON discipline)

OPEN (need your call):
⚠ .cursor/agents (14 originals) not covered by sync.ps1 + differ from .claude/agents (13) — extend sync.ps1 to mirror agents, hand-port once, or leave divergent?
⚠ §6 server-side −92% schema rewrite (rust_engine_mcp emits the signature book) = a CODE task for @coder-mcp, gated on callability ≥95% — docs adopt the form; server change deferred (intentional)
⚠ formal cold-decode + Claude-tokenizer measurement still owed before any AGGRESSIVE glyph swap (spec gate now governs going-forward)
```

```text
⟦/plan_io_notation_upgrade_v1⟧
NEXT ⚑ read $REPORT §16,§17 → P1 → E1/E2/E8 → P2 → P4 → P3 → sync → P6 → P5
RULE  ∀ change ⊨ (cold-decode ≥95% ∧ token-Δ measured) ; tool layer also ⊨ callability ≥95% ; never copy $REPORT, ref it
```
