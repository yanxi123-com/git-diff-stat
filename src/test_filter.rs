use std::collections::{HashMap, HashSet};

use crate::change::FileChange;
use crate::cli::TestFilterMode;
use crate::git::Git;
use crate::lang::{detect_language, javascript, python, rust};
use crate::patch::parse_patch;
use crate::render::DisplayStat;
use crate::revision::{RevisionEndpoints, RevisionSelection};

pub fn build_test_filtered_stats(
    git: &Git,
    selection: &RevisionSelection,
    changes: &[FileChange],
    langs: &[&str],
    mode: TestFilterMode,
) -> Result<Vec<DisplayStat>, String> {
    let patch_output = git.diff_patch(&selection.git_diff_args())?;
    let patch = parse_patch(&patch_output)?;
    let patch_map = patch
        .files
        .into_iter()
        .map(|file| (file.path.clone(), file))
        .collect::<HashMap<_, _>>();
    let endpoints = selection.endpoints(git)?;
    let whole_test_paths = build_whole_test_paths(git, endpoints.as_ref(), langs)?;
    let context = BuildContext {
        git,
        endpoints: &endpoints,
        patch_map: &patch_map,
        mode,
    };
    let mut stats = Vec::new();

    for change in changes {
        if change.added + change.deleted == 0 {
            continue;
        }

        let Some(language) = change_language(change) else {
            continue;
        };

        let (added, deleted) = match language {
            "rs" => build_counts_for_rust(&context, &whole_test_paths, change)?,
            "py" => build_counts_for_python(&context, &whole_test_paths, change)?,
            "js" | "ts" | "jsx" | "tsx" | "cjs" | "mjs" => {
                build_counts_for_javascript(&context, &whole_test_paths, change)
            }
            _ => continue,
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
    old: HashMap<&'static str, HashSet<String>>,
    new: HashMap<&'static str, HashSet<String>>,
}

struct BuildContext<'a> {
    git: &'a Git,
    endpoints: &'a Option<RevisionEndpoints>,
    patch_map: &'a HashMap<String, crate::patch::FilePatch>,
    mode: TestFilterMode,
}

fn build_whole_test_paths(
    git: &Git,
    endpoints: Option<&RevisionEndpoints>,
    langs: &[&str],
) -> Result<WholeTestPaths, String> {
    let (old_paths, new_paths) = match endpoints {
        Some(endpoints) => (
            load_revision_paths(git, &endpoints.old, langs)?,
            load_revision_paths(git, &endpoints.new, langs)?,
        ),
        None => (
            load_index_paths(git, langs)?,
            load_worktree_paths(git, langs)?,
        ),
    };
    let (old_rust_sources, new_rust_sources) = if langs.contains(&"rs") {
        match endpoints {
            Some(endpoints) => (
                load_revision_sources(git, &endpoints.old, &["rs"])?,
                load_revision_sources(git, &endpoints.new, &["rs"])?,
            ),
            None => (
                load_index_sources(git, &["rs"])?,
                load_worktree_sources(git, &["rs"])?,
            ),
        }
    } else {
        (Vec::new(), Vec::new())
    };
    let old_path_entries = path_entries(&old_paths);
    let new_path_entries = path_entries(&new_paths);

    let mut old = HashMap::new();
    old.insert("rs", rust::collect_whole_test_paths(&old_rust_sources)?);
    old.insert("py", python::collect_whole_test_paths(&old_path_entries)?);
    let old_javascript_paths = javascript::collect_whole_test_paths(&old_path_entries)?;
    for language in javascript::family_langs() {
        old.insert(*language, old_javascript_paths.clone());
    }

    let mut new = HashMap::new();
    new.insert("rs", rust::collect_whole_test_paths(&new_rust_sources)?);
    new.insert("py", python::collect_whole_test_paths(&new_path_entries)?);
    let new_javascript_paths = javascript::collect_whole_test_paths(&new_path_entries)?;
    for language in javascript::family_langs() {
        new.insert(*language, new_javascript_paths.clone());
    }

    Ok(WholeTestPaths { old, new })
}

fn build_counts_for_rust(
    context: &BuildContext<'_>,
    whole_test_paths: &WholeTestPaths,
    change: &FileChange,
) -> Result<(usize, usize), String> {
    build_counts(
        context,
        whole_test_paths.old.get("rs"),
        whole_test_paths.new.get("rs"),
        change,
        rust::split_untracked_source,
        rust::split_file_patch,
    )
}

fn build_counts_for_python(
    context: &BuildContext<'_>,
    whole_test_paths: &WholeTestPaths,
    change: &FileChange,
) -> Result<(usize, usize), String> {
    build_counts(
        context,
        whole_test_paths.old.get("py"),
        whole_test_paths.new.get("py"),
        change,
        python::split_untracked_source,
        python::split_file_patch,
    )
}

fn build_counts_for_javascript(
    context: &BuildContext<'_>,
    whole_test_paths: &WholeTestPaths,
    change: &FileChange,
) -> (usize, usize) {
    let Some(language) = change_language(change) else {
        return (0, 0);
    };

    select_counts_for_whole_file_only(
        change,
        whole_test_paths.old.get(language),
        whole_test_paths.new.get(language),
        context.mode,
    )
}

fn build_counts<Split, UntrackedFn, PatchFn>(
    context: &BuildContext<'_>,
    old_whole_test_paths: Option<&HashSet<String>>,
    new_whole_test_paths: Option<&HashSet<String>>,
    change: &FileChange,
    split_untracked: UntrackedFn,
    split_patch: PatchFn,
) -> Result<(usize, usize), String>
where
    Split: TestSplitCounts,
    UntrackedFn: Fn(&str) -> Result<Split, String>,
    PatchFn: Fn(&crate::patch::FilePatch, &str, &str) -> Result<Split, String>,
{
    let old_is_whole_test = old_whole_test_paths
        .map(|paths| paths.contains(&change.old_path))
        .unwrap_or(false);
    let new_is_whole_test = new_whole_test_paths
        .map(|paths| paths.contains(&change.new_path))
        .unwrap_or(false);

    if old_is_whole_test || new_is_whole_test {
        return Ok(select_counts_from_whole_file(
            change,
            old_is_whole_test,
            new_is_whole_test,
            context.mode,
        ));
    }

    if change.untracked {
        let source = context.git.read_worktree_file(&change.new_path)?;
        let split = split_untracked(&source)?;
        return Ok(select_counts_from_split(&split, context.mode));
    }

    let file_patch = context
        .patch_map
        .get(&change.new_path)
        .ok_or_else(|| format!("missing patch data for {}", change.path))?;
    let old_source = match context.endpoints {
        Some(endpoints) => context
            .git
            .show_file_at_revision(&endpoints.old, &change.old_path)
            .unwrap_or_default(),
        None => context
            .git
            .show_index_file(&change.old_path)
            .unwrap_or_default(),
    };
    let new_source = match context.endpoints {
        Some(endpoints) => context
            .git
            .show_file_at_revision(&endpoints.new, &change.new_path)
            .unwrap_or_default(),
        None => context
            .git
            .read_worktree_file(&change.new_path)
            .unwrap_or_default(),
    };
    let split = split_patch(file_patch, &old_source, &new_source)?;
    Ok(select_counts_from_split(&split, context.mode))
}

fn select_counts_for_whole_file_only(
    change: &FileChange,
    old_whole_test_paths: Option<&HashSet<String>>,
    new_whole_test_paths: Option<&HashSet<String>>,
    mode: TestFilterMode,
) -> (usize, usize) {
    let old_is_whole_test = old_whole_test_paths
        .map(|paths| paths.contains(&change.old_path))
        .unwrap_or(false);
    let new_is_whole_test = new_whole_test_paths
        .map(|paths| paths.contains(&change.new_path))
        .unwrap_or(false);

    select_counts_from_whole_file(change, old_is_whole_test, new_is_whole_test, mode)
}

trait TestSplitCounts {
    fn test_added(&self) -> usize;
    fn test_deleted(&self) -> usize;
    fn non_test_added(&self) -> usize;
    fn non_test_deleted(&self) -> usize;
}

impl TestSplitCounts for crate::filter::RustTestSplit {
    fn test_added(&self) -> usize {
        self.test_added
    }

    fn test_deleted(&self) -> usize {
        self.test_deleted
    }

    fn non_test_added(&self) -> usize {
        self.non_test_added
    }

    fn non_test_deleted(&self) -> usize {
        self.non_test_deleted
    }
}

impl TestSplitCounts for crate::python_tests::PythonTestSplit {
    fn test_added(&self) -> usize {
        self.test_added
    }

    fn test_deleted(&self) -> usize {
        self.test_deleted
    }

    fn non_test_added(&self) -> usize {
        self.non_test_added
    }

    fn non_test_deleted(&self) -> usize {
        self.non_test_deleted
    }
}

fn select_counts_from_whole_file(
    change: &FileChange,
    old_is_whole_test: bool,
    new_is_whole_test: bool,
    mode: TestFilterMode,
) -> (usize, usize) {
    match mode {
        TestFilterMode::TestOnly => (
            if new_is_whole_test { change.added } else { 0 },
            if old_is_whole_test { change.deleted } else { 0 },
        ),
        TestFilterMode::NonTestOnly => (
            if new_is_whole_test { 0 } else { change.added },
            if old_is_whole_test { 0 } else { change.deleted },
        ),
        TestFilterMode::All => (change.added, change.deleted),
    }
}

fn select_counts_from_split(split: &impl TestSplitCounts, mode: TestFilterMode) -> (usize, usize) {
    match mode {
        TestFilterMode::TestOnly => (split.test_added(), split.test_deleted()),
        TestFilterMode::NonTestOnly => (split.non_test_added(), split.non_test_deleted()),
        TestFilterMode::All => (
            split.test_added() + split.non_test_added(),
            split.test_deleted() + split.non_test_deleted(),
        ),
    }
}

fn change_language(change: &FileChange) -> Option<&'static str> {
    detect_language(&change.new_path).or_else(|| detect_language(&change.old_path))
}

