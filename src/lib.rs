mod tree;
mod parser;
mod traversal;

#[cfg(feature = "extension-module")]
mod python {
    use pyo3::prelude::*;
    use crate::tree::DocumentTree;
    use crate::traversal;
    use crate::parser;

    #[pyclass]
    pub struct PageIndex {
        inner: DocumentTree,
    }

    #[pymethods]
    impl PageIndex {
        #[staticmethod]
        fn from_markdown(doc_id: &str, markdown: &str) -> Self {
            let tree = parser::parse_markdown(doc_id, markdown);
            PageIndex { inner: tree }
        }

        #[staticmethod]
        fn from_file(doc_id: &str, path: &str) -> PyResult<Self> {
            let content = std::fs::read_to_string(path)
                .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
            let tree = parser::parse_markdown(doc_id, &content);
            Ok(PageIndex { inner: tree })
        }

        fn title(&self) -> String {
            self.inner.title.clone()
        }

        fn outline(&self) -> String {
            traversal::get_tree_outline(&self.inner)
        }

        fn node_ids(&self) -> Vec<String> {
            self.inner.all_node_ids()
        }

        fn get_node(&self, node_id: &str) -> Option<PyNodeResult> {
            traversal::get_node(&self.inner, node_id).map(|r| PyNodeResult {
                node_id: r.node_id,
                title: r.title,
                text: r.text,
                depth: r.depth,
                breadcrumb: r.breadcrumb,
            })
        }

        fn get_node_with_children(&self, node_id: &str) -> Option<PyNodeResult> {
            traversal::get_node_with_children(&self.inner, node_id).map(|r| PyNodeResult {
                node_id: r.node_id,
                title: r.title,
                text: r.text,
                depth: r.depth,
                breadcrumb: r.breadcrumb,
            })
        }

        fn get_children(&self, node_id: &str) -> Vec<(String, String)> {
            traversal::get_children(&self.inner, node_id)
        }

        fn to_json(&self) -> String {
            self.inner.to_json()
        }
    }

    #[pyclass]
    #[derive(Clone)]
    pub struct PyNodeResult {
        #[pyo3(get)]
        pub node_id: String,
        #[pyo3(get)]
        pub title: String,
        #[pyo3(get)]
        pub text: String,
        #[pyo3(get)]
        pub depth: usize,
        #[pyo3(get)]
        pub breadcrumb: Vec<String>,
    }

    #[pymethods]
    impl PyNodeResult {
        fn __repr__(&self) -> String {
            format!(
                "NodeResult(node_id='{}', title='{}', depth={})",
                self.node_id, self.title, self.depth
            )
        }
    }

    #[pymodule]
    pub fn pageindex_rs(_py: Python, m: &PyModule) -> PyResult<()> {
        m.add_class::<PageIndex>()?;
        m.add_class::<PyNodeResult>()?;
        Ok(())
    }
}
