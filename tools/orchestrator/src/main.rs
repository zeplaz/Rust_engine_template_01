//! CLI entry: `cargo run --manifest-path tools/orchestrator/Cargo.toml`

use rust_engine_orchestrator::pipeline::{find_repo_root, run_build_pipeline, PipelineOptions};
use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut repo_root = None;
    let mut skip_clippy = false;
    let mut skip_test = false;
    let mut skip_cargo = false;
    let mut runtime_snapshot = None;
    let mut main_thread_shift = false;
    let mut plan_slice = false;
    let mut plan_slice_top = 5usize;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--repo-root" => {
                i += 1;
                repo_root = Some(PathBuf::from(args.get(i).expect("--repo-root value")));
            }
            "--runtime-snapshot" => {
                i += 1;
                runtime_snapshot = Some(PathBuf::from(
                    args.get(i).expect("--runtime-snapshot path"),
                ));
            }
            "--skip-clippy" => skip_clippy = true,
            "--skip-test" => skip_test = true,
            "--skip-cargo" => skip_cargo = true,
            "--main-thread-shift" => main_thread_shift = true,
            "--plan-slice" => plan_slice = true,
            "--plan-slice-top" => {
                i += 1;
                plan_slice_top = args
                    .get(i)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(5);
            }
            "--" => {}
            "--help" | "-h" => {
                print_help();
                return;
            }
            other => {
                eprintln!("unknown arg: {other}");
                print_help();
                std::process::exit(2);
            }
        }
        i += 1;
    }

    let repo_root = repo_root.unwrap_or_else(|| find_repo_root(&env::current_dir().unwrap()));
    let opts = PipelineOptions {
        repo_root: repo_root.clone(),
        skip_clippy,
        skip_test,
        skip_cargo,
        runtime_snapshot,
    };

    println!(
        "rust_engine_orchestrator: repo={}",
        repo_root.display()
    );

    if plan_slice {
        use rust_engine_orchestrator::plan_slice::run_plan_slice;
        let report = run_plan_slice(&repo_root, plan_slice_top, true);
        println!(
            "plan_slice: {} recommended slices → tools/orchestrator/reports/plan_slice.md",
            report.recommended.len()
        );
        for t in &report.recommended {
            println!("  P{} {} @{} — {}", t.priority, t.id, t.agent, t.title);
        }
        for n in &report.health_notes {
            println!("  note: {n}");
        }
        if skip_cargo && !main_thread_shift {
            return;
        }
    }

    if main_thread_shift {
        use rust_engine_orchestrator::main_thread_shift::{
            run_main_thread_shift, write_live_proof, LIVE_PROOF_PATH,
        };
        let report = run_main_thread_shift(&repo_root);
        let out = write_live_proof(&repo_root, &report);
        println!(
            "main_thread_shift: ok={} severity={} proof={}",
            report.ok,
            report.highest_severity,
            out.display()
        );
        println!("debug routing id={}", report.shift_b_debug.id);
        if !report.ok {
            eprintln!("action required — see {LIVE_PROOF_PATH} shift B");
        }
    }

    let snapshot = run_build_pipeline(&opts);
    println!(
        "done run={} issues={} do_not_touch={}",
        snapshot.meta.run_id, snapshot.meta.issue_count, snapshot.meta.do_not_touch_count
    );
    println!("reports: tools/orchestrator/reports/");
}

fn print_help() {
    eprintln!(
        r"rust_engine_orchestrator — architectural diagnostics pipeline

USAGE:
  orchestrate [--repo-root PATH] [--runtime-snapshot PATH] [--skip-clippy] [--skip-test] [--skip-cargo]
            [--main-thread-shift] [--plan-slice] [--plan-slice-top N]

Runs cargo check/clippy/test (JSON diagnostics), classifies warnings by migration
state, emits reports under tools/orchestrator/.

  --plan-slice         Digest debug_runs witnesses + triage boards; write
                       reports/plan_slice.md and queues/continuation_queue.json
  --plan-slice-top N   Max recommended slices (default 5)

  --main-thread-shift  Shift A→B: witness digest, debug routing YAML, cleanup
                       classification, simulation-grade authority scan.
                       Writes debug_runs/main_thread_orchestrator_live.json
"
    );
}
