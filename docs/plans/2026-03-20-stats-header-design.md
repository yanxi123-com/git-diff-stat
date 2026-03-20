# Stats Header Design

**Context**

`git-diff-stat` currently prints only file rows and the summary row. After the recent default change to `--lang rs` and default non-test filtering, users can no longer infer the active comparison/filter context from the output alone.

**Goal**

Always print one descriptive line above the stats that states:

- what is being compared
- which languages are included
- whether the output is test-only, non-test-only, or unfiltered

**Approaches**

1. Build the final sentence directly in `main`.
   - Smallest patch, but mixes CLI interpretation with rendering and makes future output changes harder.

2. Introduce a small structured render context and let `render` own the final sentence.
   - Slightly more code, but keeps wording rules centralized and testable.

3. Infer the sentence from filtered stats.
   - Rejected. The rendered rows do not preserve enough information to reliably reconstruct user intent.

**Decision**

Use a structured render context.

`main` already has the CLI, revision selection, parsed languages, and test-filter mode. It should convert those inputs into a small description object and pass that object to the renderer together with the stats. The renderer should then prepend one Chinese sentence before the existing file rows and summary row.

**Description Rules**

- Comparison scope:
  - working tree: `未提交的`
  - `--last`: `最后一次提交的`
  - `--commit <rev>`: `<rev> 这个提交的`
  - revision ranges / positional revisions: `<old> 到 <new> 的`
- Language scope:
  - `rs`: `rs 文件`
  - `rs,py`: `rs,py 文件`
- Test scope:
  - default / `--no-test`: `非测试代码`
  - `--test`: `测试代码`
  - `--no-test-filter`: `测试与非测试代码`

**Example Output**

- `未提交的 rs 文件中，非测试代码统计如下：`
- `最后一次提交的 rs 文件中，测试与非测试代码统计如下：`
- `HEAD~1 到 HEAD 的 rs,py 文件中，测试代码统计如下：`

**Testing**

- add CLI smoke coverage for default working tree output
- add CLI smoke coverage for `--last --no-test-filter`
- add a focused unit/integration test for explicit revision ranges and multi-language headers
