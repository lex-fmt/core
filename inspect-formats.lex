Comprehensive Trait Analysis & Design Opportunities

  Available Traits

  | Trait           | Purpose                       | Key Methods
                |
  |-----------------|-------------------------------|---------------------------------------------------------
  --------------|
  | AstNode         | Universal node interface      | node_type(), display_label(), range(), accept()
                |
  | Container       | Nodes with label + children   | label(), children(), children_mut()
                |
  | TextNode        | Leaf text nodes               | text(), lines()
                |
  | VisualStructure | Line-oriented rendering hints | is_source_line_node(), has_visual_header(),
  collapses_with_children() |
  | Visitor         | AST traversal pattern         | visit_*() methods
                |

  Trait Implementation Matrix

  | Node           | AstNode | Container | TextNode | VisualStructure                        | Notes
            |
  |----------------|---------|-----------|----------|----------------------------------------|----------------
  ----------|
  | Session        | ✅       | ✅         | ❌        | ✅ (is_source_line + has_visual_header) | Label =
  title            |
  | Definition     | ✅       | ✅         | ❌        | ✅ (is_source_line + has_visual_header) | Label =
  subject          |
  | Annotation     | ✅       | ✅         | ❌        | ✅ (is_source_line + has_visual_header) | Label =
  data.label.value |
  | VerbatimBlock  | ✅       | ✅         | ❌        | ✅ (is_source_line + has_visual_header) | Label =
  subject          |
  | List           | ✅       | ❌         | ❌        | ✅ (collapses_with_children)            | Not a
  Container!         |
  | Paragraph      | ✅       | ❌         | ❌        | ✅ (collapses_with_children)            | Not a
  Container!         |
  | ListItem       | ✅       | ✅         | ❌        | ✅ (is_source_line)                     | Label =
  text[0]          |
  | TextLine       | ✅       | ❌         | ❌        | ✅ (is_source_line)                     |
                |
  | VerbatimLine   | ✅       | ❌         | ❌        | ✅ (is_source_line)                     |
                |
  | BlankLineGroup | ✅       | ❌         | ❌        | ✅ (is_source_line)                     |
                |

  Key Insights

  1. Container Trait Already Has Labels!

  Current duplication in --ast-full mode:
  // treeviz/tag currently do this manually:
  ContentItem::Session(s) => {
      let title = s.title.as_string();  // DUPLICATE!
      // Show as SessionTitle synthetic child
  }

  Could use Container trait instead:
  // If node implements Container, its label() IS the synthetic header!
  if let Some(container) = item.as_container() {
      if include_all && container.has_visual_header() {
          show_synthetic_header(container.label());  // ✅ No duplication!
      }
  }

  2. VisualStructure Already Tells Us Everything

  | Method                    | Tells us                                       | Current usage
          |
  |---------------------------|------------------------------------------------|------------------------------
  --------|
  | is_source_line_node()     | Node corresponds to a source line              | ❌ Not used
           |
  | has_visual_header()       | Node has label that should be shown separately | ❌ Not used (treeviz/tag
  hardcode it) |
  | collapses_with_children() | Should collapse with children                  | ✅ linetreeviz uses it
           |

  Nodes with visual headers (has_visual_header = true):
  - Session (title)
  - Definition (subject)
  - Annotation (label)
  - VerbatimBlock (subject)

  These are EXACTLY the nodes treeviz/tag show synthetic children for!

  3. The Synthetic Children Pattern

  Current code manually extracts:
  - Session → SessionTitle (from s.title)
  - Definition → Subject (from d.subject)
  - Annotation → Label (from a.data.label.value)
  - ListItem → Marker + Text (from li.marker + li.text)

  But wait:
  - Session/Definition/Annotation/VerbatimBlock implement Container → label() gives us the header!
  - ListItem also implements Container → label() gives us first text

  The only special cases are:
  - ListItem.marker (not in label)
  - ListItem.text (multiple, label only returns first)
  - Annotation parameters (not in label)

  Proposed Trait-Based Design

  // Common module: formats/common/ast_traversal.rs

  /// Determines which nodes to show based on filter mode
  enum FilterMode {
      Full,      // --ast-full: shows everything including synthetic
      Block,     // Default: shows structural nodes
      Line,      // Line-based: only non-collapsible nodes
  }

  /// Information about a node to render
  struct RenderNode<'a> {
      item: &'a ContentItem,
      synthetic_header: Option<String>,  // From Container::label() if has_visual_header()
      should_show: bool,
      collapsed: bool,
  }

  /// Analyze a node using traits to determine how to render it
  fn analyze_node(item: &ContentItem, mode: FilterMode) -> RenderNode {
      let should_show = match mode {
          FilterMode::Full => true,
          FilterMode::Block => !item.collapses_with_children(),
          FilterMode::Line => item.is_source_line_node(),
      };

      let collapsed = match mode {
          FilterMode::Line => item.collapses_with_children(),
          _ => false,
      };

      // Use traits to extract synthetic header!
      let synthetic_header = if mode == FilterMode::Full {
          if item.has_visual_header() {
              // Try Container trait first
              if let Some(label) = try_as_container(item).map(|c| c.label()) {
                  Some(label.to_string())
              } else {
                  None
              }
          } else {
              None
          }
      } else {
          None
      };

      RenderNode {
          item,
          synthetic_header,
          should_show,
          collapsed,
      }
  }

  // Helper to try casting to Container
  fn try_as_container(item: &ContentItem) -> Option<&dyn Container> {
      match item {
          ContentItem::Session(s) => Some(s as &dyn Container),
          ContentItem::Definition(d) => Some(d as &dyn Container),
          ContentItem::Annotation(a) => Some(a as &dyn Container),
          ContentItem::ListItem(li) => Some(li as &dyn Container),
          ContentItem::VerbatimBlock(v) => Some(v as &dyn Container),
          _ => None,
      }
  }

  What This Gives Us

  1. Eliminate synthetic child duplication:
  // OLD (treeviz/tag): 50+ lines of match arms
  match item {
      ContentItem::Session(s) => {
          show_child("SessionTitle", s.title.as_string());
      }
      ContentItem::Definition(d) => {
          show_child("Subject", d.subject.as_string());
      }
      // etc...
  }

  // NEW: 3 lines using traits
  if let (Some(header), true) = (node.synthetic_header, include_all) {
      show_synthetic_child(item.node_type() + "Title", &header);
  }

  2. Handle special cases explicitly:
  // Only ListItem and Annotation need special handling
  match item {
      ContentItem::ListItem(li) if include_all => {
          show_synthetic("Marker", li.marker.as_string());
          for text in &li.text {
              show_synthetic("Text", text.as_string());
          }
      }
      ContentItem::Annotation(a) if include_all => {
          for param in &a.data.parameters {
              show_synthetic("Parameter", format!("{}={}", param.key, param.value));
          }
      }
      _ => {}
  }

  3. Unified node selection:
  // All formats use same logic, just different FilterMode
  let nodes = doc.traverse(|item| {
      analyze_node(item, mode)
  });

  Answer to Your Questions

  "can't we just use the Container trait and get the label?"

  YES! For Session, Definition, Annotation, and VerbatimBlock:
  - They implement Container
  - container.label() returns exactly what we show as synthetic header
  - No need to manually extract title/subject/label.value

  "I think we could use visual line + other traits to actually have a solid design"

  ABSOLUTELY! The traits capture the exact semantics:

  - has_visual_header() → show synthetic header in --ast-full
  - collapses_with_children() → collapse in linetreeviz
  - is_source_line_node() → useful for line-based formats
  - Container::label() → the header text

  The current code was written before these traits existed, so it manually reimplements what the traits
  already provide!

  Recommended Refactoring Path

  1. Extract try_as_container() helper
  2. Use has_visual_header() + Container::label() for synthetic headers
  3. Keep special handling only for ListItem (marker) and Annotation (parameters)
  4. Extract common analyze_node() logic

