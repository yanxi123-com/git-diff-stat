# Git Diff Stat Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust CLI named `git-diff-stat` that extends `git diff --stat` with untracked-file inclusion, language filters, Rust test/non-test stat splitting, and commit/range support.

**Architecture:** The binary shells out to `git` for repository truth, normalizes file and patch data into internal models, applies language and Rust test-region filters, and renders a stat view close to `git diff --stat`. Rust test filtering is implemented by mapping unified diff lines onto `tree-sitter-rust` test-node line ranges from old and new file versions.

**Tech Stack:** Rust, `clap`, `anyhow`, `thiserror`, `tree-sitter`, `tree-sitter-rust`, `assert_cmd`, `predicates`, temporary git repos in integration tests.

---

### Task 1: Bootstrap the Cargo project

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `tests/cli_smoke.rs`

**Step 1: Write the failing test**

```rust
use assert_cmd::Command;

#[test]
fn prints_help() {
    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test tests::cli_smoke -- --nocapture`
Expected: FAIL because the project does not exist yet.

**Step 3: Write minimal implementation**

- create a Cargo binary named `git-diff-stat`
- wire `main.rs` to parse CLI args and print help

**Step 4: Run test to verify it passes**

Run: `cargo test --test cli_smoke -v`
Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml src/main.rs src/lib.rs tests/cli_smoke.rs
git commit -m "feat: bootstrap git-diff-stat cli"
```

### Task 2: Add CLI argument parsing and validation

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/main.rs`
- Create: `src/cli.rs`
- Create: `tests/cli_args.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn rejects_test_and_no_test_together() {
    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .args(["--test", "--no-test"])
        .assert()
        .failure();
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli_args rejects_test_and_no_test_together -v`
Expected: FAIL because validation is missing.

**Step 3: Write minimal implementation**

- add `clap`
- support positional revisions
- support `--commit <rev>`
- support `--lang <csv>`
- support `--test`
- support `--no-test`
- enforce mutual exclusion and revision rules

**Step 4: Run test to verify it passes**

