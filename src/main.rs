use std::env;

use git_diff_stat::change::collect_changes;
use git_diff_stat::cli::{Cli, TestFilterMode};
use git_diff_stat::git::Git;
use git_diff_stat::lang::{filter_by_langs, parse_langs};
use git_diff_stat::render::{DisplayStat, StatsDescription, render_stats};
use git_diff_stat::revision::RevisionSelection;
use git_diff_stat::test_filter::build_test_filtered_stats;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse_args();
    let git = Git::new(env::current_dir().map_err(|error| format!("failed to read cwd: {error}"))?);
    let selection = RevisionSelection::from_cli(&cli)?;
    let mut changes = collect_changes(&git, &selection)?;
    let langs = parse_langs(cli.lang.as_deref());

    if !langs.is_empty() {
        changes = filter_by_langs(&changes, &langs)?;
    }

    let stats = match cli.test_filter_mode() {
        TestFilterMode::TestOnly | TestFilterMode::NonTestOnly => {
            build_test_filtered_stats(&git, &selection, &changes, &langs, cli.test_filter_mode())?
        }
        TestFilterMode::All => changes
            .into_iter()
            .map(|change| DisplayStat {
                path: change.path,
                added: change.added,
                deleted: change.deleted,
            })
            .collect(),
    };

    let description = StatsDescription {
        comparison_scope: selection.describe_scope(&git, cli.last)?,
        language_scope: describe_language_scope(&langs),
        test_scope: describe_test_scope(cli.test_filter_mode()),
    };

    println!("{}", render_stats(&description, &stats));
    Ok(())
}

fn describe_language_scope(langs: &[&str]) -> String {
    if langs.is_empty() {
        "所有文件".to_string()
    } else {
        format!("{} 文件", langs.join(","))
    }
}

fn describe_test_scope(mode: TestFilterMode) -> String {
    match mode {
        TestFilterMode::TestOnly => "测试代码".to_string(),
        TestFilterMode::NonTestOnly => "非测试代码".to_string(),
        TestFilterMode::All => "测试与非测试代码".to_string(),
    }
}
