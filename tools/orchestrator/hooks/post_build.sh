#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$REPO_ROOT"
echo "[orchestrator] running pipeline..."
cargo run --quiet --manifest-path tools/orchestrator/Cargo.toml -- --skip-test
echo "[orchestrator] reports -> tools/orchestrator/reports/"
