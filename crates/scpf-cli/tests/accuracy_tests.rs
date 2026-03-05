use assert_cmd::assert::OutputAssertExt;
use predicates::str::contains;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn accuracy_report_binary_generates_report_with_f1_score() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_accuracy_report"));
    cmd.current_dir(workspace_root)
        .assert()
        .success()
        .stdout(contains("F1 Score"))
        .stdout(contains("Quality Grade"));
}
