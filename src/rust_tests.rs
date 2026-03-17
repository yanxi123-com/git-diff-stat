use tree_sitter::{Node, Parser};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestRegions {
    regions: Vec<LineRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CfgTestModuleImport {
    pub module_name: String,
    pub path_attribute: Option<String>,
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
    let tree = parse_rust_source(source)?;
    let mut regions = Vec::new();
    collect_regions(tree.root_node(), source.as_bytes(), &mut regions)?;

    Ok(TestRegions { regions })
}

pub fn detect_cfg_test_module_imports(source: &str) -> Result<Vec<CfgTestModuleImport>, String> {
    let tree = parse_rust_source(source)?;
    let mut imports = Vec::new();
    collect_cfg_test_module_imports(tree.root_node(), source.as_bytes(), &mut imports)?;

    Ok(imports)
}

fn parse_rust_source(source: &str) -> Result<tree_sitter::Tree, String> {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE.into();
    parser
        .set_language(&language)
        .map_err(|error| format!("failed to load rust grammar: {error}"))?;

    parser
        .parse(source, None)
        .ok_or_else(|| "failed to parse rust source".to_string())
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

fn collect_cfg_test_module_imports(
    node: Node<'_>,
    source: &[u8],
    imports: &mut Vec<CfgTestModuleImport>,
) -> Result<(), String> {
    if node.kind() == "mod_item"
        && let Some(import) = cfg_test_module_import(node, source)?
    {
        imports.push(import);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_cfg_test_module_imports(child, source, imports)?;
    }

    Ok(())
}

fn is_test_node(node: Node<'_>, source: &[u8]) -> Result<bool, String> {
    has_leading_attribute(node, source, |text| {
        text.contains("cfg(test)") || text.contains("[test]") || text.contains("::test]")
    })
}

fn cfg_test_module_import(
    node: Node<'_>,
    source: &[u8],
) -> Result<Option<CfgTestModuleImport>, String> {
    if !is_cfg_test_node(node, source)? || !is_external_module(node, source)? {
        return Ok(None);
    }

    let Some(module_name) = extract_module_name(node, source)? else {
        return Ok(None);
    };

    let path_attribute = leading_attribute_texts(node, source)?
        .into_iter()
        .find_map(|text| parse_path_attribute(&text));

    Ok(Some(CfgTestModuleImport {
        module_name,
        path_attribute,
    }))
}

fn is_cfg_test_node(node: Node<'_>, source: &[u8]) -> Result<bool, String> {
    has_leading_attribute(node, source, |text| text.contains("cfg(test)"))
}

fn has_leading_attribute(
    node: Node<'_>,
    source: &[u8],
    predicate: impl Fn(&str) -> bool,
) -> Result<bool, String> {
    let mut sibling = node.prev_sibling();
    while let Some(previous) = sibling {
        if previous.kind() != "attribute_item" {
            break;
        }

        let text = previous
            .utf8_text(source)
            .map_err(|error| format!("invalid utf8 in attribute: {error}"))?;
        if predicate(text) {
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

fn leading_attribute_texts(node: Node<'_>, source: &[u8]) -> Result<Vec<String>, String> {
    let mut texts = Vec::new();
    let mut sibling = node.prev_sibling();

    while let Some(previous) = sibling {
        if previous.kind() != "attribute_item" {
            break;
        }

        texts.push(
            previous
                .utf8_text(source)
                .map_err(|error| format!("invalid utf8 in attribute: {error}"))?
                .to_string(),
        );
        sibling = previous.prev_sibling();
    }

    texts.reverse();
    Ok(texts)
}

fn is_external_module(node: Node<'_>, source: &[u8]) -> Result<bool, String> {
    Ok(node
        .utf8_text(source)
        .map_err(|error| format!("invalid utf8 in module: {error}"))?
        .trim_end()
        .ends_with(';'))
}

fn extract_module_name(node: Node<'_>, source: &[u8]) -> Result<Option<String>, String> {
    if let Some(name) = node.child_by_field_name("name") {
        return name
            .utf8_text(source)
            .map(|text| Some(text.to_string()))
            .map_err(|error| format!("invalid utf8 in module name: {error}"));
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" {
            return child
                .utf8_text(source)
                .map(|text| Some(text.to_string()))
                .map_err(|error| format!("invalid utf8 in module name: {error}"));
        }
    }

    Ok(None)
}

fn parse_path_attribute(attribute_text: &str) -> Option<String> {
    if !attribute_text.contains("path") {
        return None;
    }

    let start = attribute_text.find('"')?;
    let rest = &attribute_text[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}
