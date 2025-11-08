AST Tree Querying and Traversal

	The AST provides a comprehensive query API for finding and iterating nodes without manual traversal or custom visitors. This guide explains when and how to use these APIs.

1. The Problem with Manual Iteration

	Direct iteration over children requires significant boilerplate and is error-prone. Consider finding all paragraphs in a document with nested sessions:

		for item in &doc.root.children {
			match item {
				ContentItem::Paragraph(p) => { /* process */ },
				ContentItem::Session(s) => {
					for child in &s.children {
						if let ContentItem::Paragraph(p) = child {
							/* process nested paragraph */
						}
					}
				}
				_ => {}
			}
		}
	:: rust ::

	This approach has several issues: it doesn't scale to deeper nesting, requires explicit pattern matching, and mixes iteration logic with business logic.

2. Query API Overview

	The query API provides two levels of functionality: recursive iteration and predicate-based filtering.

	2.1 Recursive Iteration

		The recursive iterators find all nodes of a specific type at any depth in the tree:

			let paragraphs: Vec<_> = doc.iter_paragraphs_recursive().collect();
			let sessions: Vec<_> = doc.iter_sessions_recursive().collect();
			let all_nodes: Vec<_> = doc.iter_all_nodes().collect();
		:: rust ::

		These methods return iterators that can be chained with standard iterator combinators. Unlike the non-recursive variants (iter_paragraphs, iter_sessions), they traverse the entire tree.

	2.2 Predicate-Based Filtering

		For more complex queries, use the find methods with predicates:

			let intro_paras = doc.find_paragraphs(|p| {
				p.text().to_lowercase().contains("introduction")
			});

			let chapters = doc.find_sessions(|s| {
				s.title.as_string().starts_with("Chapter")
			});
		:: rust ::

		The predicate is any closure that takes a reference to the node type and returns a boolean.

	2.3 Depth-Based Queries

		When tree depth matters, use depth-aware methods:

			let top_level = doc.find_nodes_at_depth(0);
			let deep_sessions = doc.find_sessions_at_depth(3);
			let mid_range = doc.find_nodes_in_depth_range(1, 3);
		:: rust ::

		Depth is measured from the document root, where direct children are at depth 0.

	2.4 Combined Queries

		Depth and predicates can be combined:

			let results = doc.find_nodes_with_depth(2, |node| {
				node.as_paragraph()
					.map(|p| p.text().len() > 100)
					.unwrap_or(false)
			});
		:: rust ::

3. When to Use Each Method

	3.1 Use Recursive Iterators When:

		- You need all nodes of a specific type regardless of depth
		- You want to chain with iterator combinators (filter, map, collect)
		- You're counting or aggregating across the entire tree

	3.2 Use Find Methods When:

		- You need nodes matching specific criteria
		- The criteria involve node content or properties
		- You want a Vec<&T> result immediately

	3.3 Use Depth Methods When:

		- Document structure matters (outline levels, heading depth)
		- You need to distinguish top-level from nested content
		- Building navigation or table of contents

	3.4 Use iter_all_nodes When:

		- You need to process every node regardless of type
		- Implementing tree-wide operations (validation, transformation)
		- Building indices or caches

4. Practical Examples

	4.1 Finding All Code Blocks

			let code_blocks = doc.iter_verbatim_blocks_recursive()
				.filter(|v| v.label().map(|l| l == "code").unwrap_or(false))
				.collect::<Vec<_>>();
		:: rust ::

	4.2 Counting Nodes at Each Depth

			let counts_by_depth = doc.iter_all_nodes_with_depth()
				.fold(HashMap::new(), |mut acc, (_, depth)| {
					*acc.entry(depth).or_insert(0) += 1;
					acc
				});
		:: rust ::

	4.3 Finding Empty Sessions

			let empty_sessions = doc.find_sessions(|s| s.children.is_empty());
		:: rust ::

	4.4 Extracting Top-Level Headings

			let headings = doc.find_sessions_at_depth(0)
				.iter()
				.map(|s| s.title.as_string())
				.collect::<Vec<_>>();
		:: rust ::

5. Performance Considerations

	The query APIs use Rust's iterator pattern, which provides zero-cost abstractions. Iterators are lazy - they only traverse when consumed. This means:

		let iter = doc.iter_paragraphs_recursive(); // No work done yet
		let first = iter.next(); // Only traverses until first paragraph found
	:: rust ::

	For operations requiring multiple passes, collect once and reuse:

		let all_nodes: Vec<_> = doc.iter_all_nodes().collect();
		let paragraph_count = all_nodes.iter().filter(|n| n.is_paragraph()).count();
		let session_count = all_nodes.iter().filter(|n| n.is_session()).count();
	:: rust ::

6. Migration from Manual Iteration

	Existing code using manual iteration can be refactored incrementally:

	Before:
		for item in &doc.root.children {
			if let ContentItem::Paragraph(p) = item {
				if p.text().starts_with("Note:") {
					// process
				}
			}
		}
	:: rust ::

	After:
		for p in doc.find_paragraphs(|p| p.text().starts_with("Note:")) {
			// process
		}
	:: rust ::

	The new API is more declarative, handles nesting automatically, and separates traversal from business logic.

7. Available Query Methods

	On Document:
		Recursive iteration: iter_paragraphs_recursive, iter_sessions_recursive, iter_lists_recursive, iter_definitions_recursive, iter_annotations_recursive, iter_verbatim_blocks_recursive, iter_list_items_recursive

		General traversal: iter_all_nodes, iter_all_nodes_with_depth

		Predicate filtering: find_paragraphs, find_sessions, find_lists, find_definitions, find_annotations, find_nodes

		Depth filtering: find_nodes_at_depth, find_nodes_in_depth_range, find_sessions_at_depth, find_paragraphs_at_depth, find_nodes_with_depth

	On ContentItem:
		descendants: Get all descendants of any node
		descendants_with_depth: Get descendants with relative depth tracking

8. Design Rationale

	8.1 Why Iterators Over Collections

		Returning iterators instead of Vec allows callers to control allocation. Iterators can be short-circuited, chained, and composed without intermediate allocations.

	8.2 Why Predicates Over Method Chains

		While a fluent API (query().sessions().with_title("X")) is possible, predicates with closures provide maximum flexibility without API explosion. The caller can use any logic without waiting for specific methods to be added.

	8.3 Why Both Iterators and Find Methods

		iter_* methods return iterators for composability. find_* methods return Vec for convenience when immediate collection is needed. Use iterators for chaining, use find for simple queries.
