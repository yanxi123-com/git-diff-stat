use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
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
fn working_tree_output_mentions_scope_lang_and_test_mode() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    fs::write(
        tempdir.path().join("src/lib.rs"),
        "pub fn answer() -> i32 {\n    41\n}\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "src/lib.rs"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("src/lib.rs"),
        "pub fn answer() -> i32 {\n    42\n}\n",
    )
    .unwrap();

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "未提交的 rs 文件中，非测试代码统计如下：",
        ))
        .stdout(predicate::str::contains("src/lib.rs"));
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
        .stdout(predicate::str::contains(
            "git diff-stat --last --no-test-filter",
        ))
        .stdout(predicate::str::contains("--no-test-filter"));
}

#[test]
fn last_flag_reports_head_patch() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    fs::write(
        tempdir.path().join("src/tracked.rs"),
        "pub fn tracked() -> &'static str {\n    \"before\"\n}\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "src/tracked.rs"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("src/tracked.rs"),
        "pub fn tracked() -> &'static str {\n    \"after\"\n}\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "src/tracked.rs"]);
    run_git(tempdir.path(), ["commit", "-m", "latest"]);

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--last", "--no-test-filter"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "最后一次提交的 rs 文件中，测试与非测试代码统计如下：",
        ))
        .stdout(predicate::str::contains("src/tracked.rs"))
        .stdout(predicate::str::contains("1 insertion"));
}

#[test]
fn default_filters_to_rust_non_test_changes() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    fs::create_dir_all(tempdir.path().join("tests")).unwrap();
    fs::write(
        tempdir.path().join("src/lib.rs"),
        "pub fn answer() -> i32 {\n    41\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("tests/integration.rs"),
        "#[test]\nfn it_works() {\n    assert_eq!(1, 1);\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("web.js"),
        "export const answer = () => 41;\n",
    )
    .unwrap();
    run_git(
        tempdir.path(),
        ["add", "src/lib.rs", "tests/integration.rs", "web.js"],
    );
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("src/lib.rs"),
        "pub fn answer() -> i32 {\n    42\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("tests/integration.rs"),
        "#[test]\nfn it_works() {\n    assert_eq!(1, 2);\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("web.js"),
        "export const answer = () => 42;\n",
    )
    .unwrap();
    run_git(
        tempdir.path(),
        ["add", "src/lib.rs", "tests/integration.rs", "web.js"],
    );
    run_git(tempdir.path(), ["commit", "-m", "latest"]);

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .arg("--last")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "最后一次提交的 rs 文件中，非测试代码统计如下：",
        ))
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("tests/integration.rs").not())
        .stdout(predicate::str::contains("web.js").not());
}

#[test]
fn no_test_filter_includes_all_rust_changes_but_keeps_default_lang() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    fs::create_dir_all(tempdir.path().join("tests")).unwrap();
    fs::write(
        tempdir.path().join("src/lib.rs"),
        "pub fn answer() -> i32 {\n    41\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("tests/integration.rs"),
        "#[test]\nfn it_works() {\n    assert_eq!(1, 1);\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("web.js"),
        "export const answer = () => 41;\n",
    )
    .unwrap();
    run_git(
        tempdir.path(),
        ["add", "src/lib.rs", "tests/integration.rs", "web.js"],
    );
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("src/lib.rs"),
        "pub fn answer() -> i32 {\n    42\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("tests/integration.rs"),
        "#[test]\nfn it_works() {\n    assert_eq!(1, 2);\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("web.js"),
        "export const answer = () => 42;\n",
    )
    .unwrap();
    run_git(
        tempdir.path(),
        ["add", "src/lib.rs", "tests/integration.rs", "web.js"],
    );
    run_git(tempdir.path(), ["commit", "-m", "latest"]);

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--last", "--no-test-filter"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "最后一次提交的 rs 文件中，测试与非测试代码统计如下：",
        ))
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("tests/integration.rs"))
        .stdout(predicate::str::contains("web.js").not());
}

