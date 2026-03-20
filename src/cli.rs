use clap::Parser;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TestFilterMode {
    TestOnly,
    NonTestOnly,
    All,
}

#[derive(Debug, Parser)]
#[command(name = "git-diff-stat")]
#[command(about = "Enhanced git diff --stat with untracked and test filtering")]
#[command(
    after_help = "Examples:\n  git diff-stat\n  git diff-stat --commit HEAD\n  git diff-stat --last\n  git diff-stat --last --no-test-filter\n  git diff-stat HEAD~1..HEAD --lang py --no-test-filter\n  git diff-stat --lang py --test\n  git diff-stat --test\n\nDefaults:\n  --lang rs,py\n  test filter: --no-test"
)]
pub struct Cli {
    #[arg(long, conflicts_with_all = ["no_test", "no_test_filter"])]
    pub test: bool,

    #[arg(long, conflicts_with_all = ["test", "no_test_filter"])]
    pub no_test: bool,

    #[arg(long, conflicts_with_all = ["test", "no_test"])]
    pub no_test_filter: bool,

    #[arg(long, value_name = "REV", conflicts_with_all = ["last", "revisions"])]
    pub commit: Option<String>,

    #[arg(long, conflicts_with_all = ["commit", "revisions"])]
    pub last: bool,

    #[arg(long, value_name = "LANGS", default_value = "rs,py")]
    pub lang: Option<String>,

    #[arg(value_name = "REVISION")]
    pub revisions: Vec<String>,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn test_filter_mode(&self) -> TestFilterMode {
        if self.test {
            TestFilterMode::TestOnly
        } else if self.no_test_filter {
            TestFilterMode::All
        } else {
            TestFilterMode::NonTestOnly
        }
    }
}
