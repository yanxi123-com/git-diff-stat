#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayStat {
    pub path: String,
    pub added: usize,
    pub deleted: usize,
}

pub fn render_stats(stats: &[DisplayStat]) -> String {
    let mut lines = Vec::new();
    let mut total_added = 0usize;
    let mut total_deleted = 0usize;

    for stat in stats {
        total_added += stat.added;
        total_deleted += stat.deleted;
        let changed = stat.added + stat.deleted;
        let graph = format!("{}{}", "+".repeat(stat.added), "-".repeat(stat.deleted));
        lines.push(format!("{} | {} {}", stat.path, changed, graph));
    }

    lines.push(format!(
        "{} files changed, {} insertions(+), {} deletions(-)",
        stats.len(),
        total_added,
        total_deleted
    ));

    lines.join("\n")
}
