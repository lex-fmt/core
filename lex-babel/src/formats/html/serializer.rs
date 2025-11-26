//! HTML serialization (Lex → HTML export)
//!
//! Converts Lex documents to semantic HTML5 with embedded CSS.
//! Pipeline: Lex AST → IR → Events → RcDom → HTML string

use crate::common::nested_to_flat::tree_to_events;
use crate::error::FormatError;
use crate::formats::html::HtmlTheme;
use crate::ir::events::Event;
use crate::ir::nodes::{DocNode, InlineContent};
use html5ever::{
    ns, serialize, serialize::SerializeOpts, serialize::TraversalScope, Attribute, LocalName,
    QualName,
};
use lex_parser::lex::ast::Document;
use markup5ever_rcdom::{Handle, Node, NodeData, RcDom, SerializableHandle};
use std::cell::{Cell, RefCell};
use std::default::Default;
use std::rc::Rc;

/// Serialize a Lex document to HTML with the given theme
pub fn serialize_to_html(doc: &Document, theme: HtmlTheme) -> Result<String, FormatError> {
    // Step 1: Lex AST → IR
    let ir_doc = crate::to_ir(doc);

    // Step 2: IR → Events
    let events = tree_to_events(&DocNode::Document(ir_doc));

    // Step 3: Events → RcDom (HTML DOM tree)
    let dom = build_html_dom(&events)?;

    // Step 4: RcDom → HTML string
    let html_string = serialize_dom(&dom)?;

    // Step 5: Wrap in complete HTML document with CSS
    let complete_html = wrap_in_document(&html_string, theme)?;

    Ok(complete_html)
}

/// Build an HTML DOM tree from IR events
fn build_html_dom(events: &[Event]) -> Result<RcDom, FormatError> {
    let dom = RcDom::default();

    // Create document container
    let doc_container = create_element("div", vec![("class", "lex-document")]);

    let mut current_parent: Handle = doc_container.clone();
    let mut parent_stack: Vec<Handle> = vec![];

    // State for collecting verbatim content
    let mut in_verbatim = false;
    let mut verbatim_language: Option<String> = None;
    let mut verbatim_content = String::new();

    // State for heading context
    let mut current_heading: Option<Handle> = None;

    for event in events {
        match event {
            Event::StartDocument => {
                // Already created doc_container
            }

            Event::EndDocument => {
                // Done
            }

            Event::StartHeading(level) => {
                // Create section wrapper for this session
                let class = format!("lex-session lex-session-{}", level);
                let section = create_element("section", vec![("class", &class)]);
                current_parent.children.borrow_mut().push(section.clone());
                parent_stack.push(current_parent.clone());
                current_parent = section;

                // Create heading element (h1-h6, max at h6)
                let heading_tag = format!("h{}", (*level as u8).min(6));
                let heading = create_element(&heading_tag, vec![]);
                current_parent.children.borrow_mut().push(heading.clone());
                current_heading = Some(heading);
            }

            Event::EndHeading(_) => {
                current_heading = None;
                // Close section
                current_parent = parent_stack.pop().ok_or_else(|| {
                    FormatError::SerializationError("Unbalanced heading end".to_string())
                })?;
            }

            Event::StartContent => {
                // Create content wrapper (mirrors AST container structure for indentation)
                current_heading = None;
                let content = create_element("div", vec![("class", "lex-content")]);
                current_parent.children.borrow_mut().push(content.clone());
                parent_stack.push(current_parent.clone());
                current_parent = content;
            }

            Event::EndContent => {
                // Close content wrapper
                current_parent = parent_stack.pop().ok_or_else(|| {
                    FormatError::SerializationError("Unbalanced content end".to_string())
                })?;
            }

            Event::StartParagraph => {
                current_heading = None;
                let para = create_element("p", vec![("class", "lex-paragraph")]);
                current_parent.children.borrow_mut().push(para.clone());
                parent_stack.push(current_parent.clone());
                current_parent = para;
            }

            Event::EndParagraph => {
                current_parent = parent_stack.pop().ok_or_else(|| {
                    FormatError::SerializationError("Unbalanced paragraph end".to_string())
                })?;
            }

            Event::StartList { ordered } => {
                current_heading = None;
                let tag = if *ordered { "ol" } else { "ul" };
                let list = create_element(tag, vec![("class", "lex-list")]);
                current_parent.children.borrow_mut().push(list.clone());
                parent_stack.push(current_parent.clone());
                current_parent = list;
            }

            Event::EndList => {
                current_parent = parent_stack.pop().ok_or_else(|| {
                    FormatError::SerializationError("Unbalanced list end".to_string())
                })?;
            }

            Event::StartListItem => {
                current_heading = None;
                let item = create_element("li", vec![("class", "lex-list-item")]);
                current_parent.children.borrow_mut().push(item.clone());
                parent_stack.push(current_parent.clone());
                current_parent = item;
            }

            Event::EndListItem => {
                current_parent = parent_stack.pop().ok_or_else(|| {
                    FormatError::SerializationError("Unbalanced list item end".to_string())
                })?;
            }

            Event::StartVerbatim(language) => {
                current_heading = None;
                in_verbatim = true;
                verbatim_language = language.clone();
                verbatim_content.clear();
            }

            Event::EndVerbatim => {
                // Create pre + code block
                let mut attrs = vec![("class", "lex-verbatim")];
                let lang_string;
                if let Some(ref lang) = verbatim_language {
                    lang_string = lang.clone();
                    attrs.push(("data-language", &lang_string));
                }

                let pre = create_element("pre", attrs);
                let code = create_element("code", vec![]);
                let text = create_text(&verbatim_content);
                code.children.borrow_mut().push(text);
                pre.children.borrow_mut().push(code);
                current_parent.children.borrow_mut().push(pre);

                in_verbatim = false;
                verbatim_language = None;
                verbatim_content.clear();
            }

            Event::StartDefinition => {
                current_heading = None;
                let dl = create_element("dl", vec![("class", "lex-definition")]);
                current_parent.children.borrow_mut().push(dl.clone());
                parent_stack.push(current_parent.clone());
                current_parent = dl;
            }

            Event::EndDefinition => {
                current_parent = parent_stack.pop().ok_or_else(|| {
                    FormatError::SerializationError("Unbalanced definition end".to_string())
                })?;
            }

            Event::StartDefinitionTerm => {
                let dt = create_element("dt", vec![]);
                current_parent.children.borrow_mut().push(dt.clone());
                parent_stack.push(current_parent.clone());
                current_parent = dt;
            }

            Event::EndDefinitionTerm => {
                current_parent = parent_stack.pop().ok_or_else(|| {
                    FormatError::SerializationError("Unbalanced definition term end".to_string())
                })?;
            }

            Event::StartDefinitionDescription => {
                let dd = create_element("dd", vec![]);
                current_parent.children.borrow_mut().push(dd.clone());
                parent_stack.push(current_parent.clone());
                current_parent = dd;
            }

            Event::EndDefinitionDescription => {
                current_parent = parent_stack.pop().ok_or_else(|| {
                    FormatError::SerializationError(
                        "Unbalanced definition description end".to_string(),
                    )
                })?;
            }

            Event::Inline(inline_content) => {
                if in_verbatim {
                    // Accumulate verbatim content
                    if let InlineContent::Text(text) = inline_content {
                        verbatim_content.push_str(text);
                    }
                } else if let Some(ref heading) = current_heading {
                    // Add to heading
                    add_inline_to_node(heading, inline_content)?;
                } else {
                    // Add to current parent
                    add_inline_to_node(&current_parent, inline_content)?;
                }
            }

            Event::StartAnnotation { label, parameters } => {
                current_heading = None;
                // Create HTML comment
                let mut comment = format!(" lex:{}", label);
                for (key, value) in parameters {
                    comment.push_str(&format!(" {}={}", key, value));
                }
                comment.push(' ');
                let comment_node = create_comment(&comment);
                current_parent.children.borrow_mut().push(comment_node);
            }

            Event::EndAnnotation { label } => {
                // Closing comment
                let comment = format!(" /lex:{} ", label);
                let comment_node = create_comment(&comment);
                current_parent.children.borrow_mut().push(comment_node);
            }
        }
    }

    // Set the document container as the root
    dom.document.children.borrow_mut().push(doc_container);

    Ok(dom)
}

