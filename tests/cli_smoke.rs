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
    fs::write(
        tempdir.path().join("web.js"),
        "export const answer = () => 41;\n",
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
            "未提交的 rs,py,js,ts,jsx,tsx,cjs,mjs 文件中，非测试代码统计如下：",
        ))
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("web.js"));
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
        .stdout(predicate::str::contains("git diff-stat --lang tsx --test"))
        .stdout(predicate::str::contains(
            "--lang all supported languages (rs,py,js,ts,jsx,tsx,cjs,mjs)",
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
            "最后一次提交的 rs,py,js,ts,jsx,tsx,cjs,mjs 文件中，测试与非测试代码统计如下：",
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
            "最后一次提交的 rs,py,js,ts,jsx,tsx,cjs,mjs 文件中，非测试代码统计如下：",
        ))
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("tests/integration.rs").not())
        .stdout(predicate::str::contains("web.js"));
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
            "最后一次提交的 rs,py,js,ts,jsx,tsx,cjs,mjs 文件中，测试与非测试代码统计如下：",
        ))
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("tests/integration.rs"))
        .stdout(predicate::str::contains("web.js"));
}

#[test]
fn no_test_filter_does_not_parse_invalid_rust_sources() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    fs::write(tempdir.path().join("src/lib.rs"), [0xff, 0xfe, 0xfd]).unwrap();
    fs::write(
        tempdir.path().join("web.js"),
        "export const answer = () => 41;\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "src/lib.rs", "web.js"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("web.js"),
        "export const answer = () => 42;\n",
    )
    .unwrap();

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--no-test-filter"])
        .assert()
        .success()
        .stdout(predicate::str::contains("web.js"))
        .stdout(predicate::str::contains("src/lib.rs").not());
}

#[test]
fn default_lang_includes_rust_and_python_non_test_changes() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    fs::create_dir_all(tempdir.path().join("app")).unwrap();
    fs::create_dir_all(tempdir.path().join("tests")).unwrap();
    fs::write(
        tempdir.path().join("src/lib.rs"),
        "pub fn answer() -> i32 {\n    41\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("app/main.py"),
        "def answer() -> int:\n    return 41\n\n\ndef test_inline() -> None:\n    assert True\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("web.js"),
        "export const answer = () => 41;\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("tests/test_app.py"),
        "def test_external() -> None:\n    assert True\n",
    )
    .unwrap();
    run_git(
        tempdir.path(),
        [
            "add",
            "src/lib.rs",
            "app/main.py",
            "web.js",
            "tests/test_app.py",
        ],
    );
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("src/lib.rs"),
        "pub fn answer() -> i32 {\n    42\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("app/main.py"),
        "def answer() -> int:\n    return 42\n\n\ndef test_inline() -> None:\n    assert False\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("web.js"),
        "export const answer = () => 42;\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("tests/test_app.py"),
        "def test_external() -> None:\n    assert False\n",
    )
    .unwrap();

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "未提交的 rs,py,js,ts,jsx,tsx,cjs,mjs 文件中，非测试代码统计如下：",
        ))
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("app/main.py"))
        .stdout(predicate::str::contains("web.js"))
        .stdout(predicate::str::contains("tests/test_app.py").not());
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
fn explicit_python_lang_uses_python_files() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    fs::create_dir_all(tempdir.path().join("app")).unwrap();
    fs::write(
        tempdir.path().join("src/lib.rs"),
        "pub fn answer() -> i32 {\n    41\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("app/main.py"),
        "def answer() -> int:\n    return 41\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "src/lib.rs", "app/main.py"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("src/lib.rs"),
        "pub fn answer() -> i32 {\n    42\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("app/main.py"),
        "def answer() -> int:\n    return 42\n",
    )
    .unwrap();

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--lang", "py", "--no-test-filter"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "未提交的 py 文件中，测试与非测试代码统计如下：",
        ))
        .stdout(predicate::str::contains("app/main.py"))
        .stdout(predicate::str::contains("src/lib.rs").not());
}

