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
