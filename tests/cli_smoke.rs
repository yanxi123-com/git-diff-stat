use assert_cmd::Command;
use predicates::prelude::predicate;
use std::fs;
use std::path::Path;
use std::process::Command as ProcessCommand;
use tempfile::tempdir;

#[test]
fn prints_help() {
    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn help_mentions_common_examples() {
    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("git diff-stat --commit HEAD"))
        .stdout(predicate::str::contains("git diff-stat --last"))
        .stdout(predicate::str::contains("git diff-stat --lang rs --test"));
}

#[test]
fn last_flag_reports_head_patch() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::write(tempdir.path().join("tracked.txt"), "before\n").unwrap();
    run_git(tempdir.path(), ["add", "tracked.txt"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(tempdir.path().join("tracked.txt"), "before\nafter\n").unwrap();
    run_git(tempdir.path(), ["add", "tracked.txt"]);
    run_git(tempdir.path(), ["commit", "-m", "latest"]);

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .arg("--last")
        .assert()
        .success()
        .stdout(predicate::str::contains("tracked.txt"))
        .stdout(predicate::str::contains("1 insertion"));
}

#[test]
fn test_filter_counts_rust_integration_test_files_as_test() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("tests")).unwrap();
    fs::write(
        tempdir.path().join("tests/integration.rs"),
        "fn helper() {\n    assert_eq!(1, 1);\n}\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "tests/integration.rs"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("tests/integration.rs"),
        "fn helper() {\n    assert_eq!(1, 2);\n}\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "tests/integration.rs"]);
    run_git(tempdir.path(), ["commit", "-m", "latest"]);

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--last", "--lang", "rs", "--test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tests/integration.rs"))
        .stdout(predicate::str::contains("1 insertion"))
        .stdout(predicate::str::contains("1 deletion"));
}

#[test]
fn no_test_filter_excludes_rust_integration_test_files() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("tests")).unwrap();
    fs::write(
        tempdir.path().join("tests/integration.rs"),
        "fn helper() {\n    assert_eq!(1, 1);\n}\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "tests/integration.rs"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("tests/integration.rs"),
        "fn helper() {\n    assert_eq!(1, 2);\n}\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "tests/integration.rs"]);
    run_git(tempdir.path(), ["commit", "-m", "latest"]);

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--last", "--lang", "rs", "--no-test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0 files changed"));
}

fn init_repo(repo: &Path) {
    run_git(repo, ["init"]);
    run_git(repo, ["config", "user.name", "Codex"]);
    run_git(repo, ["config", "user.email", "codex@example.com"]);
}

fn run_git<const N: usize>(repo: &Path, args: [&str; N]) {
    let status = ProcessCommand::new("git")
        .args(args)
        .current_dir(repo)
        .status()
        .unwrap();
    assert!(status.success());
}
