use assert_cmd::Command;
use predicates::prelude::predicate;

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
        .stdout(predicate::str::contains("git diff-stat --lang rs --test"));
}
