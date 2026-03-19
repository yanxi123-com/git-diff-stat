use git_diff_stat::change::FileChange;
use git_diff_stat::lang::filter_by_langs;

#[test]
fn filters_to_requested_extensions() {
    let changes = vec![
        FileChange {
            path: "src/lib.rs".to_string(),
            old_path: "src/lib.rs".to_string(),
            new_path: "src/lib.rs".to_string(),
            added: 3,
            deleted: 1,
            untracked: false,
        },
        FileChange {
            path: "web/app.ts".to_string(),
            old_path: "web/app.ts".to_string(),
            new_path: "web/app.ts".to_string(),
            added: 4,
            deleted: 0,
            untracked: false,
        },
    ];

    let filtered = filter_by_langs(&changes, &["rs"]).unwrap();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].path, "src/lib.rs");
}