#[test]
fn explicit_python_lang_skips_loading_unselected_rust_sources() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    fs::create_dir_all(tempdir.path().join("app")).unwrap();
    fs::write(tempdir.path().join("src/lib.rs"), [0xff, 0xfe, 0xfd]).unwrap();
    fs::write(
        tempdir.path().join("app/main.py"),
        "def answer() -> int:\n    return 41\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "src/lib.rs", "app/main.py"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("app/main.py"),
        "def answer() -> int:\n    return 42\n",
    )
    .unwrap();

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--lang", "py"])
        .assert()
        .success()
        .stdout(predicate::str::contains("app/main.py"))
        .stdout(predicate::str::contains("src/lib.rs").not());
}

#[test]
fn python_default_non_test_filter_excludes_test_files() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    fs::create_dir_all(tempdir.path().join("tests")).unwrap();
    fs::write(
        tempdir.path().join("src/app.py"),
        "def answer() -> int:\n    return 41\n\n\ndef test_inline() -> None:\n    assert True\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("tests/test_app.py"),
        "def test_external() -> None:\n    assert True\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "src/app.py", "tests/test_app.py"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("src/app.py"),
        "def answer() -> int:\n    return 42\n\n\ndef test_inline() -> None:\n    assert False\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("tests/test_app.py"),
        "def test_external() -> None:\n    assert False\n",
    )
    .unwrap();

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--lang", "py"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "未提交的 py 文件中，非测试代码统计如下：",
        ))
        .stdout(predicate::str::contains("src/app.py"))
        .stdout(predicate::str::contains("tests/test_app.py").not());
}

#[test]
fn python_test_filter_includes_test_files_and_regions() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    fs::create_dir_all(tempdir.path().join("tests")).unwrap();
    fs::write(
        tempdir.path().join("src/app.py"),
        "def answer() -> int:\n    return 41\n\n\ndef test_inline() -> None:\n    assert True\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("tests/test_app.py"),
        "def test_external() -> None:\n    assert True\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "src/app.py", "tests/test_app.py"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("src/app.py"),
        "def answer() -> int:\n    return 42\n\n\ndef test_inline() -> None:\n    assert False\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("tests/test_app.py"),
        "def test_external() -> None:\n    assert False\n",
    )
    .unwrap();

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--lang", "py", "--test"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "未提交的 py 文件中，测试代码统计如下：",
        ))
        .stdout(predicate::str::contains("src/app.py"))
        .stdout(predicate::str::contains("tests/test_app.py"));
}

#[test]
fn mixed_rust_and_python_non_test_filter_handles_both_languages() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    fs::create_dir_all(tempdir.path().join("app")).unwrap();
    fs::write(
        tempdir.path().join("src/lib.rs"),
        "pub fn answer() -> i32 {\n    41\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("app/main.py"),
        "def answer() -> int:\n    return 41\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "src/lib.rs", "app/main.py"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("src/lib.rs"),
        "pub fn answer() -> i32 {\n    42\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("app/main.py"),
        "def answer() -> int:\n    return 42\n",
    )
    .unwrap();

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--lang", "rs,py"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "未提交的 rs,py 文件中，非测试代码统计如下：",
        ))
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("app/main.py"));
}

