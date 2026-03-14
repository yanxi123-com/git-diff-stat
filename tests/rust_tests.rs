use git_diff_stat::rust_tests::detect_test_regions;

const RUST_SOURCE: &str = "\
fn production() {}

#[cfg(test)]
mod tests {
    #[test]
    fn basic() {
        assert_eq!(2 + 2, 4);
    }
}

#[tokio::test]
async fn async_test() {
    assert!(true);
}
";

#[test]
fn identifies_cfg_test_modules_and_test_functions() {
    let regions = detect_test_regions(RUST_SOURCE).unwrap();

    assert!(regions.contains_line(3));
    assert!(regions.contains_line(4));
    assert!(regions.contains_line(6));
    assert!(regions.contains_line(11));
    assert!(regions.contains_line(12));
    assert!(!regions.contains_line(1));
    assert!(!regions.contains_line(10));
}
