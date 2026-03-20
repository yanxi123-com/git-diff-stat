# Python Language Support Design

**Context**

`git-diff-stat` currently treats `--lang` as a thin file-extension filter in [`src/lang.rs`](../../src/lang.rs), while Rust test-aware behavior is implemented separately in [`src/filter.rs`](../../src/filter.rs) and [`src/rust_tests.rs`](../../src/rust_tests.rs). This works for a single language, but it couples the main execution path to Rust-specific logic and makes each new language support request disproportionately expensive.

The immediate goal is to support `--lang py` with the same test-filter semantics that Rust already participates in:

- default / `--no-test`: report only non-test code
- `--test`: report only test code
- `--no-test-filter`: report full-file stats without test splitting

The target Python test style is the one used by `winq_bt`: `pytest`-style test discovery centered around `tests/`, `test_*.py`, `*_test.py`, `conftest.py`, top-level `def test_*`, and `class Test*`.

**Goal**

Add first-class Python support without turning `main.rs` into a per-language switchboard. The structure should make future languages easier to add, while keeping the current Rust behavior unchanged.

**Approaches**

1. Extend the current code path with Python-specific branches in `main.rs` and `filter.rs`.
   - Lowest short-term cost.
   - Rejected because it keeps the architecture centered on Rust special-cases and makes future additions harder.

2. Introduce lightweight language backends and move test-aware behavior behind a shared interface.
   - Slightly more up-front work.
   - Keeps CLI/rendering stable while moving language-specific rules into isolated modules.
   - Recommended.

3. Build a fully generic plugin system now.
   - Over-designed for the current repository size and only one additional language.
   - Rejected as unnecessary complexity.

**Decision**

Use lightweight language backends.

The refactor should not aim for a public plugin API. It only needs enough structure to answer these questions in one place per language:

- does this path belong to the language?
- which files are whole-file tests?
- for mixed files, which changed lines are test lines vs non-test lines?

`main.rs` should keep orchestrating Git I/O, revision selection, rendering, and CLI interpretation. It should stop knowing Rust-specific details.

**Proposed Structure**

- `src/lang/mod.rs`
  - language parsing and normalization
  - registry of supported languages
  - path-to-language detection
- `src/lang/rust.rs`
  - wraps current Rust-specific behavior
  - owns Rust whole-file test-path detection and line-region splitting
- `src/lang/python.rs`
  - Python path matching and test-region detection
- `src/test_filter.rs`
  - shared orchestration for building test-only or non-test-only stats across requested languages
- `src/rust_tests.rs`
  - can remain as a Rust parser helper used by `src/lang/rust.rs`

This is a moderate refactor, not a rewrite. Existing types such as `FileChange`, `FilePatch`, `DisplayStat`, and `TestFilterMode` remain useful as-is.

**Backend Shape**

The shared test-filter orchestration should operate in terms of a small internal backend contract. The exact Rust type names can vary, but the responsibilities should look like this:

- language identity and aliases, such as `rs` and `py`
- file matching by extension
- optional whole-file-test classification
- optional per-file region splitting for tracked and untracked files

One practical model is:

- `LanguageKind` enum for supported languages
- helper functions in each language module instead of trait objects
- a dispatcher in `test_filter.rs` that groups changed files by language and invokes the relevant backend helpers

This avoids unnecessary dynamic dispatch while still removing language conditionals from `main.rs`.

**Python Test Semantics**

Python support should match common `pytest` conventions first.

Whole-file test rules:

- any `.py` file under a `tests/` path component
- `test_*.py`
- `*_test.py`
- `conftest.py`

Mixed-file region rules:

- top-level `def test_*`
- methods named `test_*`
- `class Test*`

That gives useful behavior for projects like `winq_bt` without trying to model every Python test framework on day one.

Not in scope for the first version:

- full `unittest.TestCase` inference beyond names already covered by `test_*`
- custom pytest discovery configuration
- doctests
- dynamic test generation

**Parsing Strategy**

Use `tree-sitter-python` alongside the existing `tree-sitter` setup.

This aligns with the current Rust implementation style:

- accurate line ranges for test functions and classes
- support for tracked diffs and untracked files
- no need to invent a fragile indentation-based parser

The Python parser only needs to detect class and function definition ranges. It does not need semantic import resolution similar to Rust's `#[cfg(test)] mod` handling.

**Data Flow**

The runtime flow should become:

1. Parse CLI and revision selection.
2. Parse requested languages into supported language kinds.
3. Filter `FileChange` values by requested languages.
4. If `--no-test-filter`, render full-file stats directly.
5. Otherwise, call a shared test-aware stats builder.
6. The builder:
   - parses the diff patch once
   - loads per-revision or worktree sources as needed
   - asks each language backend for whole-file test paths
   - asks each language backend to split changed lines into test/non-test counts
7. Render using the existing header machinery.

The important shift is that the builder should work over "requested supported languages", not over "Rust files only".

**Compatibility Rules**

- Default `--lang` remains `rs` for now.
- `--lang py` participates in the same `--test`, `--no-test`, and `--no-test-filter` flags.
- `--lang rs,py` should combine both backends in one run.
- Non-test-filtered output remains full-file diff stats for any selected language.
- Unknown or unsupported language names should continue to be ignored or rejected consistently with current behavior; if validation is added, it should happen centrally in `src/lang/mod.rs`.

**Testing Strategy**

Add coverage at three layers:

1. Unit tests for language recognition and normalization.
2. Unit tests for Python test-region detection and whole-file test-path classification.
3. CLI smoke tests proving end-to-end behavior for:
   - `--lang py` default non-test filtering
   - `--lang py --test`
   - `--lang py --no-test-filter`
   - mixed `--lang rs,py`

Python smoke tests should include:

- a production file under `src/`
- a `tests/test_*.py` file
- a mixed file containing both production code and `def test_*`
- optionally `conftest.py` to prove whole-file test classification

**Risks**

- The current loader helpers in `main.rs` are named and shaped around Rust sources. Moving them into shared test-filter orchestration will require careful renaming so behavior does not regress.
- Path normalization rules must stay consistent across languages, especially for whole-file test matching.
- Python region detection should stay intentionally narrow; a too-clever first version is more likely to misclassify production code.

**Outcome**

After this refactor, adding a new language should mainly mean:

1. register a new language kind
2. implement one language module
3. add backend tests and one or two CLI regressions

That is the right level of structure for the repository at its current size.
