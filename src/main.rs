use std::collections::HashMap;
use std::env;

use git_diff_stat::change::{FileChange, collect_changes};
use git_diff_stat::cli::Cli;
use git_diff_stat::filter::{
    collect_rust_whole_test_paths, split_file_patch_for_rust_tests, split_untracked_rust_source,
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
    let whole_test_paths = build_whole_test_paths(git, endpoints.as_ref())?;
    let mut stats = Vec::new();

    for change in changes {
        if !change.old_path.ends_with(".rs") && !change.new_path.ends_with(".rs") {
            continue;
        }

        if change.added + change.deleted == 0 {
            continue;
        }

        let old_is_whole_test = whole_test_paths.old.contains(&change.old_path);
        let new_is_whole_test = whole_test_paths.new.contains(&change.new_path);
        let (added, deleted) = if old_is_whole_test || new_is_whole_test {
            if test_only {
                (
                    if new_is_whole_test { change.added } else { 0 },
                    if old_is_whole_test { change.deleted } else { 0 },
                )
            } else {
                (
                    if new_is_whole_test { 0 } else { change.added },
                    if old_is_whole_test { 0 } else { change.deleted },
                )
            }
        } else if change.untracked {
            let source = git.read_worktree_file(&change.new_path)?;
            let split = split_untracked_rust_source(&source)?;
            if test_only {
                (split.test_added, split.test_deleted)
            } else {
                (split.non_test_added, split.non_test_deleted)
            }
        } else {
            let file_patch = patch_map
                .get(&change.new_path)
                .ok_or_else(|| format!("missing patch data for {}", change.path))?;
            let old_source = match &endpoints {
                Some(endpoints) => git
                    .show_file_at_revision(&endpoints.old, &change.old_path)
                    .unwrap_or_default(),
                None => git.show_index_file(&change.old_path).unwrap_or_default(),
            };
            let new_source = match &endpoints {
                Some(endpoints) => git
                    .show_file_at_revision(&endpoints.new, &change.new_path)
                    .unwrap_or_default(),
                None => git.read_worktree_file(&change.new_path).unwrap_or_default(),
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

struct WholeTestPaths {
    old: std::collections::HashSet<String>,
    new: std::collections::HashSet<String>,
}

fn build_whole_test_paths(
    git: &Git,
    endpoints: Option<&git_diff_stat::revision::RevisionEndpoints>,
) -> Result<WholeTestPaths, String> {
    let (old_sources, new_sources) = match endpoints {
        Some(endpoints) => (
            load_revision_rust_sources(git, &endpoints.old)?,
            load_revision_rust_sources(git, &endpoints.new)?,
        ),
        None => (
            load_index_rust_sources(git)?,
            load_worktree_rust_sources(git)?,
        ),
    };
    Ok(WholeTestPaths {
        old: collect_rust_whole_test_paths(&old_sources)?,
        new: collect_rust_whole_test_paths(&new_sources)?,
    })
}

fn load_index_rust_sources(git: &Git) -> Result<Vec<(String, String)>, String> {
    load_rust_sources(git.tracked_files()?, |path| git.show_index_file(path))
}

fn load_worktree_rust_sources(git: &Git) -> Result<Vec<(String, String)>, String> {
    let mut paths = git.tracked_files()?;
    paths.retain(|path| git.worktree_file_exists(path));
    paths.extend(git.untracked_files()?);
    load_rust_sources(paths, |path| git.read_worktree_file(path))
}

fn load_revision_rust_sources(git: &Git, revision: &str) -> Result<Vec<(String, String)>, String> {
    load_rust_sources(git.revision_files(revision)?, |path| {
        git.show_file_at_revision(revision, path)
    })
}

fn load_rust_sources<F>(
    paths: Vec<String>,
    mut read_source: F,
) -> Result<Vec<(String, String)>, String>
where
    F: FnMut(&str) -> Result<String, String>,
{
    let mut sources = Vec::new();

    for path in paths {
        if !path.ends_with(".rs") {
            continue;
        }

        sources.push((path.clone(), read_source(&path)?));
    }

    Ok(sources)
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
