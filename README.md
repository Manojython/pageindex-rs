# pageindex-rs

A Rust-powered Python library for structured document retrieval in RAG pipelines.

---

I kept running into the same problem building LLM agents over financial documents and technical manuals: chunk-based RAG is terrible at it. You embed a 10-K, split it into 512-token chunks, and at query time you get three chunks from different sections that happen to share vocabulary, none of which actually answers the question. The retrieval is noisy, the context window fills up with irrelevant text, and you end up paying for tokens that actively hurt the answer.

The fix is obvious once you see it — structured documents already tell you how they're organized. Every heading is a natural retrieval boundary. `pageindex-rs` just respects that structure. It parses a markdown document into a tree of nodes, one per heading, and at retrieval time you hand the outline to your LLM and ask it which section to look in. One node, exactly the text you need, no embeddings required.

This is a Rust reimplementation of the original [PageIndex](https://github.com/TRAIS-Lab/PageIndex) library with Python bindings via PyO3. The Rust version is faster at scale and more consistent under load — details in the benchmarks below.

## Installation

```bash
pip install pageindex-rs
```

## Usage

```python
import pageindex_rs

# Build an index from a markdown file
index = pageindex_rs.PageIndex.from_file("annual_report", "report.md")

# Or directly from a string
index = pageindex_rs.PageIndex.from_markdown("annual_report", markdown_string)

# Get the outline — this is what you send to your LLM
print(index.outline())
# [1] Executive Summary
# [2] Financial Results
#   [2.1] Revenue
#   [2.2] Expenses
#   [2.3] Net Income
# [3] Risk Factors
#   [3.1] Market Risk
#   [3.2] Regulatory Risk

# Fetch the node your LLM picked
node = index.get_node("3.2")
print(node.title)       # Regulatory Risk
print(node.text)        # New AI regulations in the EU...
print(node.breadcrumb)  # ['Risk Factors', 'Regulatory Risk']

# Need the full section including subsections?
section = index.get_node_with_children("2")
print(section.text)     # Revenue + Expenses + Net Income combined

# Peek at what's inside a section before going deeper
children = index.get_children("2")
# [('2.1', 'Revenue'), ('2.2', 'Expenses'), ('2.3', 'Net Income')]

# Full tree as JSON if you need it
print(index.to_json())
```

## How retrieval works

```python
outline = index.outline()

response = llm(f"""
Document outline:
{outline}

Question: {user_query}

Return only the node_id of the most relevant section. Nothing else.
""")

node_id = response.strip()
result = index.get_node(node_id)
# Pass result.text to your LLM to generate the final answer
```

The dot-notation node IDs (`1.2.3`) give the LLM a natural sense of document structure — it can see that `2.3` is a subsection of `2` without any extra explanation. This turns out to matter for accuracy.

## API

### PageIndex

| Method | Description |
|--------|-------------|
| `PageIndex.from_markdown(doc_id, markdown)` | Build from a markdown string |
| `PageIndex.from_file(doc_id, path)` | Build from a file path |
| `index.title()` | Document title (first H1) |
| `index.outline()` | Compact tree for LLM prompts |
| `index.node_ids()` | All node IDs in the tree |
| `index.get_node(node_id)` | Single node lookup |
| `index.get_node_with_children(node_id)` | Node with all descendant text merged |
| `index.get_children(node_id)` | Direct children as `(node_id, title)` pairs |
| `index.to_json()` | Full tree as JSON |

### NodeResult

| Attribute | Type | Description |
|-----------|------|-------------|
| `node_id` | str | Dot-separated ID, e.g. `"2.1"` |
| `title` | str | Heading text |
| `text` | str | Body text of this node |
| `depth` | int | Heading level (1 = `#`, 2 = `##`, etc.) |
| `breadcrumb` | list[str] | Path from root to this node |

## Benchmarks

Benchmarked against the original Python PageIndex library. 500 iterations per build test, 1000 random lookups per retrieval test. Run the full benchmark yourself: `tests/pageindex_rs_benchmark.ipynb`.

### Index build speed

| Document size | Rust mean | Python mean | Speedup |
|--------------|-----------|-------------|---------|
| 42 KB | 0.207 ms | 0.153 ms | 0.74x ❌ |
| 395 KB | 0.873 ms | 1.369 ms | 1.57x |
| 1055 KB | 2.549 ms | 4.278 ms | **1.68x** |

Below ~200KB, PyO3 FFI overhead cancels out the parsing speedup. At realistic document sizes (several hundred KB and above) Rust pulls ahead. The more important number is consistency:

| Document size | Rust stdev | Python stdev | Rust p99 | Python p99 |
|--------------|-----------|--------------|----------|------------|
| 42 KB | 0.835 ms | 0.014 ms | 1.335 ms | 0.206 ms |
| 395 KB | 0.060 ms | 0.053 ms | 1.129 ms | 1.511 ms |
| 1055 KB | 0.104 ms | 2.782 ms | 2.781 ms | 20.993 ms |

At 1055 KB, Python's p99 is **20ms** and its max is **42ms**. Rust's p99 is **2.8ms** and max is **3.7ms**. In a pipeline handling hundreds of documents, those Python spikes add up.

### Node retrieval speed

Rust uses a `HashMap` so lookups are O(1). Python does a linear scan, so performance degrades as the tree grows.

| Document size | Nodes | Rust mean | Python mean | Speedup |
|--------------|-------|-----------|-------------|---------|
| 42 KB | 28 | 0.0072 ms | 0.0060 ms | 0.83x |
| 395 KB | 261 | 0.0119 ms | 0.0272 ms | 2.29x |
| 1055 KB | 765 | 0.0216 ms | 0.0686 ms | **3.18x** |

The gap keeps widening. At 765 nodes Rust is 3.18x faster on average. For large technical manuals or combined document corpora this becomes meaningful.

### Answer accuracy

Tested on 10 financial questions against a ~3MB document corpus:

| | Correct |
|--|---------|
| pageindex-rs | 9 / 10 |
| PageIndex (Python) | 7 / 10 |

The accuracy difference comes down to node IDs. `1.2.3` is self-explanatory to an LLM — it signals hierarchy directly. `0012` is just a number with no structural meaning, so the LLM occasionally picks the wrong node.

## Roadmap

- PDF support (the big one)
- Cross-document retrieval across a corpus
- PageRank-style importance scoring on the tree

### Credits
https://github.com/VectifyAI/PageIndex?tab=readme-ov-file

Medium article which inspired this: https://agentnativedev.medium.com/vectorless-rag-for-agents-pageindex-is-why-their-demo-works-and-yours-needs-context-6a9219dcc20e

## License

MIT
