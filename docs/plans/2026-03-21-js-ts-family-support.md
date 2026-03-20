# JS/TS Family Language Support Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add `js`, `ts`, `jsx`, `tsx`, `cjs`, and `mjs` as first-class `--lang` values, make omitted `--lang` mean "all supported languages", and extend `--test` / `--no-test` to JS/TS unit and e2e test files.

**Architecture:** Centralize supported/default languages in `src/lang/mod.rs`, add a `src/lang/javascript.rs` backend for whole-file test classification, and update `src/test_filter.rs` to support languages that only provide whole-file test semantics. Preserve Rust and Python behavior while broadening default language coverage.

**Tech Stack:** Rust, clap, assert_cmd, predicates, cargo test, cargo clippy

---

### Task 1: Lock down registry-driven default language behavior

**Files:**
- Modify: `tests/lang_filter.rs`
- Modify: `tests/cli_smoke.rs`
- Modify: `src/lang/mod.rs`
- Modify: `src/cli.rs`

**Step 1: Write the failing test**

Add unit coverage proving that omitted `--lang` expands to all supported languages instead of a fixed `rs,py` subset. Add one CLI smoke regression proving plain `git diff-stat` includes a frontend source file in addition to Rust/Python when that file has non-test changes.

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --test lang_filter --test cli_smoke -v
```

Expected: FAIL because the CLI and language parser still default to `rs,py`.

**Step 3: Write minimal implementation**

Move default-language responsibility into `src/lang/mod.rs`. Remove the hard-coded CLI default value and make omitted `--lang` resolve to all supported tokens from the registry.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --test lang_filter --test cli_smoke -v
```

Expected: PASS

### Task 2: Add full JS/TS family extension support

**Files:**
- Modify: `src/lang/mod.rs`
- Create: `src/lang/javascript.rs`
- Modify: `src/lib.rs`
- Modify: `tests/lang_filter.rs`

**Step 1: Write the failing test**

Add unit tests proving language detection and filtering recognize:

- `app.js`
- `app.ts`
- `component.jsx`
- `component.tsx`
- `config.cjs`
- `entry.mjs`

Also add a regression proving explicit `--lang tsx` only keeps `.tsx` files.

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --test lang_filter -v
```

Expected: FAIL because the new extensions are not fully supported yet.

**Step 3: Write minimal implementation**

Add `src/lang/javascript.rs` and route JS/TS family extension detection through it. Keep the token model explicit: each extension remains its own `--lang` value rather than collapsing into one alias.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --test lang_filter -v
```

Expected: PASS

### Task 3: Add JS/TS whole-file test classification primitives

**Files:**
- Create: `tests/javascript_tests.rs`
- Modify: `src/lang/javascript.rs`

**Step 1: Write the failing test**

Add focused unit tests proving these are classified as whole-file tests:

- `src/__tests__/app.ts`
- `web/app.test.tsx`
- `web/app.spec.jsx`
- `tests/e2e/login.ts`
- `cypress/e2e/home.cy.js`
- `playwright/auth.spec.ts`

Also add negative cases such as:

- `src/app.ts`
- `scripts/build.mjs`
- `playwright.config.ts`

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --test javascript_tests -v
```

Expected: FAIL because JS/TS whole-file test classification does not exist yet.

**Step 3: Write minimal implementation**

Implement path-based whole-file classification in `src/lang/javascript.rs` using path components and filename patterns only. Do not add AST parsing or inline test detection.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --test javascript_tests -v
```

Expected: PASS

### Task 4: Teach the shared test filter to handle whole-file-only languages

**Files:**
- Modify: `src/test_filter.rs`
- Modify: `src/main.rs`
- Modify: `src/lang/javascript.rs`
- Modify: `tests/cli_smoke.rs`

**Step 1: Write the failing test**

Add CLI smoke coverage proving:

- default `--no-test` excludes `*.test.tsx` and e2e files from the output
- `--test` includes those same files
- `--no-test-filter` reports them as ordinary full-file stats
- non-test JS/TS source files are still counted under `--no-test`

Use a mixed repository that contains at least one frontend source file and at least one frontend test file.

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --test cli_smoke -v
```

Expected: FAIL because the shared builder currently only dispatches Rust and Python test-aware behavior.

**Step 3: Write minimal implementation**

Extend `src/test_filter.rs` so JS/TS family backends can contribute whole-file test paths without region splitting. For files that are not whole-file tests, treat the full diff as non-test code.

Keep the builder scoped to selected languages only.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --test cli_smoke -v
```

Expected: PASS

### Task 5: Avoid unnecessary frontend source reads during whole-file classification

**Files:**
- Modify: `src/test_filter.rs`
- Modify: `src/lang/rust.rs`
- Modify: `src/lang/python.rs`
- Modify: `src/lang/javascript.rs`
- Modify: `tests/cli_smoke.rs`

**Step 1: Write the failing test**

Add a regression similar to the existing Python review fix: create a tracked JS/TS family file with non-UTF8 bytes that should be ignored when the selected languages do not include it, and prove the command still succeeds.

Also add coverage proving JS/TS whole-file classification can be computed without bulk-reading frontend source contents when only path-based rules are needed.

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --test cli_smoke -v
```

Expected: FAIL if the builder still eagerly reads irrelevant frontend sources.

**Step 3: Write minimal implementation**

Separate path-only whole-file classification from source-assisted whole-file classification inside `src/test_filter.rs`. Preserve Rust behavior for imported `#[cfg(test)]` modules while avoiding unnecessary JS/TS content reads.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --test cli_smoke -v
```

Expected: PASS

### Task 6: Update help text and README for all-supported-language defaults

**Files:**
- Modify: `src/cli.rs`
- Modify: `README.md`
- Modify: `tests/cli_smoke.rs`
- Modify: `tests/readme_presence.rs`

**Step 1: Write the failing test**

Update help-text expectations and README assertions so they require:

- default `--lang` meaning all supported languages
- examples that mention JS/TS family usage
- notes describing frontend whole-file test detection

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --test cli_smoke help_mentions_common_examples -v
cargo test --test readme_presence -v
```

Expected: FAIL because docs and help still describe the older default set.

**Step 3: Write minimal implementation**

Update help examples, defaults text, README usage, and notes so the supported language list and default behavior are explicit and consistent with the registry.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --test cli_smoke help_mentions_common_examples -v
cargo test --test readme_presence -v
```

Expected: PASS

### Task 7: Run the full verification suite

**Files:**
- Modify: none

**Step 1: Run targeted tests**

Run:

```bash
cargo test --test lang_filter -v
cargo test --test javascript_tests -v
cargo test --test python_tests -v
cargo test --test rust_tests -v
cargo test --test cli_smoke -v
```

Expected: PASS

**Step 2: Run the full test suite**

Run:

```bash
cargo test -v
```

Expected: PASS

**Step 3: Run lint**

Run:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Expected: PASS

**Step 4: Commit**

```bash
git add README.md src tests docs/plans/2026-03-21-js-ts-family-support-design.md docs/plans/2026-03-21-js-ts-family-support.md
git commit -m "feat: add js and ts family language support"
```
