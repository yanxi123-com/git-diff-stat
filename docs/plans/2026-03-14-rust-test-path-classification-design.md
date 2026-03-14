# Rust Test Path Classification Design

## Goal

Make `--test` and `--no-test` treat Rust integration test files under `tests/` as test code in their entirety, so path-based integration tests match user expectations.

## User Experience

- `git diff-stat --lang rs --test` includes the full diff stat for Rust files located under any `tests/` path segment.
- `git diff-stat --lang rs --no-test` excludes those same Rust integration test files entirely.
- Rust source files outside `tests/` continue to use AST-based partial splitting for `#[cfg(test)]` modules and test-annotated functions.
- Non-Rust files remain unchanged; the feature still applies only to `.rs` files.

## Recommended Approach

Add a path-level classification rule ahead of the existing AST-based Rust test-region splitter:

- if a Rust file has any path segment exactly equal to `tests`, classify the whole file as test
- otherwise, keep the current AST-based split behavior

This keeps the semantics explicit and predictable:

- integration tests are recognized by Cargo-style directory convention
- inline unit tests in production files still split correctly
- no broad substring matching is introduced

## Alternatives Considered

### Keep the current AST-only behavior

Rejected because it produces surprising `--no-test` output for integration tests, where helper functions and non-annotated lines inside `tests/*.rs` appear as non-test code.

### Match any path containing the substring `test`

Rejected because it would misclassify unrelated names such as `testdata`, `contest`, or arbitrary filenames with `test` in them.

### Treat every file in `tests/` as test regardless of language

Rejected for now because `--test` and `--no-test` are currently Rust-only features, and expanding the semantics to other languages would be a larger product change.

## Testing

- add a regression test showing a Rust file under `tests/` is counted fully by `--test`
- add a regression test showing the same file is omitted by `--no-test`
- keep existing mixed-file tests proving AST-based partial splitting still works for `src/*.rs`
