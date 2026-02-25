use crate::tree::{DocumentTree, Node};

pub fn parse_markdown(doc_id: &str, markdown: &str) -> DocumentTree {
    let mut blocks: Vec<(usize, String, String)> = Vec::new();
    let mut doc_title = doc_id.to_string();

    let mut current_depth: usize = 0;
    let mut current_title = String::new();
    let mut current_body: Vec<&str> = Vec::new();
    let mut started = false;

    for line in markdown.lines() {
        if let Some((depth, title)) = parse_heading(line) {
            if started {
                blocks.push((
                    current_depth,
                    current_title.clone(),
                    current_body.join("\n").trim().to_string(),
                ));
            }

            if !started && depth == 1 {
                doc_title = title.clone();
            }

            current_depth = depth;
            current_title = title;
            current_body = Vec::new();
            started = true;
        } else if started {
            current_body.push(line);
        }
    }

    if started {
        blocks.push((
            current_depth,
            current_title,
            current_body.join("\n").trim().to_string(),
        ));
    }

    let root = build_tree(&blocks);
    DocumentTree::new(doc_id.to_string(), doc_title, root)
}

fn parse_heading(line: &str) -> Option<(usize, String)> {
    if !line.starts_with('#') {
        return None;
    }
    let depth = line.chars().take_while(|c| *c == '#').count();
    let title = line[depth..].trim().to_string();
    if title.is_empty() {
        return None;
    }
    Some((depth, title))
}

fn build_tree(blocks: &[(usize, String, String)]) -> Node {
    let mut root = Node::new("0".to_string(), "root".to_string(), 0, String::new());
    let mut node_stack: Vec<Node> = vec![root.clone()];
    let mut depth_counters = vec![0usize; 10];

    for (depth, title, body) in blocks {
        let depth = *depth;

        depth_counters[depth] += 1;
        for i in (depth + 1)..10 {
            depth_counters[i] = 0;
        }

        let node_id = depth_counters[1..=depth]
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(".");

        let node = Node::new(node_id, title.clone(), depth, body.clone());

        while node_stack.len() > 1 {
            let top_depth = node_stack.last().unwrap().depth;
            if top_depth >= depth {
                let child = node_stack.pop().unwrap();
                node_stack.last_mut().unwrap().children.push(child);
            } else {
                break;
            }
        }

        node_stack.push(node);
    }

    while node_stack.len() > 1 {
        let child = node_stack.pop().unwrap();
        node_stack.last_mut().unwrap().children.push(child);
    }

    let mut final_root = node_stack.pop().unwrap();

    // Promote to root if there's only one top-level section
    if final_root.node_id == "0" && final_root.children.len() == 1 {
        final_root.children.remove(0)
    } else {
        final_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_doc_title_from_first_heading() {
        let tree = parse_markdown("doc1", SAMPLE);
        assert_eq!(tree.title, "Introduction");
    }

    #[test]
    fn test_node_ids_are_correct() {
        let tree = parse_markdown("doc1", SAMPLE);
        let ids = tree.all_node_ids();
        assert!(ids.contains(&"1".to_string()));
        assert!(ids.contains(&"1.1".to_string()));
        assert!(ids.contains(&"1.2".to_string()));
        assert!(ids.contains(&"2".to_string()));
        assert!(ids.contains(&"2.1".to_string()));
    }

    #[test]
    fn test_node_body_text() {
        let tree = parse_markdown("doc1", SAMPLE);
        let node = tree.find_node("1.1").unwrap();
        assert_eq!(node.text, "Background details.");
    }

    #[test]
    fn test_child_count_for_introduction() {
        let tree = parse_markdown("doc1", SAMPLE);
        let node = tree.find_node("1").unwrap();
        assert_eq!(node.children.len(), 2);
    }

    #[test]
    fn test_total_node_count() {
        let tree = parse_markdown("doc1", SAMPLE);
        assert_eq!(tree.all_node_ids().len(), 5);
    }

    #[test]
    fn test_empty_markdown_produces_root() {
        let tree = parse_markdown("empty", "");
        assert_eq!(tree.doc_id, "empty");
    }

    #[test]
    fn test_single_heading() {
        let md = "# Only Section\nSome text.";
        let tree = parse_markdown("single", md);
        assert_eq!(tree.title, "Only Section");
        let node = tree.find_node("1").unwrap();
        assert_eq!(node.text, "Some text.");
    }
}
