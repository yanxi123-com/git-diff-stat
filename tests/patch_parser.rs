use git_diff_stat::patch::{LineKind, parse_patch};

const EXAMPLE_PATCH: &str = "\
diff --git a/src/lib.rs b/src/lib.rs
index 1111111..2222222 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,2 +1,3 @@
 fn alpha() {}
-fn beta() {}
+fn beta_renamed() {}
+fn gamma() {}
";

#[test]
fn maps_added_and_deleted_lines_to_file_positions() {
    let patch = parse_patch(EXAMPLE_PATCH).unwrap();
    let file = &patch.files[0];

    assert_eq!(file.line_events.len(), 3);
    assert_eq!(file.line_events[0].kind, LineKind::Deleted);
    assert_eq!(file.line_events[0].old_line, Some(2));
    assert_eq!(file.line_events[0].new_line, None);
    assert_eq!(file.line_events[1].kind, LineKind::Added);
    assert_eq!(file.line_events[1].old_line, None);
    assert_eq!(file.line_events[1].new_line, Some(2));
    assert_eq!(file.line_events[2].kind, LineKind::Added);
    assert_eq!(file.line_events[2].new_line, Some(3));
}