/// Add inline content to an HTML node, handling references → anchors conversion
fn add_inline_to_node(parent: &Handle, inline: &InlineContent) -> Result<(), FormatError> {
    match inline {
        InlineContent::Text(text) => {
            let text_node = create_text(text);
            parent.children.borrow_mut().push(text_node);
        }

        InlineContent::Bold(children) => {
            let strong = create_element("strong", vec![]);
            parent.children.borrow_mut().push(strong.clone());
            for child in children {
                add_inline_to_node(&strong, child)?;
            }
        }

        InlineContent::Italic(children) => {
            let em = create_element("em", vec![]);
            parent.children.borrow_mut().push(em.clone());
            for child in children {
                add_inline_to_node(&em, child)?;
            }
        }

        InlineContent::Code(code_text) => {
            let code = create_element("code", vec![]);
            let text = create_text(code_text);
            code.children.borrow_mut().push(text);
            parent.children.borrow_mut().push(code);
        }

        InlineContent::Math(math_text) => {
            // Math rendered in a span with class
            let math_span = create_element("span", vec![("class", "lex-math")]);
            let dollar_open = create_text("$");
            let math_content = create_text(math_text);
            let dollar_close = create_text("$");
            math_span.children.borrow_mut().push(dollar_open);
            math_span.children.borrow_mut().push(math_content);
            math_span.children.borrow_mut().push(dollar_close);
            parent.children.borrow_mut().push(math_span);
        }

        InlineContent::Reference(ref_text) => {
            // Convert to anchor
            // Handle citations (@...) by targeting a reference ID
            let href = if let Some(citation) = ref_text.strip_prefix('@') {
                format!("#ref-{}", citation)
            } else {
                ref_text.to_string()
            };

            let anchor = create_element("a", vec![("href", &href)]);
            let anchor_text = create_text(ref_text);
            anchor.children.borrow_mut().push(anchor_text);
            parent.children.borrow_mut().push(anchor);
        }
    }

    Ok(())
}

