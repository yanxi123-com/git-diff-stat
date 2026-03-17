use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use git_diff_stat::audit::{AuditConfig, OutputFormat, render_report, scan_paths};

#[derive(Debug, Parser)]
#[command(name = "rust-test-audit")]
#[command(about = "Audit Rust source trees for oversized inline test blocks")]
struct Cli {
    #[arg(long, default_value = ".")]
    root: PathBuf,

    #[arg(long = "path", value_name = "PATH")]
    paths: Vec<PathBuf>,

    #[arg(long, value_enum, default_value_t = CliFormat::Table)]
    format: CliFormat,

    #[arg(long, default_value_t = 120)]
    min_total_lines: usize,

    #[arg(long, default_value_t = 120)]
    consider_test_lines: usize,

    #[arg(long, default_value_t = 0.30)]
    consider_ratio: f64,

    #[arg(long, default_value_t = 200)]
    extract_test_lines: usize,

    #[arg(long, default_value_t = 0.45)]
    extract_ratio: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum CliFormat {
    Json,
    Markdown,
    Table,
}

impl Default for CliFormat {
    fn default() -> Self {
        Self::Table
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let paths = if cli.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        cli.paths
    };
    let config = AuditConfig {
        min_total_lines: cli.min_total_lines,
        consider_test_lines: cli.consider_test_lines,
        consider_ratio: cli.consider_ratio,
        extract_test_lines: cli.extract_test_lines,
        extract_ratio: cli.extract_ratio,
    };
    let report = scan_paths(&cli.root, &paths, &config)?;
    let output = render_report(
        &report,
        match cli.format {
            CliFormat::Json => OutputFormat::Json,
            CliFormat::Markdown => OutputFormat::Markdown,
            CliFormat::Table => OutputFormat::Table,
        },
    )?;
    println!("{output}");
    Ok(())
}
