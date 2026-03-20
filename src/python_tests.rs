use std::collections::HashSet;
use std::path::Path;

use tree_sitter::{Node, Parser};

use crate::change::line_count;
use crate::patch::{FilePatch, LineKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestRegions {
    regions: Vec<LineRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonTestSplit {
    pub test_added: usize,
    pub test_deleted: usize,
    pub non_test_added: usize,
    pub non_test_deleted: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LineRange {
    start: usize,
    end: usize,
}

impl TestRegions {
    pub fn contains_line(&self, line: usize) -> bool {
        self.regions
            .iter()
            .any(|region| region.start <= line && line <= region.end)
    }
}

pub fn collect_python_whole_test_paths(
    sources: &[(String, String)],
) -> Result<HashSet<String>, String> {
    Ok(sources
        .iter()
        .map(|(path, _)| path)
        .filter(|path| is_python_whole_test_path(path))
        .cloned()
        .collect())
}

pub fn detect_test_regions(source: &str) -> Result<TestRegions, String> {
    let tree = parse_python_source(source)?;
    let mut regions = Vec::new();
    collect_regions(tree.root_node(), source.as_bytes(), &mut regions)?;

    Ok(TestRegions { regions })
}

pub fn split_file_patch_for_python_tests(
    patch: &FilePatch,
    old_source: &str,
    new_source: &str,
) -> Result<PythonTestSplit, String> {
    let old_regions = detect_test_regions(old_source)?;
    let new_regions = detect_test_regions(new_source)?;
    let mut split = PythonTestSplit {
        test_added: 0,
        test_deleted: 0,
        non_test_added: 0,
        non_test_deleted: 0,
    };

    for event in &patch.line_events {
        match event.kind {
            LineKind::Added => {
                let line = event
                    .new_line
                    .ok_or_else(|| "added line event missing new line".to_string())?;
                if new_regions.contains_line(line) {
                    split.test_added += 1;
                } else {
                    split.non_test_added += 1;
                }
            }
            LineKind::Deleted => {
                let line = event
                    .old_line
                    .ok_or_else(|| "deleted line event missing old line".to_string())?;
                if old_regions.contains_line(line) {
                    split.test_deleted += 1;
                } else {
                    split.non_test_deleted += 1;
                }
            }
        }
    }

    Ok(split)
}

pub fn split_untracked_python_source(source: &str) -> Result<PythonTestSplit, String> {
    let regions = detect_test_regions(source)?;
    let mut split = PythonTestSplit {
        test_added: 0,
        test_deleted: 0,
        non_test_added: 0,
        non_test_deleted: 0,
    };

    for line in 1..=line_count(source) {
        if regions.contains_line(line) {
            split.test_added += 1;
        } else {
            split.non_test_added += 1;
        }
    }

    Ok(split)
}

pub fn is_python_whole_test_path(path: &str) -> bool {
    if Path::new(path).extension().and_then(|ext| ext.to_str()) != Some("py") {
        return false;
    }

    let filename = Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if filename == "conftest.py" || filename.starts_with("test_") || filename.ends_with("_test.py")
    {
        return true;
    }

    Path::new(path)
        .components()
        .any(|component| component.as_os_str() == "tests")
}

fn parse_python_source(source: &str) -> Result<tree_sitter::Tree, String> {
    let mut parser = Parser::new();
    let language = tree_sitter_python::LANGUAGE.into();
    parser
        .set_language(&language)
        .map_err(|error| format!("failed to load python grammar: {error}"))?;

    parser
        .parse(source, None)
        .ok_or_else(|| "failed to parse python source".to_string())
}

fn collect_regions(
    node: Node<'_>,
    source: &[u8],
    regions: &mut Vec<LineRange>,
) -> Result<(), String> {
    if let Some(range) = test_range_for_node(node, source)? {
        regions.push(range);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_regions(child, source, regions)?;
    }

    Ok(())
}

fn test_range_for_node(node: Node<'_>, source: &[u8]) -> Result<Option<LineRange>, String> {
    if node.kind() == "decorated_definition" {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if is_test_definition(child, source)? {
                let range = node.range();
                return Ok(Some(LineRange {
                    start: range.start_point.row + 1,
                    end: range.end_point.row + 1,
                }));
            }
        }
        return Ok(None);
    }

    if is_test_definition(node, source)? {
        let range = node.range();
        return Ok(Some(LineRange {
            start: range.start_point.row + 1,
            end: range.end_point.row + 1,
        }));
    }

    Ok(None)
}

fn is_test_definition(node: Node<'_>, source: &[u8]) -> Result<bool, String> {
    match node.kind() {
        "function_definition" => Ok(extract_name(node, source)?
            .map(|name| name.starts_with("test_"))
            .unwrap_or(false)),
        "class_definition" => Ok(extract_name(node, source)?
            .map(|name| name.starts_with("Test"))
            .unwrap_or(false)),
        _ => Ok(false),
    }
}

fn extract_name(node: Node<'_>, source: &[u8]) -> Result<Option<String>, String> {
    if let Some(name) = node.child_by_field_name("name") {
        return name
            .utf8_text(source)
            .map(|text| Some(text.to_string()))
            .map_err(|error| format!("invalid utf8 in python identifier: {error}"));
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() == "identifier" {
            return child
                .utf8_text(source)
                .map(|text| Some(text.to_string()))
                .map_err(|error| format!("invalid utf8 in python identifier: {error}"));
        }
    }

    Ok(None)
}