Run: `cargo test --test cli_args -v`
Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml src/main.rs src/cli.rs tests/cli_args.rs
git commit -m "feat: add cli argument parsing"
```

### Task 3: Model revision selection

**Files:**
- Create: `src/revision.rs`
- Modify: `src/lib.rs`
- Create: `tests/revision.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn maps_commit_flag_to_single_commit_patch() {
    let selection = RevisionSelection::from_commit("abc123").unwrap();
    assert_eq!(selection.git_diff_args(), vec!["abc123^!".to_string()]);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test revision maps_commit_flag_to_single_commit_patch -v`
Expected: FAIL because revision modeling does not exist.

**Step 3: Write minimal implementation**

- add a revision selection enum
- convert CLI inputs into normalized git diff args

**Step 4: Run test to verify it passes**

Run: `cargo test --test revision -v`
Expected: PASS

**Step 5: Commit**

```bash
git add src/lib.rs src/revision.rs tests/revision.rs
git commit -m "feat: normalize revision selection"
```

### Task 4: Add Git command wrapper

**Files:**
- Create: `src/git.rs`
- Modify: `src/lib.rs`
- Create: `tests/git_errors.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn returns_clear_error_outside_repository() {
    let err = Git::new().diff_numstat(&[]).unwrap_err();
    assert!(err.to_string().contains("git repository"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test git_errors returns_clear_error_outside_repository -v`
Expected: FAIL because no git adapter exists.

**Step 3: Write minimal implementation**

- wrap `Command`
- capture stdout, stderr, exit status
- translate common git failures into readable errors

**Step 4: Run test to verify it passes**

Run: `cargo test --test git_errors -v`
Expected: PASS

**Step 5: Commit**

```bash
git add src/lib.rs src/git.rs tests/git_errors.rs
git commit -m "feat: add git command adapter"
```

### Task 5: Parse file-level diff metadata and untracked files

**Files:**
- Create: `src/change.rs`
- Modify: `src/git.rs`
- Modify: `src/lib.rs`
- Create: `tests/untracked.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn includes_untracked_files_as_added_lines() {
    let stats = run_in_fixture_repo(["--lang", "rs"]);
    assert!(stats.stdout.contains("new_file.rs"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test untracked includes_untracked_files_as_added_lines -v`
Expected: FAIL because untracked files are not modeled.

**Step 3: Write minimal implementation**

- parse `git diff --numstat -z`
- parse untracked files from `git ls-files --others --exclude-standard`
- count current line totals for untracked files
- normalize both into one change list

**Step 4: Run test to verify it passes**

Run: `cargo test --test untracked -v`
Expected: PASS

**Step 5: Commit**

```bash
git add src/lib.rs src/git.rs src/change.rs tests/untracked.rs
git commit -m "feat: include untracked files in normalized changes"
```

### Task 6: Implement language filtering

**Files:**
- Create: `src/lang.rs`
- Modify: `src/change.rs`
- Modify: `src/lib.rs`
- Create: `tests/lang_filter.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn filters_to_requested_extensions() {
    let stats = run_in_fixture_repo(["--lang", "rs"]);
    assert!(stats.stdout.contains("src/lib.rs"));
    assert!(!stats.stdout.contains("web/app.ts"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test lang_filter filters_to_requested_extensions -v`
Expected: FAIL because file classification is missing.

**Step 3: Write minimal implementation**

- map extensions to language ids
- filter normalized changes by requested language set

**Step 4: Run test to verify it passes**

Run: `cargo test --test lang_filter -v`
Expected: PASS

**Step 5: Commit**

```bash
git add src/lib.rs src/change.rs src/lang.rs tests/lang_filter.rs
git commit -m "feat: add language filtering"
```

### Task 7: Parse unified diffs into hunk line events

**Files:**
- Create: `src/patch.rs`
- Modify: `src/git.rs`
- Modify: `src/lib.rs`
- Create: `tests/patch_parser.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn maps_added_and_deleted_lines_to_file_positions() {
    let patch = parse_patch(EXAMPLE_PATCH).unwrap();
    assert_eq!(patch.files[0].line_events.len(), 3);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test patch_parser maps_added_and_deleted_lines_to_file_positions -v`
Expected: FAIL because unified diff parsing is missing.

**Step 3: Write minimal implementation**

- call `git diff --unified=0 --no-ext-diff --find-renames`
- parse file headers and hunk headers
- emit added and deleted line events with old/new line numbers

**Step 4: Run test to verify it passes**

Run: `cargo test --test patch_parser -v`
Expected: PASS

**Step 5: Commit**

```bash
git add src/lib.rs src/git.rs src/patch.rs tests/patch_parser.rs
git commit -m "feat: parse unified diff hunks"
```

### Task 8: Detect Rust test regions with tree-sitter

**Files:**
- Modify: `Cargo.toml`
- Create: `src/rust_tests.rs`
- Modify: `src/lib.rs`
- Create: `tests/rust_tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn identifies_cfg_test_modules_and_test_functions() {
    let regions = detect_test_regions(RUST_SOURCE).unwrap();
    assert!(regions.contains_line(12));
    assert!(regions.contains_line(28));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test rust_tests identifies_cfg_test_modules_and_test_functions -v`
Expected: FAIL because Rust test detection does not exist.

**Step 3: Write minimal implementation**

- add `tree-sitter` and `tree-sitter-rust`
- parse source into syntax tree
- collect line intervals for:
  - `#[cfg(test)]` modules
  - functions with test-like attributes

**Step 4: Run test to verify it passes**

Run: `cargo test --test rust_tests -v`
Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml src/lib.rs src/rust_tests.rs tests/rust_tests.rs
git commit -m "feat: detect rust test regions"
```

### Task 9: Split Rust stats into test and non-test line counts

**Files:**
- Create: `src/filter.rs`
- Modify: `src/change.rs`
- Modify: `src/patch.rs`
- Modify: `src/rust_tests.rs`
- Modify: `src/lib.rs`
- Create: `tests/test_split.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn counts_test_and_non_test_changes_separately_in_same_file() {
    let test_stats = run_in_fixture_repo(["--test", "--lang", "rs"]);
    let non_test_stats = run_in_fixture_repo(["--no-test", "--lang", "rs"]);
    assert!(test_stats.stdout.contains("src/lib.rs"));
    assert!(non_test_stats.stdout.contains("src/lib.rs"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test test_split counts_test_and_non_test_changes_separately_in_same_file -v`
Expected: FAIL because patch events are not mapped to test regions.

**Step 3: Write minimal implementation**

- load old and new file content for changed `.rs` files
- compute old/new test-region intervals
- classify deleted lines using old intervals
- classify added lines using new intervals
- aggregate per-file test and non-test counts

**Step 4: Run test to verify it passes**

Run: `cargo test --test test_split -v`
Expected: PASS

**Step 5: Commit**

```bash
git add src/lib.rs src/change.rs src/patch.rs src/rust_tests.rs src/filter.rs tests/test_split.rs
git commit -m "feat: split rust stats by test regions"
```

### Task 10: Render stat output

**Files:**
- Create: `src/render.rs`
- Modify: `src/main.rs`
- Modify: `src/lib.rs`
- Create: `tests/render.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn prints_summary_line_with_insertions_and_deletions() {
    let stats = run_in_fixture_repo([]);
    assert!(stats.stdout.contains("files changed"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test render prints_summary_line_with_insertions_and_deletions -v`
Expected: FAIL because no renderer exists.

**Step 3: Write minimal implementation**

- render file rows from normalized stats
- render total changed file count
- render insertion and deletion totals
- support partial file counts in `--test` and `--no-test`

**Step 4: Run test to verify it passes**

Run: `cargo test --test render -v`
Expected: PASS

**Step 5: Commit**

```bash
git add src/main.rs src/lib.rs src/render.rs tests/render.rs
git commit -m "feat: render diff stat output"
```

### Task 11: Add end-to-end revision coverage

**Files:**
- Create: `tests/revisions_integration.rs`
- Modify: `tests/cli_args.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn supports_commit_flag_and_native_ranges() {
    let commit_stats = run_in_fixture_repo(["--commit", "HEAD"]);
    let range_stats = run_in_fixture_repo(["HEAD~1..HEAD"]);
    assert!(commit_stats.status.success());
    assert!(range_stats.status.success());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test revisions_integration supports_commit_flag_and_native_ranges -v`
Expected: FAIL because integration coverage is incomplete.

**Step 3: Write minimal implementation**

- ensure revision selection threads correctly through git adapter
- cover `A B`, `A..B`, `A...B`, and `--commit`

**Step 4: Run test to verify it passes**

Run: `cargo test --test revisions_integration -v`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/revisions_integration.rs tests/cli_args.rs src/cli.rs src/revision.rs src/git.rs
git commit -m "test: cover revision forms end to end"
```

### Task 12: Tighten failure behavior for unsupported and parse-error cases

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/rust_tests.rs`
- Modify: `src/main.rs`
- Create: `tests/errors.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn fails_fast_when_rust_test_filter_cannot_be_computed() {
    let result = run_with_broken_rust_input(["--test"]);
    assert!(!result.status.success());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test errors fails_fast_when_rust_test_filter_cannot_be_computed -v`
Expected: FAIL because error handling is incomplete.

**Step 3: Write minimal implementation**

- return clear failures for invalid flag combinations
- return clear failures when Rust test-region parsing is required but unavailable
- ensure stderr messaging is readable

**Step 4: Run test to verify it passes**

Run: `cargo test --test errors -v`
Expected: PASS

**Step 5: Commit**

```bash
git add src/cli.rs src/rust_tests.rs src/main.rs tests/errors.rs
git commit -m "feat: improve error handling for core filters"
```

### Task 13: Verify the full suite and document usage

**Files:**
- Create: `README.md`
- Modify: `tests/cli_smoke.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn help_mentions_commit_lang_and_test_filters() {
    Command::cargo_bin("git-diff-stat")
        .unwrap()
        .arg("--help")
        .assert()
        .stdout(predicates::str::contains("--commit"))
        .stdout(predicates::str::contains("--lang"))
        .stdout(predicates::str::contains("--test"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli_smoke help_mentions_commit_lang_and_test_filters -v`
Expected: FAIL until help text and docs are finalized.

**Step 3: Write minimal implementation**

- finalize help text
- add README usage examples:
  - default mode with untracked files
  - `--lang rs`
  - `--test`
  - `--no-test`
  - `--commit HEAD`

**Step 4: Run test to verify it passes**

Run: `cargo test --test cli_smoke -v`
Expected: PASS

**Step 5: Commit**

```bash
git add README.md tests/cli_smoke.rs src/main.rs src/cli.rs
git commit -m "docs: add usage examples and finalize help output"
```

### Task 14: Final verification

**Files:**
- Modify: `none`

**Step 1: Run the full test suite**

Run: `cargo test -v`
Expected: PASS

**Step 2: Run a formatting check**

Run: `cargo fmt --check`
Expected: PASS

**Step 3: Run linting**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: PASS

**Step 4: Smoke test against a real repository**

Run:

```bash
cargo run -- --help
cargo run -- --commit HEAD
cargo run -- --lang rs --test
```

Expected: commands succeed in a suitable repository and produce readable output.

**Step 5: Commit**

```bash
git add -A
git commit -m "chore: finalize git-diff-stat v1"
```
