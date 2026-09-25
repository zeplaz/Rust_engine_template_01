//! One-shot SCAN / DEBUG / WITNESS conclusion chain.
//!
//! Subprograms yield a report and return. There is no Bevy schedule,
//! heartbeat, or always-on scanner. Call [`run_scan_debug_conclusion`]
//! from a lib test or a `cargo test` harness.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

pub const SCAN_DEBUG_CONCLUSION_LIVE_JSON: &str = "debug_runs/scan_debug_conclusion_live.json";
const PIXEL_CONCLUDE_TIMEOUT: Duration = Duration::from_secs(3);
const PIXEL_CLI_MARKER: &str = "pixel-conclude";

/// Pointers only. The chain does not execute these drainages.
pub const SUB_DRAINAGES: &[(&str, &str)] = &[
    ("vfx", "debug_runs/product_fire_vfx_live.json"),
    ("construction", "debug_runs/construction_stage_live.json"),
    ("mcp", "tools/mcp/python/rust_engine_mcp/pixel_pipeline"),
    ("gpu", "debug_runs/gpu_p1_p2_001_live.json"),
    ("ci", "tools/orchestrator/ci/run.ps1"),
];

struct ScanTarget {
    id: &'static str,
    witness_rel: &'static str,
    pixel_rel: &'static str,
    /// `pixel-conclude --kind` choice: gui, ui, world, or art.
    pixel_kind: &'static str,
}

const CATALOG: &[ScanTarget] = &[
    ScanTarget {
        id: "product_fire_vfx",
        witness_rel: "debug_runs/product_fire_vfx_live.json",
        pixel_rel: "debug_runs/product_fire_vfx_pixel_matrix.png",
        pixel_kind: "world",
    },
    ScanTarget {
        id: "visual_artifact_jank",
        witness_rel: "debug_runs/visual_artifact_jank_live.json",
        pixel_rel: "debug_runs/visual_artifact_jank_pixel_matrix.png",
        pixel_kind: "gui",
    },
    ScanTarget {
        id: "building_look",
        witness_rel: "debug_runs/building_look_v2_live.json",
        pixel_rel: "debug_runs/building_look_v2/after/victorian_4x2_s1_d8e2.png",
        pixel_kind: "art",
    },
];

