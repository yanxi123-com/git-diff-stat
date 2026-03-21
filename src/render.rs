#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayStat {
    pub path: String,
    pub added: usize,
    pub deleted: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatsDescription {
    pub comparison_scope: String,
    pub language_scope: String,
    pub test_scope: String,
}

pub fn render_stats(description: &StatsDescription, stats: &[DisplayStat]) -> String {
    let mut lines = Vec::new();
    let mut total_added = 0usize;
    let mut total_deleted = 0usize;

    lines.push(format!(
        "{} stats for {} {}:",
        description.test_scope, description.language_scope, description.comparison_scope
    ));

    for stat in stats {
        total_added += stat.added;
        total_deleted += stat.deleted;
        let changed = stat.added + stat.deleted;
        let graph = format!("{}{}", "+".repeat(stat.added), "-".repeat(stat.deleted));
        lines.push(format!("{} | {} {}", stat.path, changed, graph));
    }

    let net_change = total_added as isize - total_deleted as isize;
    let net_change = if net_change > 0 {
        format!("+{net_change}")
    } else {
        net_change.to_string()
    };

    lines.push(format!(
        "{} files changed, {} insertions(+), {} deletions(-), net change: {}",
        stats.len(),
        total_added,
        total_deleted,
        net_change
    ));

    lines.join("\n")
}
