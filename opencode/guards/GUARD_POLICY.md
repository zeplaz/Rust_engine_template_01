# Guard Policy — protect kindly, raise up

> The guard is a **safety net, not a wall** (charter commitment 5). It exists so we can be *generous
> with trust* and still be safe. Lives as an OpenCode `tool.execute.before` plugin:
> `.opencode/plugins/guard.js`. `$ref:opencode/STEWARD_CHARTER.md`.

## Principles

```text
ASK-DEFAULT    opencode.json sets edit/write/bash = "ask" — a human (or the steward) sees risky ops. The plugin adds targeted accident-catching on top.
THROW-RARELY   the plugin BLOCKS only true irreversibles (the list below). Everything else passes — over-blocking is combative.
WARM+SAFE-PATH every block message credits intent, names what it protected, and hands back the safe way. No blame, no lock-out.
FAIL-OPEN      if the guard can't parse an op, it ALLOWS. A guard that breaks honest work is worse than the risk.
RAISE-UP       the agent leaves more capable — it learned the convention, not a punishment.
```

## Risk tiers

```text
◆ tool.execute.before
 ├─ LOW    read · grep · glob · small edit in-flight        ═▶ ● allow (silent)
 ├─ MED    write/edit any file · ordinary bash              ═▶ ◐ permission="ask" handles it (human/steward in loop) — plugin no-op
 └─ HIGH   irreversible / clear accident                    ═▶ ⊘ PAUSE + warm throw + safe path
```

## HIGH — the only things the plugin throws on (irreversibles & clear accidents)

```text
1 write/edit path OUTSIDE the repo root                     → likely a path typo; protect the wider machine
2 bash destructive pattern                                  → rm -rf · rm -r · git reset --hard · git clean -fd · git checkout -- . ·
                                                              git push -f/--force · truncate/:> · del /s · rmdir /s · Remove-Item -Recurse -Force
3 write that would BLANK an authored file                   → target exists, non-empty, and new content is empty/near-empty
4 mass delete (rm with a glob or many targets)              → classify-before-delete first (cleanup-completion-intelligence)
```

## The raise-up message (template the plugin emits on a throw)

```text
🛟 Guard paused this to protect <what>.  Your intent looks reasonable — here's the safe path:
   <concrete safe alternative>.
   Why: <one line — single-authority / classify-before-delete / irreversibility>.
   The @steward can help, and opencode/guards/GUARD_POLICY.md has the full picture. Nothing about you is at fault.
```

## The deeper guards (knowledge the steward + specialists draw on — MCP validators)

These aren't shell accidents — they're *convention* guards, enforced by the existing MCP layer, applied
with the same kindness:

```text
single-authority      ⊚ one writer per resource — a 2nd writer ⟶ GUIDE to the owner / @planner (¬block)   ($ref bevy-simulation-grade)
classify-before-delete A/B/C/D before any delete — prefer a completion_plan                                ($ref cleanup-completion-intelligence)
witness-honesty        BLANG:WIT-HON — validate-report witness_honesty / queue_integrity before Q✓          ($ref agent-lang skill)
production-bar         no subs/hacks shipped — but corrected as a *path*, the worker is never shamed         ($ref coder)
```

## Tuning

```text
ALLOWLIST   add safe-but-flagged commands to the plugin's allow list rather than loosening a whole tier
SCOPE       the plugin guards file/shell accidents; convention guards live in the agents + MCP validators
ESCALATE    a repeated same accident ⟶ the steward offers a short orientation, never a strike
```

```text
⟦/GUARD_POLICY⟧ NEXT ⚑ ask-default → catch irreversibles → warm throw + safe path → steward raises up
```
