use std::collections::HashMap;
use std::env;

use git_diff_stat::change::{FileChange, collect_changes};
use git_diff_stat::cli::Cli;
use git_diff_stat::filter::{
    is_rust_integration_test_path, split_file_patch_for_rust_tests, split_untracked_rust_source,
};
use git_diff_stat::git::Git;
use git_diff_stat::lang::filter_by_langs;
use git_diff_stat::patch::parse_patch;
use git_diff_stat::render::{DisplayStat, render_stats};
use git_diff_stat::revision::RevisionSelection;

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

    if let Some(langs) = parse_langs(cli.lang.as_deref()) {
        changes = filter_by_langs(&changes, &langs)?;
    }

    let stats = if cli.test || cli.no_test {
        build_rust_test_stats(&git, &selection, &changes, cli.test)?
    } else {
        changes
            .into_iter()
            .map(|change| DisplayStat {
                path: change.path,
                added: change.added,
                deleted: change.deleted,
            })
            .collect()
    };

    println!("{}", render_stats(&stats));
    Ok(())
}

fn build_rust_test_stats(
    git: &Git,
    selection: &RevisionSelection,
    changes: &[FileChange],
    test_only: bool,
) -> Result<Vec<DisplayStat>, String> {
    let patch_output = git.diff_patch(&selection.git_diff_args())?;
    let patch = parse_patch(&patch_output)?;
    let patch_map = patch
        .files
        .into_iter()
        .map(|file| (file.path.clone(), file))
        .collect::<HashMap<_, _>>();
    let endpoints = selection.endpoints(git)?;
    let mut stats = Vec::new();

    for change in changes {
        if !change.path.ends_with(".rs") {
            continue;
        }

        let (added, deleted) = if is_rust_integration_test_path(&change.path) {
            if test_only {
                (change.added, change.deleted)
            } else {
                (0, 0)
            }
        } else if change.untracked {
            let source = git.read_worktree_file(&change.path)?;
            let split = split_untracked_rust_source(&source)?;
            if test_only {
                (split.test_added, split.test_deleted)
            } else {
                (split.non_test_added, split.non_test_deleted)
            }
        } else {
            let file_patch = patch_map
                .get(&change.path)
                .ok_or_else(|| format!("missing patch data for {}", change.path))?;
            let old_source = match &endpoints {
                Some(endpoints) => git
                    .show_file_at_revision(&endpoints.old, &change.path)
                    .unwrap_or_default(),
                None => git.show_index_file(&change.path).unwrap_or_default(),
            };
            let new_source = match &endpoints {
                Some(endpoints) => git
                    .show_file_at_revision(&endpoints.new, &change.path)
                    .unwrap_or_default(),
                None => git.read_worktree_file(&change.path).unwrap_or_default(),
            };
            let split = split_file_patch_for_rust_tests(file_patch, &old_source, &new_source)?;
            if test_only {
                (split.test_added, split.test_deleted)
            } else {
                (split.non_test_added, split.non_test_deleted)
            }
        };

        if added + deleted == 0 {
            continue;
        }

        stats.push(DisplayStat {
            path: change.path.clone(),
            added,
            deleted,
        });
    }

    Ok(stats)
}

fn parse_langs(value: Option<&str>) -> Option<Vec<&str>> {
    value.map(|value| {
        value
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .collect()
    })
}
