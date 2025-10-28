//! Intermediate AST structures that hold spans instead of extracted text
//! These are converted to final AST structures after parsing completes
//!
//! ## Phase 3b Note
//! The content field of DocumentWithSpans is now converted to final ContentItem types at parse time,
//! skipping the intermediate WithSpans types. This is achieved by passing source text through the parser.

use super::parameters::ParameterWithSpans;
use crate::txxt_nano::ast::ContentItem;

#[derive(Debug, Clone)]
#[allow(dead_code)] // Used internally in parser, may not be directly constructed elsewhere
pub(crate) struct ParagraphWithSpans {
    pub(crate) line_spans: Vec<Vec<std::ops::Range<usize>>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct SessionWithSpans {
    pub(crate) title_spans: Vec<std::ops::Range<usize>>,
    pub(crate) content: Vec<ContentItemWithSpans>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct DefinitionWithSpans {
    pub(crate) subject_spans: Vec<std::ops::Range<usize>>,
    pub(crate) content: Vec<ContentItemWithSpans>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ForeignBlockWithSpans {
    pub(crate) subject_spans: Vec<std::ops::Range<usize>>,
    pub(crate) content_spans: Option<Vec<std::ops::Range<usize>>>,
    pub(crate) closing_annotation: AnnotationWithSpans,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct AnnotationWithSpans {
    pub(crate) label_span: Option<std::ops::Range<usize>>, // Optional: can have label, params, or both
    pub(crate) parameters: Vec<ParameterWithSpans>,
    pub(crate) content: Vec<ContentItemWithSpans>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ListItemWithSpans {
    pub(crate) text_spans: Vec<std::ops::Range<usize>>,
    pub(crate) content: Vec<ContentItemWithSpans>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ListWithSpans {
    pub(crate) items: Vec<ListItemWithSpans>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum ContentItemWithSpans {
    Paragraph(ParagraphWithSpans),
    Session(SessionWithSpans),
    List(ListWithSpans),
    Definition(DefinitionWithSpans),
    Annotation(AnnotationWithSpans),
    ForeignBlock(ForeignBlockWithSpans),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct DocumentWithSpans {
    pub(crate) metadata: Vec<AnnotationWithSpans>,
    /// Phase 3b: Content is converted to final ContentItem types at parse time
    pub(crate) content: Vec<ContentItem>,
}
