// guard.js — OpenCode benevolent guard (tool.execute.before)
// Charter: opencode/STEWARD_CHARTER.md · Policy: opencode/guards/GUARD_POLICY.md
//
// Philosophy: a SAFETY NET, not a wall. It lets us be generous with trust and still be safe.
//   • ASK-DEFAULT — opencode.json sets edit/write/bash = "ask"; a human/steward already sees risky ops.
//   • THROW-RARELY — this plugin BLOCKS only true irreversibles / clear accidents (below). Everything else passes.
//   • WARM + SAFE-PATH — every block credits intent, names what it protected, hands back the safe way. No blame.
//   • FAIL-OPEN — any internal error ⟶ ALLOW. A guard that breaks honest work is worse than the risk.
//   • RAISE-UP — the agent leaves more capable; @steward is the kind explainer.

import { existsSync, statSync } from "node:fs";
import { resolve, sep } from "node:path";

const DESTRUCTIVE = [
  { re: /\brm\s+-[a-z]*r/i,                 safe: "delete one path at a time after classifying it (A/B/C/D) — see cleanup-completion-intelligence; prefer a completion_plan over deletion" },
  { re: /\brm\s+[^|&;]*[*?]/i,              safe: "avoid glob deletes; list the exact files, classify, then remove individually" },
  { re: /\bgit\s+reset\s+--hard/i,          safe: "git stash (keeps your work) or commit on a branch; --hard discards changes irreversibly" },
  { re: /\bgit\s+clean\s+-[a-z]*f/i,        safe: "run `git clean -n` (dry-run) first; it shows what would be deleted without deleting" },
  { re: /\bgit\s+checkout\s+--\s/i,         safe: "git stash first; `checkout -- .` throws away uncommitted edits" },
  { re: /\bgit\s+push\s+[^|&;]*(--force\b|-f\b)/i, safe: "use --force-with-lease, or coordinate via @steward — a plain force-push can erase others' commits" },
  { re: /\bRemove-Item\b[^|&;]*-Recurse/i,  safe: "remove one item at a time after classifying; -Recurse can wipe a tree" },
  { re: /\b(rmdir|rd)\s+\/s/i,              safe: "delete contents deliberately after classifying, not /s recursively" },
  { re: /\bdel\s+\/s/i,                     safe: "delete named files, not /s recursively" },
];

function pathArg(args) {
  return args?.filePath ?? args?.path ?? args?.file ?? args?.target ?? args?.filename ?? null;
}
function contentArg(args) {
  return args?.content ?? args?.text ?? args?.newString ?? args?.new_string ?? null;
}
function cmdArg(args) {
  return args?.command ?? args?.cmd ?? args?.script ?? null;
}

function raise(what, safe, why) {
  // The one warm message (GUARD_POLICY template). Throwing is OpenCode's only block mechanism;
  // we make the block a teaching moment, never a strike.
  return new Error(
    `🛟 Guard paused this to protect ${what}. Your intent looks reasonable — here's the safe path: ${safe}. ` +
      `Why: ${why}. The @steward can help, and opencode/guards/GUARD_POLICY.md has the full picture. ` +
      `Nothing about you is at fault.`
  );
}

export const GuardPlugin = async (ctx) => {
  const root = resolve(ctx?.worktree || ctx?.directory || process.cwd());

  return {
    "tool.execute.before": async (input, output) => {
      try {
        const tool = String(input?.tool || "").toLowerCase();
        const args = output?.args ?? {};

        // 1 + 3 — file writes/edits
        if (tool === "write" || tool === "edit" || tool === "patch") {
          const p = pathArg(args);
          if (p) {
            const abs = resolve(root, p);
            // (1) outside the repo root → likely a path typo; protect the wider machine
            if (abs !== root && !abs.startsWith(root + sep)) {
              throw raise(
                `a path outside this repo (${p})`,
                "write inside the repo, or double-check the path — this looked like a typo escaping the project",
                "writes outside the project can touch unrelated files irreversibly"
              );
            }
            // (3) would blank an authored, non-empty file
            if (tool === "write" && existsSync(abs)) {
              let size = 0;
              try { size = statSync(abs).size; } catch { /* fail-open */ }
              const content = contentArg(args);
              const newLen = content == null ? 0 : String(content).trim().length;
              if (size > 50 && newLen < 10) {
                throw raise(
                  `an existing authored file from being blanked (${p})`,
                  "if you mean to clear it, read it first and confirm; if you mean to edit, use a targeted edit instead of an empty overwrite",
                  "overwriting a non-empty file with empty content is irreversible without git"
                );
              }
            }
          }
        }

        // 2 — destructive shell
        if (tool === "bash" || tool === "shell") {
          const cmd = cmdArg(args);
          if (cmd) {
            for (const d of DESTRUCTIVE) {
              if (d.re.test(cmd)) {
                throw raise(`against a destructive shell command`, d.safe, "this pattern can delete files or rewrite history irreversibly");
              }
            }
          }
        }
      } catch (e) {
        // Re-throw OUR intentional guard blocks; swallow everything else (FAIL-OPEN).
        if (e instanceof Error && e.message.startsWith("🛟 Guard paused")) throw e;
        return; // any internal/parse error ⟶ allow honest work to proceed
      }
    },
  };
};

export default GuardPlugin;
