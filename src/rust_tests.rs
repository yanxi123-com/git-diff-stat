use tree_sitter::{Node, Parser};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestRegions {
    regions: Vec<LineRange>,
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

pub fn detect_test_regions(source: &str) -> Result<TestRegions, String> {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE.into();
    parser
        .set_language(&language)
        .map_err(|error| format!("failed to load rust grammar: {error}"))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| "failed to parse rust source".to_string())?;

    let mut regions = Vec::new();
    collect_regions(tree.root_node(), source.as_bytes(), &mut regions)?;

    Ok(TestRegions { regions })
}

fn collect_regions(
    node: Node<'_>,
    source: &[u8],
    regions: &mut Vec<LineRange>,
) -> Result<(), String> {
    if matches!(node.kind(), "mod_item" | "function_item") && is_test_node(node, source)? {
        let range = node.range();
        let start = find_region_start_line(node);
        regions.push(LineRange {
            start,
            end: range.end_point.row + 1,
        });
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_regions(child, source, regions)?;
    }

    Ok(())
}

fn is_test_node(node: Node<'_>, source: &[u8]) -> Result<bool, String> {
    let mut sibling = node.prev_sibling();
    while let Some(previous) = sibling {
        if previous.kind() != "attribute_item" {
            break;
        }

        let text = previous
            .utf8_text(source)
            .map_err(|error| format!("invalid utf8 in attribute: {error}"))?;
        if text.contains("cfg(test)") || text.contains("[test]") || text.contains("::test]") {
            return Ok(true);
        }

        sibling = previous.prev_sibling();
    }

    Ok(false)
}

fn find_region_start_line(node: Node<'_>) -> usize {
    let mut start = node.range().start_point.row + 1;
    let mut sibling = node.prev_sibling();

    while let Some(previous) = sibling {
        if previous.kind() != "attribute_item" {
            break;
        }

        start = previous.range().start_point.row + 1;
        sibling = previous.prev_sibling();
    }

    start
}