#[test]
fn default_non_test_filter_excludes_javascript_and_typescript_test_files() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("web")).unwrap();
    fs::create_dir_all(tempdir.path().join("tests/e2e")).unwrap();
    fs::write(
        tempdir.path().join("web/app.tsx"),
        "export function App() {\n    return <div>before</div>;\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("web/app.test.tsx"),
        "test('app', () => {\n    expect(true).toBe(true);\n});\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("tests/e2e/login.ts"),
        "test('login', async () => {\n    expect(true).toBe(true);\n});\n",
    )
    .unwrap();
    run_git(
        tempdir.path(),
        [
            "add",
            "web/app.tsx",
            "web/app.test.tsx",
            "tests/e2e/login.ts",
        ],
    );
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("web/app.tsx"),
        "export function App() {\n    return <div>after</div>;\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("web/app.test.tsx"),
        "test('app', () => {\n    expect(false).toBe(false);\n});\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("tests/e2e/login.ts"),
        "test('login', async () => {\n    expect(false).toBe(false);\n});\n",
    )
    .unwrap();

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("web/app.tsx"))
        .stdout(predicate::str::contains("web/app.test.tsx").not())
        .stdout(predicate::str::contains("tests/e2e/login.ts").not());
}

#[test]
fn test_filter_includes_javascript_and_typescript_test_files() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("web")).unwrap();
    fs::create_dir_all(tempdir.path().join("cypress/e2e")).unwrap();
    fs::write(
        tempdir.path().join("web/app.tsx"),
        "export function App() {\n    return <div>before</div>;\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("web/app.spec.tsx"),
        "test('app', () => {\n    expect(true).toBe(true);\n});\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("cypress/e2e/home.cy.js"),
        "it('home', () => {\n    expect(true).to.eq(true);\n});\n",
    )
    .unwrap();
    run_git(
        tempdir.path(),
        [
            "add",
            "web/app.tsx",
            "web/app.spec.tsx",
            "cypress/e2e/home.cy.js",
        ],
    );
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("web/app.tsx"),
        "export function App() {\n    return <div>after</div>;\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("web/app.spec.tsx"),
        "test('app', () => {\n    expect(false).toBe(false);\n});\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("cypress/e2e/home.cy.js"),
        "it('home', () => {\n    expect(false).to.eq(false);\n});\n",
    )
    .unwrap();

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .arg("--test")
        .assert()
        .success()
        .stdout(predicate::str::contains("web/app.tsx").not())
        .stdout(predicate::str::contains("web/app.spec.tsx"))
        .stdout(predicate::str::contains("cypress/e2e/home.cy.js"));
}

#[test]
fn no_test_filter_includes_javascript_and_typescript_test_files() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("web")).unwrap();
    fs::create_dir_all(tempdir.path().join("playwright")).unwrap();
    fs::write(
        tempdir.path().join("web/app.jsx"),
        "export function App() {\n    return <div>before</div>;\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("playwright/auth.spec.ts"),
        "test('auth', async () => {\n    expect(true).toBe(true);\n});\n",
    )
    .unwrap();
    run_git(
        tempdir.path(),
        ["add", "web/app.jsx", "playwright/auth.spec.ts"],
    );
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("web/app.jsx"),
        "export function App() {\n    return <div>after</div>;\n}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("playwright/auth.spec.ts"),
        "test('auth', async () => {\n    expect(false).toBe(false);\n});\n",
    )
    .unwrap();

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .arg("--no-test-filter")
        .assert()
        .success()
        .stdout(predicate::str::contains("web/app.jsx"))
        .stdout(predicate::str::contains("playwright/auth.spec.ts"));
}

#[test]
fn default_non_test_filter_skips_bulk_reading_frontend_sources_for_path_rules() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("web")).unwrap();
    fs::create_dir_all(tempdir.path().join("scripts")).unwrap();
    fs::write(
        tempdir.path().join("web/app.tsx"),
        "export function App() {\n    return <div>before</div>;\n}\n",
    )
    .unwrap();
    fs::write(tempdir.path().join("scripts/build.mjs"), [0xff, 0xfe, 0xfd]).unwrap();
    run_git(tempdir.path(), ["add", "web/app.tsx", "scripts/build.mjs"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("web/app.tsx"),
        "export function App() {\n    return <div>after</div>;\n}\n",
    )
    .unwrap();

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("web/app.tsx"))
        .stdout(predicate::str::contains("scripts/build.mjs").not());
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