const PIXEL_CONCLUDE_OUT_REL: &str = "debug_runs/scan_debug_pixel_conclude";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YieldRow {
    pub id: &'static str,
    pub green: bool,
    pub path: &'static str,
    pub present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlignFlag {
    pub id: &'static str,
    pub pixel_path: &'static str,
    pub idle: bool,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelConclude {
    pub status: String,
    pub png: Option<String>,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanCounters {
    pub row_count: u32,
    pub present: u32,
    pub missing: u32,
    pub green: u32,
    pub idle_misaligned: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanDebugConclusion {
    pub matrix: Vec<YieldRow>,
    pub misaligned: Vec<AlignFlag>,
    pub pixel: PixelConclude,
    pub counters: ScanCounters,
    pub green: bool,
    pub wrote: bool,
}

fn repo_root() -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_green(path: &Path) -> (bool, bool) {
    if !path.is_file() {
        return (false, false);
    }
    let Ok(text) = fs::read_to_string(path) else {
        return (true, false);
    };
    let Ok(value) = serde_json::from_str::<Value>(&text) else {
        return (true, false);
    };
    let green = value.get("green").and_then(Value::as_bool).unwrap_or(false);
    (true, green)
}

/// SUBPROGRAM `scan_yield` — one row per catalog witness. A missing file is a row.
#[must_use]
pub fn scan_yield(root: &Path) -> Vec<YieldRow> {
    CATALOG
        .iter()
        .map(|target| {
            let (present, green) = read_green(&root.join(target.witness_rel));
            YieldRow {
                id: target.id,
                green,
                path: target.witness_rel,
                present,
            }
        })
        .collect()
}

/// SUBROUTINE `align_check` — green witness whose paired pixel matrix is absent.
#[must_use]
pub fn align_check(root: &Path, rows: &[YieldRow]) -> Vec<AlignFlag> {
    let mut flags = Vec::new();
    for row in rows {
        if !row.green {
            continue;
        }
        let Some(target) = CATALOG.iter().find(|item| item.id == row.id) else {
            continue;
        };
        if !root.join(target.pixel_rel).is_file() {
            flags.push(AlignFlag {
                id: row.id,
                pixel_path: target.pixel_rel,
                idle: true,
                reason: "green_witness_pixel_matrix_absent",
            });
        }
    }
    flags
}

fn pixel_pipeline_dir(root: &Path) -> PathBuf {
    root.join("tools/mcp/python/rust_engine_mcp/pixel_pipeline")
}

fn file_has_marker(path: &Path) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    if meta.len() > 256 * 1024 {
        return false;
    }
    fs::read_to_string(path)
        .map(|text| text.contains(PIXEL_CLI_MARKER))
        .unwrap_or(false)
}

fn tree_has_marker(dir: &Path, depth: u32) -> bool {
    if depth > 2 || !dir.is_dir() {
        return false;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    for (index, entry) in entries.flatten().enumerate() {
        if index >= 40 {
            break;
        }
        let path = entry.path();
        if path.is_dir() {
            if tree_has_marker(&path, depth + 1) {
                return true;
            }
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("py") && file_has_marker(&path) {
            return true;
        }
    }
    false
}

fn pixel_cli_present(root: &Path) -> bool {
    tree_has_marker(&pixel_pipeline_dir(root), 0)
}

fn first_known_png(root: &Path) -> Option<&'static ScanTarget> {
    CATALOG.iter().find(|target| root.join(target.pixel_rel).is_file())
}

struct BoundedRun {
    status: &'static str,
    exit_code: Option<i32>,
}

fn run_bounded(mut cmd: Command, timeout: Duration) -> BoundedRun {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return BoundedRun {
                status: "pixel_cli_absent",
                exit_code: None,
            };
        }
        Err(_) => {
            return BoundedRun {
                status: "invoke_failed",
                exit_code: None,
            };
        }
    };
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return BoundedRun {
                    status: "invoked",
                    exit_code: status.code(),
                };
            }
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return BoundedRun {
                    status: "timed_out",
                    exit_code: None,
                };
            }
            Ok(None) => thread::sleep(Duration::from_millis(20)),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return BoundedRun {
                    status: "invoke_failed",
                    exit_code: None,
                };
            }
        }
    }
}

/// FUNCTION `trigger_pixel_conclude` — one invocation, or `pixel_cli_absent` when the CLI is missing.
#[must_use]
pub fn trigger_pixel_conclude(root: &Path) -> PixelConclude {
    if !pixel_cli_present(root) {
        return PixelConclude {
            status: "pixel_cli_absent".to_string(),
            png: None,
            exit_code: None,
        };
    }
    let Some(target) = first_known_png(root) else {
        return PixelConclude {
            status: "png_absent".to_string(),
            png: None,
            exit_code: None,
        };
    };
    let png = root.join(target.pixel_rel);
    let out_dir = root.join(PIXEL_CONCLUDE_OUT_REL);
    let mut cmd = Command::new("python");
    cmd.current_dir(root.join("tools/mcp/python"))
        .env("PYTHONUTF8", "1")
        .arg("-m")
        .arg("rust_engine_mcp.pixel_pipeline")
        .arg("--image")
        .arg(&png)
        .arg("--kind")
        .arg(target.pixel_kind)
        .arg("--out")
        .arg(&out_dir);
    let ran = run_bounded(cmd, PIXEL_CONCLUDE_TIMEOUT);
    PixelConclude {
        status: ran.status.to_string(),
        png: Some(target.pixel_rel.to_string()),
        exit_code: ran.exit_code,
    }
}

