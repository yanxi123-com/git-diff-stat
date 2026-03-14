# Rust Test Path Classification Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `--test` and `--no-test` classify Rust files under `tests/` as test code for the entire file.

**Architecture:** Add a small path classifier in the Rust test-filter layer, run it before AST-based region splitting, and verify the behavior with focused regression tests plus full-suite verification. This preserves the current partial-splitting logic for production files while making integration test files match Cargo conventions.

**Tech Stack:** Rust, `clap`, existing diff/patch pipeline, `assert_cmd`, temporary git repositories, current Rust test-region utilities.

---

### Task 1: Add failing regression tests for `tests/*.rs`

**Files:**
- Modify: `tests/cli_smoke.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_filter_counts_rust_integration_test_files_as_test() {
    let repo = init_repo_with_commit([
        ("tests/integration.rs", "fn helper() {}\n"),
    ]);

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(repo.path())
        .args(["--last", "--lang", "rs", "--test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tests/integration.rs"));
}

#[test]
fn no_test_filter_excludes_rust_integration_test_files() {
    let repo = init_repo_with_commit([
        ("tests/integration.rs", "fn helper() {}\n"),
    ]);

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(repo.path())
        .args(["--last", "--lang", "rs", "--no-test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0 files changed"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli_smoke test_filter_counts_rust_integration_test_files_as_test -- --exact`
Expected: FAIL because `tests/integration.rs` is currently treated as non-test unless lines are inside annotated test regions.

Run: `cargo test --test cli_smoke no_test_filter_excludes_rust_integration_test_files -- --exact`
Expected: FAIL because the current implementation reports the helper function lines as non-test.

**Step 3: Write minimal implementation**

- add a helper that detects whether a Rust file has a path segment exactly equal to `tests`
- use it before AST splitting so path-matched files return full test counts

**Step 4: Run test to verify it passes**

Run: `cargo test --test cli_smoke test_filter_counts_rust_integration_test_files_as_test no_test_filter_excludes_rust_integration_test_files -v`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/cli_smoke.rs src/main.rs src/filter.rs
git commit -m "feat: classify rust integration tests by path"
```

### Task 2: Wire path classification into the Rust filter layer

**Files:**
- Modify: `src/filter.rs`
- Modify: `src/main.rs`

**Step 1: Write the failing unit test**

```rust
#[test]
fn rust_file_under_tests_segment_is_full_test_file() {
    assert!(is_rust_integration_test_path("tests/foo.rs"));
    assert!(is_rust_integration_test_path("crates/app/tests/foo.rs"));
    assert!(!is_rust_integration_test_path("src/tests_support/foo.rs"));
    assert!(!is_rust_integration_test_path("src/lib.rs"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test is_rust_integration_test_path -v`
Expected: FAIL because the helper does not exist yet.

**Step 3: Write minimal implementation**

- add `is_rust_integration_test_path(path: &str) -> bool`
- use the helper so `split_untracked_rust_source` and `split_file_patch_for_rust_tests` can short-circuit to full-file test counts when the path matches
- thread the file path through the call sites without changing non-Rust behavior

**Step 4: Run test to verify it passes**

Run: `cargo test is_rust_integration_test_path -v`
Expected: PASS

**Step 5: Commit**

```bash
git add src/filter.rs src/main.rs
git commit -m "refactor: add rust test path classifier"
```

### Task 3: Update docs and verify no regression

**Files:**
- Modify: `README.md`

**Step 1: Update docs**

- document that Rust files under `tests/` are treated as test files in full
- keep the existing note that non-`tests/` Rust files still use AST-based detection

**Step 2: Run targeted tests**

Run: `cargo test --test cli_smoke --test test_split`
Expected: PASS

**Step 3: Run full test suite**

Run: `cargo test`
Expected: PASS

**Step 4: Commit**

```bash
git add README.md src/filter.rs src/main.rs tests/cli_smoke.rs
git commit -m "docs: clarify rust integration test filtering"
```
