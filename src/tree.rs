use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub node_id: String,
    pub title: String,
    pub depth: usize,
    pub text: String,
    pub summary: Option<String>,
    pub children: Vec<Node>,
}

impl Node {
    pub fn new(node_id: String, title: String, depth: usize, text: String) -> Self {
        Node {
            node_id,
            title,
            depth,
            text,
            summary: None,
            children: Vec::new(),
        }
    }

    pub fn find(&self, node_id: &str) -> Option<&Node> {
        if self.node_id == node_id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find(node_id) {
                return Some(found);
            }
        }
        None
    }

    pub fn all_ids(&self) -> Vec<String> {
        let mut ids = vec![self.node_id.clone()];
        for child in &self.children {
            ids.extend(child.all_ids());
        }
        ids
    }

    pub fn flatten(&self) -> Vec<&Node> {
        let mut nodes = vec![self as &Node];
        for child in &self.children {
            nodes.extend(child.flatten());
        }
        nodes
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTree {
    pub doc_id: String,
    pub title: String,
    pub description: Option<String>,
    pub root: Node,
}

impl DocumentTree {
    pub fn new(doc_id: String, title: String, root: Node) -> Self {
        DocumentTree {
            doc_id,
            title,
            description: None,
            root,
        }
    }

    pub fn find_node(&self, node_id: &str) -> Option<&Node> {
        self.root.find(node_id)
    }

    // Excludes the synthetic root node used when a document has multiple top-level headings
    pub fn all_node_ids(&self) -> Vec<String> {
        if self.root.node_id == "0" {
            self.root.children.iter().flat_map(|c| c.all_ids()).collect()
        } else {
            self.root.all_ids()
        }
    }

    pub fn all_nodes(&self) -> Vec<&Node> {
        if self.root.node_id == "0" {
            self.root.children.iter().flat_map(|c| c.flatten()).collect()
        } else {
            self.root.flatten()
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tree() -> DocumentTree {
        let child1 = Node::new("1.1".to_string(), "Background".to_string(), 2, "Background text.".to_string());
        let child2 = Node::new("1.2".to_string(), "Goals".to_string(), 2, "Goals text.".to_string());
        let mut root = Node::new("1".to_string(), "Introduction".to_string(), 1, "Intro text.".to_string());
        root.children.push(child1);
        root.children.push(child2);
        DocumentTree::new("doc1".to_string(), "Introduction".to_string(), root)
    }

    #[test]
    fn test_find_root_node() {
        let tree = make_tree();
        let node = tree.find_node("1");
        assert!(node.is_some());
        assert_eq!(node.unwrap().title, "Introduction");
    }

    #[test]
    fn test_find_child_node() {
        let tree = make_tree();
        let node = tree.find_node("1.2");
        assert!(node.is_some());
        assert_eq!(node.unwrap().title, "Goals");
    }

    #[test]
    fn test_find_missing_node_returns_none() {
        let tree = make_tree();
        assert!(tree.find_node("9.9").is_none());
    }

    #[test]
    fn test_all_node_ids() {
        let tree = make_tree();
        let ids = tree.all_node_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&"1".to_string()));
        assert!(ids.contains(&"1.1".to_string()));
        assert!(ids.contains(&"1.2".to_string()));
    }

    #[test]
    fn test_flatten_count() {
        let tree = make_tree();
        assert_eq!(tree.all_nodes().len(), 3);
    }

    #[test]
    fn test_to_json_contains_title() {
        let tree = make_tree();
        let json = tree.to_json();
        assert!(json.contains("Introduction"));
        assert!(json.contains("1.1"));
    }
}
