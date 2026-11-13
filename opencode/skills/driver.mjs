#!/usr/bin/env node
// agent-lang driver — the harness a Claude Code agent uses to drive the
// Rust_engine_template_01 multi-agent (BLANG / agent-lang) system.
//
// It is a thin, robust wrapper over the project's MCP CLI:
//     python -m rust_engine_mcp.cli <command> [args]
//
// It exists because three things bite every time you call that CLI raw:
//   1. The CLI emits symbolic Unicode (⟨ ⟩ ★ ⇢ emoji). On a Windows cp1252
//      console that throws UnicodeEncodeError. We force PYTHONUTF8=1.
//   2. The package only imports from tools/mcp/python — we set cwd there.
//   3. The deps (mcp, pydantic, jsonschema, Pillow) live in a specific
//      Python (3.13 on this machine), NOT necessarily the default `python`.
//      We probe candidates and pick the first that can `import mcp`.
//
// Usage: node driver.mjs --help

import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));

// Repo detection is deliberately location-independent: this driver is meant to
// be copied into a shared/cloud ~/.claude skills repo and reused across systems,
// so it must NOT assume a fixed depth below the repo root.
//   1. RUST_ENGINE_REPO env var wins (same var the CLI itself honors).
//   2. Walk up from the shell's cwd looking for the MCP package marker.
//   3. Walk up from the driver's own location (covers project-local installs).
//   4. Last-resort guess: three levels up (project-local default layout).
// Adapting to a different pipeline repo == change MARKER or set RUST_ENGINE_REPO.
const MARKER = join("tools", "mcp", "python", "rust_engine_mcp", "cli.py");

function findRepoFrom(start) {
  let d = start;
  for (;;) {
    if (existsSync(join(d, MARKER))) return d;
    const up = dirname(d);
    if (up === d) return null;
    d = up;
  }
}

function resolveRepo() {
  const env = (process.env.RUST_ENGINE_REPO || "").trim();
  if (env) return env;
  return findRepoFrom(process.cwd()) || findRepoFrom(HERE) || join(HERE, "..", "..", "..");
}

const REPO = resolveRepo();
const PKG_DIR = join(REPO, "tools", "mcp", "python");

function pythonCandidates() {
  const c = [];
  if (process.env.RUST_ENGINE_MCP_PYTHON) c.push([process.env.RUST_ENGINE_MCP_PYTHON]);
  const la = process.env.LOCALAPPDATA;
  if (la) {
    const p = join(la, "Programs", "Python", "Python313", "python.exe");
    if (existsSync(p)) c.push([p]);
  }
  c.push(["py", "-3.13"], ["py", "-3"], ["python3.13"], ["python3"], ["python"]);
  return c;
}

function probe(cand) {
  const r = spawnSync(cand[0], [...cand.slice(1), "-c", "import mcp, jsonschema"], {
    cwd: PKG_DIR,
    env: { ...process.env, PYTHONUTF8: "1" },
    encoding: "utf8",
  });
  return r.status === 0;
}

let _py = null;
function resolvePython() {
  if (_py) return _py;
  for (const cand of pythonCandidates()) {
    try {
      if (probe(cand)) return (_py = cand);
    } catch {
      /* not on PATH — try next */
    }
  }
  console.error(
    "[agent-lang] No Python with MCP deps (mcp, jsonschema) found.\n" +
      "  Tried: $RUST_ENGINE_MCP_PYTHON, %LOCALAPPDATA%\\Programs\\Python\\Python313, py -3.13, python.\n" +
      "  Fix: pip install -r tools/mcp/requirements.txt  (into a Python >= 3.11)\n" +
      "  Or set RUST_ENGINE_MCP_PYTHON to that interpreter."
  );
  process.exit(2);
}

function cli(args, { capture = false } = {}) {
  const py = resolvePython();
  return spawnSync(py[0], [...py.slice(1), "-m", "rust_engine_mcp.cli", ...args], {
    cwd: PKG_DIR,
    env: { ...process.env, PYTHONUTF8: "1" },
    stdio: capture ? ["ignore", "pipe", "inherit"] : "inherit",
    encoding: "utf8",
  });
}

const USAGE = `agent-lang driver — run the Rust_engine_template_01 MCP CLI with the right Python.

  node driver.mjs boot <agent>        Orientation: where things are + the run-review path.
  node driver.mjs where               Show resolved repo + Python (portability check)
  node driver.mjs cli --help          CLI command catalog
  node driver.mjs <args...>           Passthrough to: python -m rust_engine_mcp.cli <args>

Repo is found via $RUST_ENGINE_REPO, else by walking up from cwd / this file to the
MCP package. Driver auto-sets PYTHONUTF8=1 and runs from <repo>/tools/mcp/python.`;

const ORIENT = `Read AGENTS.md first. Review a real run (no witness/queue system any more):
  RUST_ENGINE_SIM_DUMP=1 cargo run -p proc_A_dine01        (debug build only)
  python tools/sim_review/score_dump.py                   -> debug_runs/sim_review/dashboard.html
  python tools/sim_review/playcheck.py                    (build + run + capture + score in one go)
Art tools: python -m rust_engine_mcp.cli --help (via this driver), APS: tools/mcp/art_pipeline_suite.`;

function main() {
  const argv = process.argv.slice(2);
  const sub = argv[0];

  if (!sub || sub === "--help" || sub === "-h") {
    console.log(USAGE);
    process.exit(0);
  }

  if (sub === "boot") {
    console.log(`orient ${argv[1] ?? "agent"}:\n${ORIENT}`);
    process.exit(0);
  }

  if (sub === "where") {
    const py = resolvePython();
    console.log(
      JSON.stringify(
        {
          repo: REPO,
          repo_source: process.env.RUST_ENGINE_REPO ? "RUST_ENGINE_REPO" : "marker-search",
          pkg_dir: PKG_DIR,
          pkg_found: existsSync(join(REPO, MARKER)),
          python: py.join(" "),
        },
        null,
        2
      )
    );
    process.exit(0);
  }

  if (sub === "cli") {
    process.exit(cli(argv.slice(1)).status ?? 1);
  }

  // implicit passthrough: any unrecognized verb is a raw CLI command.
  process.exit(cli(argv).status ?? 1);
}

main();
