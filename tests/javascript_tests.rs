use git_diff_stat::lang::javascript::collect_whole_test_paths;

#[test]
fn collects_javascript_and_typescript_test_paths() {
    let sources = vec![
        ("src/app.ts".to_string(), String::new()),
        ("src/__tests__/app.ts".to_string(), String::new()),
        ("web/app.test.tsx".to_string(), String::new()),
        ("web/app.spec.jsx".to_string(), String::new()),
        ("tests/e2e/login.ts".to_string(), String::new()),
        ("cypress/e2e/home.cy.js".to_string(), String::new()),
        ("playwright/auth.spec.ts".to_string(), String::new()),
        ("playwright.config.ts".to_string(), String::new()),
        ("scripts/build.mjs".to_string(), String::new()),
    ];

    let whole_test_paths = collect_whole_test_paths(&sources).unwrap();

    assert!(whole_test_paths.contains("src/__tests__/app.ts"));
    assert!(whole_test_paths.contains("web/app.test.tsx"));
    assert!(whole_test_paths.contains("web/app.spec.jsx"));
    assert!(whole_test_paths.contains("tests/e2e/login.ts"));
    assert!(whole_test_paths.contains("cypress/e2e/home.cy.js"));
    assert!(whole_test_paths.contains("playwright/auth.spec.ts"));
    assert!(!whole_test_paths.contains("src/app.ts"));
    assert!(!whole_test_paths.contains("playwright.config.ts"));
    assert!(!whole_test_paths.contains("scripts/build.mjs"));
}
