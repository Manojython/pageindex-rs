use crate::tree::{DocumentTree, Node};

#[derive(Debug, Clone)]
pub struct TraversalResult {
    pub node_id: String,
    pub title: String,
    pub text: String,
    pub summary: Option<String>,
    pub depth: usize,
    pub breadcrumb: Vec<String>,
}

pub fn get_node(tree: &DocumentTree, node_id: &str) -> Option<TraversalResult> {
    let breadcrumb = build_breadcrumb(tree, node_id);
    tree.find_node(node_id).map(|node| TraversalResult {
        node_id: node.node_id.clone(),
        title: node.title.clone(),
        text: node.text.clone(),
        summary: node.summary.clone(),
        depth: node.depth,
        breadcrumb,
    })
}

pub fn get_node_with_children(tree: &DocumentTree, node_id: &str) -> Option<TraversalResult> {
    let breadcrumb = build_breadcrumb(tree, node_id);
    tree.find_node(node_id).map(|node| {
        let full_text = collect_subtree_text(node);
        TraversalResult {
            node_id: node.node_id.clone(),
            title: node.title.clone(),
            text: full_text,
            summary: node.summary.clone(),
            depth: node.depth,
            breadcrumb,
        }
    })
}

// Produces a compact outline for LLM consumption, e.g.:
// [1] Introduction
//   [1.1] Background
//   [1.2] Goals
pub fn get_tree_outline(tree: &DocumentTree) -> String {
    let mut lines = Vec::new();
    outline_node(&tree.root, &mut lines);
    lines.join("\n")
}

pub fn get_children(tree: &DocumentTree, node_id: &str) -> Vec<(String, String)> {
    tree.find_node(node_id)
        .map(|node| {
            node.children
                .iter()
                .map(|c| (c.node_id.clone(), c.title.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn collect_subtree_text(node: &Node) -> String {
    let mut parts = vec![node.text.clone()];
    for child in &node.children {
        let heading = "#".repeat(child.depth);
        parts.push(format!("{} {}", heading, child.title));
        parts.push(collect_subtree_text(child));
    }
    parts.into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join("\n\n")
}

fn build_breadcrumb(tree: &DocumentTree, node_id: &str) -> Vec<String> {
    let parts: Vec<&str> = node_id.split('.').collect();
    let mut breadcrumb = Vec::new();
    for i in 1..=parts.len() {
        let prefix = parts[..i].join(".");
        if let Some(node) = tree.find_node(&prefix) {
            breadcrumb.push(node.title.clone());
        }
    }
    breadcrumb
}

fn outline_node(node: &Node, lines: &mut Vec<String>) {
    if node.node_id != "0" {
        let indent = "  ".repeat(node.depth.saturating_sub(1));
        lines.push(format!("{}[{}] {}", indent, node.node_id, node.title));
    }
    for child in &node.children {
        outline_node(child, lines);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_markdown;

    const SAMPLE: &str = r#"
# Introduction
Introductory text.

## Background
Background details.

## Goals
Goal details.

# Methods
Method details.

## Experiment
Experiment details.
"#;

    #[test]
    fn test_get_node_returns_correct_title() {
        let tree = parse_markdown("doc1", SAMPLE);
        let result = get_node(&tree, "2").unwrap();
        assert_eq!(result.title, "Methods");
    }

    #[test]
    fn test_get_node_returns_correct_text() {
        let tree = parse_markdown("doc1", SAMPLE);
        let result = get_node(&tree, "1.1").unwrap();
        assert_eq!(result.text, "Background details.");
    }

    #[test]
    fn test_get_node_missing_returns_none() {
        let tree = parse_markdown("doc1", SAMPLE);
        assert!(get_node(&tree, "9.9").is_none());
    }

    #[test]
    fn test_breadcrumb_for_nested_node() {
        let tree = parse_markdown("doc1", SAMPLE);
        let result = get_node(&tree, "2.1").unwrap();
        assert_eq!(result.breadcrumb, vec!["Methods", "Experiment"]);
    }

    #[test]
    fn test_breadcrumb_for_top_level_node() {
        let tree = parse_markdown("doc1", SAMPLE);
        let result = get_node(&tree, "1").unwrap();
        assert_eq!(result.breadcrumb, vec!["Introduction"]);
    }

    #[test]
    fn test_get_node_with_children_includes_child_text() {
        let tree = parse_markdown("doc1", SAMPLE);
        let result = get_node_with_children(&tree, "1").unwrap();
        assert!(result.text.contains("Background details."));
        assert!(result.text.contains("Goal details."));
    }

    #[test]
    fn test_get_children_returns_correct_pairs() {
        let tree = parse_markdown("doc1", SAMPLE);
        let children = get_children(&tree, "1");
        assert_eq!(children.len(), 2);
        assert_eq!(children[0], ("1.1".to_string(), "Background".to_string()));
        assert_eq!(children[1], ("1.2".to_string(), "Goals".to_string()));
    }

    #[test]
    fn test_get_children_leaf_node_returns_empty() {
        let tree = parse_markdown("doc1", SAMPLE);
        let children = get_children(&tree, "1.1");
        assert!(children.is_empty());
    }

    #[test]
    fn test_outline_contains_all_nodes() {
        let tree = parse_markdown("doc1", SAMPLE);
        let outline = get_tree_outline(&tree);
        assert!(outline.contains("[1] Introduction"));
        assert!(outline.contains("[1.1] Background"));
        assert!(outline.contains("[2] Methods"));
        assert!(outline.contains("[2.1] Experiment"));
    }

    #[test]
    fn test_outline_indentation() {
        let tree = parse_markdown("doc1", SAMPLE);
        let outline = get_tree_outline(&tree);
        assert!(outline.contains("  [1.1]"));
        assert!(outline.contains("  [2.1]"));
    }
}
