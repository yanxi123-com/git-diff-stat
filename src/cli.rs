use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "git-diff-stat")]
#[command(about = "Enhanced git diff --stat with untracked and test filtering")]
#[command(
    after_help = "Examples:\n  git diff-stat --commit HEAD\n  git diff-stat HEAD~1..HEAD --lang rs\n  git diff-stat --lang rs --test"
)]
pub struct Cli {
    #[arg(long, conflicts_with = "no_test")]
    pub test: bool,

    #[arg(long, conflicts_with = "test")]
    pub no_test: bool,

    #[arg(long, value_name = "REV", conflicts_with = "revisions")]
    pub commit: Option<String>,

    #[arg(long, value_name = "LANGS")]
    pub lang: Option<String>,

    #[arg(value_name = "REVISION")]
    pub revisions: Vec<String>,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