#[test]
fn revision_range_output_mentions_range_langs_and_test_mode() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    fs::write(
        tempdir.path().join("src/lib.rs"),
        "pub fn answer() -> i32 {\n    41\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("web.js"),
        "export const answer = () => 41;\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "src/lib.rs", "web.js"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("src/lib.rs"),
        "pub fn answer() -> i32 {\n    42\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("web.js"),
        "export const answer = () => 42;\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "src/lib.rs", "web.js"]);
    run_git(tempdir.path(), ["commit", "-m", "latest"]);

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["HEAD~1..HEAD", "--lang", "rs,js", "--no-test-filter"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "HEAD~1 到 HEAD 的 rs,js 文件中，测试与非测试代码统计如下：",
        ))
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("web.js"));
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

#[test]
fn test_filter_counts_cfg_test_path_module_files_as_test() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("src/runtime")).unwrap();
    fs::write(
        tempdir.path().join("src/runtime.rs"),
        "#[cfg(test)]\n#[path = \"runtime/tests.rs\"]\nmod tests;\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("src/runtime/tests.rs"),
        "fn helper() {\n    assert_eq!(1, 1);\n}\n",
    )
    .unwrap();
    run_git(
        tempdir.path(),
        ["add", "src/runtime.rs", "src/runtime/tests.rs"],
    );
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("src/runtime/tests.rs"),
        "fn helper() {\n    assert_eq!(1, 2);\n}\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "src/runtime/tests.rs"]);
    run_git(tempdir.path(), ["commit", "-m", "latest"]);

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--last", "--lang", "rs", "--test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("src/runtime/tests.rs"))
        .stdout(predicate::str::contains("1 insertion"))
        .stdout(predicate::str::contains("1 deletion"));
}

#[test]
fn no_test_filter_excludes_cfg_test_path_module_files() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("src/runtime")).unwrap();
    fs::write(
        tempdir.path().join("src/runtime.rs"),
        "#[cfg(test)]\n#[path = \"runtime/tests.rs\"]\nmod tests;\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("src/runtime/tests.rs"),
        "fn helper() {\n    assert_eq!(1, 1);\n}\n",
    )
    .unwrap();
    run_git(
        tempdir.path(),
        ["add", "src/runtime.rs", "src/runtime/tests.rs"],
    );
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("src/runtime/tests.rs"),
        "fn helper() {\n    assert_eq!(1, 2);\n}\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "src/runtime/tests.rs"]);
    run_git(tempdir.path(), ["commit", "-m", "latest"]);

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--last", "--lang", "rs", "--no-test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0 files changed"));
}

#[test]
fn no_test_filter_ignores_zero_line_deleted_rust_files() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    fs::write(tempdir.path().join("src/empty.rs"), "").unwrap();
    run_git(tempdir.path(), ["add", "src/empty.rs"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::remove_file(tempdir.path().join("src/empty.rs")).unwrap();
    run_git(tempdir.path(), ["rm", "src/empty.rs"]);
    run_git(tempdir.path(), ["commit", "-m", "delete empty rust file"]);

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--last", "--lang", "rs", "--no-test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0 files changed"));
}

#[test]
fn no_test_filter_handles_renamed_rust_files() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("src/old")).unwrap();
    fs::write(
        tempdir.path().join("src/old/logging.rs"),
        "pub fn log_level() -> &'static str {\n    \"info\"\n}\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "src/old/logging.rs"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::create_dir_all(tempdir.path().join("src/new")).unwrap();
    run_git(
        tempdir.path(),
        ["mv", "src/old/logging.rs", "src/new/logging.rs"],
    );
    fs::write(
        tempdir.path().join("src/new/logging.rs"),
        "pub fn log_level() -> &'static str {\n    \"debug\"\n}\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "src/new/logging.rs"]);
    run_git(tempdir.path(), ["commit", "-m", "rename rust file"]);

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--last", "--lang", "rs", "--no-test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("logging.rs"))
        .stdout(predicate::str::contains("1 insertion"))
        .stdout(predicate::str::contains("1 deletion"));
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
