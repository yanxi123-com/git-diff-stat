use std::cmp::Reverse;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::change::line_count;
use crate::filter::is_rust_integration_test_path;
use crate::rust_tests::{detect_cfg_test_module_imports, detect_test_regions};

#[derive(Debug, Clone, PartialEq)]
pub struct AuditConfig {
    pub min_total_lines: usize,
    pub consider_test_lines: usize,
    pub consider_ratio: f64,
    pub extract_test_lines: usize,
    pub extract_ratio: f64,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            min_total_lines: 120,
            consider_test_lines: 120,
            consider_ratio: 0.30,
            extract_test_lines: 200,
            extract_ratio: 0.45,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditDecision {
    ConsiderExtract,
    ExtractNow,
}

impl AuditDecision {
    fn severity(self) -> usize {
        match self {
            AuditDecision::ConsiderExtract => 1,
            AuditDecision::ExtractNow => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AuditFinding {
    pub path: String,
    pub total_lines: usize,
    pub test_lines: usize,
    pub test_ratio: f64,
    pub has_external_test_module: bool,
    pub decision: AuditDecision,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AuditReport {
    pub root: String,
    pub scanned_paths: Vec<String>,
    pub findings: Vec<AuditFinding>,
}

pub fn scan_paths(
    root: &Path,
    paths: &[PathBuf],
    config: &AuditConfig,
) -> Result<AuditReport, String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("failed to canonicalize root {}: {error}", root.display()))?;
    let scan_paths = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths.to_vec()
    };
    let mut files = Vec::new();

    for path in &scan_paths {
        let absolute = if path.is_absolute() {
            path.clone()
        } else {
            root.join(path)
        };
        collect_rs_files(&absolute, &mut files)?;
    }

    files.sort();
    files.dedup();

    let mut findings = files
        .into_iter()
        .filter_map(|file| audit_file(&root, &file, config).transpose())
        .collect::<Result<Vec<_>, _>>()?;

    findings.sort_by_key(|finding| {
        (
            Reverse(finding.decision.severity()),
            Reverse(finding.test_lines),
            finding.path.clone(),
        )
    });

    Ok(AuditReport {
        root: normalize_path(&root),
        scanned_paths: scan_paths.iter().map(|path| normalize_path(path)).collect(),
        findings,
    })
}

pub fn render_report(report: &AuditReport, format: OutputFormat) -> Result<String, String> {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(report)
            .map_err(|error| format!("failed to render json: {error}")),
        OutputFormat::Markdown => Ok(render_markdown(report)),
        OutputFormat::Table => Ok(render_table(report)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Markdown,
    Table,
}

fn audit_file(
    root: &Path,
    file: &Path,
    config: &AuditConfig,
) -> Result<Option<AuditFinding>, String> {
    let relative = file.strip_prefix(root).map_err(|error| {
        format!(
            "failed to strip root prefix from {}: {error}",
            file.display()
        )
    })?;
    let relative = normalize_path(relative);

    if should_skip_path(&relative) {
        return Ok(None);
    }

    let source = fs::read_to_string(file)
        .map_err(|error| format!("failed to read {}: {error}", file.display()))?;
    let total_lines = line_count(&source);
    if total_lines < config.min_total_lines {
        return Ok(None);
    }

    let regions = detect_test_regions(&source)?;
    let test_lines = (1..=total_lines)
        .filter(|line| regions.contains_line(*line))
        .count();
    if test_lines == 0 {
        return Ok(None);
    }

    let test_ratio = test_lines as f64 / total_lines as f64;
    let has_external_test_module = !detect_cfg_test_module_imports(&source)?.is_empty();
    let decision = classify_finding(total_lines, test_lines, test_ratio, config)?;

    Ok(decision.map(|decision| AuditFinding {
        path: relative,
        total_lines,
        test_lines,
        test_ratio,
        has_external_test_module,
        reason: build_reason(decision, test_lines, test_ratio, config),
        decision,
    }))
}

fn classify_finding(
    total_lines: usize,
    test_lines: usize,
    test_ratio: f64,
    config: &AuditConfig,
) -> Result<Option<AuditDecision>, String> {
    if total_lines == 0 {
        return Ok(None);
    }

    if test_lines >= config.extract_test_lines
        || (test_lines >= config.consider_test_lines && test_ratio >= config.extract_ratio)
    {
        return Ok(Some(AuditDecision::ExtractNow));
    }

    if test_lines >= config.consider_test_lines && test_ratio >= config.consider_ratio {
        return Ok(Some(AuditDecision::ConsiderExtract));
    }

    Ok(None)
}

fn build_reason(
    decision: AuditDecision,
    test_lines: usize,
    test_ratio: f64,
    config: &AuditConfig,
) -> String {
    match decision {
        AuditDecision::ExtractNow if test_lines >= config.extract_test_lines => format!(
            "test lines {} reached extract threshold {}",
            test_lines, config.extract_test_lines
        ),
        AuditDecision::ExtractNow => format!(
            "test ratio {:.1}% reached extract threshold {:.1}% with at least {} test lines",
            test_ratio * 100.0,
            config.extract_ratio * 100.0,
            config.consider_test_lines
        ),
        AuditDecision::ConsiderExtract => format!(
            "test ratio {:.1}% reached consider threshold {:.1}% with {} test lines",
            test_ratio * 100.0,
            config.consider_ratio * 100.0,
            test_lines
        ),
    }
}

fn should_skip_path(path: &str) -> bool {
    path.ends_with("/tests.rs") || path == "tests.rs" || is_rust_integration_test_path(path)
}

fn collect_rs_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_file() {
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }

    let mut entries = fs::read_dir(path)
        .map_err(|error| format!("failed to read directory {}: {error}", path.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to iterate directory {}: {error}", path.display()))?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            collect_rs_files(&entry_path, files)?;
        } else if entry_path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(entry_path);
        }
    }

    Ok(())
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn render_markdown(report: &AuditReport) -> String {
    if report.findings.is_empty() {
        return "| file | total | test | ratio | action | reason |\n| --- | ---: | ---: | ---: | --- | --- |\n".to_string();
    }

    let mut lines = vec![
        "| file | total | test | ratio | action | reason |".to_string(),
        "| --- | ---: | ---: | ---: | --- | --- |".to_string(),
    ];

    for finding in &report.findings {
        lines.push(format!(
            "| {} | {} | {} | {:.1}% | {} | {} |",
            finding.path,
            finding.total_lines,
            finding.test_lines,
            finding.test_ratio * 100.0,
            decision_label(finding.decision),
            finding.reason
        ));
    }

    lines.join("\n")
}

fn render_table(report: &AuditReport) -> String {
    if report.findings.is_empty() {
        return "no findings".to_string();
    }

    let mut lines = vec!["file | total | test | ratio | action | reason".to_string()];
    for finding in &report.findings {
        lines.push(format!(
            "{} | {} | {} | {:.1}% | {} | {}",
            finding.path,
            finding.total_lines,
            finding.test_lines,
            finding.test_ratio * 100.0,
            decision_label(finding.decision),
            finding.reason
        ));
    }
    lines.join("\n")
}

fn decision_label(decision: AuditDecision) -> &'static str {
    match decision {
        AuditDecision::ConsiderExtract => "consider_extract",
        AuditDecision::ExtractNow => "extract_now",
    }
}