fn load_index_sources(git: &Git, langs: &[&str]) -> Result<Vec<(String, String)>, String> {
    load_sources(git.tracked_files()?, langs, |path| {
        git.show_index_file(path)
    })
}

fn load_index_paths(git: &Git, langs: &[&str]) -> Result<Vec<String>, String> {
    Ok(filter_paths(git.tracked_files()?, langs))
}

fn load_worktree_sources(git: &Git, langs: &[&str]) -> Result<Vec<(String, String)>, String> {
    let mut paths = git.tracked_files()?;
    paths.retain(|path| git.worktree_file_exists(path));
    paths.extend(git.untracked_files()?);
    load_sources(paths, langs, |path| git.read_worktree_file(path))
}

fn load_worktree_paths(git: &Git, langs: &[&str]) -> Result<Vec<String>, String> {
    let mut paths = git.tracked_files()?;
    paths.retain(|path| git.worktree_file_exists(path));
    paths.extend(git.untracked_files()?);
    Ok(filter_paths(paths, langs))
}

fn load_revision_sources(
    git: &Git,
    revision: &str,
    langs: &[&str],
) -> Result<Vec<(String, String)>, String> {
    load_sources(git.revision_files(revision)?, langs, |path| {
        git.show_file_at_revision(revision, path)
    })
}

fn load_revision_paths(git: &Git, revision: &str, langs: &[&str]) -> Result<Vec<String>, String> {
    Ok(filter_paths(git.revision_files(revision)?, langs))
}

fn load_sources<F>(
    paths: Vec<String>,
    langs: &[&str],
    mut read_source: F,
) -> Result<Vec<(String, String)>, String>
where
    F: FnMut(&str) -> Result<String, String>,
{
    let mut sources = Vec::new();

    for path in filter_paths(paths, langs) {
        sources.push((path.clone(), read_source(&path)?));
    }

    Ok(sources)
}

fn filter_paths(paths: Vec<String>, langs: &[&str]) -> Vec<String> {
    paths.into_iter()
        .filter(|path| should_include_path(path, langs))
        .collect()
}

fn should_include_path(path: &str, langs: &[&str]) -> bool {
    let Some(language) = detect_language(path) else {
        return false;
    };

    langs.is_empty() || langs.contains(&language)
}

fn path_entries(paths: &[String]) -> Vec<(String, String)> {
    paths.iter()
        .cloned()
        .map(|path| (path, String::new()))
        .collect()
}
