use assert_cmd::Command;
use predicates::prelude::predicate;

#[test]
fn rejects_test_and_no_test_together() {
    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .args(["--test", "--no-test"])
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
        .args(["--no-test", "--no-test-filter"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
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
