# JS/TS Family Language Support Design

**Context**

`git-diff-stat` currently supports Rust and Python as first-class test-aware languages. The language layer in [`src/lang/mod.rs`](../../src/lang/mod.rs) still has two structural limits:

- the default `--lang` behavior is represented as a hard-coded CLI string instead of "all supported languages"
- JS and TS are only partially recognized as file extensions, and they do not participate in `--test` or `--no-test`

The next goal is broader frontend language coverage:

- support `js`, `ts`, `jsx`, `tsx`, `cjs`, and `mjs`
- treat unit tests and e2e tests as test code
- change default `--lang` semantics from a fixed subset to "all supported languages"

The user explicitly approved a narrow first version for JS/TS test semantics:

- whole-file test classification only
- no file-internal `describe` / `it` / `test` region splitting

**Goal**

Add first-class JS/TS family support with test-aware filtering, while making default language selection come from the language registry rather than from a duplicated string literal in the CLI layer.

The result should make future language additions easier, not harder.

**Approaches**

1. Minimal patching
   - Add more extensions directly in [`src/lang/mod.rs`](../../src/lang/mod.rs)
   - Change the CLI default string from `rs,py` to a longer comma-separated list
   - Add ad hoc JS/TS branches in [`src/test_filter.rs`](../../src/test_filter.rs)
   - Rejected because the default language list would still be duplicated across language detection, CLI help, README, and tests.

2. Registry-driven defaults with lightweight JS/TS backend support
   - Introduce a single source of truth for supported languages
   - Derive default `--lang` behavior from that registry
   - Add a JS/TS backend that performs whole-file test classification only
   - Recommended.

3. Full backend capability framework
   - Build a more generic trait/capability model for path matching, whole-file classification, region splitting, aliasing, and default inclusion
   - Technically clean, but over-designed for the current repository size
   - Rejected for now.

**Decision**

Use registry-driven defaults plus a lightweight JS/TS backend.

This keeps the current Rust/Python design direction, but tightens two pieces that are now becoming important:

- "supported languages" must live in one place
- test-aware orchestration must support languages that only provide whole-file test classification

**Default Language Semantics**

`--lang` should no longer default to a hard-coded subset such as `rs,py`.

Instead:

- if the user passes `--lang`, respect exactly that explicit set
- if the user omits `--lang`, treat it as "all supported languages"

For the current repository state after this change, "all supported languages" means:

- `rs`
- `py`
- `js`
- `ts`
- `jsx`
- `tsx`
- `cjs`
- `mjs`

This should be surfaced consistently in:

- CLI parsing
- output headers
- help text examples
- README defaults
- tests

The critical rule is that the support list should be declared once in the language layer and reused everywhere else.

**Proposed Structure**

- `src/lang/mod.rs`
  - registry of supported language tokens
  - parsing for explicit `--lang` values
  - default-language expansion when `--lang` is omitted
  - path-to-language detection
- `src/lang/rust.rs`
  - existing Rust support
  - whole-file test classification plus region splitting
- `src/lang/python.rs`
  - existing Python support
  - whole-file test classification plus region splitting
- `src/lang/javascript.rs`
  - JS/TS family path matching
  - whole-file test classification only
- `src/test_filter.rs`
  - shared orchestration across selected languages
  - support for backends that only classify whole-file test paths

This is still a moderate refactor, not a rewrite.

**Language Registry Shape**

The registry only needs to answer a few central questions:

- which language tokens are supported?
- which token matches a given path?
- what is the default language set when `--lang` is omitted?

One practical model is:

- `supported_langs() -> &'static [&'static str]`
- `default_langs() -> &'static [&'static str]`
- `parse_langs(value: Option<&str>) -> Vec<&str>`
- `detect_language(path: &str) -> Option<&'static str>`

For now, `default_langs()` can simply return the same list as `supported_langs()`.

This avoids duplicating the support list in [`src/cli.rs`](../../src/cli.rs) and [`README.md`](../../README.md).

**JS/TS Family Matching**

The new frontend backend should recognize these extensions directly:

- `.js`
- `.ts`
- `.jsx`
- `.tsx`
- `.cjs`
- `.mjs`

Each extension should map to its own `--lang` token. This keeps filtering precise:

- `--lang js` should not automatically include `ts`
- `--lang tsx` should only include `.tsx`
- omitting `--lang` includes all of them