fn counters_of(matrix: &[YieldRow], misaligned: &[AlignFlag]) -> ScanCounters {
    let present = matrix.iter().filter(|row| row.present).count() as u32;
    let row_count = matrix.len() as u32;
    ScanCounters {
        row_count,
        present,
        missing: row_count.saturating_sub(present),
        green: matrix.iter().filter(|row| row.green).count() as u32,
        idle_misaligned: misaligned.len() as u32,
    }
}

fn conclusion_is_green(counters: &ScanCounters, pixel: &PixelConclude) -> bool {
    counters.missing == 0
        && counters.idle_misaligned == 0
        && counters.row_count > 0
        && counters.green == counters.row_count
        && pixel.status == "invoked"
        && pixel.exit_code == Some(0)
}

fn conclusion_body(report: &ScanDebugConclusion) -> Value {
    let matrix: Vec<Value> = report
        .matrix
        .iter()
        .map(|row| {
            json!({
                "id": row.id,
                "green": row.green,
                "path": row.path,
                "present": row.present,
            })
        })
        .collect();
    let misaligned: Vec<Value> = report
        .misaligned
        .iter()
        .map(|flag| {
            json!({
                "id": flag.id,
                "pixel_path": flag.pixel_path,
                "idle": flag.idle,
                "misaligned": true,
                "reason": flag.reason,
            })
        })
        .collect();
    let sub_drainages: Vec<Value> = SUB_DRAINAGES
        .iter()
        .map(|(id, pointer)| json!({"id": id, "pointer": pointer}))
        .collect();
    json!({
        "schema": "scan_debug_conclusion_live_v1",
        "gate": "SCAN-DEBUG-WITNESS-001",
        "green": report.green,
        "one_shot": true,
        "heartbeat": false,
        "schedule": null,
        "status": "returned",
        "matrix": matrix,
        "misaligned": misaligned,
        "pixel": {
            "status": report.pixel.status,
            "png": report.pixel.png,
            "exit_code": report.pixel.exit_code,
        },
        "counters": {
            "row_count": report.counters.row_count,
            "present": report.counters.present,
            "missing": report.counters.missing,
            "green": report.counters.green,
            "idle_misaligned": report.counters.idle_misaligned,
        },
        "sub_drainages": sub_drainages,
    })
}

/// SUBROUTINE `write_conclusion` — honest counters at [`SCAN_DEBUG_CONCLUSION_LIVE_JSON`].
#[must_use]
pub fn write_conclusion(report: &ScanDebugConclusion) -> bool {
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "SCAN-DEBUG-WITNESS-001",
        "run_scan_debug_conclusion",
        SCAN_DEBUG_CONCLUSION_LIVE_JSON,
        conclusion_body(report),
    );
    crate::dev::debug_run_envelope::write_debug_run_json(SCAN_DEBUG_CONCLUSION_LIVE_JSON, wrapped)
}

/// Entry subprogram. Yields, checks alignment, optionally concludes pixels, writes, returns.
#[must_use]
pub fn run_scan_debug_conclusion() -> ScanDebugConclusion {
    let root = repo_root();
    let matrix = scan_yield(&root);
    let misaligned = align_check(&root, &matrix);
    let pixel = trigger_pixel_conclude(&root);
    let counters = counters_of(&matrix, &misaligned);
    let mut report = ScanDebugConclusion {
        green: conclusion_is_green(&counters, &pixel),
        matrix,
        misaligned,
        pixel,
        counters,
        wrote: false,
    };
    report.wrote = write_conclusion(&report);
    report
}

