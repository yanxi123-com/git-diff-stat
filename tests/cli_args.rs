use assert_cmd::Command;
use predicates::prelude::predicate;

#[test]
fn rejects_test_and_no_test_together() {
    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .args(["--test", "--non-test"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn rejects_test_and_no_test_filter_together() {
    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .args(["--test", "--no-test-filter"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn rejects_no_test_and_no_test_filter_together() {
    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .args(["--non-test", "--no-test-filter"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn rejects_legacy_no_test_flag() {
    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .arg("--no-test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument '--no-test'"));
}

#[test]
fn rejects_last_with_commit() {
    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .args(["--last", "--commit", "HEAD"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--last"));
}
