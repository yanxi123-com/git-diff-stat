use git_diff_stat::change::FileChange;
use git_diff_stat::lang::{filter_by_langs, parse_langs};

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
fn omitted_lang_defaults_to_all_supported_languages() {
    assert_eq!(
        parse_langs(None),
        vec!["rs", "py", "js", "ts", "jsx", "tsx", "cjs", "mjs"]
    );
}

#[test]
fn filters_js_ts_family_extensions_individually() {
    let changes = vec![
        FileChange {
            path: "web/app.js".to_string(),
            old_path: "web/app.js".to_string(),
            new_path: "web/app.js".to_string(),
            added: 1,
            deleted: 0,
            untracked: false,
        },
        FileChange {
            path: "web/app.ts".to_string(),
            old_path: "web/app.ts".to_string(),
            new_path: "web/app.ts".to_string(),
            added: 1,
            deleted: 0,
            untracked: false,
        },
        FileChange {
            path: "web/component.jsx".to_string(),
            old_path: "web/component.jsx".to_string(),
            new_path: "web/component.jsx".to_string(),
            added: 1,
            deleted: 0,
            untracked: false,
        },
        FileChange {
            path: "web/component.tsx".to_string(),
            old_path: "web/component.tsx".to_string(),
            new_path: "web/component.tsx".to_string(),
            added: 1,
            deleted: 0,
            untracked: false,
        },
        FileChange {
            path: "web/config.cjs".to_string(),
            old_path: "web/config.cjs".to_string(),
            new_path: "web/config.cjs".to_string(),
            added: 1,
            deleted: 0,
            untracked: false,
        },
        FileChange {
            path: "web/entry.mjs".to_string(),
            old_path: "web/entry.mjs".to_string(),
            new_path: "web/entry.mjs".to_string(),
            added: 1,
            deleted: 0,
            untracked: false,
        },
    ];

    let jsx = filter_by_langs(&changes, &["jsx"]).unwrap();
    let tsx = filter_by_langs(&changes, &["tsx"]).unwrap();
    let cjs = filter_by_langs(&changes, &["cjs"]).unwrap();
    let mjs = filter_by_langs(&changes, &["mjs"]).unwrap();

    assert_eq!(jsx.len(), 1);
    assert_eq!(jsx[0].path, "web/component.jsx");
    assert_eq!(tsx.len(), 1);
    assert_eq!(tsx[0].path, "web/component.tsx");
    assert_eq!(cjs.len(), 1);
    assert_eq!(cjs[0].path, "web/config.cjs");
    assert_eq!(mjs.len(), 1);
    assert_eq!(mjs[0].path, "web/entry.mjs");
}