#[cfg(test)]
fn temp_root(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("scan_debug_{label}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

#[cfg(test)]
fn write_witness(root: &Path, rel: &str, green: bool) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, format!("{{\"green\":{green}}}")).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_debug_missing_file_is_a_row() {
        let root = temp_root("missing");
        let rows = scan_yield(&root);
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().all(|row| !row.present && !row.green));
        assert!(rows.iter().any(|row| row.id == "product_fire_vfx"));
        assert!(rows.iter().any(|row| row.id == "visual_artifact_jank"));
        assert!(rows.iter().any(|row| row.id == "building_look"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn scan_debug_align_flags_green_without_pixel_matrix() {
        let root = temp_root("align");
        write_witness(&root, "debug_runs/visual_artifact_jank_live.json", true);
        let rows = scan_yield(&root);
        let flags = align_check(&root, &rows);
        assert_eq!(flags.len(), 1);
        assert_eq!(flags[0].id, "visual_artifact_jank");
        assert!(flags[0].idle);
        assert_eq!(flags[0].reason, "green_witness_pixel_matrix_absent");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn scan_debug_align_quiet_when_pixel_matrix_present() {
        let root = temp_root("aligned");
        write_witness(&root, "debug_runs/building_look_v2_live.json", true);
        let png = root.join("debug_runs/building_look_v2/after/victorian_4x2_s1_d8e2.png");
        fs::create_dir_all(png.parent().unwrap()).unwrap();
        fs::write(&png, b"png").unwrap();
        let rows = scan_yield(&root);
        let flags = align_check(&root, &rows);
        assert!(flags.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn scan_debug_pixel_cli_absent_without_package() {
        let root = temp_root("nocli");
        let concluded = trigger_pixel_conclude(&root);
        assert_eq!(concluded.status, "pixel_cli_absent");
        assert!(concluded.png.is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn scan_debug_bounded_process_reaps_on_timeout() {
        let mut cmd = Command::new("python");
        cmd.arg("-c").arg("import time; time.sleep(30)");
        let started = Instant::now();
        let ran = run_bounded(cmd, Duration::from_millis(400));
        assert!(started.elapsed() < Duration::from_secs(3));
        assert!(ran.status == "timed_out" || ran.status == "pixel_cli_absent");
    }

    #[test]
    fn scan_debug_conclusion_chain_writes_honest_counters() {
        let report = run_scan_debug_conclusion();
        assert!(report.wrote);
        assert_eq!(report.matrix.len(), 3);
        assert_eq!(report.counters.row_count, 3);
        assert_eq!(
            report.counters.present + report.counters.missing,
            report.counters.row_count
        );
        assert_eq!(
            report.counters.idle_misaligned,
            report.misaligned.len() as u32
        );
        assert_eq!(SUB_DRAINAGES.len(), 5);
        for flag in &report.misaligned {
            assert!(flag.idle);
            let row = report
                .matrix
                .iter()
                .find(|row| row.id == flag.id)
                .expect("flag id is a matrix row");
            assert!(row.green);
            assert!(!repo_root().join(flag.pixel_path).is_file());
        }

        let text = fs::read_to_string(repo_root().join(SCAN_DEBUG_CONCLUSION_LIVE_JSON))
            .expect("conclusion witness");
        let value: Value = serde_json::from_str(&text).expect("conclusion json");
        assert_eq!(value["schema"], "scan_debug_conclusion_live_v1");
        assert_eq!(value["one_shot"], true);
        assert_eq!(value["heartbeat"], false);
        assert_eq!(value["status"], "returned");
        assert_eq!(value["counters"]["row_count"], 3);
        assert_eq!(value["sub_drainages"].as_array().map(Vec::len), Some(5));
        assert_eq!(value["matrix"].as_array().map(Vec::len), Some(3));
        let pixel_status = value["pixel"]["status"].as_str().unwrap_or("");
        assert!(
            matches!(
                pixel_status,
                "pixel_cli_absent" | "png_absent" | "invoked" | "timed_out" | "invoke_failed"
            ),
            "unexpected pixel status {pixel_status}"
        );
        if !pixel_pipeline_dir(&repo_root()).is_dir() {
            assert_eq!(pixel_status, "pixel_cli_absent");
        }
        assert_eq!(value["green"], report.green);
    }
}