#[test]
fn no_test_filter_keeps_rename_only_entries() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("web/old")).unwrap();
    fs::write(
        tempdir.path().join("web/old/app.js"),
        "export const answer = () => 41;\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "web/old/app.js"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::create_dir_all(tempdir.path().join("web/new")).unwrap();
    run_git(tempdir.path(), ["mv", "web/old/app.js", "web/new/app.js"]);
    run_git(tempdir.path(), ["commit", "-m", "rename js file"]);

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--last", "--lang", "js", "--no-test-filter"])
        .assert()
        .success()
        .stdout(predicate::str::contains("web/{old => new}/app.js"))
        .stdout(predicate::str::contains("1 files changed"))
        .stdout(predicate::str::contains("0 insertions(+), 0 deletions(-)"));
}

#[test]
fn test_filter_counts_deleted_python_tests_across_language_rename() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("tests")).unwrap();
    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    fs::write(
        tempdir.path().join("tests/test_mod.py"),
        "def test_old():\n    value = 1\n    value = value + 1\n    value = value + 1\n    value = value + 1\n    assert value == 4\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "tests/test_mod.py"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    run_git(tempdir.path(), ["mv", "tests/test_mod.py", "src/lib.rs"]);
    fs::write(
        tempdir.path().join("src/lib.rs"),
        "def test_old():\n    value = 1\n    value = value + 1\n    value = value + 1\n    value = value + 2\n    assert value == 5\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "src/lib.rs"]);
    run_git(
        tempdir.path(),
        ["commit", "-m", "rename python test to rust file"],
    );

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--last", "--lang", "py,rs", "--test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test_mod.py => src/lib.rs"))
        .stdout(predicate::str::contains("2 deletions(-)"))
        .stdout(predicate::str::contains("0 files changed").not());
}

#[test]
fn no_test_filter_splits_cross_language_rename_by_selected_language() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::create_dir_all(tempdir.path().join("tests")).unwrap();
    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    fs::write(
        tempdir.path().join("tests/test_mod.py"),
        "def test_old():\n    value = 1\n    value = value + 1\n    value = value + 1\n    value = value + 1\n    assert value == 4\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "tests/test_mod.py"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    run_git(tempdir.path(), ["mv", "tests/test_mod.py", "src/lib.rs"]);
    fs::write(
        tempdir.path().join("src/lib.rs"),
        "def test_old():\n    value = 1\n    value = value + 1\n    value = value + 1\n    value = value + 2\n    assert value == 5\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "src/lib.rs"]);
    run_git(
        tempdir.path(),
        ["commit", "-m", "rename python test to rust file"],
    );

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--last", "--lang", "py", "--no-test-filter"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test_mod.py => src/lib.rs"))
        .stdout(predicate::str::contains("0 insertions(+), 2 deletions(-)"));

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--last", "--lang", "rs", "--no-test-filter"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test_mod.py => src/lib.rs"))
        .stdout(predicate::str::contains("2 insertions(+), 0 deletions(-)"));
}

#[test]
fn non_test_filter_splits_supported_to_unsupported_rename_by_selected_language() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::write(
        tempdir.path().join("README.md"),
        "pub fn answer() -> i32 {\n    41\n}\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "README.md"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    run_git(tempdir.path(), ["mv", "README.md", "src/lib.rs"]);
    fs::write(
        tempdir.path().join("src/lib.rs"),
        "pub fn answer() -> i32 {\n    42\n}\n",
    )
    .unwrap();
    run_git(tempdir.path(), ["add", "src/lib.rs"]);
    run_git(tempdir.path(), ["commit", "-m", "rename markdown to rust"]);

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--last", "--lang", "rs", "--no-test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("README.md => src/lib.rs"))
        .stdout(predicate::str::contains("1 insertions(+), 0 deletions(-)"));
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
