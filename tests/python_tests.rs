use git_diff_stat::patch::{FilePatch, LineEvent, LineKind};
use git_diff_stat::python_tests::{
    collect_python_whole_test_paths, detect_test_regions, split_file_patch_for_python_tests,
    split_untracked_python_source,
};

const PYTHON_SOURCE: &str = "\
def build_report():
    return 1


def test_basic():
    assert True


class TestApi:
    def test_fetch(self):
        assert True


class Helper:
    def build(self):
        return 1
";

#[test]
fn identifies_python_test_regions() {
    let regions = detect_test_regions(PYTHON_SOURCE).unwrap();

    assert!(!regions.contains_line(1));
    assert!(!regions.contains_line(2));
    assert!(regions.contains_line(5));
    assert!(regions.contains_line(6));
    assert!(regions.contains_line(9));
    assert!(regions.contains_line(10));
    assert!(regions.contains_line(11));
    assert!(!regions.contains_line(14));
}

#[test]
fn collects_pytest_style_whole_test_paths() {
    let sources = vec![
        ("src/app.py".to_string(), String::new()),
        ("tests/test_app.py".to_string(), String::new()),
        ("pkg/test_utils.py".to_string(), String::new()),
        ("pkg/helpers_test.py".to_string(), String::new()),
        ("pkg/conftest.py".to_string(), String::new()),
    ];

    let whole_test_paths = collect_python_whole_test_paths(&sources).unwrap();

    assert!(whole_test_paths.contains("tests/test_app.py"));
    assert!(whole_test_paths.contains("pkg/test_utils.py"));
    assert!(whole_test_paths.contains("pkg/helpers_test.py"));
    assert!(whole_test_paths.contains("pkg/conftest.py"));
    assert!(!whole_test_paths.contains("src/app.py"));
}

#[test]
fn splits_python_patch_lines_into_test_and_non_test_counts() {
    let old_source = "\
def build_report():
    return 1


def test_basic():
    assert True
";
    let new_source = "\
def build_report():
    return 2


def test_basic():
    assert False
";
    let patch = FilePatch {
        path: "src/report.py".to_string(),
        line_events: vec![
            LineEvent {
                kind: LineKind::Deleted,
                old_line: Some(2),
                new_line: None,
            },
            LineEvent {
                kind: LineKind::Added,
                old_line: None,
                new_line: Some(2),
            },
            LineEvent {
                kind: LineKind::Deleted,
                old_line: Some(6),
                new_line: None,
            },
            LineEvent {
                kind: LineKind::Added,
                old_line: None,
                new_line: Some(6),
            },
        ],
    };

    let split = split_file_patch_for_python_tests(&patch, old_source, new_source).unwrap();

    assert_eq!(split.non_test_added, 1);
    assert_eq!(split.non_test_deleted, 1);
    assert_eq!(split.test_added, 1);
    assert_eq!(split.test_deleted, 1);
}

#[test]
fn splits_untracked_python_source_into_test_and_non_test_counts() {
    let split = split_untracked_python_source(PYTHON_SOURCE).unwrap();

    assert_eq!(split.non_test_added, 11);
    assert_eq!(split.test_added, 5);
    assert_eq!(split.test_deleted, 0);
    assert_eq!(split.non_test_deleted, 0);
}
