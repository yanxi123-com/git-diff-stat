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

#[test]
fn filters_to_python_extension() {
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
            path: "app/main.py".to_string(),
            old_path: "app/main.py".to_string(),
            new_path: "app/main.py".to_string(),
            added: 5,
            deleted: 0,
            untracked: false,
        },
    ];

    let filtered = filter_by_langs(&changes, &["py"]).unwrap();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].path, "app/main.py");
}

#[test]
fn keeps_cross_language_renames_for_either_selected_language() {
    let changes = vec![FileChange {
        path: "tests/test_mod.py => src/lib.rs".to_string(),
        old_path: "tests/test_mod.py".to_string(),
        new_path: "src/lib.rs".to_string(),
        added: 2,
        deleted: 2,
        untracked: false,
    }];

    let python = filter_by_langs(&changes, &["py"]).unwrap();
    let rust = filter_by_langs(&changes, &["rs"]).unwrap();
    let javascript = filter_by_langs(&changes, &["js"]).unwrap();

    assert_eq!(python.len(), 1);
    assert_eq!(rust.len(), 1);
    assert!(javascript.is_empty());
}
