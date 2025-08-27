use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap().parent().unwrap().to_path_buf()
}

#[test]
fn runs_factorial_example() {
    let root = workspace_root();
    let mut cmd = Command::cargo_bin("zirc-cli").unwrap();
    cmd.arg(root.join("examples/factorial.zirc"));
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("fact(5) = 120"));
}

#[test]
fn runs_conditionals_example() {
    let root = workspace_root();
    let mut cmd = Command::cargo_bin("zirc-cli").unwrap();
    cmd.arg(root.join("examples/conditionals.zirc"));
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("3 is less than 5"));
}

#[test]
fn parse_error_is_nonzero() {
    let bad = "fun x(\n"; // malformed on purpose
    let tmp_dir = tempfile::tempdir().unwrap();
    let bad_path = tmp_dir.path().join("bad.zirc");
    std::fs::write(&bad_path, bad).unwrap();

    let mut cmd = Command::cargo_bin("zirc-cli").unwrap();
    cmd.arg(bad_path);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Parse error"));
}