/// Create an HTML element with attributes
fn create_element(tag: &str, attrs: Vec<(&str, &str)>) -> Handle {
    let qual_name = QualName::new(None, ns!(html), LocalName::from(tag));
    let attributes = attrs
        .into_iter()
        .map(|(name, value)| Attribute {
            name: QualName::new(None, ns!(), LocalName::from(name)),
            value: value.to_string().into(),
        })
        .collect();

    Rc::new(Node {
        parent: Cell::new(None),
        children: RefCell::new(Vec::new()),
        data: NodeData::Element {
            name: qual_name,
            attrs: RefCell::new(attributes),
            template_contents: Default::default(),
            mathml_annotation_xml_integration_point: false,
        },
    })
}

/// Create a text node
fn create_text(text: &str) -> Handle {
    Rc::new(Node {
        parent: Cell::new(None),
        children: RefCell::new(Vec::new()),
        data: NodeData::Text {
            contents: RefCell::new(text.to_string().into()),
        },
    })
}

/// Create a comment node
fn create_comment(text: &str) -> Handle {
    Rc::new(Node {
        parent: Cell::new(None),
        children: RefCell::new(Vec::new()),
        data: NodeData::Comment {
            contents: text.to_string().into(),
        },
    })
}

/// Serialize the DOM to an HTML string (just the inner content)
fn serialize_dom(dom: &RcDom) -> Result<String, FormatError> {
    let mut output = Vec::new();

    // Get the document container (first child of document root)
    let doc_container = dom
        .document
        .children
        .borrow()
        .first()
        .ok_or_else(|| FormatError::SerializationError("Empty document".to_string()))?
        .clone();

    // Serialize each child of the doc_container
    // Use TraversalScope::IncludeNode to serialize the element AND its children
    let opts = SerializeOpts {
        traversal_scope: TraversalScope::IncludeNode,
        ..Default::default()
    };

    for child in doc_container.children.borrow().iter() {
        let serializable = SerializableHandle::from(child.clone());
        serialize(&mut output, &serializable, opts.clone()).map_err(|e| {
            FormatError::SerializationError(format!("HTML serialization failed: {}", e))
        })?;
    }

    String::from_utf8(output)
        .map_err(|e| FormatError::SerializationError(format!("UTF-8 conversion failed: {}", e)))
}

/// Wrap the content in a complete HTML document with embedded CSS
fn wrap_in_document(body_html: &str, theme: HtmlTheme) -> Result<String, FormatError> {
    let baseline_css = include_str!("../../../css/baseline.css");
    let theme_css = match theme {
        HtmlTheme::FancySerif => include_str!("../../../css/themes/theme-fancy-serif.css"),
        HtmlTheme::Modern => include_str!("../../../css/themes/theme-modern.css"),
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="generator" content="lex-babel">
  <title>Lex Document</title>
  <style>
{}
{}
  </style>
</head>
<body>
<div class="lex-document">
{}
</div>
</body>
</html>"#,
        baseline_css, theme_css, body_html
    );

    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lex_parser::lex::transforms::standard::STRING_TO_AST;

    #[test]
    fn test_simple_paragraph() {
        let lex_src = "This is a simple paragraph.\n";
        let lex_doc = STRING_TO_AST.run(lex_src.to_string()).unwrap();

        let html = serialize_to_html(&lex_doc, HtmlTheme::Modern).unwrap();

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<p class=\"lex-paragraph\">"));
        assert!(html.contains("This is a simple paragraph."));
    }

    #[test]
    fn test_heading() {
        let lex_src = "1. Introduction\n\n    Content here.\n";
        let lex_doc = STRING_TO_AST.run(lex_src.to_string()).unwrap();

        let html = serialize_to_html(&lex_doc, HtmlTheme::Modern).unwrap();

        assert!(html.contains("<section class=\"lex-session lex-session-1\">"));
        assert!(html.contains("<h1>"));
        assert!(html.contains("Introduction"));
    }

    #[test]
    fn test_css_embedded() {
        let lex_src = "Test document.\n";
        let lex_doc = STRING_TO_AST.run(lex_src.to_string()).unwrap();

        let html = serialize_to_html(&lex_doc, HtmlTheme::Modern).unwrap();

        assert!(html.contains("<style>"));
        assert!(html.contains(".lex-document"));
        assert!(html.contains("Helvetica")); // Modern theme uses Helvetica font
    }

    #[test]
    fn test_fancy_serif_theme() {
        let lex_src = "Test document.\n";
        let lex_doc = STRING_TO_AST.run(lex_src.to_string()).unwrap();

        let html = serialize_to_html(&lex_doc, HtmlTheme::FancySerif).unwrap();

        assert!(html.contains("Cormorant")); // Fancy serif theme uses Cormorant font
    }
}