This is a better fit for current CLI semantics than collapsing everything into a single `web` alias.

**JS/TS Test Semantics**

The approved first version is whole-file classification only.

Treat these as test files:

- any file under a `__tests__/` path component
- filenames matching `*.test.<ext>`
- filenames matching `*.spec.<ext>`
- any file under an `e2e/` path component
- any file under a `cypress/` path component
- any file under a `playwright/` path component
- filenames matching `*.cy.<ext>`

Where `<ext>` is one of:

- `js`
- `ts`
- `jsx`
- `tsx`
- `cjs`
- `mjs`

These rules intentionally cover both unit and e2e test conventions.

**Out of Scope**

Not in scope for the first JS/TS version:

- file-internal test block detection using `describe`, `it`, `test`, or `suite`
- `vitest` inline test detection such as `import.meta.vitest`
- framework-specific config discovery from Jest, Vitest, Playwright, Cypress, or custom tooling
- special handling for snapshot files

This is intentional. Most real-world JS/TS repositories still place tests in dedicated files or directories, so whole-file classification captures the highest-value cases with low false-positive risk.

**Test-Filter Orchestration**

The shared builder in [`src/test_filter.rs`](../../src/test_filter.rs) currently assumes that selected languages either:

- have whole-file test paths and region splitting, or
- are ignored entirely

JS/TS adds a third useful case:

- whole-file test classification only

The orchestration should therefore support:

1. languages with whole-file and region split support
2. languages with whole-file-only support

For JS/TS family files:

- if a file matches a whole-file test rule, count it as test code
- otherwise, count the full file diff as non-test code

That preserves correct semantics for `--test`, `--no-test`, and `--no-test-filter` without introducing AST parsing.

**Source Loading Strategy**

This addition makes source-loading efficiency more important.

Rust still needs source contents for path-imported `#[cfg(test)]` module detection. Python and JS/TS whole-file classification are path-driven. The design should avoid eager content reads for languages that only need paths.

That means the shared builder should distinguish between:

- path-only whole-file classification
- source-assisted whole-file classification
- region splitting

Even if the implementation stays simple, it should at least avoid bulk-reading JS/TS files just to classify them by filename or directory name.

**Data Flow**

After the refactor, runtime behavior should look like this:

1. Parse CLI.
2. Resolve revision selection.
3. Parse explicit `--lang`, or expand to all supported languages if omitted.
4. Filter `FileChange` values to the selected languages.
5. If `--no-test-filter`, render full-file stats directly.
6. Otherwise:
   - compute whole-file test paths for each selected language backend
   - use region splitting only for languages that implement it
   - treat JS/TS non-test files as full-file non-test diffs
7. Render the existing header with the updated language scope.

The biggest behavioral change is that plain `git diff-stat` now means "all supported languages" instead of "Rust and Python only".

**Testing Strategy**

Add coverage at three levels:

1. Registry and extension tests
   - supported language parsing
   - default language expansion
   - path detection for `js`, `ts`, `jsx`, `tsx`, `cjs`, `mjs`
2. JS/TS test classification unit tests
   - `__tests__/`
   - `*.test.*`
   - `*.spec.*`
   - `e2e/`
   - `cypress/`
   - `playwright/`
   - `*.cy.*`
3. CLI smoke tests
   - default run includes supported frontend files
   - default `--no-test` excludes JS/TS unit and e2e test files
   - `--test` includes those files
   - `--no-test-filter` restores full-file counting
   - explicit `--lang tsx` or `--lang cjs` behaves narrowly

The smoke suite should also prove that mixed repositories still combine Rust, Python, and JS/TS families correctly.

**Risks**

- If default-language logic remains duplicated, future additions will drift again between CLI, README, and actual behavior.
- If JS/TS whole-file rules are too broad, application code under directories like `tests-data/` or `playwright.config.ts` could be misclassified; the patterns should stay intentionally narrow and component-based.
- If the builder eagerly reads all JS/TS sources, repositories with many frontend assets could take an unnecessary performance hit.

**Outcome**

After this change:

1. plain `git diff-stat` covers all supported languages
2. JS/TS family files participate in `--test` and `--no-test`
3. test-aware orchestration no longer assumes every language must support region splitting

That is enough structure to add more path-driven languages later without reworking the CLI defaults again.
