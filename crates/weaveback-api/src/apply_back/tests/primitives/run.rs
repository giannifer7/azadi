// weaveback-api/src/apply_back/tests/primitives/run.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;

// ── run_apply_back with missing db ─────────────────────────────────────

#[test]
fn run_apply_back_reports_missing_database() {
    use std::path::PathBuf;
    let opts = ApplyBackOptions {
        db_path: PathBuf::from("/nonexistent/weaveback.db"),
        gen_dir: PathBuf::from("/nonexistent/gen"),
        dry_run: true,
        files: vec![],
        eval_config: None,
    };
    let mut out = Vec::new();
    let result = run_apply_back(opts, &mut out);
    assert!(result.is_ok());
    let msg = String::from_utf8(out).unwrap();
    assert!(msg.contains("Database not found"));
}

