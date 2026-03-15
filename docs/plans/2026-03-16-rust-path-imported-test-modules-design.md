# Rust Path-Imported Test Modules Design

## Goal

Make `--test` and `--no-test` treat Rust files imported by `#[cfg(test)]` module declarations as test code in their entirety, so extracted `tests.rs` modules are classified the same way as their former inline test blocks.

## User Experience

- `git diff-stat --lang rs --test` includes diffs from files such as `src/foo/tests.rs` when they are imported by `#[cfg(test)] mod tests;` or `#[cfg(test)] #[path = ".../tests.rs"] mod tests;`.
- `git diff-stat --lang rs --no-test` excludes those same files entirely.
- Rust files that are not whole-file test modules continue to use AST-based partial splitting for `#[cfg(test)]` blocks and test-annotated functions.
- Existing `tests/*.rs` path classification remains unchanged.

## Recommended Approach

Add a second whole-file test classifier alongside the existing `tests/` path rule:

- keep classifying Rust files under any `tests/` path segment as whole-file test code
- additionally classify a Rust file as whole-file test code when some parent Rust source imports it through a `#[cfg(test)]` module declaration

For the second rule, scan tracked and untracked Rust sources in the current diff scope for declarations equivalent to:

- `#[cfg(test)] mod tests;`
- `#[cfg(test)] #[path = "foo/tests.rs"] mod tests;`

Resolve the declared module file path using Rust's usual module-file conventions for the simple `mod tests;` form, and the explicit relative path for the `#[path = "..."]` form.

## Alternatives Considered

### Treat every `tests.rs` file as test code

Rejected because it would misclassify unrelated files that happen to use that name outside any `#[cfg(test)]` module wiring.

### Expand AST line splitting to infer helper-only test files

Rejected because the file itself does not contain enough information. External test module files often contain helper types and imports outside any `#[test]` function, so line-level inference remains incomplete without module context.

### Require exact module name `tests`

Partially accepted. The first implementation only needs to support `mod tests;` because that matches the observed refactor and common Rust style. The more general principle is still “imported by `#[cfg(test)]`”, so the resolver should be structured to allow later expansion if needed.

## Testing

- add unit tests for detecting `#[cfg(test)]` module imports with and without `#[path = "..."]`
- add CLI regression tests proving `--test` includes and `--no-test` excludes a path-imported `tests.rs` file
- run targeted Rust test-filter tests plus the full test suite
