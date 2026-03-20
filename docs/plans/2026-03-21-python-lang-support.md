# Python Language Support Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add `--lang py` with Python test-aware filtering and refactor language-specific filtering so Rust and Python both run through the same orchestration path.

**Architecture:** Replace the current Rust-only test-filter path with lightweight language backends. Keep CLI and rendering behavior stable, move Rust-specific logic behind `src/lang/rust.rs`, add a Python backend in `src/lang/python.rs`, and centralize multi-language test-aware stat construction in a shared module.

**Tech Stack:** Rust, clap, tree-sitter, tree-sitter-rust, tree-sitter-python, assert_cmd, predicates, cargo test

---

### Task 1: Lock down `py` language selection behavior

**Files:**
- Modify: `tests/lang_filter.rs`
- Modify: `tests/cli_smoke.rs`
- Modify: `README.md`

**Step 1: Write the failing test**

Add one unit test proving `filter_by_langs` keeps `.py` files for `["py"]`, and one CLI smoke test proving `--lang py --no-test-filter` includes a Python file and excludes the default Rust-only path when Python is explicitly requested.

**Step 2: Run test to verify it fails**

Run: `cargo test --test lang_filter --test cli_smoke -v`
Expected: FAIL because `.py` is not a recognized language yet.

**Step 3: Write minimal implementation**

Teach language detection to recognize `.py` and update README usage examples so Python is documented as a supported language.

**Step 4: Run test to verify it passes**

Run: `cargo test --test lang_filter --test cli_smoke -v`
Expected: PASS

### Task 2: Introduce a language module layout that can host multiple backends

**Files:**
- Create: `src/lang/mod.rs`
- Create: `src/lang/rust.rs`
- Create: `src/lang/python.rs`
- Modify: `src/lib.rs`
- Modify: `src/main.rs`
- Modify: `src/lang.rs` or move its contents into `src/lang/mod.rs`

**Step 1: Write the failing test**

Add or adjust unit tests so language parsing still accepts `rs`, `js`, `ts`, and now `py`, and so path-based language detection works after the module split.

**Step 2: Run test to verify it fails**

Run: `cargo test --test lang_filter -v`
Expected: FAIL while the old single-file `lang.rs` layout is being replaced.

**Step 3: Write minimal implementation**

Move language parsing and extension detection into `src/lang/mod.rs`. Add backend-oriented helpers in `src/lang/rust.rs` and `src/lang/python.rs`. Keep the public surface small: parsing requested languages, detecting a file's language, and exposing backend helpers needed by test filtering.

**Step 4: Run test to verify it passes**

Run: `cargo test --test lang_filter -v`
Expected: PASS

### Task 3: Add Python test detection primitives

**Files:**
- Create: `src/python_tests.rs` or keep parser helpers inside `src/lang/python.rs`
- Create: `tests/python_tests.rs`
- Modify: `Cargo.toml`

**Step 1: Write the failing test**

Add focused tests proving Python detection marks these as test code:

```python
def test_basic():
    assert True

class TestApi:
    def test_fetch(self):
        assert True
```

Also add tests proving a production helper such as `def build_report():` is not test code, and path tests proving `tests/foo.py`, `test_bar.py`, `bar_test.py`, and `conftest.py` are whole-file tests.

**Step 2: Run test to verify it fails**

Run: `cargo test --test python_tests -v`
Expected: FAIL because Python parsing and path classification do not exist yet.

**Step 3: Write minimal implementation**

Add `tree-sitter-python` and implement Python helpers for:

- whole-file test path detection
- line-region detection for test functions and `class Test*`
- untracked file splitting
- tracked patch splitting

Keep the first version intentionally narrow and pytest-oriented.

**Step 4: Run test to verify it passes**

Run: `cargo test --test python_tests -v`
Expected: PASS

### Task 4: Replace the Rust-only builder with shared multi-language test filtering

**Files:**
- Create: `src/test_filter.rs`
- Modify: `src/main.rs`
- Modify: `src/filter.rs`
- Modify: `src/lang/rust.rs`
- Modify: `src/lang/python.rs`

**Step 1: Write the failing test**

Add or update smoke coverage to prove:

- `--lang py` default output excludes Python test-only changes
- `--lang py --test` includes only Python test changes
- `--lang rs,py` handles both languages in one run

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli_smoke -v`
Expected: FAIL because `main.rs` still only builds test-aware stats for Rust files.

**Step 3: Write minimal implementation**

Extract the current Rust-only test-aware stat builder from `main.rs` into a shared module. Generalize the source-loading helpers so they operate on selected language kinds, not only Rust. Dispatch whole-file test detection and region splitting through the relevant language backend, while preserving current Rust behavior.

**Step 4: Run test to verify it passes**

Run: `cargo test --test cli_smoke -v`
Expected: PASS

### Task 5: Preserve Rust behavior during the refactor

**Files:**
- Modify: `tests/rust_tests.rs`
- Modify: `tests/cli_smoke.rs`
- Modify: `tests/lang_filter.rs`
- Modify: `src/rust_tests.rs`
- Modify: `src/lang/rust.rs`

**Step 1: Write the failing test**

Add one regression proving a Rust integration test file is still treated as a whole-file test after the backend extraction, and one regression proving mixed Rust source with `#[cfg(test)]` still splits lines correctly.

**Step 2: Run test to verify it fails**

Run: `cargo test --test rust_tests --test cli_smoke -v`
Expected: FAIL if the refactor changes Rust classification behavior.

**Step 3: Write minimal implementation**

Move Rust-specific orchestration into `src/lang/rust.rs` without changing the existing region and imported-module semantics. Keep `src/rust_tests.rs` as a parsing helper if that reduces churn.

**Step 4: Run test to verify it passes**

Run: `cargo test --test rust_tests --test cli_smoke -v`
Expected: PASS

### Task 6: Update help text and docs for Python support

**Files:**
- Modify: `src/cli.rs`
- Modify: `README.md`
- Modify: `tests/cli_smoke.rs`

**Step 1: Write the failing test**

Adjust help-text expectations so examples mention `--lang py` as a supported, test-aware language path and keep the existing Rust-default examples intact.

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli_smoke help_mentions_common_examples -v`
Expected: FAIL because help text still only reflects the old language set.

**Step 3: Write minimal implementation**

Update `after_help`, usage examples, and README notes to explain:

- `--lang py`
- pytest-oriented Python test detection
- default `--lang rs`
- `--no-test-filter` behavior across selected languages

**Step 4: Run test to verify it passes**

Run: `cargo test --test cli_smoke help_mentions_common_examples -v`
Expected: PASS

### Task 7: Run the full verification suite

**Files:**
- Modify: none

**Step 1: Run targeted tests**

Run:

```bash
cargo test --test lang_filter -v
cargo test --test python_tests -v
cargo test --test rust_tests -v
cargo test --test cli_smoke -v
```

Expected: PASS

**Step 2: Run the full test suite**

Run: `cargo test -v`
Expected: PASS

**Step 3: Run lint if available**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: PASS

**Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock src tests README.md docs/plans/2026-03-21-python-lang-support-design.md docs/plans/2026-03-21-python-lang-support.md
git commit -m "feat: add python test-aware language support"
```
