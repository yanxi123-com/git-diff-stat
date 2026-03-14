# Last Commit Shortcut Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a `--last` flag that shows the patch introduced by `HEAD`, equivalent to `HEAD^!`.

**Architecture:** Extend the `clap` CLI with a boolean shortcut flag, normalize it in the revision-selection layer by reusing the existing single-commit patch representation, and verify the behavior through focused CLI, revision, and end-to-end tests.

**Tech Stack:** Rust, `clap`, `assert_cmd`, temporary git repositories, existing revision-selection and git diff plumbing.

---

### Task 1: Add the failing CLI and revision tests

**Files:**
- Modify: `tests/cli_args.rs`
- Modify: `tests/revision.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn rejects_last_with_commit() {
    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .args(["--last", "--commit", "HEAD"])
        .assert()
        .failure();
}

#[test]
fn maps_last_flag_to_head_patch() {
    let cli = Cli::parse_from(["git-diff-stat", "--last"]);
    let selection = RevisionSelection::from_cli(&cli).unwrap();

    assert_eq!(selection.git_diff_args(), vec!["HEAD^!".to_string()]);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli_args --test revision`
Expected: FAIL because `--last` does not exist yet.

**Step 3: Write minimal implementation**

- add `last: bool` to the CLI definition
- add conflicts with `--commit` and positional revisions
- map `--last` to `RevisionSelection::CommitPatch("HEAD".to_string())`

**Step 4: Run test to verify it passes**

Run: `cargo test --test cli_args --test revision`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/cli_args.rs tests/revision.rs src/cli.rs src/revision.rs
git commit -m "feat: add last-commit shortcut"
```

### Task 2: Add an end-to-end regression test and docs

**Files:**
- Modify: `tests/cli_smoke.rs`
- Modify: `README.md`
- Modify: `src/cli.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn last_flag_reports_head_patch() {
    let repo = init_repo_with_latest_commit();

    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .current_dir(repo.path())
        .arg("--last")
        .assert()
        .success()
        .stdout(predicate::str::contains("tracked.txt"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli_smoke last_flag_reports_head_patch -v`
Expected: FAIL before the feature is wired through.

**Step 3: Write minimal implementation**

- update help examples to mention `--last`
- update README usage and revision docs

**Step 4: Run test to verify it passes**

Run: `cargo test --test cli_smoke last_flag_reports_head_patch -v`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/cli_smoke.rs README.md src/cli.rs
git commit -m "docs: document last-commit shortcut"
```

### Task 3: Run full verification

**Files:**
- Modify: none

**Step 1: Run targeted tests**

Run: `cargo test --test cli_args --test revision --test cli_smoke`
Expected: PASS

**Step 2: Run full test suite**

Run: `cargo test`
Expected: PASS

**Step 3: Review user-facing docs**

Check that `README.md` and `--help` both mention `git diff-stat --last`.

**Step 4: Commit**

```bash
git add README.md src/cli.rs tests/cli_args.rs tests/revision.rs tests/cli_smoke.rs
git commit -m "feat: support last committed patch shortcut"
```
